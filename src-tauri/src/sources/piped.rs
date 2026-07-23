/// YouTube source — search via YouTube Music InnerTube API (instant),
/// stream via yt-dlp (reliable, ~2s, only called on play).
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use super::{Album, AlbumDetail, Artist, ArtistDetail, SearchResult, StreamUrl, Track, TrackSource};

// ── YouTube Music InnerTube search ──────────────────────────────────────────

const YTM_SEARCH_URL: &str =
    "https://music.youtube.com/youtubei/v1/search?prettyPrint=false";

// Category-specific params for YTM InnerTube search (base64-encoded protobuf)
const YTM_PARAMS: &str = "EgWKAQIIAWoKEAoQCRADEAQQBQ==";          // Songs
const YTM_PARAMS_ARTISTS: &str = "EgWKAQIgAWoKEAoQCRADEAQQBQ==";  // Artists
const YTM_PARAMS_ALBUMS: &str = "EgWKAQIYAWoKEAoQCRADEAQQBQ==";   // Albums

pub async fn search(query: &str, max: u32) -> Result<Vec<Track>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let body = json!({
        "query": query,
        "params": YTM_PARAMS,
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20241106.01.00",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    let resp = client
        .post(YTM_SEARCH_URL)
        .header("Content-Type", "application/json")
        .header("Origin", "https://music.youtube.com")
        .header("Referer", "https://music.youtube.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YouTube Music API error: {}", resp.status()));
    }

    let data: Value = resp.json().await?;
    let mut tracks = extract_ytm_tracks(&data, max);

    if tracks.is_empty() {
        tracks = ytdlp_search(query, max).await?;
    }

    Ok(tracks)
}

pub async fn search_all(query: &str, max: u32) -> Result<SearchResult> {
    use std::sync::Arc;

    let client = Arc::new(
        Client::builder().timeout(Duration::from_secs(10)).build()?,
    );

    // Three parallel requests — one per category.
    // Each uses a specific params value so the response only contains that type.
    // This is more reliable than the unfiltered "all" search whose structure varies.
    let (songs_r, artists_r, albums_r) = tokio::join!(
        ytm_fetch(client.clone(), query, YTM_PARAMS),
        ytm_fetch(client.clone(), query, YTM_PARAMS_ARTISTS),
        ytm_fetch(client.clone(), query, YTM_PARAMS_ALBUMS),
    );

    let tracks = songs_r
        .map(|d| extract_ytm_tracks(&d, max))
        .unwrap_or_default();

    let artists = artists_r
        .map(|d| ytm_shelf_items(&d, max).into_iter().filter_map(|r| extract_artist(&r)).collect())
        .unwrap_or_default();

    let albums = albums_r
        .map(|d| ytm_shelf_items(&d, max).into_iter().filter_map(|r| extract_album(&r)).collect())
        .unwrap_or_default();

    Ok(SearchResult { tracks, artists, albums })
}

/// POST a single YTM InnerTube search request and return the parsed JSON.
async fn ytm_fetch(client: std::sync::Arc<Client>, query: &str, params: &str) -> Result<Value> {
    let body = json!({
        "query": query,
        "params": params,
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20241106.01.00",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    let resp = client
        .post(YTM_SEARCH_URL)
        .header("Content-Type", "application/json")
        .header("Origin", "https://music.youtube.com")
        .header("Referer", "https://music.youtube.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YouTube Music API error: {}", resp.status()));
    }

    Ok(resp.json().await?)
}

/// Flatten all musicResponsiveListItemRenderer items from a YTM search response.
fn ytm_shelf_items(data: &Value, max: u32) -> Vec<Value> {
    let mut items = Vec::new();

    let Some(tabs) = data["contents"]["tabbedSearchResultsRenderer"]["tabs"].as_array() else {
        return items;
    };

    'outer: for tab in tabs {
        let Some(sections) = tab["tabRenderer"]["content"]["sectionListRenderer"]["contents"]
            .as_array()
        else {
            continue;
        };

        for section in sections {
            let Some(section_items) = section["musicShelfRenderer"]["contents"].as_array() else {
                continue;
            };

            for item in section_items {
                let r = &item["musicResponsiveListItemRenderer"];
                if !r.is_null() {
                    items.push(r.clone());
                    if items.len() >= max as usize {
                        break 'outer;
                    }
                }
            }
        }
    }

    items
}

fn extract_ytm_tracks(data: &Value, max: u32) -> Vec<Track> {
    let mut tracks = Vec::new();

    let tabs = match data["contents"]["tabbedSearchResultsRenderer"]["tabs"].as_array() {
        Some(t) => t,
        None => return tracks,
    };

    'outer: for tab in tabs {
        let sections = match tab["tabRenderer"]["content"]["sectionListRenderer"]["contents"]
            .as_array()
        {
            Some(s) => s,
            None => continue,
        };

        for section in sections {
            let items = match section["musicShelfRenderer"]["contents"].as_array() {
                Some(i) => i,
                None => continue,
            };

            for item in items {
                let r = &item["musicResponsiveListItemRenderer"];

                // videoId lives in the play-button endpoint
                let video_id = r["overlay"]["musicItemThumbnailOverlayRenderer"]["content"]
                    ["musicPlayButtonRenderer"]["playNavigationEndpoint"]["watchEndpoint"]
                    ["videoId"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                if video_id.is_empty() {
                    continue;
                }

                let title = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]
                    ["text"]["runs"][0]["text"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string();

                let artist = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]
                    ["text"]["runs"][0]["text"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string();

                // Duration is in fixedColumns[0] as "3:45"
                let duration_str = r["fixedColumns"][0]
                    ["musicResponsiveListItemFixedColumnRenderer"]["text"]["runs"][0]["text"]
                    .as_str()
                    .unwrap_or("");
                let duration_ms = parse_duration(duration_str);

                // Best thumbnail from the response, fallback to mqdefault
                let thumbnail = best_thumbnail(r, &video_id);

                tracks.push(Track {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    artist,
                    album: None,
                    duration_ms,
                    thumbnail: Some(thumbnail),
                    source: TrackSource::YouTube,
                    stream_id: video_id,
                });

                if tracks.len() >= max as usize {
                    break 'outer;
                }
            }
        }
    }

    tracks
}


