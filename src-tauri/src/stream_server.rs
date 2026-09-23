use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub const PORT: u16 = 9863;

/// User-Agent con el que el proxy pide el audio a googlevideo. Vive aquí para
/// que la verificación previa de la URL (en `piped`) use exactamente los mismos
/// encabezados que la petición real; si difieren, podríamos dar por buena una
/// URL que después el proxy no puede leer.
pub const PLAYBACK_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

type UrlMap = Arc<Mutex<HashMap<String, (String, Instant)>>>;

static URL_STORE: std::sync::OnceLock<UrlMap> = std::sync::OnceLock::new();

const TTL: std::time::Duration = std::time::Duration::from_secs(1500);

pub fn url_store() -> UrlMap {
    URL_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

pub fn register_stream(id: &str, cdn_url: &str) {
    let store = url_store();
    let mut map = store.lock().unwrap();
    map.retain(|_, (_, ts)| ts.elapsed() < TTL);
    map.insert(id.to_string(), (cdn_url.to_string(), Instant::now()));
}

pub async fn start() {
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", PORT)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[stream_server] Failed to bind port {}: {}", PORT, e);
            return;
        }
    };

    eprintln!("[stream_server] Listening on 127.0.0.1:{}", PORT);

    loop {
        let Ok((socket, _)) = listener.accept().await else {
            continue;
        };
        let store = url_store();
        tokio::spawn(handle(socket, store));
    }
}

async fn handle(mut socket: tokio::net::TcpStream, store: UrlMap) {
    let mut buf = vec![0u8; 8192];
    let n = match tokio::io::AsyncReadExt::read(&mut socket, &mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = String::from_utf8_lossy(&buf[..n]);

    if req.starts_with("OPTIONS") {
        let _ = socket
            .write_all(
                b"HTTP/1.1 200 OK\r\n\
                  Access-Control-Allow-Origin: *\r\n\
                  Access-Control-Allow-Headers: Range\r\n\
                  Access-Control-Allow-Methods: GET, HEAD\r\n\
                  Content-Length: 0\r\n\r\n",
            )
            .await;
        return;
    }

    let is_head = req.starts_with("HEAD");

    let range = req
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("range:"))
        .map(|l| l["range:".len()..].trim().to_string());

    let stream_id = match extract_stream_id(&req) {
        Some(id) => id,
        None => {
            let _ = socket
                .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
                .await;
            return;
        }
    };

    let yt_url = {
        let map = store.lock().unwrap();
        match map.get(&stream_id) {
            Some((url, ts)) if ts.elapsed() < TTL => Some(url.clone()),
            _ => None,
        }
    };

    let yt_url = match yt_url {
        Some(u) => u,
        None => {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await;
            return;
        }
    };

    proxy_to_youtube(&mut socket, &stream_id, &yt_url, range.as_deref(), is_head).await;
}

fn extract_stream_id(req: &str) -> Option<String> {
    let first_line = req.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    let path = path.split('?').next()?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[1] == "stream" {
        Some(parts[2..].join("/"))
    } else {
        None
    }
}

/// Tamaño de cada tramo que se le pide a googlevideo.
///
/// No es una optimización: googlevideo responde 403 a los rangos abiertos
/// (`bytes=0-`), a los que exceden el tamaño del archivo y a las peticiones sin
/// cabecera `Range`. Solo acepta tramos acotados y dentro del archivo, así que
/// el proxy trocea la descarga y va concatenando los tramos para el cliente.
const CHUNK: u64 = 1 << 20;

fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| v.to_string())
    })
}

/// Traduce la cabecera `Range` del cliente a un tramo concreto del archivo.
/// Devuelve `(inicio, fin, parcial)`; `parcial` es false cuando el cliente no
/// pidió rango y por tanto espera el recurso entero con un 200.
fn resolve_range(range: Option<&str>, total: u64) -> (u64, u64, bool) {
    let last = total.saturating_sub(1);
    let Some(spec) = range
        .and_then(|r| r.strip_prefix("bytes="))
        .map(str::trim)
        .and_then(|s| s.split(',').next())
    else {
        return (0, last, false);
    };
    let Some((from, to)) = spec.split_once('-') else {
        return (0, last, false);
    };

    let (start, end) = match (from.trim(), to.trim()) {
        ("", "") => (0, last),
        // Sufijo (`bytes=-500`): los últimos N bytes.
        ("", n) => {
            let len = n.parse::<u64>().unwrap_or(total).clamp(1, total.max(1));
            (total.saturating_sub(len), last)
        }
        (s, "") => (s.parse::<u64>().unwrap_or(0).min(last), last),
        (s, e) => {
            let start = s.parse::<u64>().unwrap_or(0).min(last);
            (start, e.parse::<u64>().unwrap_or(last).min(last).max(start))
        }
    };
    (start, end, true)
}

