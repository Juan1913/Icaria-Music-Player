<script lang="ts">
  import { player } from '$lib/stores/player';
  import { resolveStream, getRadioTracks } from '$lib/api';
  import { get } from 'svelte/store';
  import type { Track } from '$lib/stores/player';

  let audioEl: HTMLAudioElement | undefined = $state();
  let duration = $state(0);
  let seeking = $state(false);
  let seekValue = $state(0);

  let pendingPlay = $state(false);
  let expectedUrl: string | null = null;

  // Carga y reproduce una pista explícitamente (sin depender de $effect)
  async function streamAndPlay(track: Track) {
    player.setTrackLoading(track);
    try {
      const stream = await resolveStream(track);
      player.setStreamUrl(stream.url);
    } catch (e) {
      player.setError(`ERROR: ${e}`);
    }
  }

  async function handleNext() {
    const s = get(player);
    if (s.queue.length === 0) return;
    if (s.queueIndex >= s.queue.length - 1 && s.repeat !== 'all') return;
    player.next();
    const after = get(player);
    const track = after.queue[after.queueIndex];
    if (track) await streamAndPlay(track);
  }

  async function handlePrev() {
    const s = get(player);
    if (s.queue.length === 0) return;
    player.prev();
    const after = get(player);
    const track = after.queue[after.queueIndex];
    if (track) await streamAndPlay(track);
  }

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

  function onCanPlay() {
    player.setLoading(false);
    if (pendingPlay && audioEl) {
      pendingPlay = false;
      audioEl.play().catch(() => {});
    }
  }
  function onTimeUpdate() {
    if (!audioEl || seeking) return;
    player.setProgress(audioEl.currentTime * 1000, duration * 1000);
  }
  function onLoadedMetadata() { if (audioEl) duration = audioEl.duration; }
  async function onEnded() {
    if ($player.repeat === 'one') {
      if (audioEl) { audioEl.currentTime = 0; audioEl.play().catch(() => {}); }
      return;
    }
    const s = get(player);
    const isExhausted = s.queue.length === 0 || s.queueIndex < 0 || s.queueIndex >= s.queue.length - 1;
    if (isExhausted && s.repeat !== 'all' && s.currentTrack) {
      // Radio: YouTube Music RDAMVM mix (similar artists, not just same artist)
      try {
        const tracks = await getRadioTracks(s.currentTrack.streamId, 20);
        if (tracks.length > 0) {
          player.setQueue(tracks, 0);
          await streamAndPlay(tracks[0]);
        }
      } catch { /* si falla el radio, no pasa nada */ }
    } else {
      await handleNext();
    }
  }
  function togglePlay() { player.setPlaying(!$player.isPlaying); }
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
</script>

<audio
  bind:this={audioEl}
  ontimeupdate={onTimeUpdate}
  onloadedmetadata={onLoadedMetadata}
  oncanplay={onCanPlay}
  onended={onEnded}
  onerror={() => { if ($player.streamUrl) player.setError('STREAM ERROR'); }}
  preload="auto"
></audio>

{#if $player.error}
  <div class="error-bar">!! {$player.error} !!</div>
{/if}

<div class="player-bar" class:has-track={hasTrack}>

  <!-- LEFT: art + meta + extra btns -->
  <div class="track-section">
    <div class="art-wrap">
      {#if $player.currentTrack?.thumbnail}
        <img class="art" src={$player.currentTrack.thumbnail} alt="cover" />
      {:else}
        <div class="art art-empty">
          <div class="empty-dots"></div>
          {#if $player.isLoading}
            <span class="loading-pulse" style="position:relative;z-index:1">···</span>
          {:else}
            <span class="empty-g-bar">G</span>
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
      <span class="track-sub">{$player.currentTrack?.artist ?? 'GROOVE'}</span>
    </div>

    <div class="extra-btns">
      <button class="icon-btn" title="Favorito">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
        </svg>
      </button>
      <button class="icon-btn" onclick={() => { if ($player.currentTrack) player.addToQueue($player.currentTrack); }} title="Añadir">
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
</style>
