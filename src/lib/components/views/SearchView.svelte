<script lang="ts">
  import { searchYouTubeAll, preloadStream, resolveStream } from '$lib/api';
  import TrackCard from '$lib/components/TrackCard.svelte';
  import type { Track, Artist, Album, SearchResult } from '$lib/stores/player';
  import { player } from '$lib/stores/player';
  import { nav } from '$lib/stores/nav';
  import { imgFallback } from '$lib/imgFallback';

  import { onMount } from 'svelte';

  let query = $state($nav.searchQuery ?? '');
  let results = $state<SearchResult | null>(null);
  let searchError = $state<string | null>(null);
  let loading = $state(false);
  let searched = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout>;
  let activeTab = $state<'all' | 'songs' | 'artists' | 'albums'>('all');

  onMount(() => {
    if (query.trim()) {
      doSearch();
    }
    nav.clearSearchQuery();
  });

  const currentId = $derived($player.currentTrack?.streamId);

  async function doSearch() {
    const q = query.trim();
    if (!q) { results = null; searched = false; searchError = null; return; }
    loading = true;
    searched = true;
    searchError = null;
    try {
      const r = await searchYouTubeAll(q, 8);
      results = r;
      // Preload all tracks in parallel — fire-and-forget, no UI impact
      for (const track of r.tracks) {
        preloadStream(track);
      }
    } catch (e) {
      results = null;
      searchError = String(e);
    } finally {
      loading = false;
    }
  }

  function onInput() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(doSearch, 350);
  }

  async function playAll() {
    if (!results || !results.tracks.length) return;
    player.setQueue(results.tracks, 0);
    player.setTrackLoading(results.tracks[0]);
    try {
      const stream = await resolveStream(results.tracks[0]);
      player.setStreamUrl(stream.url);
    } catch {}
  }

  function openArtist(artist: Artist) {
    nav.openArtist(artist.browseId);
  }

  function openAlbum(album: Album) {
    nav.openAlbum(album.browseId);
  }

  let allTracks = $derived(results?.tracks ?? []);
  let allArtists = $derived(results?.artists ?? []);
  let allAlbums = $derived(results?.albums ?? []);
  let hasResults = $derived(allTracks.length > 0 || allArtists.length > 0 || allAlbums.length > 0);
</script>

