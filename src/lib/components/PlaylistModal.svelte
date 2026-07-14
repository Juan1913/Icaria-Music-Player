<script lang="ts">
  import { playlists } from '$lib/stores/playlists';
  import type { Track } from '$lib/stores/player';
  import { imgFallback } from '$lib/imgFallback';

  interface Props { track: Track; onClose: () => void; }
  let { track, onClose }: Props = $props();

  let newName = $state('');
  let creating = $state(false);
  let added = $state<string | null>(null);

  function addToExisting(playlistId: string) {
    playlists.addTrack(playlistId, track);
    added = playlistId;
    setTimeout(onClose, 700);
  }

  function createAndAdd() {
    if (!newName.trim()) return;
    const pl = playlists.create(newName);
    playlists.addTrack(pl.id, track);
    added = pl.id;
    setTimeout(onClose, 700);
  }

  function onKeydown(e: KeyboardEvent) { if (e.key === 'Escape') onClose(); }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="backdrop" role="button" tabindex="-1" onclick={onClose} onkeydown={() => {}}></div>

<div class="modal" role="dialog" aria-modal="true">
  <!-- Header -->
  <div class="modal-header">
    <span class="modal-title">// ADD TO PLAYLIST</span>
    <button class="close-btn" onclick={onClose}>[×]</button>
  </div>

  <!-- Track preview -->
  <div class="track-preview">
    {#if track.thumbnail}
      <img src={track.thumbnail} alt="" class="preview-art" use:imgFallback />
    {:else}
      <div class="preview-art preview-art-empty">[♪]</div>
    {/if}
    <div class="preview-info">
      <span class="preview-title">{track.title}</span>
      <span class="preview-artist">{track.artist}</span>
    </div>
  </div>

  <!-- Existing playlists -->
  {#if $playlists.length > 0}
    <div class="existing-list">
      {#each $playlists as pl (pl.id)}
        <button
          class="pl-row {added === pl.id ? 'added' : ''}"
          onclick={() => addToExisting(pl.id)}
        >
          <span class="pl-icon">♫</span>
          <span class="pl-name">{pl.name.toUpperCase()}</span>
          <span class="pl-count">[{pl.tracks.length}]</span>
          {#if added === pl.id}<span class="check-mark">[✓]</span>{/if}
        </button>
      {/each}
    </div>
  {:else}
    <p class="no-playlists">:: no playlists yet — create one below ::</p>
  {/if}

  <!-- Create new -->
  <div class="create-section">
    {#if !creating}
      <button class="create-toggle" onclick={() => (creating = true)}>[+] NEW PLAYLIST</button>
    {:else}
      <div class="create-form">
        <span class="form-prefix">&gt;</span>
        <input
          type="text"
          class="form-input"
          placeholder="playlist name..."
          bind:value={newName}
          onkeydown={e => e.key === 'Enter' && createAndAdd()}
        />
        <button class="form-ok" onclick={createAndAdd} disabled={!newName.trim()}>[OK]</button>
        <button class="form-cancel" onclick={() => (creating = false)}>[✕]</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(29,32,33,0.85);
    z-index: 200;
  }
  .modal {
    position: fixed; top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    z-index: 201;
    background: var(--bg-primary);
    border: var(--stroke-heavy) solid var(--stroke);
    box-shadow: var(--shadow-lg);
    width: min(380px, 92vw);
    max-height: 80vh;
    display: flex; flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0.85rem 1rem;
    border-bottom: var(--stroke-w) solid var(--bg-card);
    background: var(--bg-sidebar);
    flex-shrink: 0;
  }
  .modal-title { font-size: 0.72rem; font-weight: 700; letter-spacing: 0.12em; color: var(--accent); }
  .close-btn {
    background: none; border: 1px solid var(--bg-card-hover);
    color: var(--gb-red); font-size: 0.72rem; font-weight: 700;
    font-family: inherit; padding: 0.1rem 0.35rem; cursor: pointer; transition: all 0.08s;
  }
  .close-btn:hover { border-color: var(--gb-red); }

  .track-preview {
    display: flex; align-items: center; gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--bg-card);
    background: var(--bg-card);
    flex-shrink: 0;
  }
  .preview-art {
    width: 42px; height: 42px; object-fit: cover; display: block;
    border: var(--stroke-w) solid var(--stroke); flex-shrink: 0;
  }
  .preview-art-empty {
    width: 42px; height: 42px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--bg-card); border: var(--stroke-w) solid var(--bg-card-hover);
    font-size: 0.75rem; color: var(--text-muted);
  }
  .preview-info { display: flex; flex-direction: column; min-width: 0; gap: 2px; }
  .preview-title {
    font-size: 0.82rem; font-weight: 700; color: var(--text-primary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .preview-artist { font-size: 0.72rem; color: var(--gb-orange); }

  .existing-list { flex: 1; overflow-y: auto; padding: 0.4rem 0; }
  .pl-row {
    display: flex; align-items: center; gap: 0.65rem;
    width: 100%; padding: 0.55rem 1rem;
    background: none; border: none; border-bottom: 1px solid transparent;
    color: var(--text-muted); text-align: left; font-family: inherit;
    font-size: 0.78rem; font-weight: 700; letter-spacing: 0.03em;
    cursor: pointer; transition: all 0.08s;
  }
  .pl-row:hover {
    background: var(--bg-card); color: var(--text-primary);
    border-bottom-color: var(--bg-card);
  }
  .pl-row.added { background: rgba(184,187,38,0.08); color: var(--gb-green); }
  .pl-icon { color: var(--accent); flex-shrink: 0; font-size: 0.9rem; }
  .pl-name { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .pl-count { font-size: 0.65rem; color: var(--gb-gray); flex-shrink: 0; }
  .check-mark { color: var(--gb-green); font-weight: 700; font-size: 0.72rem; flex-shrink: 0; }

  .no-playlists {
    padding: 1.25rem 1rem;
    font-size: 0.72rem; font-weight: 700;
    color: var(--text-muted); text-align: center; letter-spacing: 0.04em;
  }

  .create-section {
    padding: 0.65rem 1rem;
    border-top: var(--stroke-w) solid var(--bg-card);
    background: var(--bg-sidebar);
    flex-shrink: 0;
  }
  .create-toggle {
    width: 100%;
    background: none; border: var(--stroke-w) dashed var(--bg-card-hover);
    color: var(--accent); font-size: 0.72rem; font-weight: 700;
    letter-spacing: 0.08em; font-family: inherit;
    padding: 0.55rem; cursor: pointer; transition: all 0.08s;
  }
  .create-toggle:hover { border-color: var(--accent); background: rgba(250,189,47,0.05); }

  .create-form {
    display: flex; align-items: center; gap: 0.4rem;
    border: var(--stroke-heavy) solid var(--stroke);
    background: var(--bg-primary); padding: 0.35rem 0.6rem;
  }
  .form-prefix { color: var(--accent); font-weight: 900; flex-shrink: 0; }
  .form-input {
    flex: 1; background: none; border: none; outline: none;
    color: var(--text-primary); font-size: 0.88rem; font-family: inherit;
    caret-color: var(--accent);
  }
  .form-input::placeholder { color: var(--bg-card-hover); }
  .form-ok {
    background: none; border: 1px solid var(--accent);
    color: var(--accent); font-size: 0.68rem; font-weight: 700;
    font-family: inherit; padding: 0.15rem 0.4rem; cursor: pointer; transition: all 0.08s;
    flex-shrink: 0;
  }
  .form-ok:hover:not(:disabled) { background: var(--accent); color: var(--on-accent); }
  .form-ok:disabled { opacity: 0.4; cursor: not-allowed; }
  .form-cancel {
    background: none; border: 1px solid var(--bg-card-hover);
    color: var(--gb-red); font-size: 0.68rem; font-weight: 700;
    font-family: inherit; padding: 0.15rem 0.4rem; cursor: pointer; flex-shrink: 0;
  }
  .form-cancel:hover { border-color: var(--gb-red); }
</style>
