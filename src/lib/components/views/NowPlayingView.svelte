<script lang="ts">
  import { player } from '$lib/stores/player';
  import { nav } from '$lib/stores/nav';
  import { favorites } from '$lib/stores/favorites';
  import { seek, playNext, playPrev } from '$lib/playback';
  import PlaylistModal from '$lib/components/PlaylistModal.svelte';

  function togglePlay() { player.setPlaying(!$player.isPlaying); }
  function fmtTime(ms: number) {
    const s = Math.max(0, Math.floor(ms / 1000));
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  }

  // Scroll suave propio (con easing) para la letra: más agradable que el
  // "smooth" nativo del WebView, que puede sentirse brusco o poco fluido.
  let scrollAnimFrame = 0;
  function smoothScrollTo(el: HTMLElement, target: number, duration = 700) {
    cancelAnimationFrame(scrollAnimFrame);
    const start = el.scrollTop;
    const delta = target - start;
    if (Math.abs(delta) < 1) return;
    const startTime = performance.now();
    const easeInOutQuad = (t: number) => (t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2);
    function step(now: number) {
      const t = Math.min(1, (now - startTime) / duration);
      el.scrollTop = start + delta * easeInOutQuad(t);
      if (t < 1) scrollAnimFrame = requestAnimationFrame(step);
    }
    scrollAnimFrame = requestAnimationFrame(step);
  }

  let showLyrics = $state(false);
  let lyrics = $state<{ time: number; text: string }[]>([]);
  let lyricsLoading = $state(false);
  let lyricsAvailable = $state<boolean | null>(null);
  let activeLine = $state(-1);
  let lyricsContainer: HTMLElement | undefined = $state();

  const isFav = $derived(
    !!$player.currentTrack && $favorites.some(t => t.id === $player.currentTrack!.id)
  );
  let showPlaylistModal = $state(false);

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
      const container = lyricsContainer?.querySelector<HTMLElement>('.lyrics-lines');
      const el = container?.querySelector<HTMLElement>('[data-active="true"]');
      if (container && el) {
        const target = el.offsetTop - container.clientHeight / 2 + el.clientHeight / 2;
        smoothScrollTo(container, target);
      }
    }
  });

  function toggleLyrics() {
    showLyrics = !showLyrics;
    if (showLyrics && lyricsAvailable === null && $player.currentTrack) {
      loadLyrics();
    }
  }

  // El waveform hace de línea de tiempo: click para saltar a esa posición.
  function seekFromWave(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    seek((e.clientX - rect.left) / rect.width);
  }

  // ── Waveform decorativo (barras deterministas por pista) ──
  const BAR_COUNT = 64;
  function hashStr(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
    return h;
  }
  const bars = $derived.by(() => {
    const seed = hashStr($player.currentTrack?.id ?? 'icaria') * 0.001;
    return Array.from({ length: BAR_COUNT }, (_, i) => {
      const v = (Math.sin(seed + i * 0.6) + Math.sin(seed * 1.7 + i * 0.21)) / 2;
      return 0.22 + (v * 0.5 + 0.5) * 0.78; // 0.22 .. 1.0
    });
  });
  // Índice de la barra alcanzada por el progreso (para colorear "reproducido").
  const playedBar = $derived(Math.round(($player.progress || 0) * BAR_COUNT));

  const totalMs = $derived(
    $player.currentTrack?.durationMs ??
    ($player.progress > 0 ? $player.currentMs / $player.progress : 0)
  );
  const remainingMs = $derived(Math.max(0, totalMs - $player.currentMs));
</script>

