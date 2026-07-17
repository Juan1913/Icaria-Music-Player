<script lang="ts">
  import { player } from '$lib/stores/player';
  import { favorites } from '$lib/stores/favorites';
  import { nav } from '$lib/stores/nav';
  import { get } from 'svelte/store';
  import { onMount } from 'svelte';
  import { playNext as handleNext, playPrev as handlePrev, playRadio, seekRequest, streamAndPlay } from '$lib/playback';

  // ── Media Session: controles en pantalla de bloqueo / notificación (Android/desktop) ──
  onMount(() => {
    if (!('mediaSession' in navigator)) return;
    const ms = navigator.mediaSession;
    ms.setActionHandler('play', () => player.setPlaying(true));
    ms.setActionHandler('pause', () => player.setPlaying(false));
    ms.setActionHandler('previoustrack', () => { handlePrev(); });
    ms.setActionHandler('nexttrack', () => { handleNext(); });
    try {
      ms.setActionHandler('seekto', (d) => {
        if (audioEl && typeof d.seekTime === 'number') audioEl.currentTime = d.seekTime;
      });
    } catch { /* algunos webviews no soportan seekto */ }
  });

  // Metadatos de la pista actual (título/artista/carátula) para el control del sistema
  $effect(() => {
    if (!('mediaSession' in navigator)) return;
    const t = $player.currentTrack;
    if (!t) { navigator.mediaSession.metadata = null; return; }
    navigator.mediaSession.metadata = new MediaMetadata({
      title: t.title,
      artist: t.artist,
      album: t.album ?? '',
      artwork: t.thumbnail
        ? [{ src: t.thumbnail, sizes: '512x512', type: 'image/jpeg' }]
        : [],
    });
  });

  // Estado de reproducción (play/pausa) reflejado en el control del sistema
  $effect(() => {
    if ('mediaSession' in navigator) {
      navigator.mediaSession.playbackState = $player.isPlaying ? 'playing' : 'paused';
    }
  });

  let audioEl: HTMLAudioElement | undefined = $state();
  let duration = $state(0);
  let seeking = $state(false);
  let seekValue = $state(0);

  let pendingPlay = $state(false);
  let expectedUrl: string | null = null;

  // Audio element ↔ player state sync
  $effect(() => {
    if (!audioEl) return;
    const url = $player.streamUrl;
    const shouldPlay = $player.isPlaying;

    if (url && url !== expectedUrl) {
      expectedUrl = url;
      pendingPlay = shouldPlay;
      audioEl.src = url;
      audioEl.load();
      return;
    }

    if (!url) {
      expectedUrl = null;
      audioEl.pause();
      pendingPlay = false;
      return;
    }

    if (shouldPlay) {
      if (audioEl.readyState >= 3) {
        pendingPlay = false;
        audioEl.play().catch(() => {});
      } else {
        pendingPlay = true;
      }
    } else {
      pendingPlay = false;
      audioEl.pause();
    }
  });

  $effect(() => {
    if (audioEl) audioEl.volume = $player.isMuted ? 0 : $player.volume;
  });

  // Aplica un seek pedido desde otra vista (p. ej. Now Playing).
  $effect(() => {
    const f = $seekRequest;
    if (f == null || !audioEl || !duration) return;
    audioEl.currentTime = f * duration;
    player.setProgress(audioEl.currentTime * 1000, duration * 1000);
    seekRequest.set(null);
  });

  let audioErrorTrackId: string | null = null;
  let audioErrorRetries = 0;
  const MAX_AUDIO_RETRIES = 2;

  function onCanPlay() {
    player.setLoading(false);
    audioErrorTrackId = null;
    audioErrorRetries = 0;
    if (pendingPlay && audioEl) {
      pendingPlay = false;
      audioEl.play().catch(() => {});
    }
  }
  function onTimeUpdate() {
    if (!audioEl || seeking) return;
    player.setProgress(audioEl.currentTime * 1000, duration * 1000);
    if ('mediaSession' in navigator && 'setPositionState' in navigator.mediaSession && duration > 0) {
      try {
        navigator.mediaSession.setPositionState({
          duration,
          playbackRate: audioEl.playbackRate || 1,
          position: Math.min(audioEl.currentTime, duration),
        });
      } catch { /* valores inválidos: ignorar */ }
    }
  }
  function onLoadedMetadata() { if (audioEl) duration = audioEl.duration; }
  async function onEnded() {
    if ($player.repeat === 'one') {
      if (audioEl) { audioEl.currentTime = 0; audioEl.play().catch(() => {}); }
      return;
    }
    const s = get(player);
    const hasNext = s.queue.length > 0 &&
      ((s.shuffle && s.queue.length > 1) || s.queueIndex < s.queue.length - 1 || s.repeat === 'all');
    if (hasNext) {
      await handleNext();
    } else if (s.autoplay) {
      // Fin de la cola: radio de temas similares (si autoplay está activo)
      await playRadio();
    } else {
      player.setPlaying(false);
    }
  }
  function togglePlay() { player.setPlaying(!$player.isPlaying); }
  function openNowPlaying() { if ($player.currentTrack) nav.setPage('nowplaying'); }
  function onSeekInput(e: Event) {
    seeking = true;
    seekValue = +(e.target as HTMLInputElement).value;
  }
  function onSeekChange(e: Event) {
    const v = +(e.target as HTMLInputElement).value;
    if (audioEl) audioEl.currentTime = (v / 100) * duration;
    seeking = false;
  }
  function onVolumeChange(e: Event) {
    player.setVolume(+(e.target as HTMLInputElement).value / 100);
  }
  function fmt(ms: number) {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  }
  const pct = $derived(seeking ? seekValue : $player.progress * 100);
  const hasTrack = $derived(!!$player.currentTrack);
  const isFav = $derived(!!$player.currentTrack && $favorites.some(t => t.id === $player.currentTrack!.id));
