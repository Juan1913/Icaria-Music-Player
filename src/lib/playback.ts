import { get, writable } from 'svelte/store';
import { player } from '$lib/stores/player';
import { resolveStream, getRadioTracks } from '$lib/api';
import type { Track } from '$lib/stores/player';

/**
 * Petición de seek pendiente (fracción 0..1). La PlayerBar es dueña del
 * elemento <audio>; cualquier vista puede pedir un salto poniendo aquí una
 * fracción y la PlayerBar la aplica y vuelve a poner el valor en null.
 */
export const seekRequest = writable<number | null>(null);

/** Solicita saltar a una fracción (0..1) de la pista actual. */
export function seek(fraction: number): void {
  seekRequest.set(Math.max(0, Math.min(1, fraction)));
}

/**
 * Resuelve la URL de stream de una pista y arranca la reproducción.
 * Orquestación compartida por la PlayerBar, la vista Now Playing y las listas
 * de pistas, para no duplicar el patrón setTrackLoading → resolveStream → setStreamUrl.
 */
const MAX_RESOLVE_RETRIES = 2;

// Identificador del intento de reproducción vigente. Cada llamada "fresca" a
// streamAndPlay (desde fuera) lo incrementa; solo el intento con el id más
// reciente puede escribir en el store, así un reintento viejo que resuelve
// tarde no pisa un éxito posterior (evita el falso "STREAM ERROR" cuando en
// realidad ya está sonando).
let playRequestId = 0;

export async function streamAndPlay(track: Track, attempt = 0, requestId?: number): Promise<void> {
  const myId = requestId ?? ++playRequestId;
  if (attempt === 0) player.setTrackLoading(track);
  try {
    const stream = await resolveStream(track);
    if (myId !== playRequestId) return; // superado por un intento más nuevo
    player.setStreamUrl(stream.url);
  } catch (e) {
    if (myId !== playRequestId) return;
    // Los fallos de YouTube (LOGIN_REQUIRED, etc.) suelen ser intermitentes:
    // reintentar la misma pista antes de rendirse evita falsos "no disponible".
    if (attempt < MAX_RESOLVE_RETRIES && get(player).currentTrack?.id === track.id) {
      await new Promise(r => setTimeout(r, 700 * (attempt + 1)));
      if (myId !== playRequestId || get(player).currentTrack?.id !== track.id) return;
      return streamAndPlay(track, attempt + 1, myId);
    }
    if (myId !== playRequestId) return;
    player.setError(`ERROR: ${e}`);
  }
}

/**
 * Siguiente pista:
 *  - Álbum/playlist/cola: avanza a la siguiente (respetando repeat).
 *  - Si no hay siguiente (tema suelto o final de la lista): genera una radio de
 *    temas similares y sigue con ella.
 */
export async function playNext(): Promise<void> {
  const s = get(player);
  const hasNext = s.queue.length > 0 &&
    ((s.shuffle && s.queue.length > 1) || s.queueIndex < s.queue.length - 1 || s.repeat === 'all');
  if (hasNext) {
    player.next();
    const track = get(player).queue[get(player).queueIndex];
    if (track) await streamAndPlay(track);
    return;
  }
  await playRadio();
}

/** Crea una radio (mix de temas afines) desde la pista actual y la reproduce. */
export async function playRadio(): Promise<void> {
  const s = get(player);
  if (!s.currentTrack) return;
  try {
    const tracks = await getRadioTracks(s.currentTrack.streamId, 20);
    if (tracks.length > 0) {
      player.setQueue(tracks, 0);
      await streamAndPlay(tracks[0]);
    }
  } catch { /* si falla el radio, no pasa nada */ }
}

/** Retrocede a la pista anterior de la cola y la reproduce. */
export async function playPrev(): Promise<void> {
  const s = get(player);
  if (s.queue.length === 0) return;
  player.prev();
  const track = get(player).queue[get(player).queueIndex];
  if (track) await streamAndPlay(track);
}
