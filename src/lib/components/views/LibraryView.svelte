<script lang="ts">
  import { playlists, type Playlist } from '$lib/stores/playlists';
  import { player } from '$lib/stores/player';
  import TrackCard from '$lib/components/TrackCard.svelte';
  import { resolveStream, preloadStream } from '$lib/api';
  import { imgFallback } from '$lib/imgFallback';

  let view = $state<'library' | 'playlist'>('library');
  let activePlaylist = $state<Playlist | null>(null);
  let creating = $state(false);
  let newName = $state('');
  let renaming = $state<string | null>(null);
  let renameValue = $state('');

  const currentId = $derived($player.currentTrack?.streamId);

  function openPlaylist(pl: Playlist) {
    activePlaylist = pl;
    view = 'playlist';
    for (let i = 0; i < Math.min(2, pl.tracks.length); i++) {
      preloadStream(pl.tracks[i]);
    }
  }
  function back() { view = 'library'; activePlaylist = null; }

  function createPlaylist() {
    if (!newName.trim()) return;
    playlists.create(newName);
    newName = '';
    creating = false;
  }

  function deletePlaylist(id: string) {
    playlists.delete(id);
    if (activePlaylist?.id === id) back();
  }

  function startRename(pl: Playlist) { renaming = pl.id; renameValue = pl.name; }

  function confirmRename() {
    if (renaming && renameValue.trim()) {
      playlists.rename(renaming, renameValue);
      if (activePlaylist?.id === renaming) activePlaylist = $playlists.find(p => p.id === renaming) ?? null;
    }
    renaming = null;
  }

  async function playPlaylist(pl: Playlist) {
    if (!pl.tracks.length) return;
    player.setQueue(pl.tracks, 0);
    try {
      const stream = await resolveStream(pl.tracks[0]);
      player.setTrack(pl.tracks[0], stream.url);
    } catch {}
  }

  $effect(() => {
    if (activePlaylist) {
      activePlaylist = $playlists.find(p => p.id === activePlaylist!.id) ?? null;
      if (!activePlaylist) back();
    }
  });
</script>

