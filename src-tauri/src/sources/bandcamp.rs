use anyhow::{anyhow, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;

use super::{StreamUrl, Track, TrackSource};

pub async fn search(query: &str) -> Result<Vec<Track>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    let url = format!(
        "https://bandcamp.com/search?q={}&item_type=t",
        urlencoding::encode(query)
    );

    let html = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&html);

    let item_sel = Selector::parse(".searchresult").unwrap();
    let heading_sel = Selector::parse(".heading a").unwrap();
    let subhead_sel = Selector::parse(".subhead").unwrap();
    let art_sel = Selector::parse(".art img").unwrap();

    let mut tracks = Vec::new();

    for item in document.select(&item_sel).take(15) {
        let link_el = item.select(&heading_sel).next();
        let title = link_el
            .as_ref()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let track_url = link_el
            .and_then(|el| el.value().attr("href"))
            .map(|h| h.split('?').next().unwrap_or(h).to_string())
            .unwrap_or_default();

        if title.is_empty() || track_url.is_empty() {
            continue;
        }

        let artist = item
            .select(&subhead_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let thumbnail = item
            .select(&art_sel)
            .next()
            .and_then(|el| el.value().attr("src"))
            .map(|s| s.to_string());

        tracks.push(Track {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            artist,
            album: None,
            duration_ms: None,
            thumbnail,
            source: TrackSource::Bandcamp,
            stream_id: track_url,
        });
    }

    Ok(tracks)
}

pub async fn get_stream(track_url: &str) -> Result<StreamUrl> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    let html = client.get(track_url).send().await?.text().await?;
    let document = Html::parse_document(&html);

    // Busca el atributo data-tralbum en la página
    let script_sel = Selector::parse("[data-tralbum]").unwrap();
    if let Some(el) = document.select(&script_sel).next() {
        if let Some(data) = el.value().attr("data-tralbum") {
            let json: Value = serde_json::from_str(data)?;
            if let Some(url) = json
                .get("trackinfo")
                .and_then(|t| t.get(0))
                .and_then(|t| t.get("file"))
                .and_then(|f| f.get("mp3-128"))
                .and_then(|u| u.as_str())
            {
                return Ok(StreamUrl {
                    url: url.to_string(),
                    mime_type: Some("audio/mpeg".into()),
                });
            }
        }
    }

    // Fallback: buscar en scripts inline
    let script_sel2 = Selector::parse("script[type='text/javascript']").unwrap();
    for script in document.select(&script_sel2) {
        let content = script.text().collect::<String>();
        if content.contains("TralbumData") {
            if let Some(start) = content.find("mp3-128") {
                let slice = &content[start..];
                if let Some(url_start) = slice.find("https://") {
                    let url_end = slice[url_start..]
                        .find('"')
                        .unwrap_or(slice.len() - url_start);
                    let url = &slice[url_start..url_start + url_end];
                    return Ok(StreamUrl {
                        url: url.to_string(),
                        mime_type: Some("audio/mpeg".into()),
                    });
                }
            }
        }
    }

    Err(anyhow!("Could not extract stream from Bandcamp URL: {}", track_url))
}
