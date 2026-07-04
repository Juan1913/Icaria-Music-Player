pub mod bandcamp;
pub mod piped;
pub mod spotify;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub thumbnail: Option<String>,
    pub source: TrackSource,
    pub stream_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackSource {
    YouTube,
    Spotify,
    Bandcamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamUrl {
    pub url: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub thumbnail: Option<String>,
    pub subscribers: Option<String>,
    pub browse_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: Option<String>,
    pub thumbnail: Option<String>,
    pub browse_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub tracks: Vec<Track>,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistDetail {
    pub name: String,
    pub thumbnail: Option<String>,
    pub subscribers: Option<String>,
    pub description: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetail {
    pub title: String,
    pub artist: String,
    pub year: Option<String>,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub tracks: Vec<Track>,
}
