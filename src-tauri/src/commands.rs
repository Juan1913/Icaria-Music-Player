use crate::sources::{bandcamp, piped, spotify, AlbumDetail, ArtistDetail, SearchResult, StreamUrl, Track};

#[tauri::command]
pub async fn search_youtube(query: String, max: Option<u32>) -> Result<Vec<Track>, String> {
    piped::search(&query, max.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_youtube_all(query: String, max: Option<u32>) -> Result<SearchResult, String> {
    piped::search_all(&query, max.unwrap_or(6))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_youtube_stream(
    video_id: String,
) -> Result<StreamUrl, String> {
    let stream = piped::get_stream(&video_id)
        .await
        .map_err(|e| e.to_string())?;

    crate::stream_server::register_stream(&video_id, &stream.url);

    Ok(StreamUrl {
        url: format!(
            "http://localhost:{}/stream/{}",
            crate::stream_server::PORT,
            video_id
        ),
        mime_type: Some("audio/mp4".to_string()),
    })
}

#[tauri::command]
pub async fn get_artist(browse_id: String) -> Result<ArtistDetail, String> {
    piped::get_artist(&browse_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_album(browse_id: String) -> Result<AlbumDetail, String> {
    eprintln!("[Icaria] get_album browseId={:?}", browse_id);
    piped::get_album(&browse_id)
        .await
        .map_err(|e| { eprintln!("[Icaria] get_album error: {}", e); e.to_string() })
}

#[tauri::command]
pub async fn search_spotify(
    query: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Track>, String> {
    let creds = state
        .spotify_creds
        .lock()
        .unwrap()
        .clone()
        .ok_or("Spotify credentials not configured")?;

    spotify::search(&query, creds)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_spotify_stream(
    track: Track,
) -> Result<StreamUrl, String> {
    let video_id = spotify::resolve_to_youtube(&track)
        .await
        .map_err(|e| e.to_string())?;

    let stream = piped::get_stream(&video_id)
        .await
        .map_err(|e| e.to_string())?;

    crate::stream_server::register_stream(&video_id, &stream.url);

    Ok(StreamUrl {
        url: format!(
            "http://localhost:{}/stream/{}",
            crate::stream_server::PORT,
            video_id
        ),
        mime_type: Some("audio/mp4".to_string()),
    })
}

#[tauri::command]
pub async fn save_spotify_credentials(
    client_id: String,
    client_secret: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let creds = spotify::SpotifyCredentials {
        client_id,
        client_secret,
    };
    *state.spotify_creds.lock().unwrap() = Some(creds);
    Ok(())
}

#[tauri::command]
pub async fn get_spotify_credentials(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Option<spotify::SpotifyCredentials>, String> {
    Ok(state.spotify_creds.lock().unwrap().clone())
}

#[tauri::command]
pub async fn search_bandcamp(query: String) -> Result<Vec<Track>, String> {
    bandcamp::search(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_bandcamp_stream(track_url: String) -> Result<StreamUrl, String> {
    bandcamp::get_stream(&track_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_album_raw(browse_id: String) -> Result<String, String> {
    piped::get_album_raw(&browse_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_radio(video_id: String, max: Option<u32>) -> Result<Vec<Track>, String> {
    piped::get_radio_tracks(&video_id, max.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}
