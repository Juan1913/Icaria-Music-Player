<script lang="ts">
  import { history } from '$lib/stores/history';
  import { player } from '$lib/stores/player';
  import { streamAndPlay } from '$lib/playback';
  import { preloadStream } from '$lib/api';
  import TrackCard from '$lib/components/TrackCard.svelte';

  const currentId = $derived($player.currentTrack?.streamId);

  function playFrom(index: number) {
    const list = $history;
    if (!list.length) return;
    player.setQueue(list, index);
    streamAndPlay(list[index]);
  }

  function playAll() { playFrom(0); }

  function clearHistory() {
    history.clear();
  }

  $effect(() => {
    for (let i = 0; i < Math.min(2, $history.length); i++) preloadStream($history[i]);
  });
</script>

<div class="history-view">
  <div class="view-header">
    <span class="view-title">// HISTORIAL</span>
    <span class="count">[{$history.length} CANCIONES]</span>
    {#if $history.length}
      <button class="play-all-btn" onclick={playAll}>▶ REPRODUCIR TODO</button>
      <button class="clear-btn" onclick={clearHistory}>[× BORRAR]</button>
    {/if}
  </div>

  {#if $history.length === 0}
    <div class="empty-state">
      <pre class="ascii-note">
  ╔══════════════╗
  ║   [⏱]  [ ]   ║
  ║  sin historial  ║
  ╚══════════════╝</pre>
      <p class="empty-msg">:: acá van apareciendo las canciones que reproducís ::</p>
    </div>
  {:else}
    <div class="track-list">
      {#each $history as track, i (track.id + i)}
        <div class="track-row">
          <span class="track-num">{String(i + 1).padStart(2, '0')}</span>
          <div class="track-card-wrap">
            <TrackCard
              {track}
              compact
              playing={track.streamId === currentId}
              onPlay={() => playFrom(i)}
            />
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .history-view { display: flex; flex-direction: column; height: 100%; overflow-y: auto; }

  .view-header {
    padding: 1rem 1.25rem 0.75rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.08);
    display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap;
    flex-shrink: 0;
  }
  .view-title { font-size: 0.72rem; font-weight: 700; letter-spacing: 0.12em; color: var(--accent); }
  .count { font-size: 0.68rem; font-weight: 700; letter-spacing: 0.08em; color: var(--gb-gray); flex: 1; }

  .play-all-btn {
    background: none; border: var(--stroke-w) solid var(--accent); box-shadow: var(--shadow-sm);
    color: var(--accent); font-size: 0.68rem; font-weight: 700; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.3rem 0.7rem; cursor: pointer; transition: all 0.08s;
  }
  .play-all-btn:hover { background: var(--accent); color: var(--on-accent); transform: translate(-1px,-1px); box-shadow: var(--shadow); }

  .clear-btn {
    background: none; border: var(--stroke-w) solid var(--gb-red); box-shadow: var(--shadow-sm);
    color: var(--gb-red); font-size: 0.68rem; font-weight: 700; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.3rem 0.7rem; cursor: pointer; transition: all 0.08s;
  }
  .clear-btn:hover { background: var(--gb-red); color: #ffffff; transform: translate(-1px,-1px); box-shadow: var(--shadow); }

  .track-list { display: flex; flex-direction: column; gap: 2px; padding: 0.5rem; overflow-y: auto; }
  .track-row { display: flex; align-items: center; gap: 0.25rem; }
  .track-num {
    width: 2rem; text-align: center; flex-shrink: 0;
    font-size: 0.68rem; font-weight: 700; color: var(--bg-card-hover); letter-spacing: 0.05em;
  }
  .track-card-wrap { flex: 1; min-width: 0; }

  .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    flex: 1; gap: 1rem; padding: 3rem; text-align: center; min-height: 250px;
  }
  .ascii-note { font-size: 0.6rem; color: var(--bg-card-hover); line-height: 1.4; }
  .empty-msg { font-size: 0.82rem; font-weight: 700; color: var(--accent); letter-spacing: 0.06em; }
</style>
