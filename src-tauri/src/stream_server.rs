use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub const PORT: u16 = 9863;

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

async fn proxy_to_youtube(
    socket: &mut tokio::net::TcpStream,
    stream_id: &str,
    yt_url: &str,
    range: Option<&str>,
    is_head: bool,
) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(10))
        .read_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let mut rb = client
        .get(yt_url)
        .header("Referer", "https://www.youtube.com/")
        .header("Origin", "https://www.youtube.com")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );

    if let Some(r) = range {
        rb = rb.header("Range", r);
    }

    let resp = match rb.send().await {
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

    let status = resp.status().as_u16();
    let reason = resp.status().canonical_reason().unwrap_or("Unknown");

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mp4")
        .to_string();

    let content_length = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let content_range = resp
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut head = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Accept-Ranges: bytes\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Expose-Headers: Content-Range,Content-Length,Accept-Ranges\r\n",
        status, reason, content_type
    );

    if let Some(cr) = &content_range {
        head += &format!("Content-Range: {}\r\n", cr);
    }
    if let Some(cl) = &content_length {
        head += &format!("Content-Length: {}\r\n", cl);
    } else {
        head += "Transfer-Encoding: chunked\r\n";
    }
    head += "\r\n";

    eprintln!(
        "[stream_server] id={} range={:?} → upstream {} {} content_type={} content_length={:?}",
        stream_id, range, status, reason, content_type, content_length
    );

    if !resp.status().is_success() {
        let body = resp.bytes().await.unwrap_or_default();
        let preview_len = body.len().min(300);
        eprintln!(
            "[stream_server] id={} upstream error body preview: {}",
            stream_id,
            String::from_utf8_lossy(&body[..preview_len])
        );
        let _ = socket.write_all(head.as_bytes()).await;
        if !is_head {
            let _ = socket.write_all(&body).await;
        }
        return;
    }

    if is_head {
        let _ = socket.write_all(head.as_bytes()).await;
        return;
    }

    let _ = socket.write_all(head.as_bytes()).await;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if content_length.is_some() {
                    if socket.write_all(&chunk).await.is_err() {
                        break;
                    }
                } else {
                    let len = chunk.len();
                    let size_line = format!("{:x}\r\n", len);
                    if socket.write_all(size_line.as_bytes()).await.is_err() {
                        break;
                    }
                    if socket.write_all(&chunk).await.is_err() {
                        break;
                    }
                    if socket.write_all(b"\r\n").await.is_err() {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
    if content_length.is_none() {
        let _ = socket.write_all(b"0\r\n\r\n").await;
    }
    let _ = tokio::io::AsyncWriteExt::shutdown(socket).await;
}