</script>

<audio
  bind:this={audioEl}
  ontimeupdate={onTimeUpdate}
  onloadedmetadata={onLoadedMetadata}
  oncanplay={onCanPlay}
  onended={onEnded}
  onerror={() => {
    if (!$player.streamUrl) return;
    const err = audioEl?.error;
    const codeNames: Record<number, string> = {
      1: 'ABORTED', 2: 'NETWORK', 3: 'DECODE', 4: 'SRC_NOT_SUPPORTED',
    };
    const codeName = err ? (codeNames[err.code] ?? `CODE_${err.code}`) : 'UNKNOWN';
    console.error('[Icaria] audio error:', codeName, err?.message || '(sin mensaje)', 'url:', $player.streamUrl);

    const track = $player.currentTrack;
    if (track) {
      if (audioErrorTrackId !== track.id) { audioErrorTrackId = track.id; audioErrorRetries = 0; }
      // Muchos fallos de YouTube son intermitentes (anti-bot/rate-limit): re-resolver
      // la misma pista antes de mostrar el error suele recuperarla sin intervención.
      if (audioErrorRetries < MAX_AUDIO_RETRIES) {
        audioErrorRetries++;
        setTimeout(() => {
          if ($player.currentTrack?.id === track.id) streamAndPlay(track);
        }, 700 * audioErrorRetries);
        return;
      }
    }
    player.setError(`STREAM ERROR (${codeName})`);
  }}
  preload="auto"
></audio>

