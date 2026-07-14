<script lang="ts">
  import { settings } from '$lib/stores/settings';

  let clientId = $state($settings.spotifyClientId);
  let clientSecret = $state($settings.spotifyClientSecret);
  let saving = $state(false);
  let saved = $state(false);

  async function save() {
    if (!clientId.trim() || !clientSecret.trim()) return;
    saving = true;
    try {
      await settings.saveSpotify(clientId.trim(), clientSecret.trim());
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-view">
  <div class="view-header">
    <span class="view-title">// SETTINGS</span>
  </div>

  <div class="scroll-area">
    <!-- Spotify -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag spotify">[SPOTIFY]</span>
        <span class="source-status warn">! REQUIRES SETUP</span>
      </div>
      <p class="card-desc">
        Used for search &amp; metadata only — audio streams via YouTube.
        No Premium needed. Get free credentials at
        <strong>developer.spotify.com</strong> → create app.
      </p>

      <div class="form">
        <label class="field">
          <span class="field-label">CLIENT ID</span>
          <div class="input-wrap">
            <span class="input-prefix">›</span>
            <input
              type="text"
              class="nb-input"
              placeholder="paste client id..."
              bind:value={clientId}
            />
          </div>
        </label>

        <label class="field">
          <span class="field-label">CLIENT SECRET</span>
          <div class="input-wrap">
            <span class="input-prefix">›</span>
            <input
              type="password"
              class="nb-input"
              placeholder="paste client secret..."
              bind:value={clientSecret}
            />
          </div>
        </label>

        <button class="save-btn" onclick={save} disabled={saving || (!clientId.trim() || !clientSecret.trim())}>
          {#if saved}
            [✓ SAVED]
          {:else if saving}
            [SAVING···]
          {:else}
            [SAVE CREDENTIALS]
          {/if}
        </button>
      </div>

      {#if $settings.spotifyConfigured}
        <div class="status-row ok">[✓] SPOTIFY CONFIGURED</div>
      {:else}
        <div class="status-row warn">[!] NOT CONFIGURED — YOUTUBE &amp; BANDCAMP STILL WORK</div>
      {/if}
    </div>

    <!-- YouTube -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag youtube">[YOUTUBE]</span>
        <span class="source-status ok">[✓] READY</span>
      </div>
      <p class="card-desc">
        Streams via <strong>Piped</strong> — open-source, ad-free YouTube frontend.
        5 public instances with automatic fallback. Zero config.
      </p>
      <div class="instance-list">
        {#each ['pipedapi.kavin.rocks', 'pipedapi.adminforge.de', 'piped-api.garudalinux.org', 'watchapi.whatever.social', 'api.piped.projectsegfau.lt'] as inst, i}
          <div class="instance-row">
            <span class="inst-num">{i+1}.</span>
            <span class="inst-url">{inst}</span>
            {#if i === 0}<span class="inst-badge">[PRIMARY]</span>{/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Bandcamp -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag bandcamp">[BANDCAMP]</span>
        <span class="source-status ok">[✓] READY</span>
      </div>
      <p class="card-desc">
        Streams mp3-128 directly from artist pages. HTML scraping, no API key needed.
      </p>
    </div>

    <!-- About -->
    <div class="card nb-card about-card">
      <div class="about-logo">
        <span class="ab-bracket">[</span>
        <span class="ab-text">ICARIA</span>
        <span class="ab-bracket">]</span>
      </div>
      <p class="about-desc">
        open-source music player · no ads · no premium · rust + tauri + svelte
      </p>
    </div>
  </div>
</div>

<style>
  .settings-view { display: flex; flex-direction: column; height: 100%; }
  .scroll-area { flex: 1; overflow-y: auto; padding: 1rem 1.25rem; display: flex; flex-direction: column; gap: 0.85rem; }

  .view-header {
    padding: 1rem 1.25rem 0.5rem;
    box-shadow: 0 2px 10px rgba(0,0,0,0.08);
  }
  .view-title { font-size: 0.72rem; font-weight: 700; letter-spacing: 0.12em; color: var(--accent); }

  .card {
    background: var(--bg-card);
    border: var(--stroke-w) solid var(--stroke);
    box-shadow: var(--shadow);
    padding: 1rem 1.1rem;
    display: flex; flex-direction: column; gap: 0.75rem;
    max-width: 580px;
  }

  .card-head { display: flex; align-items: center; gap: 0.75rem; }

  .source-tag {
    font-size: 0.72rem; font-weight: 900; letter-spacing: 0.08em; padding: 0.15rem 0.5rem;
    border: var(--stroke-w) solid currentColor;
  }
  .source-tag.spotify  { color: var(--gb-green); }
  .source-tag.youtube  { color: var(--gb-red);   }
  .source-tag.bandcamp { color: var(--gb-blue);  }

  .source-status {
    font-size: 0.65rem; font-weight: 700; letter-spacing: 0.06em;
  }
  .source-status.ok   { color: var(--gb-green); }
  .source-status.warn { color: var(--gb-yellow); }

  .card-desc {
    font-size: 0.78rem; color: var(--text-muted); line-height: 1.55;
  }
  .card-desc strong { color: var(--text-primary); }

  /* Form */
  .form { display: flex; flex-direction: column; gap: 0.65rem; }
  .field { display: flex; flex-direction: column; gap: 0.25rem; }
  .field-label { font-size: 0.65rem; font-weight: 700; letter-spacing: 0.1em; color: var(--gb-gray); }

  .input-wrap {
    display: flex; align-items: center; gap: 0.4rem;
    border: var(--stroke-heavy) solid var(--bg-card-hover);
    background: var(--bg-sidebar);
    padding: 0.45rem 0.65rem;
    transition: border-color 0.08s;
  }
  .input-wrap:focus-within { border-color: var(--accent); }
  .input-prefix { color: var(--accent); font-weight: 900; flex-shrink: 0; }
  .nb-input {
    flex: 1; background: none; border: none; outline: none;
    color: var(--text-primary); font-size: 0.88rem; font-family: inherit;
    caret-color: var(--accent);
  }
  .nb-input::placeholder { color: var(--bg-card-hover); }

  .save-btn {
    background: none;
    border: var(--stroke-heavy) solid var(--accent);
    box-shadow: var(--shadow-sm);
    color: var(--accent);
    font-size: 0.72rem; font-weight: 900; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.5rem 1.1rem;
    align-self: flex-start; cursor: pointer; transition: all 0.08s;
  }
  .save-btn:not(:disabled):hover {
    background: var(--accent); color: var(--on-accent);
    transform: translate(-1px,-1px); box-shadow: var(--shadow);
  }
  .save-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .status-row {
    font-size: 0.7rem; font-weight: 700; letter-spacing: 0.04em;
    padding: 0.35rem 0.6rem;
    border-left: 3px solid currentColor;
  }
  .status-row.ok   { color: var(--gb-green); background: rgba(184,187,38,0.07); }
  .status-row.warn { color: var(--gb-yellow); background: rgba(250,189,47,0.07); }

  /* Instance list */
  .instance-list { display: flex; flex-direction: column; gap: 4px; }
  .instance-row {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--bg-card); background: var(--bg-sidebar);
  }
  .inst-num { font-size: 0.65rem; color: var(--bg-card-hover); font-weight: 700; flex-shrink: 0; }
  .inst-url { font-size: 0.72rem; color: var(--gb-fg2); flex: 1; }
  .inst-badge {
    font-size: 0.6rem; font-weight: 700; letter-spacing: 0.06em;
    color: var(--accent); border: 1px solid var(--accent);
    padding: 0.05rem 0.3rem;
  }

  /* About */
  .about-card { align-items: center; padding: 1.25rem; }
  .about-logo { display: flex; align-items: center; gap: 0.25rem; }
  .ab-bracket { color: var(--accent); font-size: 1.1rem; font-weight: 700; }
  .ab-text { font-size: 1rem; font-weight: 700; letter-spacing: 0.15em; color: var(--text-primary); }
  .about-desc { font-size: 0.7rem; color: var(--text-muted); letter-spacing: 0.04em; }
</style>
