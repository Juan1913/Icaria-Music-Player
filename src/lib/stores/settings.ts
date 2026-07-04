import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface SpotifyCredentials {
  clientId: string;
  clientSecret: string;
}

function createSettingsStore() {
  const { subscribe, update } = writable({
    spotifyConfigured: false,
    spotifyClientId: '',
    spotifyClientSecret: '',
  });

  return {
    subscribe,

    async load() {
      try {
        const creds = await invoke<{ client_id: string; client_secret: string } | null>(
          'get_spotify_credentials'
        );
        if (creds) {
          update(s => ({
            ...s,
            spotifyConfigured: true,
            spotifyClientId: creds.client_id,
            spotifyClientSecret: creds.client_secret,
          }));
        }
      } catch {
        // sin credenciales guardadas
      }
    },

    async saveSpotify(clientId: string, clientSecret: string) {
      await invoke('save_spotify_credentials', { clientId, clientSecret });
      update(s => ({
        ...s,
        spotifyConfigured: true,
        spotifyClientId: clientId,
        spotifyClientSecret: clientSecret,
      }));
    },
  };
}

export const settings = createSettingsStore();
