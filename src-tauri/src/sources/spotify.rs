use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{Track, TrackSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCredentials {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    tracks: TracksObject,
}

#[derive(Deserialize)]
struct TracksObject {
    items: Vec<SpotifyTrackItem>,
}

#[derive(Deserialize)]
struct SpotifyTrackItem {
    id: String,
    name: String,
    duration_ms: u64,
    artists: Vec<SpotifyArtist>,
    album: SpotifyAlbum,
}

#[derive(Deserialize)]
struct SpotifyArtist {
    name: String,
}

#[derive(Deserialize)]
struct SpotifyAlbum {
    name: String,
    images: Vec<SpotifyImage>,
}

#[derive(Deserialize)]
struct SpotifyImage {
    url: String,
}

async fn get_token(client: &Client, creds: &SpotifyCredentials) -> Result<String> {
    let params = [("grant_type", "client_credentials")];
    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(&creds.client_id, Some(&creds.client_secret))
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Spotify auth failed: {}", resp.status()));
    }

    let token: TokenResponse = resp.json().await?;
    Ok(token.access_token)
}

pub async fn search(query: &str, creds: SpotifyCredentials) -> Result<Vec<Track>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let token = get_token(&client, &creds).await?;

    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=20",
        urlencoding::encode(query)
    );

    let resp = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Spotify search failed: {}", resp.status()));
    }

    let search_resp: SearchResponse = resp.json().await?;

    let tracks = search_resp
        .tracks
        .items
        .into_iter()
        .map(|item| {
            let artist = item
                .artists
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".into());
            let thumbnail = item.album.images.first().map(|i| i.url.clone());
            let search_query = format!("{} {}", artist, item.name);
            Track {
                id: item.id.clone(),
                title: item.name,
                artist,
                album: Some(item.album.name),
                duration_ms: Some(item.duration_ms),
                thumbnail,
                source: TrackSource::Spotify,
                stream_id: search_query, // se usa para buscar en Piped
            }
        })
        .collect();

    Ok(tracks)
}

// Encuentra el video de YouTube que mejor coincide con el track de Spotify
pub async fn resolve_to_youtube(track: &Track) -> Result<String> {
    let results = super::piped::search(&track.stream_id, 5).await?;

    if results.is_empty() {
        return Err(anyhow!("No YouTube match found for: {}", track.title));
    }

    // Si tenemos duración de Spotify, elegimos el resultado más cercano
    if let Some(spotify_ms) = track.duration_ms {
        let best = results
            .iter()
            .filter_map(|r| {
                let yt_ms = r.duration_ms?;
                let diff = (yt_ms as i64 - spotify_ms as i64).unsigned_abs();
                Some((diff, r.stream_id.clone()))
            })
            .min_by_key(|(diff, _)| *diff);

        if let Some((diff, video_id)) = best {
            // Aceptar si la diferencia es menor a 15 segundos
            if diff < 15_000 {
                return Ok(video_id);
            }
        }
    }

    // Fallback: primer resultado
    Ok(results[0].stream_id.clone())
}
