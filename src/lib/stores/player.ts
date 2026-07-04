import { writable, derived, get } from 'svelte/store';
import { preloadStream } from '$lib/api';

export interface Track {
  id: string;
  title: string;
  artist: string;
  album?: string;
  durationMs?: number;
  thumbnail?: string;
  source: 'youtube' | 'spotify' | 'bandcamp';
  streamId: string;
}

export interface Artist {
  id: string;
  name: string;
  thumbnail?: string;
  subscribers?: string;
  browseId: string;
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  year?: string;
  thumbnail?: string;
  browseId: string;
}

export interface SearchResult {
  tracks: Track[];
  artists: Artist[];
  albums: Album[];
}

export interface ArtistDetail {
  name: string;
  thumbnail?: string;
  subscribers?: string;
  description?: string;
  tracks: Track[];
  albums: Album[];
}

export interface AlbumDetail {
  title: string;
  artist: string;
  year?: string;
  thumbnail?: string;
  description?: string;
  tracks: Track[];
}

export interface PlayerState {
  currentTrack: Track | null;
  streamUrl: string | null;
  isPlaying: boolean;
  progress: number;       // 0–1
  currentMs: number;
  volume: number;         // 0–1
  isMuted: boolean;
  shuffle: boolean;
  repeat: 'none' | 'one' | 'all';
  queue: Track[];
  queueIndex: number;
  isLoading: boolean;
  error: string | null;
}

const initial: PlayerState = {
  currentTrack: null,
  streamUrl: null,
  isPlaying: false,
  progress: 0,
  currentMs: 0,
  volume: 0.8,
  isMuted: false,
  shuffle: false,
  repeat: 'none',
  queue: [],
  queueIndex: -1,
  isLoading: false,
  error: null,
};

function createPlayerStore() {
  const { subscribe, set, update } = writable<PlayerState>(initial);

  function prefetchNext(s: PlayerState) {
    const nextIdx = s.queueIndex + 1;
    if (nextIdx < s.queue.length) {
      preloadStream(s.queue[nextIdx]);
    }
    if (s.repeat === 'all' && nextIdx >= s.queue.length && s.queue.length > 0) {
      preloadStream(s.queue[0]);
    }
  }

  return {
    subscribe,

    setTrack(track: Track, url: string) {
      update(s => {
        const next = { ...s, currentTrack: track, streamUrl: url, isPlaying: true, isLoading: true, progress: 0, currentMs: 0, error: null };
        prefetchNext(next);
        return next;
      });
    },

    // Show the track in the player bar immediately, before the stream URL is ready.
    setTrackLoading(track: Track) {
      update(s => ({
        ...s,
        currentTrack: track,
        streamUrl: null,
        isPlaying: true,   // intent to play; PlayerBar handles null URL gracefully
        isLoading: true,
        progress: 0,
        currentMs: 0,
        error: null,
      }));
    },

    // Called when the stream URL is resolved, after setTrackLoading.
    setStreamUrl(url: string) {
      update(s => ({ ...s, streamUrl: url, isLoading: true, isPlaying: true }));
    },

    setPlaying(v: boolean) {
      update(s => ({ ...s, isPlaying: v }));
    },

    setProgress(ms: number, duration: number) {
      update(s => ({ ...s, currentMs: ms, progress: duration > 0 ? ms / duration : 0 }));
    },

    setVolume(v: number) {
      update(s => ({ ...s, volume: Math.max(0, Math.min(1, v)) }));
    },

    toggleMute() {
      update(s => ({ ...s, isMuted: !s.isMuted }));
    },

    toggleShuffle() {
      update(s => ({ ...s, shuffle: !s.shuffle }));
    },

    cycleRepeat() {
      update(s => {
        const next = s.repeat === 'none' ? 'all' : s.repeat === 'all' ? 'one' : 'none';
        return { ...s, repeat: next };
      });
    },

    setQueue(tracks: Track[], index = 0) {
      update(s => {
        const next = { ...s, queue: tracks, queueIndex: index };
        prefetchNext(next);
        return next;
      });
    },

    addToQueue(track: Track) {
      update(s => ({ ...s, queue: [...s.queue, track] }));
    },

    clearQueue() {
      update(s => ({ ...s, queue: [], queueIndex: -1 }));
    },

    setLoading(v: boolean) {
      update(s => ({ ...s, isLoading: v }));
    },

    setError(msg: string | null) {
      update(s => ({ ...s, error: msg, isLoading: false }));
    },

    next() {
      update(s => {
        if (s.queue.length === 0) return s;
        let idx = s.queueIndex + 1;
        if (idx >= s.queue.length) idx = s.repeat === 'all' ? 0 : s.queue.length - 1;
        const next = { ...s, queueIndex: idx };
        prefetchNext(next);
        return next;
      });
    },

    prev() {
      update(s => {
        if (s.queue.length === 0) return s;
        let idx = s.queueIndex - 1;
        if (idx < 0) idx = s.repeat === 'all' ? s.queue.length - 1 : 0;
        const next = { ...s, queueIndex: idx };
        prefetchNext(next);
        return next;
      });
    },
  };
}

export const player = createPlayerStore();

export const currentTrackIndex = derived(player, $p => $p.queueIndex);
