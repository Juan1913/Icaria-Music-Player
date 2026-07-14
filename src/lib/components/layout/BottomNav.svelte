<script lang="ts">
  import { nav, type NavPage } from '$lib/stores/nav';

  let activePage = $derived($nav.page);

  const tabs: { id: NavPage; label: string }[] = [
    { id: 'home',      label: 'Inicio'     },
    { id: 'search',    label: 'Buscar'     },
    { id: 'library',   label: 'Biblioteca' },
    { id: 'favorites', label: 'Favoritos'  },
    { id: 'settings',  label: 'Perfil'     },
  ];
</script>

<nav class="bottom-nav">
  {#each tabs as tab}
    <button
      class="tab {activePage === tab.id ? 'active' : ''}"
      onclick={() => nav.setPage(tab.id)}
    >
      <span class="tab-icon">
        {#if tab.id === 'home'}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V21h14V9.5"/></svg>
        {:else if tab.id === 'search'}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="10.5" cy="10.5" r="6.5"/><line x1="16" y1="16" x2="21" y2="21"/></svg>
        {:else if tab.id === 'library'}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="9" x2="9" y2="20"/></svg>
        {:else if tab.id === 'favorites'}
          <svg viewBox="0 0 24 24" fill={activePage === tab.id ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="4"/><path d="M4 21v-1a6 6 0 0 1 6-6h4a6 6 0 0 1 6 6v1"/></svg>
        {/if}
      </span>
      <span class="tab-label">{tab.label}</span>
    </button>
  {/each}
</nav>

<style>
  .bottom-nav {
    display: flex;
    background: var(--bg-sidebar);
    border-top: var(--stroke-heavy) solid var(--stroke);
    padding: 0.35rem 0.25rem calc(0.35rem + env(safe-area-inset-bottom, 0px));
  }
  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: 0.4rem 0;
    border: none;
    background: none;
    color: var(--text-muted);
    font-family: inherit;
    cursor: pointer;
    transition: color 0.1s;
    border-radius: var(--radius-sm);
  }
  .tab:hover { color: var(--text-primary); }
  .tab.active { color: var(--accent); }
  .tab-icon { width: 24px; height: 24px; }
  .tab-icon svg { width: 100%; height: 100%; display: block; }
  .tab-label { font-size: 0.62rem; font-weight: 700; letter-spacing: 0.02em; }
</style>
