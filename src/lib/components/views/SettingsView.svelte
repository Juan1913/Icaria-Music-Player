<script lang="ts">
  import { theme, THEMES, THEME_LABELS, THEME_COLORS, type Theme } from '$lib/stores/theme';
  import { history } from '$lib/stores/history';
  import { nav } from '$lib/stores/nav';

  function pickTheme(t: Theme) {
    theme.set(t);
  }

  let fileInput: HTMLInputElement | undefined = $state();
  let importMsg = $state('');

  function exportHistory() {
    const data = JSON.stringify($history, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'icaria-historial.json';
    a.click();
    URL.revokeObjectURL(url);
  }

  function triggerImport() {
    fileInput?.click();
  }

  async function onImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    try {
      const text = await file.text();
      const parsed = JSON.parse(text);
      if (!Array.isArray(parsed)) throw new Error('formato inválido');
      history.merge(parsed);
      importMsg = `Se importaron ${parsed.length} canciones.`;
    } catch {
      importMsg = 'No se pudo leer el archivo.';
    }
    setTimeout(() => (importMsg = ''), 3000);
  }

  function clearHistory() {
    history.clear();
  }
</script>

<div class="settings-view">
  <div class="view-header">
    <span class="view-title">// AJUSTES</span>
  </div>

  <div class="scroll-area">
    <!-- Apariencia -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag theme-tag">[APARIENCIA]</span>
      </div>
      <p class="card-desc">Elegí el tema de color de la app.</p>
      <div class="theme-grid">
        {#each THEMES as t}
          <button
            class="theme-card {$theme === t ? 'active' : ''}"
            style="--tc: {THEME_COLORS[t]}"
            onclick={() => pickTheme(t)}
          >
            <span class="card-swatch"></span>
            <span class="card-name">{THEME_LABELS[t]}</span>
            {#if $theme === t}<span class="card-check">✓</span>{/if}
          </button>
        {/each}
      </div>
    </div>

    <!-- Historial -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag history-tag">[HISTORIAL]</span>
        <span class="source-status ok">{$history.length} CANCIONES</span>
      </div>
      <p class="card-desc">
        Se guarda solo en este dispositivo y se usa para armar "Para ti" en Inicio.
        Podés exportarlo para llevarlo a otra instalación, o importar uno.
      </p>
      <div class="history-actions">
        <button class="hist-btn" onclick={() => nav.setPage('history')} disabled={$history.length === 0}>[▤ VER HISTORIAL]</button>
        <button class="hist-btn" onclick={exportHistory} disabled={$history.length === 0}>[↓ EXPORTAR]</button>
        <button class="hist-btn" onclick={triggerImport}>[↑ IMPORTAR]</button>
        <button class="hist-btn danger" onclick={clearHistory} disabled={$history.length === 0}>[× BORRAR]</button>
      </div>
      <input
        type="file"
        accept="application/json"
        bind:this={fileInput}
        onchange={onImportFile}
        style="display:none"
      />
      {#if importMsg}<div class="status-row ok">{importMsg}</div>{/if}
    </div>

    <!-- YouTube -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag youtube">[YOUTUBE]</span>
        <span class="source-status ok">[✓] LISTO</span>
      </div>
      <p class="card-desc">
        Transmite vía <strong>Piped</strong> — frontend de YouTube libre y sin anuncios.
        5 instancias públicas con respaldo automático. Sin configuración.
      </p>
      <div class="instance-list">
        {#each ['pipedapi.kavin.rocks', 'pipedapi.adminforge.de', 'piped-api.garudalinux.org', 'watchapi.whatever.social', 'api.piped.projectsegfau.lt'] as inst, i}
          <div class="instance-row">
            <span class="inst-num">{i+1}.</span>
            <span class="inst-url">{inst}</span>
            {#if i === 0}<span class="inst-badge">[PRINCIPAL]</span>{/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Bandcamp -->
    <div class="card nb-card">
      <div class="card-head">
        <span class="source-tag bandcamp">[BANDCAMP]</span>
        <span class="source-status ok">[✓] LISTO</span>
      </div>
      <p class="card-desc">
        Transmite mp3-128 directo desde las páginas de los artistas. Scraping HTML, sin necesidad de API key.
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
        reproductor de música libre · sin anuncios · sin cuentas de youtube
      </p>
      <p class="about-slogan">música y software libre por un mundo libre</p>
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
  .source-tag.youtube  { color: var(--gb-red);   }
  .source-tag.bandcamp { color: var(--gb-blue);  }
  .source-tag.theme-tag { color: var(--accent); }
  .source-tag.history-tag { color: var(--accent-2); }

  /* Selector de tema */
  .theme-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 0.65rem;
  }
  .theme-card {
    position: relative; overflow: hidden;
    display: flex; flex-direction: column; align-items: flex-start; gap: 0.4rem;
    padding: 0.8rem 0.75rem;
    background: var(--tc);
    border: var(--stroke-heavy) solid var(--stroke);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
    cursor: pointer; transition: transform 0.1s, box-shadow 0.1s;
    font-family: inherit;
  }
  .theme-card:hover { transform: translate(-2px,-2px); box-shadow: var(--shadow); }
  .theme-card:active { transform: translate(0,0); box-shadow: var(--shadow-sm); }
  .theme-card.active { box-shadow: var(--shadow); }
  .card-swatch {
    width: 24px; height: 24px; border-radius: 50%;
    background: #ffffff; opacity: 0.9;
    border: 2px solid var(--stroke);
  }
  .card-name {
    font-size: 0.78rem; font-weight: 900; color: #ffffff;
    text-shadow: 0 1px 3px rgba(0,0,0,0.5); letter-spacing: 0.03em;
  }
  .card-check {
    position: absolute; top: 6px; right: 8px;
    font-size: 0.8rem; font-weight: 900; color: #ffffff;
    text-shadow: 0 1px 3px rgba(0,0,0,0.5);
  }

  .source-status {
    font-size: 0.65rem; font-weight: 700; letter-spacing: 0.06em;
  }
  .source-status.ok   { color: var(--gb-green); }

  .card-desc {
    font-size: 0.78rem; color: var(--text-muted); line-height: 1.55;
  }
  .card-desc strong { color: var(--text-primary); }

  /* Historial */
  .history-actions { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .hist-btn {
    background: none;
    border: var(--stroke-w) solid var(--accent);
    box-shadow: var(--shadow-sm);
    color: var(--accent);
    font-size: 0.68rem; font-weight: 700; letter-spacing: 0.06em;
    font-family: inherit; padding: 0.3rem 0.7rem; cursor: pointer; transition: all 0.08s;
  }
  .hist-btn:not(:disabled):hover {
    background: var(--accent); color: var(--on-accent);
    transform: translate(-1px,-1px); box-shadow: var(--shadow);
  }
  .hist-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .hist-btn.danger { border-color: var(--gb-red); color: var(--gb-red); }
  .hist-btn.danger:not(:disabled):hover { background: var(--gb-red); color: #ffffff; }

  .status-row {
    font-size: 0.7rem; font-weight: 700; letter-spacing: 0.04em;
    padding: 0.35rem 0.6rem;
    border-left: 3px solid currentColor;
  }
  .status-row.ok   { color: var(--gb-green); background: rgba(184,187,38,0.07); }

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
  .about-slogan {
    font-size: 0.72rem; font-weight: 800; color: var(--accent);
    letter-spacing: 0.03em; margin-top: 0.3rem; text-align: center;
  }

</style>
