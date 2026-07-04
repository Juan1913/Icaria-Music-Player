<script lang="ts">
  import { playlists } from '$lib/stores/playlists';
  import { nav, type NavPage } from '$lib/stores/nav';
  import { theme, THEME_LABELS, THEME_COLORS, type Theme } from '$lib/stores/theme';

  const ALL_THEMES: Theme[] = ['paper', 'dracula', 'gruvbox', 'jamaica'];

  let activePage = $derived($nav.page);
  let showThemePopup = $state(false);

  const mainItems: { id: NavPage; label: string; icon: string }[] = [
    { id: 'home',   label: 'Inicio',    icon: '⌂' },
  ];
  const libItems: { id: NavPage; label: string; icon: string }[] = [
    { id: 'library', label: 'Biblioteca', icon: '▤' },
  ];

  function pickTheme(t: Theme) {
    theme.set(t);
    showThemePopup = false;
  }
</script>

<!-- Theme popup overlay -->
{#if showThemePopup}
  <div class="popup-backdrop" role="presentation" onclick={() => showThemePopup = false}></div>
  <div class="theme-popup">
    <p class="popup-title">TEMA</p>
    <div class="popup-grid">
      {#each ALL_THEMES as t}
        <button
          class="theme-card {$theme === t ? 'active' : ''}"
          style="--tc: {THEME_COLORS[t]}"
          onclick={() => pickTheme(t)}
        >
          <div class="card-dots"></div>
          <span class="card-swatch"></span>
          <span class="card-name">{THEME_LABELS[t]}</span>
          {#if $theme === t}<span class="card-check">✓</span>{/if}
        </button>
      {/each}
    </div>
  </div>
{/if}

<aside class="sidebar">
  <!-- Logo -->
  <div class="logo-area">
    <div class="logo-icon">♪</div>
    <div class="logo-name">
      <span class="logo-main">GROOVE</span>
      <span class="logo-sub">música libre</span>
    </div>
    <button class="bell-btn" onclick={() => showThemePopup = !showThemePopup} title="Cambiar tema">
      <svg class="bell-icon" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
        <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
      </svg>
      <span class="bell-dot" style="background:{THEME_COLORS[$theme]}"></span>
    </button>
  </div>

  <!-- Main nav -->
  <nav class="nav-section">
    {#each mainItems as item}
      <button
        class="nav-item {activePage === item.id ? 'active' : ''}"
        onclick={() => nav.setPage(item.id)}
      >
        <span class="nav-icon">{item.icon}</span>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Library -->
  <div class="section-header">
    <span>BIBLIOTECA</span>
    <div class="section-rule"></div>
  </div>

  <nav class="nav-section library-nav">
    {#each libItems as item}
      <button
        class="nav-item {activePage === item.id ? 'active' : ''}"
        onclick={() => nav.setPage(item.id)}
      >
        <span class="nav-icon">{item.icon}</span>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}

    {#if $playlists.length > 0}
      <div class="subsection-label">PLAYLISTS</div>
      {#each $playlists as pl (pl.id)}
        <button class="playlist-item" onclick={() => nav.setPage('library')} title={pl.name}>
          <span class="pl-dot">◈</span>
          <span class="pl-label">{pl.name}</span>
          <span class="pl-count">{pl.tracks.length}</span>
        </button>
      {/each}
    {/if}
  </nav>

  <div class="sidebar-spacer"></div>

  <div class="nav-section settings-section">
    <button
      class="nav-item {activePage === 'settings' ? 'active' : ''}"
      onclick={() => nav.setPage('settings')}
    >
      <span class="nav-icon">⚙</span>
      <span class="nav-label">Ajustes</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 260px;
    min-width: 260px;
    background: var(--bg-sidebar);
    background-image: radial-gradient(circle, var(--dot-color, rgba(0,0,0,0.06)) 1.5px, transparent 1.5px);
    background-size: 18px 18px;
    display: flex;
    flex-direction: column;
    border-right: var(--stroke-heavy) solid var(--stroke);
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
    z-index: 10;
  }

  /* Logo */
  .logo-area {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    height: 80px;
    padding: 0 1.25rem;
    background: var(--bg-sidebar);
    background-image: none;
    flex-shrink: 0;
  }
  .logo-icon {
    font-size: 1.5rem;
    width: 44px; height: 44px;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); color: #ffffff;
    border-radius: var(--radius-sm); flex-shrink: 0;
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-accent);
  }
  .logo-name { display: flex; flex-direction: column; gap: 2px; flex: 1; }
  .logo-main { font-size: 1.05rem; font-weight: 900; letter-spacing: 0.1em; color: var(--text-primary); line-height: 1; }
  .logo-sub { font-size: 0.62rem; letter-spacing: 0.04em; color: var(--text-muted); font-weight: 600; }

  /* Nav */
  .nav-section {
    display: flex; flex-direction: column;
    padding: 0.75rem 0.85rem 0; gap: 4px;
  }
  .settings-section { padding-bottom: 0.85rem; gap: 6px; }
  .library-nav { gap: 3px; }

  .nav-item {
    position: relative;
    display: flex; align-items: center; gap: 0.75rem;
    padding: 0.65rem 1rem;
    border: var(--stroke-heavy) solid transparent;
    background: transparent; color: var(--text-muted);
    font-size: 0.9rem; font-weight: 700; font-family: inherit;
    cursor: pointer; transition: all 0.15s;
    text-align: left; width: 100%; overflow: hidden;
    border-radius: var(--radius-sm);
  }
  .nav-item:hover {
    background: var(--bg-card);
    color: var(--text-primary);
    border-color: var(--stroke);
    box-shadow: var(--shadow-sm);
    transform: translate(-2px, -2px);
  }
  .nav-item.active {
    background: var(--accent);
    color: #ffffff;
    border-color: var(--stroke);
    box-shadow: var(--shadow-sm);
  }
  .nav-item.active:hover { opacity: 0.92; transform: none; }

  .nav-icon { font-size: 1rem; width: 1.2rem; text-align: center; flex-shrink: 0; line-height: 1; }
  .nav-label { flex: 1; }

  /* Section header */
  .section-header {
    display: flex; align-items: center; gap: 0.6rem;
    padding: 1rem 1rem 0.4rem;
    font-size: 0.6rem; font-weight: 800; letter-spacing: 0.12em;
    color: var(--text-dim); flex-shrink: 0; text-transform: uppercase;
  }
  .section-rule { flex: 1; height: 2px; background: var(--stroke); opacity: 0.15; }

  .subsection-label {
    font-size: 0.58rem; font-weight: 800; letter-spacing: 0.12em;
    color: var(--text-dim); padding: 0.7rem 1rem 0.2rem; text-transform: uppercase;
  }

  .playlist-item {
    display: flex; align-items: center; gap: 0.5rem;
    width: 100%; text-align: left; padding: 0.5rem 1rem 0.5rem 1.25rem;
    background: none; border: var(--stroke-w) solid transparent;
    color: var(--text-muted); font-size: 0.85rem; font-weight: 600;
    font-family: inherit; cursor: pointer; transition: all 0.15s;
    min-width: 0; border-radius: var(--radius-sm);
  }
  .playlist-item:hover { color: var(--accent); background: var(--bg-card); border-color: var(--accent); box-shadow: var(--shadow-sm); }
  .pl-dot { color: var(--accent); font-size: 0.62rem; flex-shrink: 0; }
  .pl-label { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .pl-count { font-size: 0.65rem; color: var(--text-dim); background: var(--bg-card-hover); padding: 0.1rem 0.45rem; border-radius: var(--radius-pill); flex-shrink: 0; }

  .sidebar-spacer { flex: 1; min-height: 0.5rem; }

  /* Bell icon button (in logo area) */
  .bell-btn {
    width: 36px; height: 36px; flex-shrink: 0;
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: transform 0.1s, box-shadow 0.1s;
    position: relative;
    color: var(--text-primary);
  }
  .bell-btn:hover { transform: translate(-1px,-1px); box-shadow: var(--shadow); }
  .bell-icon { display: block; flex-shrink: 0; }
  .bell-dot {
    position: absolute; top: 5px; right: 5px;
    width: 8px; height: 8px; border-radius: 50%;
    border: 1.5px solid var(--bg-card);
  }

  /* Dracula: borde blanco en el logo */
  :global([data-theme="dracula"]) .logo-icon { border-color: #ffffff; }

  /* Gruvbox: borde blanco en el logo */
  :global([data-theme="gruvbox"]) .logo-icon { border-color: #ffffff; }

  /* Jamaica: logo verde, sombra amarilla → tres colores rastafari visibles */
  :global([data-theme="jamaica"]) .logo-icon {
    background: var(--accent-2);
    box-shadow: 3px 3px 0 var(--accent);
  }

  /* ── Theme popup ── */
  .popup-backdrop {
    position: fixed; inset: 0; z-index: 90;
  }
  .theme-popup {
    position: fixed;
    top: 90px; left: 16px;
    width: 280px;
    background: var(--bg-card);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-lg);
    border-radius: var(--radius);
    padding: 1rem;
    z-index: 100;
  }
  .popup-title {
    font-size: 0.65rem; font-weight: 900; letter-spacing: 0.15em;
    color: var(--text-dim); text-transform: uppercase;
    margin-bottom: 0.75rem;
  }
  .popup-grid {
    display: grid; grid-template-columns: 1fr 1fr; gap: 0.65rem;
  }
  .theme-card {
    position: relative; overflow: hidden;
    display: flex; flex-direction: column; align-items: flex-start; gap: 0.5rem;
    padding: 0.9rem 0.85rem;
    background: var(--tc);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
    cursor: pointer; transition: transform 0.1s, box-shadow 0.1s;
    font-family: inherit;
  }
  .theme-card:hover { transform: translate(-2px,-2px); box-shadow: var(--shadow); }
  .theme-card:active { transform: translate(0,0); box-shadow: 1px 1px 0 var(--stroke); }
  .theme-card.active { box-shadow: var(--shadow); }

  /* polka dots overlay */
  .card-dots {
    position: absolute; inset: 0; pointer-events: none;
    background-image: radial-gradient(circle, rgba(0,0,0,0.18) 1.5px, transparent 1.5px);
    background-size: 10px 10px;
    border-radius: inherit;
  }
  .card-swatch {
    width: 28px; height: 28px; border-radius: 50%;
    background: #ffffff; opacity: 0.9;
    border: 2px solid var(--stroke);
    position: relative; z-index: 1;
    display: flex; align-items: center; justify-content: center;
  }
  .card-name {
    font-size: 0.82rem; font-weight: 900;
    color: #ffffff; text-shadow: 0 1px 3px rgba(0,0,0,0.5);
    position: relative; z-index: 1;
    letter-spacing: 0.04em;
  }
  .card-check {
    position: absolute; top: 6px; right: 8px;
    font-size: 0.85rem; font-weight: 900; color: #ffffff;
    text-shadow: 0 1px 3px rgba(0,0,0,0.5); z-index: 1;
  }
</style>