<div class="now-playing">
  <!-- Halo suave con la portada, teñido con el fondo del tema -->
  {#if $player.currentTrack?.thumbnail}
    <div class="bg-art" style="background-image:url({hqThumb($player.currentTrack.thumbnail)})"></div>
  {/if}
  <div class="bg-overlay"></div>

  <div class="np-layout" class:with-lyrics={showLyrics}>
    <div class="np-inner">
      <!-- Barra superior -->
      <div class="top-bar">
        <button class="round-btn" onclick={() => nav.goBackFromNowPlaying()} title="Volver" aria-label="Volver">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
        </button>
        <div class="top-actions">
          <button
            class="autoplay-switch"
            class:on={$player.autoplay}
            onclick={() => player.toggleAutoplay()}
            title={$player.autoplay ? 'Autoplay activado: al acabar la cola sigue música relacionada' : 'Autoplay desactivado: se detiene al acabar la cola'}
            role="switch"
            aria-checked={$player.autoplay}
            aria-label="Autoplay"
          >
            <span class="sw-label">AUTOMÁTICO</span>
            <span class="sw-track"><span class="sw-knob"></span></span>
          </button>
          <button class="lyrics-toggle" class:active={showLyrics} onclick={toggleLyrics}>
            LETRA {lyricsLoading ? '···' : ''}
          </button>
        </div>
      </div>

      <div class="np-main">
        <!-- Portada -->
        <div class="art-col">
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
                <span class="empty-g">I</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Info / letra -->
        <div class="info-col">
          {#if showLyrics}
            <div class="lyrics-panel" bind:this={lyricsContainer}>
              {#if lyricsLoading}
                <p class="lyrics-status">Buscando letra···</p>
              {:else if lyricsAvailable === false}
                <p class="lyrics-status">Letra no disponible para esta canción.</p>
              {:else}
                <div class="lyrics-lines">
                  {#each lyrics as line, i}
                    {#if line.text}
                      <p class="lyric-line" data-active={i === activeLine}>{line.text}</p>
                    {:else}
                      <p class="lyric-break"></p>
                    {/if}
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <div class="badge">
              EN REPRODUCCIÓN
              <span class="badge-eq" class:playing={$player.isPlaying}>
                <i></i><i></i><i></i>
              </span>
            </div>

            <h1 class="track-title">{$player.currentTrack?.title ?? '─── sin canción ───'}</h1>
            <p class="track-artist">{$player.currentTrack?.artist ?? ''}</p>
            {#if $player.currentTrack?.album}
              <p class="track-album">{$player.currentTrack.album}</p>
            {/if}

            <div class="actions">
              <button
                class="action-btn primary"
                class:liked={isFav}
                onclick={() => { if ($player.currentTrack) favorites.toggle($player.currentTrack); }}
                title={isFav ? 'Quitar de favoritos' : 'Añadir a favoritos'}
                aria-label="Favorito" aria-pressed={isFav}
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill={isFav ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
                </svg>
              </button>
              <button
                class="action-btn"
                onclick={() => { if ($player.currentTrack) showPlaylistModal = true; }}
                title="Añadir a playlist" aria-label="Añadir a playlist"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                  <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
              </button>
            </div>

            <!-- Waveform: también hace de línea de tiempo (click para saltar) -->
            <div
              class="wave"
              class:playing={$player.isPlaying}
              role="slider"
              tabindex="0"
              aria-label="Progreso"
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={Math.round(($player.progress || 0) * 100)}
              onclick={seekFromWave}
              onkeydown={(e) => {
                if (e.key === 'ArrowRight') seek(($player.progress || 0) + 0.02);
                else if (e.key === 'ArrowLeft') seek(($player.progress || 0) - 0.02);
              }}
            >
              {#each bars as h, i}
                <span
                  class="wave-bar"
                  class:played={i < playedBar}
                  style="--h:{h}; --d:{(i % 12) * 0.07}s"
                ></span>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- Controles (solo móvil; en escritorio están en la PlayerBar) -->
      {#if !showLyrics}
        <div class="np-controls">
          <div class="np-times">
            <span class="time">{fmtTime($player.currentMs)}</span>
            <span class="time">-{fmtTime(remainingMs)}</span>
          </div>
          <div class="controls-row">
            <button class="ctrl-icon" class:active={$player.shuffle} onclick={() => player.toggleShuffle()} aria-label="Aleatorio">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/></svg>
            </button>
            <button class="ctrl-icon" onclick={playPrev} aria-label="Anterior">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
            </button>
            <button class="play-btn" onclick={togglePlay} aria-label={$player.isPlaying ? 'Pausar' : 'Reproducir'}>
              {#if $player.isLoading}
                <span class="loading-pulse">···</span>
              {:else if $player.isPlaying}
                <svg width="26" height="26" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>
              {:else}
                <svg width="26" height="26" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
              {/if}
            </button>
            <button class="ctrl-icon" onclick={playNext} aria-label="Siguiente">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
            </button>
            <button class="ctrl-icon" class:active={$player.repeat !== 'none'} onclick={() => player.cycleRepeat()} aria-label="Repetir">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>{#if $player.repeat === 'one'}<line x1="12" y1="9" x2="12" y2="15"/><line x1="9" y1="12" x2="15" y2="12"/>{/if}</svg>
            </button>
          </div>
        </div>
      {/if}

    </div>
  </div>
</div>

{#if showPlaylistModal && $player.currentTrack}
  <PlaylistModal track={$player.currentTrack} onClose={() => (showPlaylistModal = false)} />
{/if}

<style>
  .now-playing {
    position: relative;
    height: 100%;
    overflow: hidden;
    background: var(--bg-primary);
    background-image: radial-gradient(circle, var(--dot-color, rgba(0,0,0,0.05)) 1.5px, transparent 1.5px);
    background-size: 22px 22px;
  }

  /* Halo con la portada, muy tenue, fundido con el fondo del tema */
  .bg-art {
    position: absolute; inset: -10%;
    background-size: cover; background-position: center;
    filter: blur(90px) saturate(1.5);
    opacity: 0.28;
    z-index: 0;
  }
  .bg-overlay {
    position: absolute; inset: 0;
    background:
      radial-gradient(ellipse at center, transparent 0%, var(--bg-primary) 78%);
    z-index: 1;
  }

  .np-layout {
    position: relative; z-index: 2;
    height: 100%;
    display: flex;
    justify-content: center;
    padding: 1.25rem clamp(1rem, 4vw, 3rem) 1.5rem;
    overflow-y: auto;
  }
  .np-inner {
    width: 100%;
    max-width: 960px;
    display: flex;
    flex-direction: column;
  }

  /* ── Barra superior ── */
  .top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
    flex-shrink: 0;
  }
  .round-btn {
    width: 44px; height: 44px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card);
    color: var(--text-primary);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    transition: box-shadow 0.12s, transform 0.1s;
  }
  .round-btn:hover { box-shadow: var(--shadow); transform: translate(-2px, -2px); }
  .round-btn:active { transform: translate(0, 0); box-shadow: var(--shadow-sm); }

  .top-actions { display: flex; align-items: center; gap: 0.6rem; }

  /* Autoplay: interruptor on/off neo-brutalista */
  .autoplay-switch {
    display: inline-flex; align-items: center; gap: 0.55rem;
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius-pill);
    box-shadow: var(--shadow-sm);
    padding: 0.4rem 0.75rem;
    color: var(--text-muted);
    font-size: 0.7rem; font-weight: 900; letter-spacing: 0.1em; font-family: inherit;
    cursor: pointer;
    transition: box-shadow 0.1s, transform 0.1s, color 0.1s;
  }
  .autoplay-switch:hover { box-shadow: var(--shadow); transform: translate(-1px, -1px); }
  .autoplay-switch:active { transform: translate(0, 0); box-shadow: var(--shadow-sm); }
  .autoplay-switch.on { color: var(--text-primary); }

  .sw-track {
    position: relative; width: 34px; height: 18px; flex-shrink: 0;
    background: var(--bg-card-hover);
    border: 2px solid var(--stroke);
    border-radius: var(--radius-pill);
    transition: background 0.15s;
  }
  .sw-knob {
    position: absolute; top: 1px; left: 1px;
    width: 12px; height: 12px;
    background: var(--stroke); border-radius: 50%;
    transition: transform 0.15s, background 0.15s;
  }
  .autoplay-switch.on .sw-track { background: var(--accent); }
  .autoplay-switch.on .sw-knob { transform: translateX(16px); background: var(--on-accent); }
  .lyrics-toggle {
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    color: var(--text-muted);
    font-size: 0.72rem; font-weight: 900; font-family: inherit;
    letter-spacing: 0.12em;
    padding: 0.55rem 1rem;
    border-radius: var(--radius-pill);
    box-shadow: var(--shadow-sm);
    transition: box-shadow 0.12s, transform 0.1s, background 0.12s, color 0.12s;
  }
  .lyrics-toggle:hover { box-shadow: var(--shadow); transform: translate(-2px, -2px); }
  .lyrics-toggle.active {
    background: var(--accent); color: var(--on-accent);
    box-shadow: var(--shadow-accent);
  }

  /* ── Cuerpo ── */
  .np-main {
    display: flex;
    align-items: center;
    gap: clamp(1.5rem, 4vw, 3rem);
    flex: 1;
    min-height: 0;
  }

  .art-col { flex-shrink: 0; }
  .art-wrap { width: clamp(220px, 32vw, 360px); }
  .art {
    width: 100%; aspect-ratio: 1;
    object-fit: cover; display: block;
    border-radius: var(--radius);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-lg);
  }
  .art-empty {
    width: 100%; aspect-ratio: 1;
    position: relative; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent);
    border-radius: var(--radius);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-lg);
  }
  .empty-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(255,255,255,0.25) 2px, transparent 2px);
    background-size: 16px 16px;
  }
  .empty-g {
    font-size: 6rem; font-weight: 900; color: var(--on-accent);
    position: relative; z-index: 1; letter-spacing: -0.04em;
  }

  .info-col {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .badge {
    display: inline-flex; align-items: center; gap: 0.5rem;
    align-self: flex-start;
    background: var(--accent); color: var(--on-accent);
    font-size: 0.68rem; font-weight: 900; letter-spacing: 0.12em;
    padding: 0.4rem 0.8rem;
    border-radius: var(--radius-pill);
    border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    margin-bottom: 1rem;
  }
  .badge-eq { display: inline-flex; align-items: flex-end; gap: 2px; height: 12px; }
  .badge-eq i {
    width: 2.5px; height: 40%;
    background: var(--on-accent); border-radius: 1px;
  }
  .badge-eq.playing i { animation: eq 0.8s ease-in-out infinite alternate; }
  .badge-eq.playing i:nth-child(2) { animation-delay: 0.25s; }
  .badge-eq.playing i:nth-child(3) { animation-delay: 0.5s; }
  /* Palestina: badge en rojo */
  :global([data-theme="palestina"]) .badge { background: var(--accent-3); }

  .track-title {
    font-size: clamp(2rem, 5vw, 3.4rem);
    font-weight: 900; color: var(--text-primary);
    letter-spacing: -0.03em; line-height: 1.02;
    margin-bottom: 0.6rem;
  }
  .track-artist {
    font-size: clamp(1.1rem, 2vw, 1.5rem);
    color: var(--accent); font-weight: 800;
    margin-bottom: 0.3rem;
  }
  .track-album {
    font-size: 0.95rem; color: var(--text-muted); font-weight: 600;
  }

  .actions {
    display: flex; gap: 0.75rem;
    margin-top: 1.5rem;
  }
  .action-btn {
    width: 48px; height: 48px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card);
    color: var(--text-primary);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    transition: box-shadow 0.12s, transform 0.1s, background 0.12s, color 0.12s;
  }
  .action-btn:hover { box-shadow: var(--shadow); transform: translate(-2px, -2px); }
  .action-btn:active { transform: translate(0, 0); box-shadow: var(--shadow-sm); }
  .action-btn.primary { background: var(--accent); color: var(--on-accent); box-shadow: var(--shadow-accent); }
  .action-btn.liked { background: var(--accent); color: var(--on-accent); }
  /* Palestina: favorito en rojo */
  :global([data-theme="palestina"]) .action-btn.primary,
  :global([data-theme="palestina"]) .action-btn.liked {
    background: var(--accent-3);
    box-shadow: 4px 4px 0 var(--accent-3-dim);
  }

  /* ── Waveform ── */
  .wave {
    display: flex; align-items: flex-end; gap: 3px;
    height: 64px; margin-top: 2rem;
    overflow: hidden;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }
  .wave:focus-visible { outline: var(--stroke-w) solid var(--accent); outline-offset: 4px; }
  .wave-bar {
    flex: 1;
    min-width: 2px;
    height: calc(var(--h) * 100%);
    background: var(--text-dim);
    border-radius: 2px;
    transform-origin: bottom;
    opacity: 0.55;
    transition: background 0.2s, opacity 0.2s;
  }
  .wave-bar.played { background: var(--accent); opacity: 1; }
  .wave.playing .wave-bar.played {
    animation: eq 0.9s ease-in-out infinite alternate;
    animation-delay: var(--d);
  }

  @keyframes eq { from { transform: scaleY(0.45); } to { transform: scaleY(1); } }

  /* ── Letra ── */
  /* Con letra activa, las columnas se estiran a toda la altura para que la
     letra tenga un contenedor de altura fija y haga scroll interno (no la página). */
  .np-layout.with-lyrics .np-main { align-items: stretch; }
  .np-layout.with-lyrics .art-col { align-self: center; }

  .lyrics-panel {
    display: flex; flex-direction: column;
    height: 100%; min-height: 0;
    width: 100%;
  }
  .lyrics-status {
    color: var(--text-muted); font-size: 0.9rem; line-height: 1.6;
    margin: auto 0;
  }
  .lyrics-lines {
    flex: 1; min-height: 0; overflow-y: auto;
    display: flex; flex-direction: column; gap: 0.35rem;
    padding: 0.5rem 0.75rem 2rem;
    scroll-behavior: smooth;
    mask-image: linear-gradient(180deg, transparent, #000 14%, #000 86%, transparent);
    scrollbar-width: none;
  }
  .lyrics-lines::-webkit-scrollbar { display: none; }
  .lyric-line {
    font-size: 1.25rem; font-weight: 700; color: var(--text-primary);
    line-height: 1.6; opacity: 0.85;
  }
  .lyric-break { height: 0.8rem; }

  /* Controles: ocultos en escritorio (están en la PlayerBar), visibles en móvil */
  .np-controls { display: none; }
  @keyframes pulse { 50% { opacity: 0; } }

  /* ── Responsive / móvil ── */
  @media (max-width: 768px) {
    /* Sin barra inferior propia en esta pantalla (se oculta a propósito):
       hay que dejar lugar para la barra de gestos/navegación del sistema. */
    .np-layout { padding-bottom: calc(1rem + env(safe-area-inset-bottom, 0px)); }
    .np-main { flex-direction: column; text-align: center; }
    .info-col { align-items: center; }
    .badge, .actions { align-self: center; }
    /* Tamaño de la portada atado también al alto disponible, no solo al
       ancho: en pantallas bajas se achica para que todo entre sin scroll
       ni superposiciones con los controles. */
    .art-wrap { width: min(40vh, 62vw, 280px); }
    .badge { margin-bottom: 0.6rem; }
    .track-title { font-size: clamp(1.5rem, 7vw, 2.4rem); margin-bottom: 0.3rem; }
    .track-artist { margin-bottom: 0.15rem; }
    .actions { margin-top: 0.75rem; }
    .wave { width: 100%; height: 48px; margin-top: 1rem; }

    .np-controls {
      display: flex; flex-direction: column; gap: 0.5rem;
      flex-shrink: 0; margin-top: 0.75rem;
    }
    .np-times { display: flex; justify-content: space-between; padding: 0 0.25rem; }
    .np-times .time {
      font-size: 0.72rem; color: var(--text-muted); font-weight: 700;
      font-variant-numeric: tabular-nums;
    }
    .controls-row { display: flex; align-items: center; justify-content: center; gap: 0.4rem; }
    .ctrl-icon {
      width: 46px; height: 46px; border-radius: 50%;
      background: none; border: none; color: var(--text-muted);
      display: flex; align-items: center; justify-content: center;
    }
    .ctrl-icon.active { color: var(--accent); }
    .play-btn {
      width: 64px; height: 64px; border-radius: 50%;
      background: var(--accent); color: var(--on-accent);
      border: var(--stroke-heavy) solid var(--stroke);
      display: flex; align-items: center; justify-content: center;
      box-shadow: var(--shadow-accent); margin: 0 0.4rem;
    }
    .play-btn:active { transform: translate(0,0); box-shadow: var(--shadow-sm); }
    .loading-pulse { animation: pulse 1s step-end infinite; font-weight: 700; }
  }
</style>
