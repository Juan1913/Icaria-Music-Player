import { writable, get } from 'svelte/store';
import type { Track } from './player';

const STORAGE_KEY = 'icaria_favorites';
const LEGACY_KEY = 'harmonia_favorites';

function load(): Track[] {
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

function save(tracks: Track[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tracks));
  } catch {}
}

function createFavoritesStore() {
  const { subscribe, update } = writable<Track[]>(load());

  /** ¿Está la pista en favoritos? (lectura puntual; en componentes usa $favorites) */
  function has(id: string): boolean {
    return get({ subscribe }).some(t => t.id === id);
  }

  function add(track: Track) {
    update(list => {
      if (list.some(t => t.id === track.id)) return list;
      const next = [track, ...list];
      save(next);
      return next;
    });
  }

  function remove(id: string) {
    update(list => {
      const next = list.filter(t => t.id !== id);
      save(next);
      return next;
    });
  }

  return {
    subscribe,
    has,
    add,
    remove,

    /** Alterna el estado de favorito y devuelve el nuevo estado. */
    toggle(track: Track): boolean {
      const nowFav = !has(track.id);
      if (nowFav) add(track);
      else remove(track.id);
      return nowFav;
    },
  };
}

export const favorites = createFavoritesStore();
