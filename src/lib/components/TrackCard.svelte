<script lang="ts">
  import type { Track } from '$lib/stores/player';
  import { player } from '$lib/stores/player';
  import { resolveStream, preloadStream } from '$lib/api';
  import PlaylistModal from './PlaylistModal.svelte';

  interface Props {
    track: Track;
    compact?: boolean;
    playing?: boolean;
    onPlay?: () => void;
  }
  let { track, compact = false, playing = false, onPlay }: Props = $props();

  let loading = $state(false);
  let showPlaylistModal = $state(false);
  let imgError = $state(false);

  function fmt(ms?: number) {
    if (!ms) return '--:--';
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  }

  async function play() {
    if (loading) return;
    if (onPlay) { onPlay(); return; }
    loading = true;
    player.setTrackLoading(track);
    player.addToQueue(track);
    try {
      const stream = await resolveStream(track);
      player.setStreamUrl(stream.url);
    } catch (e) {
      player.setError(`ERROR: ${e}`);
    } finally {
      loading = false;
    }
  }

  function onAddToPlaylist(e: MouseEvent) {
    e.stopPropagation();
    showPlaylistModal = true;
  }
</script>

<div
  class="track-card {playing ? 'is-playing' : ''}"
  role="button"
  tabindex="0"
  onclick={play}
  onmouseenter={() => preloadStream(track)}
  onfocus={() => preloadStream(track)}
  onkeydown={e => e.key === 'Enter' && play()}
>
  <!-- Art -->
  <div class="art-wrap">
    {#if track.thumbnail && !imgError}
      <img class="art" src={track.thumbnail} alt="" loading="lazy" onerror={() => imgError = true} />
    {:else}
      <div class="art art-empty">
        <div class="empty-dots"></div>
        <span class="empty-g">I</span>
      </div>
    {/if}
    <div class="art-overlay">
      {#if loading}
        <span class="loading">···</span>
      {:else if playing}
        <span class="eq"><span></span><span></span><span></span></span>
      {:else}
        <span>▶</span>
      {/if}
    </div>
  </div>

  <!-- Info -->
  <div class="track-info">
    <span class="title">{track.title}</span>
    <span class="artist">{track.artist}</span>
  </div>

  <!-- Duration -->
  <span class="duration">{fmt(track.durationMs)}</span>

  <!-- Add to playlist -->
  <button class="add-btn" title="Agregar a playlist" onclick={onAddToPlaylist}>+</button>
</div>

{#if showPlaylistModal}
  <PlaylistModal {track} onClose={() => (showPlaylistModal = false)} />
{/if}

<style>
  .track-card {
    display: flex; align-items: center; gap: 0.85rem;
    padding: 0.55rem 1rem; border: 2px solid transparent;
    background: transparent; cursor: pointer;
    transition: background 0.12s, box-shadow 0.1s, border-color 0.1s, transform 0.1s;
    position: relative; border-radius: var(--radius-sm);
  }
  .track-card:hover {
    background: var(--bg-card);
    border-color: var(--stroke);
    box-shadow: var(--shadow-sm);
    transform: translate(-1px, -1px);
  }
  .track-card:active { transform: translate(0,0); }
  .track-card.is-playing {
    background: var(--bg-card);
    border-color: var(--accent);
    box-shadow: var(--shadow-accent);
  }

  /* Art */
  .art-wrap { position: relative; flex-shrink: 0; }
  .art {
    width: 48px; height: 48px; object-fit: cover; display: block;
    border-radius: var(--radius-sm);
    border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow-sm);
  }
  .art-empty {
    width: 48px; height: 48px;
    position: relative; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent);
    border-radius: var(--radius-sm);
    border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow-sm);
  }
  .empty-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(255,255,255,0.25) 1.5px, transparent 1.5px);
    background-size: 8px 8px;
  }
  .empty-g {
    font-size: 1.3rem; font-weight: 900; color: #ffffff;
    position: relative; z-index: 1;
    letter-spacing: -0.02em;
  }
  .art-overlay {
    position: absolute; inset: 0;
    background: rgba(0,0,0,0.52);
    display: flex; align-items: center; justify-content: center;
    color: #ffffff; font-size: 1rem; font-weight: 700;
    opacity: 0; transition: opacity 0.1s; border-radius: var(--radius-sm);
  }
  .track-card:hover .art-overlay { opacity: 1; }
  .is-playing .art-overlay { opacity: 1; background: rgba(0,0,0,0.6); }

  .loading { animation: blink 0.8s step-end infinite; }
  @keyframes blink { 50% { opacity: 0; } }

  /* Equalizer bars */
  .eq { display: flex; align-items: flex-end; gap: 2px; height: 18px; }
  .eq span { width: 3px; background: #ffffff; animation: eq 0.7s ease-in-out infinite alternate; }
  .eq span:nth-child(1) { height: 10px; animation-delay: 0s; }
  .eq span:nth-child(2) { height: 18px; animation-delay: 0.15s; }
  .eq span:nth-child(3) { height: 7px; animation-delay: 0.3s; }
  @keyframes eq { to { height: 3px; } }

  /* Info */
  .track-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .title {
    font-size: 0.92rem; font-weight: 600; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    letter-spacing: -0.01em;
  }
  .is-playing .title { color: var(--accent); font-weight: 700; }
  .artist { font-size: 0.78rem; color: var(--text-muted); font-weight: 500; }

  .duration {
    font-size: 0.78rem; color: var(--text-dim); flex-shrink: 0;
    min-width: 2.8rem; text-align: right; font-weight: 600;
  }

  .add-btn {
    background: none; border: var(--stroke-w) solid transparent; color: var(--text-dim);
    font-size: 1.1rem; font-weight: 700; font-family: inherit;
    width: 30px; height: 30px; border-radius: var(--radius-sm);
    display: flex; align-items: center; justify-content: center;
    padding: 0; opacity: 0; transition: all 0.1s; flex-shrink: 0; line-height: 1;
  }
  .track-card:hover .add-btn { opacity: 1; }
  .add-btn:hover { color: var(--accent); border-color: var(--accent); background: var(--bg-card); box-shadow: var(--shadow-sm); }
</style>