<div class="search-view">
  <div class="search-wrap">
    <span class="search-prefix">◎</span>
    <input
      type="search"
      class="search-input"
      placeholder="Artista, canción, álbum..."
      bind:value={query}
      oninput={onInput}
      onkeydown={e => e.key === 'Enter' && doSearch()}
    />
    {#if loading}
      <span class="loading-ind">···</span>
    {:else if query}
      <button class="clear-btn" onclick={() => { query = ''; results = null; searched = false; }}>×</button>
    {/if}
  </div>

  <div class="results-area">
    {#if !searched && !loading}
      <div class="empty-state">
        <div class="empty-icon">♪</div>
        <p class="empty-msg">Busca lo que quieras</p>
        <p class="empty-sub">Sin anuncios · Sin cuenta · YouTube</p>
      </div>
    {:else if loading && !results}
      <div class="skeleton-list">
        {#each Array(10) as _}
          <div class="skeleton-row">
            <div class="sk-art"></div>
            <div class="sk-lines">
              <div class="sk-line" style="width:{45 + Math.random() * 35}%"></div>
              <div class="sk-line narrow"></div>
            </div>
          </div>
        {/each}
      </div>

    {:else if searchError}
      <div class="empty-state">
        <div class="empty-icon">!</div>
        <p class="empty-msg">Error al buscar</p>
        <p class="empty-sub">{searchError}</p>
      </div>
    {:else if !hasResults}
      <div class="empty-state">
        <div class="empty-icon">◎</div>
        <p class="empty-msg">Sin resultados para "{query}"</p>
      </div>

    {:else}
      <div class="tabs-bar">
        <button class="tab-btn" class:active={activeTab === 'all'} onclick={() => activeTab = 'all'}>Todo</button>
        <button class="tab-btn" class:active={activeTab === 'songs'} onclick={() => activeTab = 'songs'}>
          Canciones{allTracks.length ? ` (${allTracks.length})` : ''}
        </button>
        <button class="tab-btn" class:active={activeTab === 'artists'} onclick={() => activeTab = 'artists'}>
          Artistas{allArtists.length ? ` (${allArtists.length})` : ''}
        </button>
        <button class="tab-btn" class:active={activeTab === 'albums'} onclick={() => activeTab = 'albums'}>
          Álbumes{allAlbums.length ? ` (${allAlbums.length})` : ''}
        </button>
        {#if activeTab === 'all' || activeTab === 'songs'}
          <div class="tabs-spacer"></div>
          <button class="play-all-btn" onclick={playAll}>▶ Reproducir todo</button>
        {/if}
      </div>

      {#if (activeTab === 'all' || activeTab === 'artists') && allArtists.length}
        <section class="section">
          <div class="section-head">
            <span class="section-label">Artistas</span>
            <div class="section-rule"></div>
          </div>
          <div class="artists-row">
            {#each allArtists as artist (artist.id)}
              <button class="artist-card" onclick={() => openArtist(artist)}>
                {#if artist.thumbnail}
                  <img class="artist-thumb" src={artist.thumbnail} alt="" loading="lazy" use:imgFallback />
                {:else}
                  <div class="artist-thumb artist-thumb-empty">♪</div>
                {/if}
                <span class="artist-name">{artist.name}</span>
                {#if artist.subscribers}
                  <span class="artist-subs">{artist.subscribers}</span>
                {/if}
              </button>
            {/each}
          </div>
        </section>
      {/if}

      {#if (activeTab === 'all' || activeTab === 'albums') && allAlbums.length}
        <section class="section">
          <div class="section-head">
            <span class="section-label">Álbumes</span>
            <div class="section-rule"></div>
          </div>
          <div class="albums-grid">
            {#each allAlbums as album (album.id)}
              <button class="album-card" onclick={() => openAlbum(album)}>
                {#if album.thumbnail}
                  <img class="album-art" src={album.thumbnail} alt="" loading="lazy" use:imgFallback />
                {:else}
                  <div class="album-art album-art-empty">♪</div>
                {/if}
                <div class="album-meta">
                  <span class="album-title">{album.title}</span>
                  <span class="album-artist">{album.artist} {album.year ? `· ${album.year}` : ''}</span>
                </div>
              </button>
            {/each}
          </div>
        </section>
      {/if}

      {#if (activeTab === 'all' || activeTab === 'songs') && allTracks.length}
        <section class="section">
          <div class="section-head">
            <span class="section-label">Canciones</span>
            <div class="section-rule"></div>
          </div>
          <div class="track-list">
            {#each allTracks as track (track.id)}
              <TrackCard {track} compact playing={track.streamId === currentId} />
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  </div>
</div>

<style>
  .search-view { display: flex; flex-direction: column; height: 100%; }

  .search-wrap {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 1.1rem 1.5rem 0.75rem;
    border: var(--stroke-heavy) solid var(--stroke);
    background: var(--bg-card);
    border-radius: var(--radius);
    padding: 0.8rem 1.3rem;
    box-shadow: var(--shadow);
    transition: border-color 0.12s, box-shadow 0.12s;
  }
  .search-wrap:focus-within {
    border-color: var(--accent);
    box-shadow: var(--shadow-accent);
  }
  .search-prefix { color: var(--accent); font-size: 1.1rem; flex-shrink: 0; }
  .search-input {
    flex: 1; background: none; border: none !important; box-shadow: none !important;
    outline: none; color: var(--text-primary); font-size: 1rem; font-family: inherit;
    font-weight: 600; caret-color: var(--accent);
  }
  .search-input::placeholder { color: var(--text-dim); font-weight: 400; }
  .loading-ind {
    color: var(--accent); font-size: 1rem; font-weight: 700;
    animation: blink 0.6s step-end infinite; flex-shrink: 0;
  }
  @keyframes blink { 50% { opacity: 0; } }
  .clear-btn {
    background: var(--bg-card); border: var(--stroke-w) solid var(--stroke); color: var(--text-muted);
    font-size: 1rem; font-weight: 700; width: 26px; height: 26px;
    border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center;
    transition: all 0.1s; flex-shrink: 0; padding: 0;
    box-shadow: var(--shadow-sm);
  }
  .clear-btn:hover { color: var(--text-primary); background: var(--paper-4, var(--bg-card-hover)); border-color: var(--stroke); }

  .results-area { flex: 1; overflow-y: auto; padding-bottom: 1rem; }

  .tabs-bar {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.08);
    flex-wrap: wrap;
  }
  .tab-btn {
    background: var(--bg-card); border: var(--stroke-w) solid var(--stroke);
    color: var(--text-muted); font-size: 0.8rem; font-weight: 700;
    font-family: inherit; padding: 0.4rem 1rem;
    cursor: pointer; transition: all 0.1s; border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
  }
  .tab-btn.active { background: var(--accent); border-color: var(--stroke); color: #ffffff; box-shadow: var(--shadow-accent); }
  .tab-btn:hover:not(.active) { transform: translate(-1px, -1px); box-shadow: var(--shadow); color: var(--text-primary); }
  .tabs-spacer { flex: 1; }
  .play-all-btn {
    background: var(--green, var(--accent-2)); border: var(--stroke-w) solid var(--stroke);
    border-radius: var(--radius-sm); color: #ffffff; font-size: 0.8rem; font-weight: 700;
    font-family: inherit; padding: 0.4rem 1rem; box-shadow: var(--shadow-sm);
    transition: all 0.1s; cursor: pointer;
  }
  .play-all-btn:hover { transform: translate(-2px, -2px); box-shadow: var(--shadow); }
  .play-all-btn:active { transform: translate(0, 0); box-shadow: var(--shadow-sm); }

  .section { padding: 1.5rem 1.5rem 0.75rem; }
  .section-head { display: flex; align-items: center; gap: 0.85rem; margin-bottom: 1rem; }
  .section-label {
    font-size: 0.72rem; font-weight: 900; letter-spacing: 0.1em; color: var(--text-primary);
    white-space: nowrap; text-transform: uppercase;
    border: var(--stroke-w) solid var(--stroke); padding: 0.25rem 0.75rem;
    border-radius: var(--radius-sm); background: var(--bg-card); box-shadow: var(--shadow-sm);
  }
  .section-rule { flex: 1; height: 2px; background: var(--stroke); opacity: 0.15; }

  .track-list { padding: 0 0.25rem; }

  .artists-row { display: flex; gap: 1rem; overflow-x: auto; padding-bottom: 0.75rem; }
  .artist-card {
    display: flex; flex-direction: column; align-items: center; gap: 0.6rem;
    min-width: 115px; max-width: 135px; flex-shrink: 0;
    background: var(--bg-card); border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius); box-shadow: var(--shadow);
    padding: 1.1rem 0.6rem; cursor: pointer; transition: all 0.1s; text-align: center;
  }
  .artist-card:hover { transform: translate(-2px, -2px); box-shadow: var(--shadow-lg); }
  .artist-card:active { transform: translate(0, 0); box-shadow: var(--shadow-sm); }
  .artist-thumb { width: 76px; height: 76px; border-radius: 50%; object-fit: cover; border: var(--stroke-w) solid var(--stroke); }
  .artist-thumb-empty {
    width: 76px; height: 76px;
    display: flex; align-items: center; justify-content: center;
    background: var(--paper-4, var(--bg-card)); font-size: 1.4rem; color: var(--text-muted);
    border-radius: 50%; border: var(--stroke-w) solid var(--stroke);
  }
  .artist-name { font-size: 0.8rem; font-weight: 700; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }
  .artist-subs { font-size: 0.67rem; color: var(--text-muted); font-weight: 500; }

  .albums-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(155px, 1fr)); gap: 1rem; }
  .album-card {
    display: flex; flex-direction: column; background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius); box-shadow: var(--shadow);
    cursor: pointer; text-align: left; transition: all 0.1s; padding: 0; overflow: hidden;
  }
  .album-card:hover { transform: translate(-2px, -2px); box-shadow: var(--shadow-lg); }
  .album-card:active { transform: translate(0, 0); }
  .album-art { width: 100%; aspect-ratio: 1; object-fit: cover; display: block; }
  .album-art-empty {
    width: 100%; aspect-ratio: 1;
    display: flex; align-items: center; justify-content: center;
    background: var(--paper-4, var(--bg-card)); font-size: 2rem; color: var(--text-muted);
  }
  .album-meta { padding: 0.65rem 0.8rem; }
  .album-title { display: block; font-size: 0.85rem; font-weight: 700; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 3px; }
  .album-artist { font-size: 0.75rem; color: var(--accent); font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: block; }

  .skeleton-list { display: flex; flex-direction: column; gap: 6px; padding: 1rem 1.5rem; }
  .skeleton-row { display: flex; align-items: center; gap: 0.85rem; padding: 0.4rem 0; }
  .sk-art { width: 54px; height: 54px; flex-shrink: 0; background: var(--paper-4, var(--bg-card)); border-radius: var(--radius-sm); animation: shimmer 1.5s infinite; border: 2px solid var(--paper-4, #ccc); }
  .sk-lines { flex: 1; display: flex; flex-direction: column; gap: 8px; }
  .sk-line { height: 10px; background: var(--paper-4, var(--bg-card)); border-radius: 4px; animation: shimmer 1.5s infinite; }
  .sk-line.narrow { width: 30%; }
  @keyframes shimmer { 0%,100%{opacity:0.4} 50%{opacity:0.85} }

  .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 100%; min-height: 300px; gap: 1rem; padding: 2.5rem;
  }
  .empty-icon { font-size: 3.5rem; color: var(--paper-4, var(--bg-card-hover)); }
  .empty-msg { font-size: 1.1rem; font-weight: 800; color: var(--text-primary); text-align: center; }
  .empty-sub { font-size: 0.85rem; color: var(--text-muted); text-align: center; font-weight: 500; }
</style>