<div class="library-view">
  {#if view === 'library'}
    <div class="view-header">
      <span class="view-title">// LIBRARY</span>
      <button class="new-btn" onclick={() => (creating = !creating)}>
        {creating ? '[x] CANCEL' : '[+] NEW PLAYLIST'}
      </button>
    </div>

    {#if creating}
      <div class="create-row">
        <span class="create-prefix">&gt;</span>
        <input
          type="text"
          class="create-input"
          placeholder="playlist name..."
          bind:value={newName}
          onkeydown={e => e.key === 'Enter' && createPlaylist()}
        />
        <button class="create-confirm" onclick={createPlaylist} disabled={!newName.trim()}>[OK]</button>
      </div>
    {/if}

    {#if $playlists.length === 0 && !creating}
      <div class="empty-state">
        <pre class="ascii-note">
  ╔══════════════╗
  ║   [♫]  [ ]  ║
  ║   no playlists  ║
  ╚══════════════╝</pre>
        <p class="empty-msg">:: search music · tap [+] to save ::</p>
      </div>
    {:else}
      <div class="playlist-grid">
        {#each $playlists as pl (pl.id)}
          <div class="playlist-card nb-card">
            <button class="pl-art-btn" onclick={() => openPlaylist(pl)} aria-label="Open {pl.name}">
              {#if pl.tracks.some(t => t.thumbnail)}
                <div class="art-mosaic">
                  {#each pl.tracks.filter(t => t.thumbnail).slice(0,4) as t}
                    <img src={t.thumbnail} alt="" use:imgFallback />
                  {/each}
                </div>
              {:else}
                <div class="art-fallback">[♫]</div>
              {/if}
              <div class="pl-overlay">▶</div>
            </button>

            <div class="pl-meta">
              {#if renaming === pl.id}
                <input
                  class="rename-input"
                  bind:value={renameValue}
                  onblur={confirmRename}
                  onkeydown={e => { if (e.key === 'Enter') confirmRename(); if (e.key === 'Escape') renaming = null; }}
                />
              {:else}
                <button class="pl-name-btn" onclick={() => openPlaylist(pl)}>{pl.name.toUpperCase()}</button>
              {/if}
              <span class="pl-count">[{pl.tracks.length} TRACKS]</span>
            </div>

            <div class="pl-actions">
              <button class="act-btn" onclick={() => playPlaylist(pl)} title="Play">▶</button>
              <button class="act-btn" onclick={() => startRename(pl)} title="Rename">✎</button>
              <button class="act-btn danger" onclick={() => deletePlaylist(pl.id)} title="Delete">[×]</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if view === 'playlist' && activePlaylist}
    <div class="view-header">
      <button class="back-btn" onclick={back}>[← BACK]</button>
      <span class="view-title">// {activePlaylist.name.toUpperCase()}</span>
      {#if activePlaylist.tracks.length}
        <button class="play-all-btn" onclick={() => playPlaylist(activePlaylist!)}>▶ PLAY ALL</button>
      {/if}
    </div>
    <div class="pl-detail-count">[{activePlaylist.tracks.length} TRACKS]</div>

    {#if activePlaylist.tracks.length === 0}
      <div class="empty-state">
        <p class="empty-msg">:: playlist is empty ::</p>
        <p class="empty-sub">search music and tap [+] to add tracks</p>
      </div>
    {:else}
      <div class="track-list">
        {#each activePlaylist.tracks as track, i (track.id + i)}
          <div class="track-row">
            <span class="track-num">{String(i+1).padStart(2,'0')}</span>
            <div class="track-card-wrap">
              <TrackCard {track} compact playing={track.streamId === currentId} />
            </div>
            <button
              class="remove-btn"
              title="Remove"
              onclick={() => playlists.removeTrack(activePlaylist!.id, i)}
            >[×]</button>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .library-view { display: flex; flex-direction: column; height: 100%; overflow-y: auto; }

  .view-header {
    padding: 1rem 1.25rem 0.75rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.08);
    display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap;
    flex-shrink: 0;
  }
  .view-title { font-size: 0.72rem; font-weight: 700; letter-spacing: 0.12em; color: var(--accent); flex: 1; }

  .new-btn {
    background: none; border: var(--stroke-w) solid var(--gb-green); box-shadow: var(--shadow-sm);
    color: var(--gb-green); font-size: 0.68rem; font-weight: 700; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.3rem 0.7rem; cursor: pointer; transition: all 0.08s;
  }
  .new-btn:hover { transform: translate(-1px,-1px); box-shadow: var(--shadow); }

  .create-row {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    box-shadow: 0 2px 6px rgba(0,0,0,0.06);
    background: var(--bg-sidebar);
    flex-shrink: 0;
  }
  .create-prefix { color: var(--accent); font-weight: 900; font-size: 1rem; }
  .create-input {
    flex: 1; background: none; border: none; outline: none;
    color: var(--text-primary); font-size: 0.9rem; font-weight: 600; font-family: inherit;
    caret-color: var(--accent);
  }
  .create-confirm {
    background: none; border: var(--stroke-w) solid var(--accent);
    color: var(--accent); font-size: 0.72rem; font-weight: 700;
    font-family: inherit; padding: 0.2rem 0.5rem; cursor: pointer; transition: all 0.08s;
  }
  .create-confirm:hover { background: var(--accent); color: var(--on-accent); }
  .create-confirm:disabled { opacity: 0.4; cursor: not-allowed; }

  .pl-detail-count {
    padding: 0.4rem 1.25rem;
    font-size: 0.68rem; font-weight: 700; letter-spacing: 0.08em;
    color: var(--gb-gray);
    flex-shrink: 0;
  }

  /* Playlist grid */
  .playlist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 0.85rem;
    padding: 1rem 1.25rem;
    overflow-y: auto;
  }
  .playlist-card {
    display: flex; flex-direction: column; gap: 0;
    background: var(--bg-card);
    border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow);
    transition: all 0.08s;
  }
  .playlist-card:hover { transform: translate(-2px,-2px); box-shadow: var(--shadow-lg); }
  .playlist-card:hover .pl-overlay { opacity: 1; }

  .pl-art-btn {
    aspect-ratio: 1; width: 100%; position: relative;
    cursor: pointer; background: var(--bg-card);
    border: none;
    overflow: hidden; padding: 0;
  }
  .art-mosaic { display: grid; grid-template-columns: 1fr 1fr; width: 100%; height: 100%; }
  .art-mosaic img { width: 100%; height: 100%; object-fit: cover; }
  .art-fallback {
    width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
    font-size: 1.5rem; color: var(--text-muted); font-weight: 700;
  }
  .pl-overlay {
    position: absolute; inset: 0;
    background: rgba(29,32,33,0.7);
    display: flex; align-items: center; justify-content: center;
    font-size: 1.5rem; color: var(--accent); font-weight: 900;
    opacity: 0; transition: opacity 0.1s;
  }

  .pl-meta { padding: 0.6rem 0.65rem 0.4rem; display: flex; flex-direction: column; gap: 2px; }
  .pl-name-btn {
    background: none; border: none; color: var(--text-primary);
    font-size: 0.8rem; font-weight: 700; letter-spacing: 0.04em;
    text-align: left; cursor: pointer; padding: 0; font-family: inherit;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 100%;
  }
  .pl-name-btn:hover { color: var(--accent); }
  .rename-input {
    background: var(--bg-sidebar); border: 1px solid var(--accent);
    color: var(--text-primary); font-size: 0.8rem; font-family: inherit;
    padding: 2px 6px; outline: none; width: 100%;
  }
  .pl-count { font-size: 0.65rem; font-weight: 700; letter-spacing: 0.05em; color: var(--gb-gray); }

  .pl-actions { display: flex; }
  .act-btn {
    flex: 1; padding: 0.35rem;
    background: none; border: none;
    color: var(--text-muted); font-size: 0.72rem; font-weight: 700;
    font-family: inherit; cursor: pointer; transition: all 0.08s;
  }
  .act-btn:hover { color: var(--accent); background: var(--bg-card); }
  .act-btn.danger:hover { color: var(--gb-red); }

  /* Detail view */
  .back-btn {
    background: none; border: var(--stroke-w) solid var(--bg-card-hover);
    color: var(--text-muted); font-size: 0.68rem; font-weight: 700;
    font-family: inherit; padding: 0.25rem 0.6rem; cursor: pointer; transition: all 0.08s;
  }
  .back-btn:hover { border-color: var(--stroke); color: var(--text-primary); }
  .play-all-btn {
    background: none; border: var(--stroke-w) solid var(--accent); box-shadow: var(--shadow-sm);
    color: var(--accent); font-size: 0.68rem; font-weight: 700; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.3rem 0.7rem; cursor: pointer; transition: all 0.08s;
  }
  .play-all-btn:hover { background: var(--accent); color: var(--on-accent); transform: translate(-1px,-1px); box-shadow: var(--shadow); }

  .track-list { display: flex; flex-direction: column; gap: 2px; padding: 0.5rem 0.5rem; overflow-y: auto; }
  .track-row { display: flex; align-items: center; gap: 0.25rem; }
  .track-num {
    width: 2rem; text-align: center; flex-shrink: 0;
    font-size: 0.68rem; font-weight: 700; color: var(--bg-card-hover); letter-spacing: 0.05em;
  }
  .track-card-wrap { flex: 1; min-width: 0; }
  .remove-btn {
    background: none; border: 1px solid transparent;
    color: transparent; font-size: 0.7rem; font-weight: 700; font-family: inherit;
    padding: 0.3rem; flex-shrink: 0; cursor: pointer; transition: all 0.08s;
  }
  .track-row:hover .remove-btn { color: var(--bg-card-hover); border-color: transparent; }
  .remove-btn:hover { color: var(--gb-red) !important; border-color: var(--gb-red) !important; }

  /* Empty */
  .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    flex: 1; gap: 1rem; padding: 3rem; text-align: center; min-height: 250px;
  }
  .ascii-note { font-size: 0.6rem; color: var(--bg-card-hover); line-height: 1.4; }
  .empty-msg { font-size: 0.82rem; font-weight: 700; color: var(--accent); letter-spacing: 0.06em; }
  .empty-sub { font-size: 0.68rem; color: var(--text-muted); }
</style>
