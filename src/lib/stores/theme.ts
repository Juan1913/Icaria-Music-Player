import { writable } from 'svelte/store';

export type Theme = 'paper' | 'dracula' | 'gruvbox' | 'jamaica' | 'palestina';

export const THEMES: Theme[] = ['paper', 'dracula', 'gruvbox', 'jamaica', 'palestina'];

export const THEME_LABELS: Record<Theme, string> = {
  paper:     'Paper',
  dracula:   'Dracula',
  gruvbox:   'Gruvbox',
  jamaica:   'Jamaica',
  palestina: 'Palestina',
};

export const THEME_COLORS: Record<Theme, string> = {
  paper:     '#5e3f7b',
  dracula:   '#bd93f9',
  gruvbox:   '#d79921',
  jamaica:   '#FFD100',
  palestina: '#007A3D',
};

const STORAGE_KEY = 'icaria-theme';
const LEGACY_KEY = 'harmonia-theme';

/** Lee la preferencia, migrando desde la clave antigua si hace falta. */
function loadTheme(): string | null {
  if (typeof localStorage === 'undefined') return null;
  let v = localStorage.getItem(STORAGE_KEY);
  if (!v) {
    const legacy = localStorage.getItem(LEGACY_KEY);
    if (legacy) {
      localStorage.setItem(STORAGE_KEY, legacy);
      localStorage.removeItem(LEGACY_KEY);
      v = legacy;
    }
  }
  return v;
}

function createThemeStore() {
  const stored = loadTheme() as Theme | null;

  const valid: Theme = stored && (THEMES as readonly string[]).includes(stored) ? stored as Theme : 'paper';
  const { subscribe, set, update } = writable<Theme>(valid);

  return {
    subscribe,
    cycle() {
      update(t => {
        const next = THEMES[(THEMES.indexOf(t) + 1) % THEMES.length];
        localStorage.setItem(STORAGE_KEY, next);
        return next;
      });
    },
    set(t: Theme) {
      localStorage.setItem(STORAGE_KEY, t);
      set(t);
    },
  };
}

export const theme = createThemeStore();