fn extract_artist(r: &Value) -> Option<Artist> {
    // browseId is on the top-level navigationEndpoint for artist items,
    // not inside flexColumns like tracks.
    let browse_id = r["navigationEndpoint"]["browseEndpoint"]["browseId"]
        .as_str()
        .or_else(|| {
            r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
                ["navigationEndpoint"]["browseEndpoint"]["browseId"]
                .as_str()
        })
        .unwrap_or("")
        .to_string();

    // Accept any non-empty browseId (UC* = YT channel, MPLA* = YTM artist)
    if browse_id.is_empty() {
        return None;
    }

    let name = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
        ["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let subscribers = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]["text"]
        ["runs"]
        .as_array()
        .and_then(|runs| {
            let texts: Vec<String> = runs
                .iter()
                .filter_map(|run| run["text"].as_str().map(|s| s.to_string()))
                .collect();
            if texts.is_empty() {
                None
            } else {
                Some(texts.join(""))
            }
        });

    let thumbnail = best_thumbnail_artist(r);

    Some(Artist {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        thumbnail: Some(thumbnail),
        subscribers,
        browse_id,
    })
}

fn extract_album(r: &Value) -> Option<Album> {
    // Try multiple paths for browseId — YTM response structure varies by result type
    let browse_id = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]
        ["runs"][0]["navigationEndpoint"]["browseEndpoint"]["browseId"]
        .as_str()
        .or_else(|| r["navigationEndpoint"]["browseEndpoint"]["browseId"].as_str())
        .or_else(|| {
            r["overlay"]["musicItemThumbnailOverlayRenderer"]["content"]
                ["musicPlayButtonRenderer"]["playNavigationEndpoint"]["browseEndpoint"]["browseId"]
                .as_str()
        })
        .unwrap_or("")
        .to_string();

    if browse_id.is_empty() {
        return None;
    }

    let title = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
        ["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let subtitle_runs = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]["text"]
        ["runs"]
        .as_array();

    let (artist_name, year) = match subtitle_runs {
        Some(runs) => {
            // Subtitle runs look like: ["Album", " • ", "Artist", " • ", "2022"]
            // Filter separators, then skip type keyword, extract artist and year
            let texts: Vec<&str> = runs
                .iter()
                .filter_map(|run| run["text"].as_str())
                .filter(|t| {
                    let t = t.trim();
                    !t.is_empty() && t != "•" && t != "·" && t != " • " && t != " · "
                })
                .collect();

            let yr = texts.last()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .filter(|&y| y > 1900 && y < 2100)
                .map(|y| y.to_string());

            let type_words = ["album", "single", "ep", "álbum", "sencillo", "compilation", "playlist"];
            let artist = texts.iter()
                .find(|&&t| {
                    let tl = t.trim().to_lowercase();
                    !type_words.contains(&tl.as_str()) && t.trim().parse::<u32>().is_err()
                })
                .map(|s| s.to_string())
                .unwrap_or_default();

            (artist, yr)
        }
        None => (String::new(), None),
    };

    let thumbnail = best_thumbnail_artist(r);

    Some(Album {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        artist: artist_name,
        year,
        thumbnail: Some(thumbnail),
        browse_id,
    })
}

fn extract_track_from_renderer(r: &Value) -> Option<Track> {
    let video_id = r["overlay"]["musicItemThumbnailOverlayRenderer"]["content"]
        ["musicPlayButtonRenderer"]["playNavigationEndpoint"]["watchEndpoint"]["videoId"]
        .as_str()
        // Album track listings often use playlistItemData instead of overlay
        .or_else(|| r["playlistItemData"]["videoId"].as_str())
        // Fallback: flexColumn navigation endpoint
        .or_else(|| {
            r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
                ["navigationEndpoint"]["watchEndpoint"]["videoId"]
                .as_str()
        })
        .unwrap_or("")
        .to_string();

    if video_id.is_empty() {
        return None;
    }

    let title = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
        ["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let artist = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"][0]
        ["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let duration_str = r["fixedColumns"][0]["musicResponsiveListItemFixedColumnRenderer"]["text"]
        ["runs"][0]["text"]
        .as_str()
        .unwrap_or("");

    let thumbnail = best_thumbnail(r, &video_id);

    Some(Track {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        artist,
        album: None,
        duration_ms: parse_duration(duration_str),
        thumbnail: Some(thumbnail),
        source: TrackSource::YouTube,
        stream_id: video_id,
    })
}

fn best_thumbnail_artist(r: &Value) -> String {
    if let Some(thumbs) = r["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
        .as_array()
    {
        if let Some(last) = thumbs.last() {
            if let Some(url) = last["url"].as_str() {
                return url.to_string();
            }
        }
    }
    if let Some(thumbs) = r["thumbnail"]["musicAvatarRenderer"]["thumbnail"]["thumbnails"]
        .as_array()
    {
        if let Some(last) = thumbs.last() {
            if let Some(url) = last["url"].as_str() {
                return url.to_string();
            }
        }
    }
    String::new()
}

fn best_thumbnail(r: &Value, video_id: &str) -> String {
    // YouTube Music includes thumbnails inside the renderer
    if let Some(thumbs) = r["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
        .as_array()
    {
        if let Some(last) = thumbs.last() {
            if let Some(url) = last["url"].as_str() {
                return url.to_string();
            }
        }
    }
    // hqdefault = 480x360 (siempre disponible, buena calidad)
    format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id)
}

fn parse_duration(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    match parts.len() {
        2 => {
            let m: u64 = parts[0].parse().ok()?;
            let s: u64 = parts[1].parse().ok()?;
            Some((m * 60 + s) * 1000)
        }
        3 => {
            let h: u64 = parts[0].parse().ok()?;
            let m: u64 = parts[1].parse().ok()?;
            let s: u64 = parts[2].parse().ok()?;
            Some((h * 3600 + m * 60 + s) * 1000)
        }
        _ => None,
    }
}

// ── yt-dlp fallback search ───────────────────────────────────────────────────

fn ytdlp_bin() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let local = format!("{}/.local/bin/yt-dlp", home);
    if std::path::Path::new(&local).exists() {
        return local;
    }
    "yt-dlp".to_string()
}

/// Navegador del que yt-dlp toma cookies para pasar el "confirm you're not a bot"
/// / HTTP 429 de YouTube. Se puede forzar con ICARIA_COOKIES_BROWSER (p.ej. "chrome");
/// si no, autodetecta un perfil existente. None si no hay ninguno.
fn cookies_browser() -> Option<String> {
    if let Ok(b) = std::env::var("ICARIA_COOKIES_BROWSER") {
        let b = b.trim().to_string();
        return if b.is_empty() { None } else { Some(b) };
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        ("firefox", format!("{home}/.mozilla/firefox")),
        ("chrome", format!("{home}/.config/google-chrome")),
        ("chromium", format!("{home}/.config/chromium")),
        ("brave", format!("{home}/.config/BraveSoftware/Brave-Browser")),
        ("vivaldi", format!("{home}/.config/vivaldi")),
    ];
    candidates
        .into_iter()
        .find(|(_, path)| std::path::Path::new(path).exists())
        .map(|(name, _)| name.to_string())
}

#[derive(serde::Deserialize)]
struct YtDlpFlat {
    id: String,
    title: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
}

async fn ytdlp_search(query: &str, max: u32) -> Result<Vec<Track>> {
    let bin = ytdlp_bin();
    let arg = format!("ytsearch{}:{}", max, query);

    let out = tokio::task::spawn_blocking(move || {
        Command::new(&bin)
            .args(["--flat-playlist", "--dump-json", "--quiet", "--no-warnings", &arg])
            .output()
    })
    .await??;

    if !out.status.success() {
        return Err(anyhow!("yt-dlp search failed"));
    }

    let mut tracks = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(e) = serde_json::from_str::<YtDlpFlat>(line) {
            let artist = e.uploader.or(e.channel).unwrap_or_else(|| "YouTube".into());
            let thumbnail = format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", e.id);
            tracks.push(Track {
                id: uuid::Uuid::new_v4().to_string(),
                title: e.title.unwrap_or_else(|| e.id.clone()),
                artist,
                album: None,
                duration_ms: e.duration.map(|d| (d * 1000.0) as u64),
                thumbnail: Some(thumbnail),
                source: TrackSource::YouTube,
                stream_id: e.id,
            });
        }
    }
    Ok(tracks)
}

// ── Artist / Album browse via InnerTube ──────────────────────────────────────

const YTM_BROWSE_URL: &str =
    "https://music.youtube.com/youtubei/v1/browse?prettyPrint=false";

fn ytm_context() -> Value {
    json!({
        "client": {
            "clientName": "WEB_REMIX",
            "clientVersion": "1.20241106.01.00",
            "hl": "en",
            "gl": "US"
        }
    })
}

fn ytm_headers(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    req.header("Content-Type", "application/json")
        .header("Origin", "https://music.youtube.com")
        .header("Referer", "https://music.youtube.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
}

pub async fn get_artist(browse_id: &str) -> Result<ArtistDetail> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let body = json!({
        "browseId": browse_id,
        "context": { "client": ytm_context()["client"] }
    });

    let resp = ytm_headers(client.post(YTM_BROWSE_URL))
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YouTube Music browse error: {}", resp.status()));
    }

    let data: Value = resp.json().await?;
    parse_artist_detail(&data)
}

