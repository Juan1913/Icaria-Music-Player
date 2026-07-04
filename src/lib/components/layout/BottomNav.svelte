<script lang="ts">
  import { nav, type NavPage } from '$lib/stores/nav';

  let activePage = $derived($nav.page);

  const tabs: { id: NavPage; label: string; icon: string }[] = [
    { id: 'home',     label: 'HOME',    icon: '⌂' },
    { id: 'search',   label: 'SEARCH',  icon: '/' },
    { id: 'library',  label: 'LIBRARY', icon: '≡' },
    { id: 'settings', label: 'OPTS',    icon: '*' },
  ];
</script>

<nav class="bottom-nav">
  {#each tabs as tab}
    <button
      class="tab {activePage === tab.id ? 'active' : ''}"
      onclick={() => nav.setPage(tab.id)}
    >
      <span class="tab-icon">{tab.icon}</span>
      <span class="tab-label">{tab.label}</span>
    </button>
  {/each}
</nav>

<style>
  .bottom-nav {
    display: flex;
    background: var(--bg-sidebar);
    border-top: var(--stroke-heavy) solid var(--stroke);
    padding: 0;
  }
  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 0.6rem 0;
    border: none;
    border-right: 1px solid var(--bg-card);
    background: none;
    color: var(--text-muted);
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    cursor: pointer;
    transition: all 0.08s;
  }
  .tab:last-child { border-right: none; }
  .tab:hover { background: var(--bg-card); color: var(--text-primary); }
  .tab.active {
    background: var(--accent);
    color: var(--bg-sidebar);
    border-bottom: 3px solid var(--gb-orange-dim);
  }
  .tab-icon { font-size: 1.1rem; line-height: 1; }
  .tab-label { font-size: 0.58rem; }
</style>
