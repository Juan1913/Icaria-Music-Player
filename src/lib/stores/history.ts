import { writable, get } from 'svelte/store';
import type { Track } from './player';

const STORAGE_KEY = 'icaria_history';
const MAX = 60;

function load(): Track[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function save(list: Track[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  } catch {}
}

function createHistoryStore() {
  const { subscribe, update } = writable<Track[]>(load());

  return {
    subscribe,
    get: () => get({ subscribe }),

    /** Registra una pista reproducida (recientes primero, sin duplicados). */
    record(track: Track) {
      update(list => {
        if (list[0]?.id === track.id) return list; // ya es la más reciente
        const next = [track, ...list.filter(t => t.id !== track.id)].slice(0, MAX);
        save(next);
        return next;
      });
    },

    clear() {
      save([]);
      update(() => []);
    },
  };
}

export const history = createHistoryStore();