{#if $player.error}
  <div class="error-bar">!! {$player.error} !!</div>
{/if}

<div class="player-bar" class:has-track={hasTrack}>

  <!-- Progreso fino (solo visible en móvil) -->
  <div class="mobile-progress"><div class="mobile-progress-fill" style="width:{pct}%"></div></div>

  <!-- LEFT: art + meta + extra btns (en móvil, toca para abrir la reproducción) -->
  <div
    class="track-section"
    role="button"
    tabindex="0"
    onclick={openNowPlaying}
    onkeydown={(e) => { if (e.key === 'Enter') openNowPlaying(); }}
  >
    <div class="art-wrap">
      {#if $player.currentTrack?.thumbnail}
        <img class="art" src={$player.currentTrack.thumbnail} alt="cover" />
      {:else}
        <div class="art art-empty">
          <div class="empty-dots"></div>
          {#if $player.isLoading}
            <span class="loading-pulse" style="position:relative;z-index:1">···</span>
          {:else}
            <span class="empty-g-bar">I</span>
          {/if}
        </div>
      {/if}
      {#if $player.isLoading && $player.currentTrack?.thumbnail}
        <div class="art-loading-overlay"><span class="loading-pulse">···</span></div>
      {/if}
    </div>

    <div class="track-meta">
      <span class="track-title" title={$player.currentTrack?.title ?? ''}>
        {$player.currentTrack?.title ?? '─── sin canción ───'}
      </span>
      <span class="track-sub">{$player.currentTrack?.artist ?? 'Icaria'}</span>
    </div>

    <div class="extra-btns">
      <button
        class="icon-btn"
        class:fav={isFav}
        onclick={(e) => { e.stopPropagation(); if ($player.currentTrack) favorites.toggle($player.currentTrack); }}
        title={isFav ? 'Quitar de favoritos' : 'Favorito'}
        aria-pressed={isFav}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill={isFav ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
        </svg>
      </button>
      <button class="icon-btn" onclick={(e) => { e.stopPropagation(); if ($player.currentTrack) player.addToQueue($player.currentTrack); }} title="Añadir">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- CENTER: controls + progress -->
  <div class="center-section">
    <div class="controls-row">
      <!-- Shuffle -->
      <button class="ctrl-icon {$player.shuffle ? 'active' : ''}" onclick={() => player.toggleShuffle()} title="Aleatorio">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/>
          <polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/>
        </svg>
      </button>

      <!-- Prev -->
      <button class="ctrl-icon" onclick={handlePrev} title="Anterior">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/>
        </svg>
      </button>

      <!-- Play/Pause -->
      <button class="play-btn" onclick={togglePlay} title={$player.isPlaying ? 'Pausar' : 'Reproducir'}>
        {#if $player.isLoading}
          <span class="loading-pulse">···</span>
        {:else if $player.isPlaying}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>
          </svg>
        {:else}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5 3 19 12 5 21 5 3"/>
          </svg>
        {/if}
      </button>

      <!-- Next -->
      <button class="ctrl-icon" onclick={handleNext} title="Siguiente">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/>
        </svg>
      </button>

      <!-- Repeat -->
      <button class="ctrl-icon {$player.repeat !== 'none' ? 'active' : ''}" onclick={() => player.cycleRepeat()} title="Repetir">
        {#if $player.repeat === 'one'}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/>
            <polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>
            <line x1="12" y1="9" x2="12" y2="15"/><line x1="9" y1="12" x2="15" y2="12"/>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/>
            <polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>
          </svg>
        {/if}
      </button>
    </div>

    <div class="progress-row">
      <span class="time">{fmt($player.currentMs)}</span>
      <div class="progress-track">
        <div class="progress-fill" style="width:{pct}%"></div>
        <input
          class="progress-slider"
          type="range" min="0" max="100" step="0.1"
          value={pct}
          oninput={onSeekInput}
          onchange={onSeekChange}
        />
      </div>
      <span class="time">{fmt(duration * 1000)}</span>
    </div>
  </div>

  <!-- RIGHT: volume + queue list -->
  <div class="right-section">
    <button class="icon-btn" onclick={() => player.toggleMute()} title="Volumen">
      {#if $player.isMuted || $player.volume === 0}
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>
        </svg>
      {:else if $player.volume > 0.5}
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
          <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
        </svg>
      {:else}
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
        </svg>
      {/if}
    </button>
    <div class="volume-track">
      <div class="volume-fill" style="width:{$player.isMuted ? 0 : $player.volume * 100}%"></div>
      <input
        class="volume-slider"
        type="range" min="0" max="100"
        value={$player.isMuted ? 0 : $player.volume * 100}
        oninput={onVolumeChange}
      />
    </div>
    <button class="icon-btn" title="Cola">
      <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
        <line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>
      </svg>
    </button>
  </div>

</div>

<style>
  /* ── PLAYER BAR ── */
  .player-bar {
    height: 104px;
    background: var(--bg-primary);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    margin: 0 0.75rem 0.75rem;
    display: grid;
    grid-template-columns: 280px 1fr 220px;
    align-items: center;
    padding: 0 1.25rem;
    gap: 0.75rem;
    position: relative;
  }

  /* Progreso fino (solo móvil) */
  .mobile-progress {
    display: none;
    position: absolute; top: 0; left: 0; right: 0; height: 3px;
    background: var(--bg-card-hover);
  }
  .mobile-progress-fill {
    height: 100%; background: var(--accent);
    transition: width 0.1s linear;
  }

  /* ── LEFT: art + meta + extra btns ── */
  .track-section {
    display: flex; align-items: center; gap: 0.85rem; min-width: 0;
  }
  .art-wrap { position: relative; flex-shrink: 0; }
  .art {
    width: 64px; height: 64px; object-fit: cover; display: block;
    border-radius: var(--radius-sm);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
  }
  .art-empty {
    width: 64px; height: 64px;
    position: relative; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent);
    border-radius: var(--radius-sm);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
  }
  .empty-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(255,255,255,0.2) 1.5px, transparent 1.5px);
    background-size: 10px 10px;
  }
  .empty-g-bar {
    font-size: 1.6rem; font-weight: 900; color: #fff;
    position: relative; z-index: 1; letter-spacing: -0.03em;
    text-shadow: 2px 2px 0 rgba(0,0,0,0.25);
  }
  .art-loading-overlay {
    position: absolute; inset: 0;
    background: rgba(0,0,0,0.55);
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-sm);
  }
  .loading-pulse {
    animation: pulse 1s step-end infinite;
    font-size: 1rem; color: var(--accent); font-weight: 700; letter-spacing: 0.1em;
  }
  @keyframes pulse { 50% { opacity: 0; } }

  .track-meta {
    display: flex; flex-direction: column; gap: 3px;
    min-width: 0; overflow: hidden; flex: 1;
  }
  .track-title {
    font-size: 0.9rem; font-weight: 700; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .track-sub {
    font-size: 0.75rem; font-weight: 600; color: var(--text-muted);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  :global([data-theme="paper"]) .track-sub { color: #00a85a; }
  :global([data-theme="jamaica"]) .track-sub { color: #009B45; }

  .extra-btns {
    display: flex; gap: 0.25rem; flex-shrink: 0;
  }
  .icon-btn {
    width: 32px; height: 32px; border-radius: 50%;
    background: none; border: none;
    color: var(--text-muted); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: color 0.12s, background 0.12s;
  }
  .icon-btn:hover { color: var(--text-primary); background: var(--bg-card-hover); }
  .icon-btn.fav { color: var(--accent); }
  :global([data-theme="palestina"]) .icon-btn.fav { color: var(--accent-3); }

  /* ── CENTER: controls + progress ── */
  .center-section {
    display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  }
  .controls-row {
    display: flex; align-items: center; gap: 0.35rem;
  }
  .ctrl-icon {
    width: 36px; height: 36px; border-radius: 50%;
    background: none; border: none;
    color: var(--text-muted); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: color 0.12s, background 0.12s;
  }
  .ctrl-icon:hover { color: var(--text-primary); background: var(--bg-card-hover); }
  .ctrl-icon.active { color: var(--accent); }

  .play-btn {
    width: 52px; height: 52px; border-radius: 50%;
    background: var(--accent); color: #ffffff;
    border: var(--stroke-heavy) solid var(--stroke); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    box-shadow: var(--shadow-accent);
    transition: transform 0.1s, box-shadow 0.1s;
    margin: 0 0.25rem;
  }
  .play-btn:hover { transform: translate(-2px, -2px); box-shadow: var(--shadow-lg); }
  .play-btn:active { transform: translate(0, 0); box-shadow: 1px 1px 0 var(--stroke); }

  .progress-row {
    display: flex; align-items: center; gap: 0.6rem;
    width: 100%; max-width: 500px;
  }
  .time {
    font-size: 0.7rem; color: var(--text-dim);
    min-width: 2.8rem; text-align: center; font-weight: 600;
  }
  .progress-track {
    flex: 1; position: relative; height: 4px;
    background: var(--bg-card-hover); border-radius: 2px; cursor: pointer;
  }
  .progress-fill {
    position: absolute; left: 0; top: 0; bottom: 0;
    background: var(--accent); border-radius: 2px;
    pointer-events: none; transition: width 0.1s linear;
  }
  .progress-track:hover { height: 6px; }
  .progress-track:hover .progress-fill { border-radius: 3px; }
  .progress-slider {
    position: absolute; inset: -10px 0;
    width: 100%; opacity: 0; cursor: pointer; margin: 0;
    height: calc(100% + 20px);
  }

  /* ── RIGHT: volume ── */
  .right-section {
    display: flex; align-items: center; gap: 0.5rem; justify-content: flex-end;
  }
  .volume-track {
    position: relative; width: 90px; height: 4px;
    background: var(--bg-card-hover); border-radius: 2px;
  }
  .volume-fill {
    position: absolute; left: 0; top: 0; bottom: 0;
    background: var(--accent); border-radius: 2px;
    pointer-events: none;
  }
  .volume-track:hover { height: 6px; }
  .volume-slider {
    position: absolute; inset: -10px 0;
    width: 100%; height: calc(100% + 20px);
    opacity: 0; cursor: pointer; margin: 0;
  }

  .error-bar {
    background: var(--gb-red-dim); color: #ffffff;
    font-size: 0.78rem; font-weight: 700;
    text-align: center; padding: 0.35rem;
    border-top: var(--stroke-w) solid var(--gb-red);
  }

  :global([data-theme="jamaica"]) .play-btn { background: #009B45; }

  /* Palestina: verde + borde negro + sombra roja → los tres colores en un botón */
  :global([data-theme="palestina"]) .play-btn { box-shadow: 4px 4px 0 var(--accent-3-dim); }
  :global([data-theme="palestina"]) .play-btn:hover { box-shadow: 6px 6px 0 var(--accent-3-dim); }

  /* ── Móvil: mini-reproductor (art + título + play) ── */
  @media (max-width: 768px) {
    .player-bar {
      height: 62px;
      margin: 0;
      border-radius: 0;
      border-left: none; border-right: none; border-bottom: none;
      box-shadow: none;
      grid-template-columns: 1fr auto;
      padding: 0 0.75rem;
      gap: 0.5rem;
    }
    .mobile-progress { display: block; }
    .track-section { cursor: pointer; gap: 0.6rem; }
    .art, .art-empty { width: 46px; height: 46px; box-shadow: none; border-width: var(--stroke-w); }
    .extra-btns { display: none; }
    .progress-row { display: none; }
    .right-section { display: none; }
    .controls-row .ctrl-icon { display: none; }
    .center-section { flex-direction: row; gap: 0; }
    .play-btn {
      width: 44px; height: 44px; margin: 0;
      box-shadow: 3px 3px 0 var(--stroke);
    }
    .play-btn:hover { transform: none; }
  }
</style>
