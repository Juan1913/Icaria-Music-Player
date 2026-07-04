<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import BottomNav from '$lib/components/layout/BottomNav.svelte';
  import PlayerBar from '$lib/components/layout/PlayerBar.svelte';
  import HomeView from '$lib/components/views/HomeView.svelte';
  import SearchView from '$lib/components/views/SearchView.svelte';
  import LibraryView from '$lib/components/views/LibraryView.svelte';
  import SettingsView from '$lib/components/views/SettingsView.svelte';
  import ArtistView from '$lib/components/views/ArtistView.svelte';
  import AlbumView from '$lib/components/views/AlbumView.svelte';
  import { settings } from '$lib/stores/settings';
  import { nav } from '$lib/stores/nav';
  import { theme } from '$lib/stores/theme';
  import { player } from '$lib/stores/player';
  import NowPlayingView from '$lib/components/views/NowPlayingView.svelte';

  let isMobile = $state(false);

  function checkMobile() {
    isMobile = window.innerWidth < 768;
  }

  onMount(() => {
    checkMobile();
    settings.load();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  });

  // Apply theme class to document root
  $effect(() => {
    document.documentElement.setAttribute('data-theme', $theme);
  });

  // Auto-navegar a Now Playing cuando empieza una canción nueva
  let prevTrackId = $state<string | null>(null);
  $effect(() => {
    const id = $player.currentTrack?.id ?? null;
    if (id && id !== prevTrackId) {
      prevTrackId = id;
      nav.setPage('nowplaying');
    }
  });

  let activePage = $derived($nav.page);
  let artistBrowseId = $derived($nav.artistBrowseId);
  let albumBrowseId = $derived($nav.albumBrowseId);
</script>

