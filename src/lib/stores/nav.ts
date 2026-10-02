import { writable, get } from 'svelte/store';
import type { Artist, Album } from './player';

export type NavPage =
  | 'home'
  | 'search'
  | 'library'
  | 'favorites'
  | 'history'
  | 'settings'
  | 'artist'
  | 'album'
  | 'nowplaying';

interface NavState {
  page: NavPage;
  prevPage: NavPage;
  artistBrowseId: string | null;
  albumBrowseId: string | null;
  searchQuery: string;
}

const initial: NavState = {
  page: 'home',
  prevPage: 'home',
  artistBrowseId: null,
  albumBrowseId: null,
  searchQuery: '',
};

function createNavStore() {
  const { subscribe, set, update } = writable<NavState>(initial);

  return {
    subscribe,
    get: () => get({ subscribe }),
    setPage(page: NavPage) {
      update(s => ({ ...s, prevPage: s.page, page }));
    },
    goBackFromNowPlaying() {
      update(s => ({ ...s, page: s.prevPage !== 'nowplaying' ? s.prevPage : 'home' }));
    },
    openArtist(browseId: string) {
      update(s => ({ ...s, page: 'artist', artistBrowseId: browseId, albumBrowseId: null }));
    },
    openAlbum(browseId: string) {
      update(s => ({ ...s, page: 'album', albumBrowseId: browseId, artistBrowseId: null }));
    },
    goBack() {
      update(s => ({ ...s, page: 'search', artistBrowseId: null, albumBrowseId: null }));
    },
    openSearch(query = '') {
      update(s => ({ ...s, prevPage: s.page, page: 'search', searchQuery: query }));
    },
    clearSearchQuery() {
      update(s => ({ ...s, searchQuery: '' }));
    },
  };
}

export const nav = createNavStore();
