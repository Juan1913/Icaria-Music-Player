<script lang="ts">
  import { getArtist, preloadStream, resolveStream } from '$lib/api';
  import { player } from '$lib/stores/player';
  import type { ArtistDetail, Track, Album } from '$lib/stores/player';
  import { nav } from '$lib/stores/nav';
  import { imgFallback } from '$lib/imgFallback';
  import TrackCard from '$lib/components/TrackCard.svelte';

  interface Props {
    browseId: string;
  }
  let { browseId }: Props = $props();

  let detail = $state<ArtistDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const currentId = $derived($player.currentTrack?.streamId);

  $effect(() => {
    const id = browseId;
    loading = true;
    error = null;
    detail = null;
    getArtist(id)
      .then(d => { detail = d; for (let i = 0; i < Math.min(3, d.tracks.length); i++) preloadStream(d.tracks[i]); })
      .catch(e => { error = String(e); })
      .finally(() => { loading = false; });
  });

  async function playTrack(track: Track) {
    player.setTrackLoading(track);
    try {
      const stream = await resolveStream(track);
      player.setStreamUrl(stream.url);
    } catch (e) {
      player.setError(`ERROR: ${e}`);
    }
  }

  async function playAll() {
    if (!detail || !detail.tracks.length) return;
    const track = detail.tracks[0];
    player.setQueue(detail.tracks, 0);
    player.setTrackLoading(track);
    try {
      const stream = await resolveStream(track);
      player.setStreamUrl(stream.url);
    } catch {}
  }

  function openAlbum(album: Album) {
    nav.openAlbum(album.browseId);
  }
</script>

<div class="artist-view">
  {#if loading}
    <div class="loading-state">
      <span class="loading-ind">···</span>
    </div>
  {:else if error}
    <div class="error-state">
      <p class="error-msg">!! ERROR: {error} !!</p>
    </div>
  {:else if detail}
    <div class="view-header">
      <button class="back-btn" onclick={() => nav.goBack()}>[← BACK]</button>
    </div>

    <div class="artist-hero">
      {#if detail.thumbnail}
        <img class="artist-art" src={detail.thumbnail} alt="" use:imgFallback />
      {:else}
        <div class="artist-art artist-art-empty">[♪]</div>
      {/if}
      <div class="artist-info">
        <h1 class="artist-name">{detail.name.toUpperCase()}</h1>
        {#if detail.subscribers}
          <span class="artist-subs">{detail.subscribers}</span>
        {/if}
        {#if detail.description}
          <p class="artist-desc">{detail.description.slice(0, 200)}{detail.description.length > 200 ? '...' : ''}</p>
        {/if}
        {#if detail.tracks.length}
          <button class="play-all-btn nb-btn" onclick={playAll}>▶ PLAY ALL</button>
        {/if}
      </div>
    </div>

    {#if detail.tracks.length}
      <section class="section">
        <div class="section-head">
          <span class="section-label">[ TOP SONGS ]</span>
          <div class="section-rule"></div>
        </div>
        <div class="track-list">
          {#each detail.tracks as track (track.id)}
            <TrackCard {track} compact playing={track.streamId === currentId} />
          {/each}
        </div>
      </section>
    {/if}

    {#if detail.albums.length}
      <section class="section">
        <div class="section-head">
          <span class="section-label">[ ALBUMS ]</span>
          <div class="section-rule"></div>
        </div>
        <div class="albums-grid">
          {#each detail.albums as album (album.id)}
            <button class="album-card" onclick={() => openAlbum(album)}>
              {#if album.thumbnail}
                <img class="album-art" src={album.thumbnail} alt="" loading="lazy" use:imgFallback />
              {:else}
                <div class="album-art album-art-empty">[♪]</div>
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
  {/if}
</div>

<style>
  .artist-view { display: flex; flex-direction: column; height: 100%; overflow-y: auto; }

  .view-header {
    padding: 1rem 1.25rem 0.75rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.08);
    display: flex; align-items: center; gap: 0.75rem;
  }
  .back-btn {
    background: none; border: var(--stroke-w) solid var(--bg-card-hover);
    color: var(--text-muted); font-size: 0.68rem; font-weight: 700;
    font-family: inherit; padding: 0.25rem 0.6rem; cursor: pointer; transition: all 0.08s;
  }
  .back-btn:hover { border-color: var(--stroke); color: var(--text-primary); }

  .loading-state, .error-state {
    display: flex; align-items: center; justify-content: center;
    height: 100%; min-height: 300px;
  }
  .loading-ind { color: var(--accent); font-size: 1.5rem; font-weight: 900; animation: blink 0.6s step-end infinite; }
  @keyframes blink { 50% { opacity: 0; } }
  .error-msg { color: var(--gb-red); font-size: 0.82rem; font-weight: 700; }

  .artist-hero {
    display: flex; align-items: center; gap: 1.25rem;
    padding: 1.25rem; box-shadow: 0 2px 10px rgba(0,0,0,0.08);
  }
  .artist-art {
    width: 120px; height: 120px; object-fit: cover; flex-shrink: 0;
    border: var(--stroke-heavy) solid var(--stroke); box-shadow: var(--shadow);
  }
  .artist-art-empty {
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card); font-size: 2rem; color: var(--text-muted);
  }
  .artist-info { display: flex; flex-direction: column; gap: 0.35rem; min-width: 0; }
  .artist-name {
    font-size: 1.4rem; font-weight: 900; color: var(--text-primary);
    letter-spacing: 0.04em; margin: 0;
  }
  .artist-subs { font-size: 0.72rem; color: var(--gb-gray); font-weight: 700; letter-spacing: 0.06em; }
  .artist-desc { font-size: 0.72rem; color: var(--gb-fg3); line-height: 1.4; margin: 0; }
  .play-all-btn {
    background: var(--gb-green-dim); border: var(--stroke-w) solid var(--gb-green);
    box-shadow: var(--shadow-sm); color: var(--on-accent); font-size: 0.72rem;
    font-weight: 700; font-family: inherit; letter-spacing: 0.06em; padding: 0.3rem 0.8rem;
    width: fit-content;
  }
  .play-all-btn:hover { transform: translate(2px, 2px); box-shadow: 0 0 0; }

  .section { padding: 1rem 1.25rem 0.5rem; }
  .section-head { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.75rem; }
  .section-label { font-size: 0.68rem; font-weight: 700; letter-spacing: 0.1em; color: var(--gb-gray); white-space: nowrap; }
  .section-rule { flex: 1; height: 1px; background: var(--bg-card); }

  .track-list { padding: 0 0.5rem; }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.75rem;
  }
  .album-card {
    display: flex; flex-direction: column; background: var(--bg-card);
    border: var(--stroke-w) solid var(--stroke); box-shadow: var(--shadow);
    cursor: pointer; text-align: left; transition: all 0.08s; padding: 0;
  }
  .album-card:hover { transform: translate(-2px,-2px); box-shadow: var(--shadow-lg); }
  .album-art {
    width: 100%; aspect-ratio: 1; object-fit: cover; display: block;
  }
  .album-art-empty {
    width: 100%; aspect-ratio: 1;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card); font-size: 1.5rem; color: var(--text-muted);
  }
  .album-meta { padding: 0.5rem; }
  .album-title {
    display: block; font-size: 0.75rem; font-weight: 700; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 2px;
  }
  .album-artist { font-size: 0.67rem; color: var(--gb-orange); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: block; }
</style>
