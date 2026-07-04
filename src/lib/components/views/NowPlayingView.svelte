<script lang="ts">
  import { player } from '$lib/stores/player';
  import { nav } from '$lib/stores/nav';

  let showLyrics = $state(false);
  let lyrics = $state<{ time: number; text: string }[]>([]);
  let lyricsLoading = $state(false);
  let lyricsAvailable = $state<boolean | null>(null);
  let activeLine = $state(-1);
  let lyricsContainer: HTMLElement | undefined = $state();

  function parseLRC(lrc: string): { time: number; text: string }[] {
    return lrc.split('\n')
      .flatMap(line => {
        const m = line.match(/^\[(\d+):(\d+\.\d+)\]\s?(.*)$/);
        if (!m) return [];
        return [{ time: +m[1] * 60 + parseFloat(m[2]), text: m[3] }];
      });
  }

  async function loadLyrics() {
    const track = $player.currentTrack;
    if (!track) return;
    lyricsLoading = true;
    lyricsAvailable = null;
    lyrics = [];
    try {
      const p = new URLSearchParams({ artist_name: track.artist, track_name: track.title });
      if (track.durationMs) p.set('duration', String(Math.floor(track.durationMs / 1000)));
      const res = await fetch('https://lrclib.net/api/get?' + p.toString());
      if (res.ok) {
        const data = await res.json();
        if (data.syncedLyrics) {
          lyrics = parseLRC(data.syncedLyrics).filter(l => l.text.trim() !== '');
          lyricsAvailable = lyrics.length > 0;
        } else if (data.plainLyrics) {
          lyrics = data.plainLyrics
            .split('\n')
            .filter((t: string) => t.trim())
            .map((text: string, i: number) => ({ time: -(i + 1), text }));
          lyricsAvailable = true;
        } else {
          lyricsAvailable = false;
        }
      } else {
        lyricsAvailable = false;
      }
    } catch {
      lyricsAvailable = false;
    } finally {
      lyricsLoading = false;
    }
  }

  function hqThumb(url: string | undefined): string | undefined {
    if (!url) return url;
    return url
      .replace(/mqdefault\.jpg/, 'maxresdefault.jpg')
      .replace(/sddefault\.jpg/, 'maxresdefault.jpg')
      .replace(/hqdefault\.jpg/, 'maxresdefault.jpg');
  }

  let prevTrackId = $state<string | null>(null);
  $effect(() => {
    const id = $player.currentTrack?.id ?? null;
    if (id !== prevTrackId) {
      prevTrackId = id;
      lyrics = [];
      lyricsAvailable = null;
      activeLine = -1;
      if (showLyrics && id) loadLyrics();
    }
  });

  $effect(() => {
    if (!lyrics.length || lyrics[0].time < 0) return; // plain lyrics, no sync
    const sec = $player.currentMs / 1000;
    let idx = -1;
    for (let i = 0; i < lyrics.length; i++) {
      if (lyrics[i].time <= sec) idx = i;
      else break;
    }
    if (idx !== activeLine) {
      activeLine = idx;
      const el = lyricsContainer?.querySelector<HTMLElement>('[data-active="true"]');
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  });

  function toggleLyrics() {
    showLyrics = !showLyrics;
    if (showLyrics && lyricsAvailable === null && $player.currentTrack) {
      loadLyrics();
    }
  }
</script>

<div class="now-playing">
  <!-- Fondo borroso con la portada -->
  {#if $player.currentTrack?.thumbnail}
    <div class="bg-art" style="background-image:url({hqThumb($player.currentTrack.thumbnail)})"></div>
  {/if}
  <div class="bg-overlay"></div>
  <div class="bg-dots"></div>

  <div class="np-layout" class:with-lyrics={showLyrics}>
    <!-- Panel izquierdo: arte + info -->
    <div class="art-panel">
      <div class="top-bar">
        <button class="back-btn" onclick={() => nav.goBackFromNowPlaying()}>
          ← volver
        </button>
        <button
          class="lyrics-toggle {showLyrics ? 'active' : ''}"
          onclick={toggleLyrics}
          title="Letra"
        >
          LETRA {lyricsLoading ? '···' : ''}
        </button>
      </div>

      <div class="art-wrap">
        {#if $player.currentTrack?.thumbnail}
          <img
            class="art"
            src={hqThumb($player.currentTrack.thumbnail)}
            alt=""
            onerror={(e) => { (e.target as HTMLImageElement).src = $player.currentTrack!.thumbnail!; }}
          />
        {:else}
          <div class="art art-empty">
            <div class="empty-dots"></div>
            <span class="empty-g">G</span>
          </div>
        {/if}
      </div>

      <div class="track-info">
        <p class="track-title">{$player.currentTrack?.title ?? '─── sin canción ───'}</p>
        <p class="track-artist">{$player.currentTrack?.artist ?? ''}</p>
        {#if $player.currentTrack?.album}
          <p class="track-album">{$player.currentTrack.album}</p>
        {/if}
      </div>
    </div>

    <!-- Panel derecho: letra -->
    {#if showLyrics}
      <div class="lyrics-panel" bind:this={lyricsContainer}>
        <p class="lyrics-title">LETRA</p>
        {#if lyricsLoading}
          <p class="lyrics-status">Buscando letra···</p>
        {:else if lyricsAvailable === false}
          <p class="lyrics-status">Letra no disponible para esta canción.</p>
        {:else}
          <div class="lyrics-lines">
            {#each lyrics as line, i}
              {#if line.text}
                <p
                  class="lyric-line"
                  class:active={i === activeLine}
                  class:past={i < activeLine}
                  data-active={i === activeLine}
                >{line.text}</p>
              {:else}
                <p class="lyric-break"></p>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .now-playing {
    position: relative;
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Fondo borroso */
  .bg-art {
    position: absolute; inset: 0;
    background-size: cover; background-position: center;
    filter: blur(60px) saturate(1.4) brightness(0.4);
    transform: scale(1.1);
    z-index: 0;
  }
  .bg-overlay {
    position: absolute; inset: 0;
    background: linear-gradient(180deg, rgba(0,0,0,0.55) 0%, rgba(0,0,0,0.3) 50%, rgba(0,0,0,0.7) 100%);
    z-index: 1;
  }
  .bg-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(255,255,255,0.04) 1.5px, transparent 1.5px);
    background-size: 22px 22px;
    z-index: 2;
    pointer-events: none;
  }

  /* Layout */
  .np-layout {
    position: relative; z-index: 3;
    display: flex;
    height: 100%;
    gap: 0;
  }
  .np-layout.with-lyrics { gap: 0; }

  /* Panel arte */
  .art-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1.5rem 2rem 1.5rem;
    min-width: 0;
  }

  .top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    margin-bottom: 1.5rem;
  }
  .back-btn {
    background: rgba(255,255,255,0.12);
    border: 2px solid rgba(255,255,255,0.25);
    color: #ffffff;
    font-size: 0.78rem; font-weight: 800; font-family: inherit;
    letter-spacing: 0.06em;
    padding: 0.45rem 1rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s;
    text-transform: uppercase;
  }
  .back-btn:hover { background: rgba(255,255,255,0.22); border-color: rgba(255,255,255,0.5); }

  .lyrics-toggle {
    background: rgba(255,255,255,0.1);
    border: 2px solid rgba(255,255,255,0.2);
    color: rgba(255,255,255,0.7);
    font-size: 0.72rem; font-weight: 800; font-family: inherit;
    letter-spacing: 0.1em;
    padding: 0.4rem 0.9rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.1s;
  }
  .lyrics-toggle:hover { background: rgba(255,255,255,0.2); color: #ffffff; }
  .lyrics-toggle.active {
    background: var(--accent);
    border-color: var(--accent);
    color: #0d0d0d;
    box-shadow: 3px 3px 0 rgba(0,0,0,0.4);
  }

  /* Álbum art */
  .art-wrap {
    margin: 0 auto;
    width: min(280px, 70vw);
    flex-shrink: 0;
  }
  .art {
    width: 100%; aspect-ratio: 1;
    object-fit: cover;
    border-radius: 12px;
    border: 3px solid rgba(255,255,255,0.15);
    box-shadow: 6px 6px 0 rgba(0,0,0,0.6), 0 20px 60px rgba(0,0,0,0.5);
    display: block;
  }
  .art-empty {
    width: 100%; aspect-ratio: 1;
    position: relative; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent);
    border-radius: 12px;
    border: 3px solid rgba(255,255,255,0.15);
    box-shadow: 6px 6px 0 rgba(0,0,0,0.6);
  }
  .empty-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(255,255,255,0.25) 2px, transparent 2px);
    background-size: 16px 16px;
  }
  .empty-g {
    font-size: 6rem; font-weight: 900; color: #ffffff;
    position: relative; z-index: 1; letter-spacing: -0.04em;
    text-shadow: 4px 4px 0 rgba(0,0,0,0.3);
  }

  /* Track info */
  .track-info {
    text-align: center;
    margin-top: 1.75rem;
    max-width: 320px;
  }
  .track-title {
    font-size: 1.35rem; font-weight: 900; color: #ffffff;
    letter-spacing: -0.02em; line-height: 1.2;
    text-shadow: 0 2px 8px rgba(0,0,0,0.5);
    margin-bottom: 0.4rem;
  }
  .track-artist {
    font-size: 1rem; color: var(--accent);
    font-weight: 700; margin-bottom: 0.2rem;
  }
  .track-album {
    font-size: 0.8rem; color: rgba(255,255,255,0.5);
    font-weight: 500;
  }

  /* Panel letra */
  .lyrics-panel {
    width: 320px;
    min-width: 280px;
    background: rgba(0,0,0,0.55);
    border-left: 2px solid rgba(255,255,255,0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    backdrop-filter: blur(20px);
  }
  .lyrics-title {
    font-size: 0.65rem; font-weight: 900; letter-spacing: 0.15em;
    color: rgba(255,255,255,0.4); text-transform: uppercase;
    padding: 1.5rem 1.5rem 0.75rem; flex-shrink: 0;
  }
  .lyrics-status {
    color: rgba(255,255,255,0.4); font-size: 0.85rem;
    padding: 0 1.5rem; line-height: 1.6;
  }
  .lyrics-lines {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 1.5rem 3rem;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .lyrics-lines::-webkit-scrollbar { width: 3px; }
  .lyrics-lines::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 2px; }

  .lyric-line {
    font-size: 1rem; font-weight: 600; color: rgba(255,255,255,0.3);
    line-height: 1.7; cursor: default;
    transition: color 0.3s, font-size 0.3s, font-weight 0.3s;
    padding: 0.1rem 0;
  }
  .lyric-line.past { color: rgba(255,255,255,0.25); }
  .lyric-line.active {
    color: #ffffff;
    font-size: 1.1rem; font-weight: 800;
  }
  .lyric-break { height: 1rem; }
</style>
