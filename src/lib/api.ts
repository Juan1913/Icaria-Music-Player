import { invoke } from '@tauri-apps/api/core';
import type { Track, Artist, Album, SearchResult, ArtistDetail, AlbumDetail } from './stores/player';

export interface StreamUrl {
  url: string;
  mimeType?: string;
}

const cache = new Map<string, Promise<StreamUrl>>();
const MAX_CACHE = 60;
const TTL = 25 * 60 * 1000;
const timestamps = new Map<string, number>();

function cacheKey(track: Track): string {
  return `${track.source}:${track.streamId}`;
}

function evictExpired() {
  const now = Date.now();
  for (const [k, ts] of timestamps) {
    if (now - ts > TTL) {
      cache.delete(k);
      timestamps.delete(k);
    }
  }
  if (cache.size > MAX_CACHE) {
    let oldest = '';
    let oldestTs = Infinity;
    for (const [k, ts] of timestamps) {
      if (ts < oldestTs) { oldestTs = ts; oldest = k; }
    }
    cache.delete(oldest);
    timestamps.delete(oldest);
  }
}

export async function searchYouTube(query: string, max = 20): Promise<Track[]> {
  const results = await invoke<any[]>('search_youtube', { query, max });
  return results.map(normalizeTrack);
}

export async function searchYouTubeAll(query: string, max = 6): Promise<SearchResult> {
  const raw = await invoke<any>('search_youtube_all', { query, max });
  return {
    tracks: (raw.tracks ?? []).map(normalizeTrack),
    artists: (raw.artists ?? []).map(normalizeArtist),
    albums: (raw.albums ?? []).map(normalizeAlbum),
  };
}

export async function searchSpotify(query: string): Promise<Track[]> {
  const results = await invoke<any[]>('search_spotify', { query });
  return results.map(normalizeTrack);
}

export async function searchBandcamp(query: string): Promise<Track[]> {
  const results = await invoke<any[]>('search_bandcamp', { query });
  return results.map(normalizeTrack);
}

export async function getYouTubeStream(videoId: string): Promise<StreamUrl> {
  return invoke<StreamUrl>('get_youtube_stream', { videoId });
}

export async function getSpotifyStream(track: Track): Promise<StreamUrl> {
  return invoke<StreamUrl>('get_spotify_stream', { track: denormalizeTrack(track) });
}

export async function getBandcampStream(trackUrl: string): Promise<StreamUrl> {
  return invoke<StreamUrl>('get_bandcamp_stream', { trackUrl });
}

export async function getRadioTracks(videoId: string, max = 20): Promise<Track[]> {
  const results = await invoke<any[]>('get_radio', { videoId, max });
  return results.map(normalizeTrack);
}

export async function getArtist(browseId: string): Promise<ArtistDetail> {
  const raw = await invoke<any>('get_artist', { browseId });
  return {
    ...raw,
    tracks: (raw.tracks ?? []).map(normalizeTrack),
    albums: (raw.albums ?? []).map(normalizeAlbum),
  };
}

export async function getAlbum(browseId: string): Promise<AlbumDetail> {
  const raw = await invoke<any>('get_album', { browseId });
  return {
    ...raw,
    tracks: (raw.tracks ?? []).map(normalizeTrack),
  };
}

function normalizeTrack(raw: any): Track {
  return {
    id: raw.id,
    title: raw.title,
    artist: raw.artist,
    album: raw.album ?? undefined,
    durationMs: raw.durationMs ?? undefined,
    thumbnail: raw.thumbnail ?? undefined,
    source: raw.source,
    streamId: raw.streamId,
  };
}

function normalizeArtist(raw: any): Artist {
  return {
    id: raw.id,
    name: raw.name,
    thumbnail: raw.thumbnail ?? undefined,
    subscribers: raw.subscribers ?? undefined,
    browseId: raw.browseId,
  };
}

function normalizeAlbum(raw: any): Album {
  return {
    id: raw.id,
    title: raw.title,
    artist: raw.artist,
    year: raw.year ?? undefined,
    thumbnail: raw.thumbnail ?? undefined,
    browseId: raw.browseId,
  };
}

function denormalizeTrack(t: Track): any {
  return {
    id: t.id,
    title: t.title,
    artist: t.artist,
    album: t.album ?? null,
    durationMs: t.durationMs ?? null,
    thumbnail: t.thumbnail ?? null,
    source: t.source,
    streamId: t.streamId,
  };
}

async function fetchStream(track: Track): Promise<StreamUrl> {
  switch (track.source) {
    case 'youtube':
      return getYouTubeStream(track.streamId);
    case 'spotify':
      return getSpotifyStream(track);
    case 'bandcamp':
      return getBandcampStream(track.streamId);
  }
}

function storeInCache(key: string, promise: Promise<StreamUrl>) {
  cache.set(key, promise);
  timestamps.set(key, Date.now());
  promise.catch(() => {
    cache.delete(key);
    timestamps.delete(key);
  });
}

export async function resolveStream(track: Track): Promise<StreamUrl> {
  const key = cacheKey(track);
  const cached = cache.get(key);
  if (cached) {
    timestamps.set(key, Date.now());
    return cached;
  }
  evictExpired();
  const promise = fetchStream(track);
  storeInCache(key, promise);
  return promise;
}

export function preloadStream(track: Track): void {
  const key = cacheKey(track);
  if (cache.has(key)) return;
  const promise = fetchStream(track);
  storeInCache(key, promise);
}

export function hasPreloaded(track: Track): boolean {
  return cache.has(cacheKey(track));
}

export function clearStreamCache(): void {
  cache.clear();
  timestamps.clear();
}
