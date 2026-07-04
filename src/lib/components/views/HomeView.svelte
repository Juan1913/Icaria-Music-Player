<script lang="ts">
  import { searchYouTubeAll, resolveStream } from '$lib/api';
  import { player } from '$lib/stores/player';
  import type { Track } from '$lib/stores/player';
  import { nav } from '$lib/stores/nav';

  const CATEGORIES = [
    { id: 'Todo',        label: 'Todo'        },
    { id: 'Pop',         label: 'Pop'         },
    { id: 'Rock',        label: 'Rock'        },
    { id: 'Hip-Hop',     label: 'Hip-Hop'     },
    { id: 'Electrónica', label: 'Electrónica' },
    { id: 'Jazz',        label: 'Jazz'        },
    { id: 'Relax',       label: 'Relax'       },
    { id: 'Clásica',     label: 'Clásica'     },
  ];

  const GENRE_QUERIES: Record<string, string> = {
    'Todo':        'top hits music 2025',
    'Pop':         'top pop songs 2025',
    'Rock':        'best rock songs 2025',
    'Hip-Hop':     'top hip hop rap songs 2025',
    'Electrónica': 'best electronic dance music 2025',
    'Jazz':        'best jazz music playlist',
    'Relax':       'relaxing chill music playlist',
    'Clásica':     'classical music masterpieces',
  };

  let activeCategory = $state('Todo');

  let tracks  = $state<Track[]>([]);
  let loading = $state(true);

  async function fetchForCategory(cat: string) {
    loading = true;
    try {
      const r = await searchYouTubeAll(GENRE_QUERIES[cat] ?? GENRE_QUERIES['Todo'], 20);
      tracks = r.tracks;
    } catch {
      tracks = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    fetchForCategory(activeCategory);
  });

  function selectCategory(catId: string) {
    if (catId === activeCategory) return;
    activeCategory = catId;
  }

  const featured = $derived(tracks.slice(0, 3));
  const popular  = $derived(tracks.slice(3));

  async function playTrack(track: Track) {
    player.setTrackLoading(track);
    player.addToQueue(track);
    try {
      const stream = await resolveStream(track);
      player.setStreamUrl(stream.url);
    } catch (e) { player.setError(`ERROR: ${e}`); }
  }

  function isActive(t: Track) {
    return $player.currentTrack?.id === t.id;
  }
</script>