fn parse_artist_detail(data: &Value) -> Result<ArtistDetail> {
    // Try multiple header types that YTM uses for artist pages.
    let header = &data["header"]["musicImmersiveHeaderRenderer"];
    if !header.is_null() {
        // fall through to the main parser below
    } else {
        let header2 = &data["header"]["musicVisualHeaderRenderer"];
        if !header2.is_null() {
            return parse_artist_detail_from_visual(data, header2);
        }
        // musicHeaderRenderer — simpler header used for some channels
        let header3 = &data["header"]["musicHeaderRenderer"];
        if !header3.is_null() {
            return parse_artist_detail_from_visual(data, header3);
        }
        // No recognized header; attempt best-effort parse anyway
        return parse_artist_detail_content_only(data);
    }

    let name = header["title"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let subscribers = header["subscriptionButton"]["subscribeButtonRenderer"]["subscriberCountText"]
        ["runs"]
        .as_array()
        .and_then(|runs| {
            let texts: Vec<String> = runs
                .iter()
                .filter_map(|r| r["text"].as_str().map(|s| s.to_string()))
                .collect();
            if texts.is_empty() { None } else { Some(texts.join("")) }
        });

    let thumbnail = header["foregroundThumbnail"]["thumbnails"]
        .as_array()
        .and_then(|t| t.last())
        .and_then(|t| t["url"].as_str().map(|s| s.to_string()));

    let description = data["description"]["musicDescriptionShelfRenderer"]["description"]
        ["runs"]
        .as_array()
        .map(|runs| {
            runs.iter()
                .filter_map(|r| r["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        });

    let (tracks, albums) = parse_artist_content(data);

    Ok(ArtistDetail {
        name,
        thumbnail,
        subscribers,
        description,
        tracks,
        albums,
    })
}

fn parse_artist_detail_content_only(data: &Value) -> Result<ArtistDetail> {
    let (tracks, albums) = parse_artist_content(data);
    Ok(ArtistDetail {
        name: "Unknown Artist".to_string(),
        thumbnail: None,
        subscribers: None,
        description: None,
        tracks,
        albums,
    })
}

fn parse_artist_content(data: &Value) -> (Vec<Track>, Vec<Album>) {
    let mut tracks = Vec::new();
    let mut albums = Vec::new();

    let contents_paths: &[&[&str]] = &[
        &["contents","singleColumnBrowseResultsRenderer","tabs","0","tabRenderer","content","sectionListRenderer","contents"],
        &["contents","twoColumnBrowseResultsRenderer","secondaryContents","sectionListRenderer","contents"],
    ];

    for path in contents_paths {
        if let Some(sections) = navigate(data, path).and_then(|v| v.as_array()) {
            for section in sections {
                let section_title = section["musicShelfRenderer"]["title"]["runs"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase();

                if let Some(items) = section["musicShelfRenderer"]["contents"].as_array() {
                    for item in items {
                        let r = &item["musicResponsiveListItemRenderer"];
                        if section_title.contains("album") || section_title.contains("ep") || section_title.contains("release") {
                            if let Some(al) = extract_album(r) { albums.push(al); }
                        } else {
                            if let Some(t) = extract_track_from_renderer(r) { tracks.push(t); }
                        }
                    }
                }

                if let Some(carousel) = section["musicCarouselShelfRenderer"]["contents"].as_array() {
                    let carousel_title = section["musicCarouselShelfRenderer"]["header"]
                        ["musicCarouselShelfBasicHeaderRenderer"]["title"]["runs"][0]["text"]
                        .as_str()
                        .or_else(|| {
                            section["musicCarouselShelfRenderer"]["header"]
                                ["musicCarouselShelfBasicInfoRenderer"]["title"]["runs"][0]["text"]
                                .as_str()
                        })
                        .unwrap_or("")
                        .to_lowercase();

                    for item in carousel {
                        let r = &item["musicTwoRowItemRenderer"];
                        if r.is_null() { continue; }
                        if carousel_title.contains("album") || carousel_title.contains("ep") || carousel_title.contains("release") || carousel_title.contains("single") {
                            if let Some(al) = extract_album_two_row(r) { albums.push(al); }
                        }
                    }
                }
            }
            if !tracks.is_empty() || !albums.is_empty() { break; }
        }
    }

    (tracks, albums)
}

fn navigate<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for key in path {
        cur = if let Ok(idx) = key.parse::<usize>() { &cur[idx] } else { &cur[*key] };
        if cur.is_null() { return None; }
    }
    Some(cur)
}

fn parse_artist_detail_from_visual(data: &Value, header: &Value) -> Result<ArtistDetail> {
    let name = header["title"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let thumbnail = header["foregroundThumbnail"]["thumbnails"]
        .as_array()
        .and_then(|t| t.last())
        .and_then(|t| t["url"].as_str().map(|s| s.to_string()))
        .or_else(|| {
            header["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
                .as_array()
                .and_then(|t| t.last())
                .and_then(|t| t["url"].as_str().map(|s| s.to_string()))
        });

    let (tracks, albums) = parse_artist_content(data);

    Ok(ArtistDetail {
        name,
        thumbnail,
        subscribers: None,
        description: None,
        tracks,
        albums,
    })
}

fn extract_album_two_row(r: &Value) -> Option<Album> {
    let browse_id = r["title"]["runs"][0]["navigationEndpoint"]["browseEndpoint"]["browseId"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if browse_id.is_empty() {
        return None;
    }

    let title = r["title"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let (artist, year) = r["subtitle"]["runs"]
        .as_array()
        .map(|runs| parse_album_subtitle(runs))
        .unwrap_or_default();

    let thumbnail = r["thumbnailRenderer"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
        .as_array()
        .and_then(|t| t.last())
        .and_then(|t| t["url"].as_str().map(|s| s.to_string()))
        .or_else(|| {
            r["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
                .as_array()
                .and_then(|t| t.last())
                .and_then(|t| t["url"].as_str().map(|s| s.to_string()))
        });

    Some(Album {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        artist,
        year,
        thumbnail,
        browse_id,
    })
}

pub async fn get_album(browse_id: &str) -> Result<AlbumDetail> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let body = json!({
        "browseId": browse_id,
        "context": { "client": ytm_context()["client"] }
    });

    let resp = ytm_headers(client.post(YTM_BROWSE_URL))
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YouTube Music browse error: {}", resp.status()));
    }

    let data: Value = resp.json().await?;
    let mut detail = parse_album_detail(&data)?;

    // If the YTM browse didn't find tracks (MPREb_ responses often have no contents),
    // try a VL browse which puts tracks in secondaryContents.
    if detail.tracks.is_empty() {
        let playlist_id = ytm_playlist_id_from_browse(browse_id, &data);
        eprintln!("[Icaria] no tracks from MPREb browse, trying VL browse: {}", playlist_id);
        match ytm_vl_playlist_tracks(&playlist_id).await {
            Ok(tracks) if !tracks.is_empty() => {
                detail.tracks = tracks;
            }
            _ => {
                eprintln!("[Icaria] VL browse empty, trying Invidious: {}", playlist_id);
                if let Ok(tracks) = invidious_playlist_tracks(&playlist_id).await {
                    detail.tracks = tracks;
                }
            }
        }
    }

    Ok(detail)
}

/// Extract the YouTube playlist ID from a browse response or browseId.
/// YTM's microformat.microformatDataRenderer.urlCanonical contains "?list=OLAK5uy_..."
fn ytm_playlist_id_from_browse(browse_id: &str, data: &Value) -> String {
    // Try microformat URL first
    if let Some(url) = data["microformat"]["microformatDataRenderer"]["urlCanonical"].as_str() {
        if let Some((_, after)) = url.split_once("list=") {
            let id = after.split('&').next().unwrap_or("").trim().to_string();
            if !id.is_empty() {
                return id;
            }
        }
    }
    // Fallback: use the browseId directly (works when it's already an OLAK5uy_ or VL... ID)
    browse_id.trim_start_matches("VL").to_string()
}

/// Fetch album tracks from the Invidious playlists API (reliable, stable structure).
async fn invidious_playlist_tracks(playlist_id: &str) -> Result<Vec<Track>> {
    use futures_util::StreamExt;
    use std::sync::Arc;

    let client = Arc::new(Client::builder().timeout(Duration::from_secs(8)).build()?);
    let mut futs = futures_util::stream::FuturesUnordered::new();

    for instance in INVIDIOUS_INSTANCES {
        let client = client.clone();
        let url = format!("https://{}/api/v1/playlists/{}", instance, playlist_id);
        futs.push(async move {
            let resp = client.get(&url).send().await.ok()?;
            if !resp.status().is_success() { return None; }
            let data: Value = resp.json().await.ok()?;
            let videos = data["videos"].as_array()?;
            let tracks: Vec<Track> = videos.iter().filter_map(|v| {
                let vid = v["videoId"].as_str().filter(|s| !s.is_empty())?.to_string();
                let title = v["title"].as_str().unwrap_or("Unknown").to_string();
                let artist = v["author"].as_str().unwrap_or("Unknown").to_string();
                let duration_ms = v["lengthSeconds"].as_u64().map(|s| s * 1000);
                let thumbnail = v["videoThumbnails"].as_array()
                    .and_then(|ts| {
                        ts.iter().find(|t| t["quality"].as_str() == Some("medium"))
                            .or_else(|| ts.last())
                    })
                    .and_then(|t| t["url"].as_str())
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", vid));
                Some(Track {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    artist,
                    album: None,
                    duration_ms,
                    thumbnail: Some(thumbnail),
                    source: TrackSource::YouTube,
                    stream_id: vid,
                })
            }).collect();
            if tracks.is_empty() { None } else { Some(tracks) }
        });
    }

    while let Some(result) = futs.next().await {
        if let Some(tracks) = result {
            return Ok(tracks);
        }
    }
    Err(anyhow!("invidious playlist failed for {}", playlist_id))
}

/// Browse a YTM playlist via VL prefix and extract tracks from secondaryContents.
/// YTM's MPREb_ browse has no tracks in contents; the VL browse puts them in
/// contents.twoColumnBrowseResultsRenderer.secondaryContents which scan_for_tracks normally skips.
async fn ytm_vl_playlist_tracks(playlist_id: &str) -> Result<Vec<Track>> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let body = json!({
        "browseId": format!("VL{}", playlist_id),
        "context": { "client": ytm_context()["client"] }
    });
    let resp = ytm_headers(client.post(YTM_BROWSE_URL)).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("VL browse error: {}", resp.status()));
    }
    let data: Value = resp.json().await?;
    let mut tracks = Vec::new();
    // Scan directly inside secondaryContents (not its parent, so blacklist doesn't trigger)
    let secondary = &data["contents"]["twoColumnBrowseResultsRenderer"]["secondaryContents"];
    scan_for_tracks(secondary, &mut tracks);
    // Also check tabs for alternate response layouts
    if tracks.is_empty() {
        scan_for_tracks(&data["contents"]["twoColumnBrowseResultsRenderer"]["tabs"], &mut tracks);
    }
    // Full contents scan as last resort
    if tracks.is_empty() {
        scan_all_for_tracks(&data["contents"], &mut tracks);
    }
    eprintln!("[Icaria] VL browse {} → {} tracks", playlist_id, tracks.len());
    if tracks.is_empty() {
        Err(anyhow!("no tracks in VL browse for {}", playlist_id))
    } else {
        Ok(tracks)
    }
}

/// Like scan_for_tracks but doesn't skip secondaryContents — used for VL browse fallback.
fn scan_all_for_tracks(node: &Value, tracks: &mut Vec<Track>) {
    match node {
        Value::Object(obj) => {
            for (key, val) in obj {
                match key.as_str() {
                    "playlistVideoRenderer" => {
                        if let Some(t) = extract_playlist_video_track_inner(val) { tracks.push(t); }
                    }
                    "musicResponsiveListItemRenderer" => {
                        if let Some(t) = extract_track_from_renderer(val) { tracks.push(t); }
                    }
                    "relatedChipCloudRenderer" | "continuationItemRenderer" => {}
                    _ => scan_all_for_tracks(val, tracks),
                }
            }
        }
        Value::Array(arr) => {
            for item in arr { scan_all_for_tracks(item, tracks); }
        }
        _ => {}
    }
}

/// Debug: returns the raw YTM browse JSON so we can inspect the actual response structure.
pub async fn get_album_raw(browse_id: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let body = json!({
        "browseId": browse_id,
        "context": { "client": ytm_context()["client"] }
    });

    let resp = ytm_headers(client.post(YTM_BROWSE_URL))
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YouTube Music browse error: {}", resp.status()));
    }

    // Return only the header section to keep response small
    let data: Value = resp.json().await?;
    let header_keys: Vec<String> = data["header"]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    let header_summary = json!({
        "header_keys": header_keys,
        "header": data["header"],
        "contents_keys": data["contents"].as_object().map(|o| o.keys().cloned().collect::<Vec<_>>())
    });
    Ok(serde_json::to_string_pretty(&header_summary)?)
}

fn parse_album_subtitle(runs: &[Value]) -> (String, Option<String>) {
    let texts: Vec<&str> = runs
        .iter()
        .filter_map(|r| r["text"].as_str())
        .filter(|t| {
            let t = t.trim();
            !t.is_empty() && t != "•" && t != "·" && t != " • " && t != " · "
        })
        .collect();
    let yr = texts.last()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .filter(|&y| y > 1900 && y < 2100)
        .map(|y| y.to_string());
    let type_words = ["album", "single", "ep", "álbum", "sencillo", "compilation", "playlist"];
    let artist = texts.iter()
        .find(|&&t| {
            let tl = t.trim().to_lowercase();
            !type_words.contains(&tl.as_str()) && t.trim().parse::<u32>().is_err()
        })
        .map(|s| s.to_string())
        .unwrap_or_default();
    (artist, yr)
}

/// Collects all tracks from an album browse response by recursively scanning the contents
/// for any `playlistVideoRenderer` or `musicResponsiveListItemRenderer` node.
/// This works regardless of nesting depth or YTM response version.
fn collect_album_tracks(data: &Value, tracks: &mut Vec<Track>) {
    scan_for_tracks(&data["contents"], tracks);
}

fn scan_for_tracks(node: &Value, tracks: &mut Vec<Track>) {
    match node {
        Value::Object(obj) => {
            for (key, val) in obj {
                match key.as_str() {
                    "playlistVideoRenderer" => {
                        if let Some(t) = extract_playlist_video_track_inner(val) {
                            tracks.push(t);
                        }
                    }
                    "musicResponsiveListItemRenderer" => {
                        if let Some(t) = extract_track_from_renderer(val) {
                            tracks.push(t);
                        }
                    }
                    // Don't recurse into sidebar, ads, or related sections
                    "secondaryContents" | "relatedChipCloudRenderer" | "continuationItemRenderer" => {}
                    _ => scan_for_tracks(val, tracks),
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                scan_for_tracks(item, tracks);
            }
        }
        _ => {}
    }
}

/// Extracts a track from the `playlistVideoRenderer` object itself.
fn extract_playlist_video_track_inner(r: &Value) -> Option<Track> {
    if r.is_null() {
        return None;
    }
    let vid = r["videoId"].as_str().filter(|s| !s.is_empty())?.to_string();
    let title = r["title"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let artist = r["shortBylineText"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let duration_str = r["lengthText"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("");
    let thumbnail = r["thumbnail"]["thumbnails"]
        .as_array()
        .and_then(|t| t.last())
        .and_then(|t| t["url"].as_str())
        .map(|u| u.to_string())
        .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", vid));

    Some(Track {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        artist,
        album: None,
        duration_ms: parse_duration(duration_str),
        thumbnail: Some(thumbnail),
        source: TrackSource::YouTube,
        stream_id: vid,
    })
}

fn header_extract(h: &Value) -> (String, String, Option<String>, Option<String>, Option<String>) {
    let title = h["title"]["runs"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let (artist, yr) = h["subtitle"]["runs"]
        .as_array()
        .map(|r| parse_album_subtitle(r))
        .unwrap_or_default();

    // Thumbnail — try several known paths
    let thumb = h["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
        .as_array()
        .or_else(|| h["thumbnail"]["croppedSquareImageRenderer"]["thumbnail"]["thumbnails"].as_array())
        .or_else(|| h["foregroundThumbnail"]["thumbnails"].as_array())
        .and_then(|t| t.last())
        .and_then(|t| t["url"].as_str().map(|s| s.to_string()));

    let desc = h["description"]["runs"]
        .as_array()
        .map(|runs| {
            runs.iter()
                .filter_map(|r| r["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        });

    (title, artist, yr, thumb, desc)
}

fn parse_album_detail(data: &Value) -> Result<AlbumDetail> {
    // Try every recognized header type in priority order.
    // If none match, fall back to whatever key is in data["header"].
    const KNOWN_HEADERS: &[&str] = &[
        "musicDetailHeaderRenderer",
        "musicImmersiveHeaderRenderer",
        "musicResponsiveHeaderRenderer",
        "musicImmersiveCarouselHeaderRenderer",
        "musicVisualHeaderRenderer",
        "musicHeaderRenderer",
        "musicEditorialCardRenderer",
    ];

    // Log all top-level keys and header keys for diagnostics
    let top_keys: Vec<String> = data.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
    let all_header_keys: Vec<String> = data["header"].as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
    eprintln!("[Icaria] album top-level keys: {:?}", top_keys);
    eprintln!("[Icaria] album header keys: {:?}", all_header_keys);

    // Which key matched (for debug logging)
    let mut matched_key = "";
    let found = KNOWN_HEADERS.iter().find_map(|key| {
        let v = &data["header"][key];
        if !v.is_null() { matched_key = key; Some(v) } else { None }
    });

    let h = found.or_else(|| {
        data["header"].as_object().and_then(|o| o.values().next())
    });

    let (title, artist, year, thumbnail, description) = match h {
        Some(h) => {
            eprintln!("[Icaria] album header matched: {:?}", matched_key);
            header_extract(h)
        }
        None => (String::new(), String::new(), None, None, None),
    };

    // Fallback: try microformat (used by some album/release pages)
    let title = if title.is_empty() {
        data["microformat"]["microformatDataRenderer"]["title"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("?top:{}", top_keys.join(",")))
    } else { title };

    let thumbnail = thumbnail.or_else(|| {
        data["microformat"]["microformatDataRenderer"]["thumbnail"]["thumbnails"]
            .as_array()
            .and_then(|t| t.last())
            .and_then(|t| t["url"].as_str().map(|s| s.to_string()))
    });

    // Also look for description in a separate shelf (used by some header types)
    let description = description.or_else(|| {
        data["description"]["musicDescriptionShelfRenderer"]["description"]["runs"]
            .as_array()
            .map(|runs| {
                runs.iter().filter_map(|r| r["text"].as_str()).collect::<Vec<_>>().join("")
            })
    });

    let mut tracks = Vec::new();

    collect_album_tracks(data, &mut tracks);

    eprintln!("[Icaria] album tracks found: {}", tracks.len());

    Ok(AlbumDetail {
        title,
        artist,
        year,
        thumbnail,
        description,
        tracks,
    })
}

static STREAM_CACHE: LazyLock<Mutex<HashMap<String, (String, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
const STREAM_TTL: Duration = Duration::from_secs(1500);

const PIPED_INSTANCES: &[&str] = &[
    "pipedapi.kavin.rocks",
    "pipedapi.adminforge.de",
    "piped-api.garudalinux.org",
    "watchapi.whatever.social",
    "api.piped.projectsegfau.lt",
];

// Invidious serves stream URLs with n-sig already decoded — no yt-dlp needed.
const INVIDIOUS_INSTANCES: &[&str] = &[
    "inv.nadeko.net",
    "invidious.protokolla.fi",
    "invidious.nerdvpn.de",
    "yewtu.be",
    "invidious.private.coffee",
    "invidious.fdn.fr",
];

/// Descarta una URL cacheada que resultó estar rota (p. ej. 403 del CDN al
/// reproducirla), para que el próximo intento la vuelva a resolver en vez de
/// repetir la misma URL muerta durante los 25 min de TTL.
pub fn invalidate_stream(video_id: &str) {
    STREAM_CACHE.lock().unwrap().remove(video_id);
}

pub async fn get_stream(video_id: &str) -> Result<StreamUrl> {
    {
        let cache = STREAM_CACHE.lock().unwrap();
        if let Some((url, ts)) = cache.get(video_id) {
            if ts.elapsed() < STREAM_TTL {
                return Ok(StreamUrl { url: url.clone(), mime_type: None });
            }
        }
    }

    // race_stream primero (Invidious/Piped/yt-dlp en paralelo, rápido en escritorio).
    // Si falla (p.ej. Android sin yt-dlp), InnerTube como respaldo.
    let url = match race_stream(video_id).await {
        Ok(u) => u,
        Err(race_err) => {
            eprintln!("[Icaria] race_stream falló: {}", race_err);
            match innertube_stream(video_id).await {
                Ok(u) => u,
                Err(it_err) => return Err(anyhow!("No se pudo obtener el audio. InnerTube: {}", it_err)),
            }
        }
    };

    let mut cache = STREAM_CACHE.lock().unwrap();
    cache.retain(|_, (_, ts)| ts.elapsed() < STREAM_TTL);
    cache.insert(video_id.to_string(), (url.clone(), Instant::now()));

    Ok(StreamUrl { url, mime_type: None })
}

// Clave pública InnerTube de YouTube.
const INNERTUBE_KEY: &str = "AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w";

// Clientes que devuelven URLs de audio directas (sin PO token ni descifrado).
// Se prueban en orden porque distintos clientes funcionan para distintos videos.
// (clientName, clientVersion, X-YouTube-Client-Name, User-Agent, campos extra)
fn innertube_clients() -> Vec<(&'static str, &'static str, &'static str, &'static str, Value)> {
    vec![
        ("IOS", "20.10.4", "5",
         "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X)",
         json!({"deviceMake":"Apple","deviceModel":"iPhone16,2","osName":"iPhone","osVersion":"18.3.2.22D82"})),
        ("ANDROID", "20.10.38", "3",
         "com.google.android.youtube/20.10.38 (Linux; U; Android 14) gzip",
         json!({"androidSdkVersion":34})),
        ("ANDROID_VR", "1.60.19", "28",
         "com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12; GB) gzip",
         json!({"deviceModel":"Quest 3","androidSdkVersion":32})),
    ]
}

/// Extrae la URL de audio vía la API InnerTube de YouTube, probando varios
/// clientes hasta que uno devuelve un stream reproducible. Sin yt-dlp → funciona
/// en Android y escritorio.
async fn innertube_stream(video_id: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?;
    let url = format!("https://www.youtube.com/youtubei/v1/player?key={}", INNERTUBE_KEY);

    let mut last_reason = String::new();

    for (name, ver, cn, ua, extra) in innertube_clients() {
        let mut client_ctx = json!({
            "clientName": name,
            "clientVersion": ver,
            "hl": "en",
            "gl": "US"
        });
        if let (Some(obj), Some(ex)) = (client_ctx.as_object_mut(), extra.as_object()) {
            for (k, v) in ex { obj.insert(k.clone(), v.clone()); }
        }
        let body = json!({ "videoId": video_id, "context": { "client": client_ctx } });

        let resp = match client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", ua)
            .header("X-YouTube-Client-Name", cn)
            .header("X-YouTube-Client-Version", ver)
            .json(&body)
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };

        let data: Value = match resp.json().await { Ok(d) => d, Err(_) => continue };

        let status = data["playabilityStatus"]["status"].as_str().unwrap_or("");
        if status != "OK" {
            last_reason = data["playabilityStatus"]["reason"]
                .as_str().unwrap_or(status).to_string();
            eprintln!("[Icaria] innertube {} → {} ({})", name, status, last_reason);
            continue;
        }

        if let Some(u) = pick_innertube_audio_url(&data) {
            eprintln!("[Icaria] innertube OK vía {}", name);
            return Ok(u);
        }
    }

    Err(anyhow!(
        "innertube: sin stream ({})",
        if last_reason.is_empty() { "todos los clientes fallaron".to_string() } else { last_reason }
    ))
}

fn pick_innertube_audio_url(data: &Value) -> Option<String> {
    let formats = data["streamingData"]["adaptiveFormats"].as_array()?;

    // Preferido: itag 140 (m4a/aac 128 kbps) — ideal para WebKit/WebView
    for f in formats {
        if f["itag"].as_i64() == Some(140) {
            if let Some(u) = f["url"].as_str() {
                if !u.is_empty() { return Some(u.to_string()); }
            }
        }
    }
    // Cualquier audio mp4
    for f in formats {
        if f["mimeType"].as_str().unwrap_or("").starts_with("audio/mp4") {
            if let Some(u) = f["url"].as_str() {
                if !u.is_empty() { return Some(u.to_string()); }
            }
        }
    }
    // Cualquier audio
    for f in formats {
        if f["mimeType"].as_str().unwrap_or("").starts_with("audio/") {
            if let Some(u) = f["url"].as_str() {
                if !u.is_empty() { return Some(u.to_string()); }
            }
        }
    }
    None
}

async fn race_stream(video_id: &str) -> Result<String> {
    use futures_util::future::select_ok;

    let vid1 = video_id.to_string();
    let vid2 = video_id.to_string();
    let vid3 = video_id.to_string();

    let futs: Vec<futures_util::future::BoxFuture<'static, Result<String>>> = vec![
        // Invidious: URLs pre-descifradas, ~300ms si hay instancia viva
        Box::pin(async move { invidious_stream_parallel(&vid1).await }),
        // Piped: también pre-descifradas, pero casi todas caídas
        Box::pin(async move { piped_stream_parallel(&vid2).await }),
        // Respaldo (solo escritorio): yt-dlp (~2s). En Android no existe y falla rápido.
        Box::pin(async move { ytdlp_stream_raw(&vid3).await }),
    ];

    select_ok(futs).await.map(|(url, _)| url)
}

async fn piped_stream_parallel(video_id: &str) -> Result<String> {
    use futures_util::StreamExt;
    use std::sync::Arc;

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?,
    );

    let mut futs = futures_util::stream::FuturesUnordered::new();

    for instance in PIPED_INSTANCES {
        let client = client.clone();
        let url = format!("https://{}/streams/{}", instance, video_id);
        futs.push(async move {
            let resp = client.get(&url).send().await.ok()?;
            if !resp.status().is_success() {
                return None;
            }
            let data: Value = resp.json().await.ok()?;
            pick_audio_url(&data)
        });
    }

    while let Some(result) = futs.next().await {
        if let Some(url) = result {
            return Ok(url);
        }
    }

    Err(anyhow!("All Piped instances failed for video {}", video_id))
}

fn pick_audio_url(data: &Value) -> Option<String> {
    let streams = data["audioStreams"].as_array()?;

    let mut best: Option<&Value> = None;
    let mut best_bitrate: i64 = 0;

    for s in streams {
        let bitrate = s["bitrate"].as_i64().unwrap_or(0);
        let mime = s["mimeType"].as_str().unwrap_or("");
        let url = s["url"].as_str().unwrap_or("");

        if url.is_empty() {
            continue;
        }

        let is_m4a = mime.contains("mp4") || mime.contains("m4a");

        if best.is_none() {
            best = Some(s);
            best_bitrate = bitrate;
            continue;
        }

        let prev_mime = best.unwrap()["mimeType"].as_str().unwrap_or("");
        let prev_m4a = prev_mime.contains("mp4") || prev_mime.contains("m4a");

        let cur_preferred = is_m4a && bitrate <= 130_000;
        let prev_preferred = prev_m4a && best_bitrate <= 130_000;

        if cur_preferred && !prev_preferred {
            best = Some(s);
            best_bitrate = bitrate;
        } else if cur_preferred == prev_preferred && bitrate > best_bitrate {
            best = Some(s);
            best_bitrate = bitrate;
        }
    }

    best.and_then(|s| s["url"].as_str().map(|u| u.to_string()))
}

async fn invidious_stream_parallel(video_id: &str) -> Result<String> {
    use futures_util::StreamExt;
    use std::sync::Arc;

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?,
    );

    let mut futs = futures_util::stream::FuturesUnordered::new();

    for instance in INVIDIOUS_INSTANCES {
        let client = client.clone();
        let url = format!(
            "https://{}/api/v1/videos/{}?fields=adaptiveFormats",
            instance, video_id
        );
        futs.push(async move {
            let resp = client.get(&url).send().await.ok()?;
            if !resp.status().is_success() {
                return None;
            }
            let data: Value = resp.json().await.ok()?;
            pick_invidious_audio_url(&data)
        });
    }

    while let Some(result) = futs.next().await {
        if let Some(url) = result {
            return Ok(url);
        }
    }

    Err(anyhow!("All Invidious instances failed for video {}", video_id))
}

fn pick_invidious_audio_url(data: &Value) -> Option<String> {
    let formats = data["adaptiveFormats"].as_array()?;

    // Prefer itag 140 (m4a/aac 128kbps) — best for WebKitGTK
    for fmt in formats {
        let itag = fmt["itag"].as_str().unwrap_or("");
        if itag == "140" {
            if let Some(url) = fmt["url"].as_str() {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Fallback: any m4a container
    for fmt in formats {
        let container = fmt["container"].as_str().unwrap_or("");
        if container == "m4a" {
            if let Some(url) = fmt["url"].as_str() {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Fallback: any audio stream (Invidious uses "type" key, not "mimeType")
    for fmt in formats {
        let mime = fmt["type"].as_str().unwrap_or("");
        if mime.contains("audio") {
            if let Some(url) = fmt["url"].as_str() {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    None
}

// ── YouTube Music Radio (RDAMVM) ─────────────────────────────────────────────

const YTM_NEXT_URL: &str =
    "https://music.youtube.com/youtubei/v1/next?prettyPrint=false";

/// Fetch a radio mix seeded by `video_id` using the RDAMVM playlist system.
/// Returns songs from similar/related artists, not just the same artist.
/// Falls back to Invidious recommendedVideos if the YTM API yields nothing.
pub async fn get_radio_tracks(video_id: &str, max: u32) -> Result<Vec<Track>> {
    match ytm_radio_tracks(video_id, max).await {
        Ok(tracks) if !tracks.is_empty() => return Ok(tracks),
        _ => {}
    }
    invidious_related(video_id, max).await
}

async fn ytm_radio_tracks(video_id: &str, max: u32) -> Result<Vec<Track>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(12))
        .build()?;

    let playlist_id = format!("RDAMVM{}", video_id);
    let body = json!({
        "videoId": video_id,
        "playlistId": playlist_id,
        "context": ytm_context()
    });

    let resp = ytm_headers(client.post(YTM_NEXT_URL))
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("YTM next API error: {}", resp.status()));
    }

    let data: Value = resp.json().await?;
    let tracks = extract_radio_panel_tracks(&data, video_id, max);

    if tracks.is_empty() {
        return Err(anyhow!("no radio tracks in YTM response"));
    }
    Ok(tracks)
}

fn extract_radio_panel_tracks(data: &Value, current_id: &str, max: u32) -> Vec<Track> {
    // YTM watch-next response nests the playlist under this path
    let panel = &data["contents"]["singleColumnMusicWatchNextResultsRenderer"]
        ["tabbedRenderer"]["watchNextTabbedResultsRenderer"]["tabs"][0]
        ["tabRenderer"]["content"]["musicQueueRenderer"]["content"]
        ["playlistPanelRenderer"];

    let Some(contents) = panel["contents"].as_array() else {
        return Vec::new();
    };

    let mut tracks = Vec::new();
    for item in contents {
        if tracks.len() >= max as usize {
            break;
        }
        let r = &item["playlistPanelVideoRenderer"];
        if r.is_null() {
            continue;
        }
        let vid = match r["videoId"].as_str() {
            Some(v) if !v.is_empty() && v != current_id => v.to_string(),
            _ => continue,
        };
        let title = r["title"]["runs"][0]["text"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let artist = r["longBylineText"]["runs"][0]["text"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let duration_str = r["lengthText"]["runs"][0]["text"]
            .as_str()
            .unwrap_or("");
        let thumbnail = r["thumbnail"]["thumbnails"]
            .as_array()
            .and_then(|ts| ts.last())
            .and_then(|t| t["url"].as_str())
            .map(|u| u.to_string())
            .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", vid));

        tracks.push(Track {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            artist,
            album: None,
            duration_ms: parse_duration(duration_str),
            thumbnail: Some(thumbnail),
            source: TrackSource::YouTube,
            stream_id: vid,
        });
    }
    tracks
}

async fn invidious_related(video_id: &str, max: u32) -> Result<Vec<Track>> {
    use futures_util::StreamExt;
    use std::sync::Arc;

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(8))
            .build()?,
    );

    let mut futs = futures_util::stream::FuturesUnordered::new();

    for instance in INVIDIOUS_INSTANCES {
        let client = client.clone();
        let url = format!(
            "https://{}/api/v1/videos/{}?fields=recommendedVideos",
            instance, video_id
        );
        let vid = video_id.to_string();
        futs.push(async move {
            let resp = client.get(&url).send().await.ok()?;
            if !resp.status().is_success() {
                return None;
            }
            let data: Value = resp.json().await.ok()?;
            let videos = data["recommendedVideos"].as_array()?;
            let mut tracks = Vec::new();
            for v in videos.iter().take(max as usize) {
                let id = v["videoId"].as_str().filter(|s| !s.is_empty() && *s != vid)?;
                let title = v["title"].as_str().unwrap_or("Unknown").to_string();
                let artist = v["author"].as_str().unwrap_or("YouTube").to_string();
                let duration_ms = v["lengthSeconds"].as_u64().map(|s| s * 1000);
                let thumbnail = v["videoThumbnails"]
                    .as_array()
                    .and_then(|ts| {
                        ts.iter()
                            .find(|t| t["quality"].as_str() == Some("medium"))
                            .or_else(|| ts.first())
                    })
                    .and_then(|t| t["url"].as_str())
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id));
                tracks.push(Track {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    artist,
                    album: None,
                    duration_ms,
                    thumbnail: Some(thumbnail),
                    source: TrackSource::YouTube,
                    stream_id: id.to_string(),
                });
            }
            if tracks.is_empty() { None } else { Some(tracks) }
        });
    }

    while let Some(result) = futs.next().await {
        if let Some(tracks) = result {
            return Ok(tracks);
        }
    }

    Err(anyhow!("all Invidious instances failed for radio of {}", video_id))
}

async fn ytdlp_stream_raw(video_id: &str) -> Result<String> {
    let bin = ytdlp_bin();
    let url_arg = format!("https://www.youtube.com/watch?v={}", video_id);

    // Intenta primero con cookies del navegador (evita el bot-check/429 de YouTube).
    // Si no hay navegador o el intento falla, reintenta sin cookies (no rompe equipos
    // sin sesión de YouTube).
    let attempts: Vec<Option<String>> = match cookies_browser() {
        Some(b) => vec![Some(b), None],
        None => vec![None],
    };

    let mut last_err = String::from("yt-dlp returned no URL");
    for cookies in attempts {
        let bin = bin.clone();
        let url_arg = url_arg.clone();
        let out = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(&bin);
            cmd.args([
                "-f", "140/bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio",
                "--get-url",
                "--quiet",
                "--no-warnings",
                "--no-playlist",
                "--no-check-formats",
            ]);
            if let Some(browser) = &cookies {
                cmd.args(["--cookies-from-browser", browser]);
            }
            cmd.arg(&url_arg);
            cmd.output()
        })
        .await??;

        if out.status.success() {
            let url = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !url.is_empty() {
                return Ok(url);
            }
            last_err = "Empty stream URL from yt-dlp".to_string();
        } else {
            last_err = String::from_utf8_lossy(&out.stderr).to_string();
        }
    }

    Err(anyhow!("yt-dlp stream failed: {}", last_err))
}
