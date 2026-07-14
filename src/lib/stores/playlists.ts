import { writable, get } from 'svelte/store';
import type { Track } from './player';

export interface Playlist {
  id: string;
  name: string;
  tracks: Track[];
  createdAt: number;
}

const STORAGE_KEY = 'icaria_playlists';
const LEGACY_KEY = 'harmonia_playlists';

function load(): Playlist[] {
  try {
    let raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      const legacy = localStorage.getItem(LEGACY_KEY);
      if (legacy) {
        localStorage.setItem(STORAGE_KEY, legacy);
        localStorage.removeItem(LEGACY_KEY);
        raw = legacy;
      }
    }
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function save(playlists: Playlist[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(playlists));
  } catch {}
}

function createPlaylistsStore() {
  const { subscribe, update, set } = writable<Playlist[]>(load());

  return {
    subscribe,

    create(name: string): Playlist {
      const pl: Playlist = {
        id: crypto.randomUUID(),
        name: name.trim(),
        tracks: [],
        createdAt: Date.now(),
      };
      update(list => {
        const next = [...list, pl];
        save(next);
        return next;
      });
      return pl;
    },

    addTrack(playlistId: string, track: Track) {
      update(list => {
        const next = list.map(pl =>
          pl.id === playlistId
            ? { ...pl, tracks: [...pl.tracks, track] }
            : pl
        );
        save(next);
        return next;
      });
    },

    removeTrack(playlistId: string, trackIndex: number) {
      update(list => {
        const next = list.map(pl =>
          pl.id === playlistId
            ? { ...pl, tracks: pl.tracks.filter((_, i) => i !== trackIndex) }
            : pl
        );
        save(next);
        return next;
      });
    },

    rename(playlistId: string, name: string) {
      update(list => {
        const next = list.map(pl =>
          pl.id === playlistId ? { ...pl, name: name.trim() } : pl
        );
        save(next);
        return next;
      });
    },

    delete(playlistId: string) {
      update(list => {
        const next = list.filter(pl => pl.id !== playlistId);
        save(next);
        return next;
      });
    },

    getById(id: string): Playlist | undefined {
      return get({ subscribe }).find(pl => pl.id === id);
    },
  };
}

export const playlists = createPlaylistsStore();