<div class="app-root {isMobile ? 'mobile' : 'desktop'}">
  {#if !isMobile}
    <Sidebar />
  {/if}

  <main class="main-content">
    {#if activePage === 'home'}
      <HomeView />
    {:else if activePage === 'search'}
      <SearchView />
    {:else if activePage === 'artist' && artistBrowseId}
      <ArtistView browseId={artistBrowseId} />
    {:else if activePage === 'album' && albumBrowseId}
      <AlbumView browseId={albumBrowseId} />
    {:else if activePage === 'nowplaying'}
      <NowPlayingView />
    {:else if activePage === 'library'}
      <LibraryView />
    {:else if activePage === 'settings'}
      <SettingsView />
    {:else}
      <SearchView />
    {/if}
  </main>

  <div class="player-wrapper">
    <PlayerBar />
    {#if isMobile}
      <BottomNav />
    {/if}
  </div>
</div>

<style>
  /* ═══════════════════════════════════════════
     GROOVE DESIGN SYSTEM — 4 THEMES
     paper · dracula · gruvbox · jamaica
     ═══════════════════════════════════════════ */

  /* ── SHARED CONSTANTS ── */
  :global(:root) {
    --stroke-w:      2.5px;
    --stroke-heavy:  3px;
    --radius:        14px;
    --radius-sm:     8px;
    --radius-pill:   100px;
    --on-accent:     #ffffff;

    font-family: system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif;
    font-size: 17px;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
    color: var(--text-primary);
    background: var(--bg-primary);
  }

  /* ══════════════════════════════════════
     PAPER — warm cream + violet + green
     ══════════════════════════════════════ */
  :global(:root),
  :global([data-theme="paper"]) {
    --bg-primary:    #ede8dc;
    --bg-sidebar:    #e2dcd0;
    --bg-card:       #ffffff;
    --bg-card-hover: #e2dcd0;
    --text-primary:  #0d0d0d;
    --text-muted:    #6b6b6b;
    --text-dim:      #9a9a9a;
    --accent:        #5e3f7b;
    --accent-dim:    #452e5a;
    --accent-2:      #16a34a;
    --accent-2-dim:  #15803d;
    --accent-3:      #ec4899;
    --accent-3-dim:  #db2777;
    --stroke:        #0d0d0d;
    --shadow:        4px 4px 0 #0d0d0d;
    --shadow-sm:     3px 3px 0 #0d0d0d;
    --shadow-lg:     6px 6px 0 #0d0d0d;
    --shadow-accent: 4px 4px 0 #452e5a;
    /* legacy */
    --gb-bg-hard:    #e2dcd0; --gb-bg0: #ede8dc; --gb-bg1: #cfc9bc;
    --gb-bg2:        #cfc9bc; --gb-bg3: #bdb7aa;
    --gb-fg0:        #0d0d0d; --gb-fg:  #0d0d0d; --gb-fg2: #3a3a3a;
    --gb-fg3:        #6b6b6b; --gb-gray:#6b6b6b;
    --gb-yellow:     #5e3f7b; --gb-yellow-dim:#452e5a;
    --gb-orange:     #ec4899; --gb-orange-dim:#db2777;
    --gb-red:        #dc2626; --gb-red-dim:#b91c1c;
    --gb-green:      #16a34a; --gb-green-dim:#15803d;
    --gb-blue:       #5e3f7b; --gb-purple:#5e3f7b; --gb-aqua:#22d3ee;
    --dot-color:     rgba(0,0,0,0.06);
  }

  /* ══════════════════════════════════════
     DRACULA — fondo oscuro + purple + pink
     ══════════════════════════════════════ */
  :global([data-theme="dracula"]) {
    --bg-primary:    #282a36;
    --bg-sidebar:    #21222c;
    --bg-card:       #313341;
    --bg-card-hover: #44475a;
    --text-primary:  #f8f8f2;
    --text-muted:    #8b9dc0;
    --text-dim:      #6272a4;
    --accent:        #bd93f9;
    --accent-dim:    #9d6fdf;
    --accent-2:      #4ea672;
    --accent-2-dim:  #3e855b;
    --accent-3:      #ff79c6;
    --accent-3-dim:  #df59a6;
    --stroke:        #6272a4;
    --shadow:        4px 4px 0 #6272a4;
    --shadow-sm:     3px 3px 0 #6272a4;
    --shadow-lg:     6px 6px 0 #6272a4;
    --shadow-accent: 4px 4px 0 #9d6fdf;
    /* legacy */
    --gb-bg-hard:    #21222c; --gb-bg0: #282a36; --gb-bg1: #313341;
    --gb-bg2:        #44475a; --gb-bg3: #4d5068;
    --gb-fg0:        #f8f8f2; --gb-fg:  #f8f8f2; --gb-fg2: #e0e0dc;
    --gb-fg3:        #8b9dc0; --gb-gray:#6272a4;
    --gb-yellow:     #bd93f9; --gb-yellow-dim:#9d6fdf;
    --gb-orange:     #ff79c6; --gb-orange-dim:#df59a6;
    --gb-red:        #ff5555; --gb-red-dim:#df3535;
    --gb-green:      #4ea672; --gb-green-dim:#3e855b;
    --gb-blue:       #8be9fd; --gb-purple:#bd93f9; --gb-aqua:#8be9fd;
    --dot-color:     rgba(255,255,255,0.05);
  }

  /* ══════════════════════════════════════
     GRUVBOX DARK — warm dark + amber + green
     ══════════════════════════════════════ */
  :global([data-theme="gruvbox"]) {
    --bg-primary:    #282828;
    --bg-sidebar:    #1d2021;
    --bg-card:       #3c3836;
    --bg-card-hover: #504945;
    --text-primary:  #ebdbb2;
    --text-muted:    #928374;
    --text-dim:      #665c54;
    --accent:        #d79921;
    --accent-dim:    #b57614;
    --accent-2:      #98971a;
    --accent-2-dim:  #79740e;
    --accent-3:      #fb4934;
    --accent-3-dim:  #cc241d;
    --stroke:        #a89984;
    --shadow:        4px 4px 0 #a89984;
    --shadow-sm:     3px 3px 0 #a89984;
    --shadow-lg:     6px 6px 0 #a89984;
    --shadow-accent: 4px 4px 0 #b57614;
    /* legacy */
    --gb-bg-hard:    #1d2021; --gb-bg0: #282828; --gb-bg1: #3c3836;
    --gb-bg2:        #504945; --gb-bg3: #665c54;
    --gb-fg0:        #ebdbb2; --gb-fg:  #ebdbb2; --gb-fg2: #d5c4a1;
    --gb-fg3:        #928374; --gb-gray:#928374;
    --gb-yellow:     #d79921; --gb-yellow-dim:#b57614;
    --gb-orange:     #fe8019; --gb-orange-dim:#d65d0e;
    --gb-red:        #fb4934; --gb-red-dim:#cc241d;
    --gb-green:      #b8bb26; --gb-green-dim:#98971a;
    --gb-blue:       #83a598; --gb-purple:#d3869b; --gb-aqua:#8ec07c;
    --dot-color:     rgba(255,255,255,0.05);
  }

  /* ══════════════════════════════════════
     JAMAICA — crema + amarillo + verde + negro
     ══════════════════════════════════════ */
  :global([data-theme="jamaica"]) {
    --bg-primary:    #f5ecd8;
    --bg-sidebar:    #ece3cc;
    --bg-card:       #fffbf0;
    --bg-card-hover: #e5dcc8;
    --text-primary:  #0d0d0d;
    --text-muted:    #4a4030;
    --text-dim:      #8a7e6a;
    --accent:        #FFD100;
    --accent-dim:    #d4ac00;
    --accent-2:      #009B45;
    --accent-2-dim:  #007534;
    --accent-3:      #E63328;
    --accent-3-dim:  #b82219;
    --stroke:        #0d0d0d;
    --shadow:        4px 4px 0 #0d0d0d;
    --shadow-sm:     3px 3px 0 #0d0d0d;
    --shadow-lg:     6px 6px 0 #0d0d0d;
    --shadow-accent: 4px 4px 0 #d4ac00;
    /* legacy */
    --gb-bg-hard:    #ece3cc; --gb-bg0: #f5ecd8; --gb-bg1: #e0d8c0;
    --gb-bg2:        #e0d8c0; --gb-bg3: #ccc4ac;
    --gb-fg0:        #0d0d0d; --gb-fg:  #0d0d0d; --gb-fg2: #2a2010;
    --gb-fg3:        #4a4030; --gb-gray:#7a6e58;
    --gb-yellow:     #FFD100; --gb-yellow-dim:#d4ac00;
    --gb-orange:     #FFD100; --gb-orange-dim:#d4ac00;
    --gb-red:        #E63328; --gb-red-dim:#b82219;
    --gb-green:      #009B45; --gb-green-dim:#007534;
    --gb-blue:       #009B45; --gb-purple:#FFD100; --gb-aqua:#00aa88;
    --dot-color:     rgba(0,0,0,0.07);
  }

  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    overflow: hidden; background: var(--bg-primary);
    background-image: radial-gradient(circle, var(--dot-color, rgba(0,0,0,0.05)) 1.5px, transparent 1.5px);
    background-size: 22px 22px;
  }

  /* Scrollbar */
  :global(::-webkit-scrollbar) { width: 5px; }
  :global(::-webkit-scrollbar-track) { background: var(--bg-sidebar); }
  :global(::-webkit-scrollbar-thumb) {
    background: var(--paper-5, var(--bg-card-hover));
    border-radius: 10px;
  }

  :global(button) { cursor: pointer; font-family: inherit; font-size: inherit; }

  /* Utility card */
  :global(.nb-card) {
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
  }
  :global(.nb-btn) {
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
    transition: box-shadow 0.12s, transform 0.1s;
  }
  :global(.nb-btn:hover) { box-shadow: var(--shadow); transform: translate(-2px, -2px); }
  :global(.nb-btn:active) { transform: translate(0, 0); box-shadow: 1px 1px 0 var(--stroke); }

  :global(input[type="text"], input[type="search"], input[type="password"]) {
    font-family: inherit;
    border: var(--stroke-w) solid var(--paper-4, var(--bg-card-hover));
    background: var(--bg-card);
    color: var(--text-primary);
    outline: none;
    border-radius: var(--radius-sm);
  }
  :global(input:focus) {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(147,51,234,0.15);
  }
  :global(hr) { border: none; border-top: 1px solid var(--paper-4, var(--bg-card-hover)); }

  /* ── Layout ── */
  .app-root {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .desktop { flex-direction: row; }
  .desktop .main-content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
    padding-bottom: 120px;
    position: relative;
  }
  .desktop .player-wrapper {
    position: fixed;
    bottom: 0; left: 0; right: 0;
    z-index: 100;
  }
  .mobile { flex-direction: column; }
  .mobile .main-content { flex: 1; overflow-y: auto; min-width: 0; }
  .mobile .player-wrapper { flex-shrink: 0; }

</style>
