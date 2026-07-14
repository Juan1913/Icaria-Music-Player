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
export async function streamAndPlay(track: Track): Promise<void> {
  player.setTrackLoading(track);
  try {
    const stream = await resolveStream(track);
    player.setStreamUrl(stream.url);
  } catch (e) {
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
