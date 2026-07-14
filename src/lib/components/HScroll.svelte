<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
    gap?: string;
  }
  let { children, gap = '0.9rem' }: Props = $props();

  let track = $state<HTMLElement>();
  let atStart = $state(true);
  let atEnd = $state(true);

  function refresh() {
    const el = track;
    if (!el) return;
    atStart = el.scrollLeft <= 1;
    atEnd = Math.ceil(el.scrollLeft + el.clientWidth) >= el.scrollWidth - 1;
  }

  function nudge(dir: number) {
    track?.scrollBy({ left: dir * track.clientWidth * 0.8, behavior: 'smooth' });
  }

  // Recalcula al montar, al redimensionar y cuando cambia el contenido (carga async).
  $effect(() => {
    const el = track;
    if (!el) return;
    refresh();
    const ro = new ResizeObserver(refresh);
    ro.observe(el);
    const mo = new MutationObserver(refresh);
    mo.observe(el, { childList: true, subtree: true });
    return () => { ro.disconnect(); mo.disconnect(); };
  });
</script>

<div class="hs">
  {#if !atStart}
    <button class="hs-arrow left" onclick={() => nudge(-1)} aria-label="Desplazar a la izquierda">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="15 18 9 12 15 6"/>
      </svg>
    </button>
  {/if}

  <div class="hs-track" bind:this={track} style="--gap:{gap}" onscroll={refresh}>
    {@render children()}
  </div>

  {#if !atEnd}
    <button class="hs-arrow right" onclick={() => nudge(1)} aria-label="Desplazar a la derecha">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="9 18 15 12 9 6"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .hs { position: relative; }

  .hs-track {
    display: flex;
    gap: var(--gap);
    overflow-x: auto;
    scroll-behavior: smooth;
    padding-bottom: 0.25rem;
    scrollbar-width: none;
  }
  .hs-track::-webkit-scrollbar { display: none; }

  .hs-arrow {
    position: absolute; top: 50%; transform: translateY(-50%);
    z-index: 4;
    width: 40px; height: 40px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card); color: var(--text-primary);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    transition: transform 0.1s, box-shadow 0.1s;
  }
  .hs-arrow:hover { transform: translateY(-50%) translate(-1px, -1px); box-shadow: var(--shadow); }
  .hs-arrow:active { transform: translateY(-50%); box-shadow: 1px 1px 0 var(--stroke); }
  .hs-arrow.left  { left: -8px; }
  .hs-arrow.right { right: -8px; }
</style>
