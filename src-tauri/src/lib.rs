mod commands;
mod sources;
mod stream_server;

use std::sync::Mutex;
use sources::spotify::SpotifyCredentials;

pub struct AppState {
    pub spotify_creds: Mutex<Option<SpotifyCredentials>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::spawn(async move {
        stream_server::start().await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState {
            spotify_creds: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_youtube,
            commands::search_youtube_all,
            commands::get_youtube_stream,
            commands::get_artist,
            commands::get_album,
            commands::search_spotify,
            commands::get_spotify_stream,
            commands::save_spotify_credentials,
            commands::get_spotify_credentials,
            commands::search_bandcamp,
            commands::get_bandcamp_stream,
            commands::get_radio,
            commands::get_album_raw,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harmonia");
}