async fn fetch_chunk(
    client: &reqwest::Client,
    yt_url: &str,
    start: u64,
    end: u64,
) -> reqwest::Result<reqwest::Response> {
    client
        .get(yt_url)
        .header("Range", format!("bytes={}-{}", start, end))
        .header("Referer", "https://www.youtube.com/")
        .header("Origin", "https://www.youtube.com")
        .header("User-Agent", PLAYBACK_UA)
        .send()
        .await
}

/// Tamaño total del audio. La URL firmada ya lo trae en `clen`; si faltara, se
/// deduce del `Content-Range` de un tramo mínimo.
async fn upstream_total(client: &reqwest::Client, yt_url: &str) -> Option<u64> {
    if let Some(clen) = query_param(yt_url, "clen").and_then(|v| v.parse::<u64>().ok()) {
        if clen > 0 {
            return Some(clen);
        }
    }
    let resp = fetch_chunk(client, yt_url, 0, 1).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let cr = resp.headers().get("content-range")?.to_str().ok()?;
    cr.rsplit('/').next()?.trim().parse::<u64>().ok()
}

async fn proxy_to_youtube(
    socket: &mut tokio::net::TcpStream,
    stream_id: &str,
    yt_url: &str,
    range: Option<&str>,
    is_head: bool,
) {
    use futures_util::StreamExt;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(10))
        .read_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let Some(total) = upstream_total(&client, yt_url).await else {
        eprintln!(
            "[stream_server] id={} no se pudo determinar el tamaño del audio",
            stream_id
        );
        let _ = socket
            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
            .await;
        return;
    };

    let (start, end, partial) = resolve_range(range, total);
    let len = end - start + 1;

    // El primer tramo confirma que la URL sigue viva y trae el Content-Type real,
    // antes de comprometernos con un estado de respuesta hacia el cliente.
    let first_end = if is_head { start } else { (start + CHUNK - 1).min(end) };
    let first = match fetch_chunk(&client, yt_url, start, first_end).await {
        Ok(r) => r,
        Err(e) => {
            let body = e.to_string();
            let msg = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(msg.as_bytes()).await;
            return;
        }
    };

    eprintln!(
        "[stream_server] id={} range={:?} → tramo bytes={}-{}/{} upstream {}",
        stream_id,
        range.unwrap_or("(ninguno)"),
        start,
        first_end,
        total,
        first.status()
    );

    if !first.status().is_success() {
        let status = first.status();
        let body = first.bytes().await.unwrap_or_default();
        let preview_len = body.len().min(300);
        eprintln!(
            "[stream_server] id={} upstream error body preview: {}",
            stream_id,
            String::from_utf8_lossy(&body[..preview_len])
        );
        let msg = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown"),
            body.len()
        );
        let _ = socket.write_all(msg.as_bytes()).await;
        if !is_head {
            let _ = socket.write_all(&body).await;
        }
        return;
    }

    let content_type = first
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mp4")
        .to_string();

    let mut head = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: {}\r\n\
         Accept-Ranges: bytes\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Expose-Headers: Content-Range,Content-Length,Accept-Ranges\r\n",
        if partial { "206 Partial Content" } else { "200 OK" },
        content_type,
        len
    );
    if partial {
        head += &format!("Content-Range: bytes {}-{}/{}\r\n", start, end, total);
    }
    head += "\r\n";

    if socket.write_all(head.as_bytes()).await.is_err() || is_head {
        return;
    }

    // Se reutiliza el primer tramo y se piden los siguientes a medida que hacen falta.
    let mut pending = Some(first);
    let mut pos = start;
    while pos <= end {
        let chunk_end = (pos + CHUNK - 1).min(end);
        let resp = match pending.take() {
            Some(r) => r,
            None => match fetch_chunk(&client, yt_url, pos, chunk_end).await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    eprintln!(
                        "[stream_server] id={} tramo bytes={}-{} → {}",
                        stream_id, pos, chunk_end, r.status()
                    );
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "[stream_server] id={} tramo bytes={}-{} falló: {}",
                        stream_id, pos, chunk_end, e
                    );
                    break;
                }
            },
        };

        let mut body = resp.bytes_stream();
        while let Some(chunk) = body.next().await {
            match chunk {
                // El cliente cortó (cambio de pista, seek): no es un error.
                Ok(b) if socket.write_all(&b).await.is_err() => return,
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[stream_server] id={} corte al leer el tramo: {}", stream_id, e);
                    return;
                }
            }
        }
        pos = chunk_end + 1;
    }

    let _ = tokio::io::AsyncWriteExt::shutdown(socket).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    /// Ejercita el proxy completo contra una URL real de googlevideo, sin que el
    /// cliente mande `Range` — que es exactamente lo que hace el <audio> de
    /// WebKitGTK y lo que googlevideo rechaza con 403 si se le reenvía tal cual.
    ///
    /// Necesita red y una URL firmada viva (caducan en unas horas), así que no
    /// corre por defecto:
    ///
    /// ```text
    /// ICARIA_TEST_STREAM_URL='https://rr1---sn-…googlevideo.com/videoplayback?…' \
    ///   cargo test --no-default-features -- --ignored --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "necesita red y una URL de googlevideo viva en ICARIA_TEST_STREAM_URL"]
    async fn sirve_el_audio_entero_cuando_el_cliente_no_pide_rango() {
        let url = std::env::var("ICARIA_TEST_STREAM_URL")
            .expect("falta ICARIA_TEST_STREAM_URL");
        register_stream("test", &url);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            handle(socket, url_store()).await;
        });

        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
        client
            .write_all(b"GET /stream/test HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();

        let mut raw = Vec::new();
        client.read_to_end(&mut raw).await.unwrap();
        server.await.unwrap();

        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("respuesta sin fin de cabeceras");
        let headers = String::from_utf8_lossy(&raw[..split]).to_string();
        let body = &raw[split + 4..];

        assert!(
            headers.starts_with("HTTP/1.1 200 OK"),
            "se esperaba 200 OK, llegó:\n{}",
            headers
        );

        let declared: usize = headers
            .lines()
            .find_map(|l| l.strip_prefix("Content-Length: "))
            .and_then(|v| v.trim().parse().ok())
            .expect("respuesta sin Content-Length");

        assert_eq!(body.len(), declared, "el cuerpo no coincide con Content-Length");
        assert!(declared > 100_000, "audio sospechosamente corto: {} bytes", declared);
        // Un m4a (itag 140) empieza por la caja `ftyp`.
        assert_eq!(&body[4..8], b"ftyp", "el cuerpo no parece un MP4");
        println!("OK: {} bytes servidos, cabeceras:\n{}", body.len(), headers);
    }

    /// Cadena completa: resolver un video real y servirlo por el proxy tal como
    /// lo pide el <audio> (sin `Range`). Cubre lo que las pruebas por partes no
    /// ven: que el resolutor elegido entregue una URL que sirva el archivo
    /// entero, no solo su principio.
    ///
    /// ```text
    /// ICARIA_TEST_VIDEO_ID=SCLzBtjC9a8 cargo test --no-default-features -- --ignored --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "necesita red y habla con YouTube"]
    async fn resuelve_y_sirve_un_video_real() {
        let video_id =
            std::env::var("ICARIA_TEST_VIDEO_ID").unwrap_or_else(|_| "SCLzBtjC9a8".to_string());

        let stream = crate::sources::piped::get_stream(&video_id)
            .await
            .expect("no se pudo resolver el audio");
        register_stream(&video_id, &stream.url);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            handle(socket, url_store()).await;
        });

        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let req = format!("GET /stream/{} HTTP/1.1\r\nHost: localhost\r\n\r\n", video_id);
        client.write_all(req.as_bytes()).await.unwrap();

        let mut raw = Vec::new();
        client.read_to_end(&mut raw).await.unwrap();
        server.await.unwrap();

        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("respuesta sin fin de cabeceras");
        let headers = String::from_utf8_lossy(&raw[..split]).to_string();
        let body = &raw[split + 4..];

        assert!(
            headers.starts_with("HTTP/1.1 200 OK"),
            "se esperaba 200 OK, llegó:\n{}",
            headers
        );
        let declared: usize = headers
            .lines()
            .find_map(|l| l.strip_prefix("Content-Length: "))
            .and_then(|v| v.trim().parse().ok())
            .expect("respuesta sin Content-Length");

        assert_eq!(
            body.len(),
            declared,
            "se cortó a mitad: {} de {} bytes",
            body.len(),
            declared
        );
        assert_eq!(&body[4..8], b"ftyp", "el cuerpo no parece un MP4");
        println!("OK: {} bytes de {} servidos enteros", body.len(), video_id);
    }
}