<div class="home-view">
  <!-- Topbar -->
  <div class="topbar">
    <h1 class="page-title">Inicio</h1>
    <button class="search-fab" onclick={() => nav.openSearch('')} title="Buscar">
      <svg width="19" height="19" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
        <circle cx="8.5" cy="8.5" r="5.5"/>
        <line x1="13" y1="13" x2="18" y2="18"/>
      </svg>
    </button>
  </div>

  <div class="scroll-area">

    <!-- ── FEATURED: 3 cards verticales portrait ── -->
    <section class="section">
      <div class="section-head">
        <span class="section-icon">✦</span>
        <span class="section-title">Destacados</span>
        <div class="section-rule"></div>
        <button class="ver-todo" onclick={() => {}}>VER TODO &rsaquo;</button>
      </div>

      <div class="featured-row">
        {#if loading}
          {#each [1,2,3] as _}
            <div class="feat-skeleton"></div>
          {/each}
        {:else}
          {#each featured as track (track.id)}
            {@const playing = isActive(track)}
            <button
              class="feat-card {playing ? 'playing' : ''}"
              onclick={() => playTrack(track)}
            >
              <div class="feat-img-half">
                {#if track.thumbnail}
                  <img class="feat-img" src={track.thumbnail} alt="" />
                {:else}
                  <div class="feat-img feat-empty">
                    <div class="empty-dots"></div>
                    <span class="empty-g">G</span>
                  </div>
                {/if}
              </div>

              <div class="feat-info-half">
                <div class="feat-info">
                  <span class="feat-artist-tag">{track.artist}</span>
                  <div class="feat-accent-line"></div>
                  <span class="feat-title">{track.title}</span>
                </div>
                <div class="feat-play">
                  {playing && $player.isPlaying ? '▮▮' : '▶'}
                </div>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </section>

    <!-- ── CATEGORÍAS ── -->
    <section class="section">
      <div class="section-head">
        <span class="section-icon cat-icon-dot">●</span>
        <span class="section-title">Categorías</span>
        <div class="section-rule"></div>
        <button class="ver-todo" onclick={() => {}}>VER TODO &rsaquo;</button>
      </div>
      <div class="cats-row">
        {#each CATEGORIES as cat}
          <button
            class="cat-pill {activeCategory === cat.id ? 'active' : ''}"
            onclick={() => selectCategory(cat.id)}
          >{cat.label}</button>
        {/each}
      </div>
    </section>

    <!-- ── CANCIONES POPULARES (horizontal scroll) ── -->
    <section class="section section-last">
      <div class="section-head">
        <span class="section-icon fire-icon">🔥</span>
        <span class="section-title">Canciones populares</span>
        <div class="section-rule"></div>
        <button class="ver-todo" onclick={() => {}}>VER TODO &rsaquo;</button>
      </div>
      <div class="popular-row">
        {#if loading}
          {#each [1,2,3,4,5,6] as _}
            <div class="pop-skeleton"></div>
          {/each}
        {:else}
          {#each popular as track (track.id)}
            {@const playing = isActive(track)}
            <button
              class="pop-card {playing ? 'playing' : ''}"
              onclick={() => playTrack(track)}
            >
              <div class="pop-art-wrap">
                {#if track.thumbnail}
                  <img class="pop-art" src={track.thumbnail} alt="" />
                {:else}
                  <div class="pop-art pop-empty">
                    <div class="empty-dots"></div>
                    <span class="empty-g-sm">G</span>
                  </div>
                {/if}
                <div class="pop-overlay">{playing && $player.isPlaying ? '▮▮' : '▶'}</div>
              </div>
              <div class="pop-meta">
                <span class="pop-title">{track.title}</span>
                <span class="pop-artist">{track.artist}</span>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </section>

  </div>
</div>

<style>
  .home-view { display: flex; flex-direction: column; height: 100%; }
  .scroll-area { flex: 1; overflow-y: auto; }

  /* ── Topbar ── */
  .topbar {
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 1.5rem 1.5rem 1rem; flex-shrink: 0;
  }
  .page-title {
    font-size: 3.2rem; font-weight: 900; color: var(--text-primary);
    letter-spacing: -0.04em; line-height: 1;
  }
  .search-fab {
    width: 42px; height: 42px; border-radius: 50%;
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    display: flex; align-items: center; justify-content: center;
    cursor: pointer; flex-shrink: 0; margin-top: 4px;
    color: var(--accent);
    transition: transform 0.12s, box-shadow 0.12s;
  }
  .search-fab:hover { transform: translate(-2px, -2px); box-shadow: var(--shadow); }
  .search-fab:active { transform: translate(0, 0); box-shadow: 1px 1px 0 var(--stroke); }

  /* ── Sections ── */
  .section { padding: 0.75rem 1.5rem 0.5rem; }
  .section-last { padding-bottom: 1.25rem; }
  .section-head { display: flex; align-items: center; gap: 0.55rem; margin-bottom: 1rem; }
  .section-icon { font-size: 0.82rem; color: var(--accent); flex-shrink: 0; line-height: 1; }
  .cat-icon-dot { color: #f97316; font-size: 0.7rem; }
  .fire-icon { font-size: 0.9rem; }
  .section-title {
    font-size: 0.95rem; font-weight: 900; color: var(--text-primary);
    white-space: nowrap; letter-spacing: 0.01em;
  }
  .section-rule { flex: 1; height: 1px; background: var(--stroke); opacity: 0.12; }
  .ver-todo {
    font-size: 0.68rem; font-weight: 800; color: var(--text-muted);
    background: none; border: none; cursor: pointer; white-space: nowrap;
    font-family: inherit; padding: 0; transition: color 0.12s; letter-spacing: 0.05em;
  }
  .ver-todo:hover { color: var(--accent); }

  /* ── Featured cards (landscape 3:2) ── */
  .featured-row {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem;
  }
  .feat-card {
    display: flex; align-items: stretch;
    position: relative; aspect-ratio: 3 / 2; overflow: hidden;
    border-radius: var(--radius);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow);
    cursor: pointer; padding: 0; background: var(--bg-card);
    transition: transform 0.15s, box-shadow 0.15s;
  }
  .feat-card:hover { transform: translate(-3px, -3px); box-shadow: var(--shadow-lg); }
  .feat-card:active { transform: translate(0, 0); box-shadow: 1px 1px 0 var(--stroke); }
  .feat-card.playing {
    border-color: var(--accent);
    box-shadow: var(--shadow-accent);
  }

  /* left half: artwork */
  .feat-img-half { position: relative; flex: 1 1 50%; min-width: 0; overflow: hidden; }
  .feat-img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; display: block; }
  .feat-empty {
    position: absolute; inset: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card-hover);
  }
  .empty-dots {
    position: absolute; inset: 0;
    background-image: radial-gradient(circle, rgba(0,0,0,0.1) 2px, transparent 2px);
    background-size: 14px 14px;
  }
  .empty-g {
    font-size: 3rem; font-weight: 900; color: var(--text-dim);
    position: relative; z-index: 1; letter-spacing: -0.04em;
  }

  /* right half: color panel, themed per card */
  .feat-info-half {
    flex: 1 1 50%; min-width: 0;
    display: flex; flex-direction: column; justify-content: center;
    padding: 1rem 0.9rem;
  }
  .feat-card:nth-child(1) .feat-info-half { background: var(--accent); }
  .feat-card:nth-child(2) .feat-info-half { background: var(--accent-2); }
  .feat-card:nth-child(3) .feat-info-half { background: var(--accent-3); }

  .feat-info { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .feat-artist-tag {
    font-size: 0.6rem; font-weight: 800; color: rgba(255,255,255,0.85);
    letter-spacing: 0.1em; text-transform: uppercase;
    text-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }
  .feat-accent-line {
    height: 2px; width: 24px; background: rgba(255,255,255,0.55);
    border-radius: 1px; margin: 3px 0;
  }
  .feat-title {
    font-size: 1.05rem; font-weight: 900; color: #ffffff; line-height: 1.2;
    display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical;
    overflow: hidden; text-shadow: 0 1px 4px rgba(0,0,0,0.25);
  }
  .feat-play {
    flex-shrink: 0; width: 40px; height: 40px; border-radius: 50%;
    background: #ffffff; border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    display: flex; align-items: center; justify-content: center;
    font-size: 0.85rem; font-weight: 900; color: #0d0d0d;
    align-self: flex-end; margin-top: 0.75rem;
    transition: transform 0.12s, box-shadow 0.12s;
  }
  .feat-card:hover .feat-play { transform: translate(-1px, -1px); box-shadow: var(--shadow); }
  .feat-skeleton {
    aspect-ratio: 3 / 2; background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke); border-radius: var(--radius);
    animation: shimmer 1.5s infinite;
  }

  /* ── Category pills ── */
  .cats-row {
    display: flex; gap: 0.55rem; overflow-x: auto; padding-bottom: 0.2rem;
    scrollbar-width: none;
  }
  .cats-row::-webkit-scrollbar { display: none; }
  .cat-pill {
    flex-shrink: 0; padding: 0.42rem 1.15rem;
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius-pill);
    box-shadow: var(--shadow-sm);
    color: var(--text-muted);
    font-size: 0.82rem; font-weight: 700; font-family: inherit;
    cursor: pointer; white-space: nowrap;
    transition: transform 0.1s, box-shadow 0.1s, background 0.1s, color 0.1s;
  }
  .cat-pill:hover:not(.active) {
    background: var(--bg-card-hover); color: var(--text-primary);
    transform: translate(-2px, -2px); box-shadow: var(--shadow);
  }
  .cat-pill.active {
    background: var(--accent); color: #fff;
    font-weight: 900;
    box-shadow: var(--shadow-accent);
  }

  /* ── Popular songs (horizontal scroll) ── */
  .popular-row {
    display: flex; gap: 0.9rem; overflow-x: auto; padding-bottom: 0.5rem;
    scroll-snap-type: x mandatory; scrollbar-width: thin;
    scrollbar-color: var(--bg-card-hover) transparent;
  }
  .popular-row::-webkit-scrollbar { height: 4px; }
  .popular-row::-webkit-scrollbar-thumb { background: var(--bg-card-hover); border-radius: 2px; }
  .pop-card {
    flex-shrink: 0; scroll-snap-align: start; width: 148px;
    background: transparent; border: var(--stroke-heavy) solid transparent;
    border-radius: var(--radius); cursor: pointer; padding: 0; text-align: left;
    transition: border-color 0.12s, box-shadow 0.12s, transform 0.12s;
  }
  .pop-card:hover {
    border-color: var(--stroke);
    box-shadow: var(--shadow);
    transform: translate(-2px, -2px);
  }
  .pop-card:active { transform: translate(0, 0); box-shadow: none; }
  .pop-card.playing { border-color: var(--accent); box-shadow: var(--shadow-accent); }
  .pop-art-wrap {
    position: relative; width: 100%; aspect-ratio: 1;
    border-radius: calc(var(--radius) - 2px); overflow: hidden;
  }
  .pop-art { width: 100%; height: 100%; object-fit: cover; display: block; }
  .pop-empty {
    width: 100%; height: 100%; position: relative;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent);
  }
  .empty-g-sm {
    font-size: 2.2rem; font-weight: 900; color: #fff;
    position: relative; z-index: 1; text-shadow: 2px 2px 0 rgba(0,0,0,0.2);
  }
  .pop-overlay {
    position: absolute; inset: 0; background: rgba(0,0,0,0.48);
    display: flex; align-items: center; justify-content: center;
    font-size: 1.4rem; color: #fff; font-weight: 700;
    opacity: 0; transition: opacity 0.12s; border-radius: inherit;
  }
  .pop-card:hover .pop-overlay,
  .pop-card.playing .pop-overlay { opacity: 1; }
  .pop-meta {
    padding: 0.5rem 0.4rem 0.35rem; display: flex; flex-direction: column; gap: 2px;
  }
  .pop-title {
    font-size: 0.78rem; font-weight: 700; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; letter-spacing: -0.01em;
  }
  .pop-card.playing .pop-title { color: var(--accent); }
  .pop-artist {
    font-size: 0.68rem; color: var(--text-muted); font-weight: 500;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .pop-skeleton {
    flex-shrink: 0; width: 148px; aspect-ratio: 1;
    background: var(--bg-card); border-radius: var(--radius);
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer { 0%,100%{opacity:0.35} 50%{opacity:0.75} }
</style>
