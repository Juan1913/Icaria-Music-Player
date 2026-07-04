import { writable } from 'svelte/store';

export type Theme = 'paper' | 'dracula' | 'gruvbox' | 'jamaica';

const THEMES: Theme[] = ['paper', 'dracula', 'gruvbox', 'jamaica'];

export const THEME_LABELS: Record<Theme, string> = {
  paper:   'Paper',
  dracula: 'Dracula',
  gruvbox: 'Gruvbox',
  jamaica: 'Jamaica',
};

export const THEME_COLORS: Record<Theme, string> = {
  paper:   '#5e3f7b',
  dracula: '#bd93f9',
  gruvbox: '#d79921',
  jamaica: '#FFD100',
};

function createThemeStore() {
  const stored = (typeof localStorage !== 'undefined'
    ? localStorage.getItem('harmonia-theme')
    : null) as Theme | null;

  const valid: Theme = stored && (THEMES as readonly string[]).includes(stored) ? stored as Theme : 'paper';
  const { subscribe, set, update } = writable<Theme>(valid);

  return {
    subscribe,
    cycle() {
      update(t => {
        const next = THEMES[(THEMES.indexOf(t) + 1) % THEMES.length];
        localStorage.setItem('harmonia-theme', next);
        return next;
      });
    },
    set(t: Theme) {
      localStorage.setItem('harmonia-theme', t);
      set(t);
    },
  };
}

export const theme = createThemeStore();
