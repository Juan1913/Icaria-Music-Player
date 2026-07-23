# Tauri + SvelteKit + TypeScript

This template should help get you started developing with Tauri, SvelteKit and TypeScript in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Setup de escritorio (reproducir YouTube)

En escritorio la reproducción usa `yt-dlp` con un runtime JS (`deno`) y cookies del
navegador para evitar el bot-check / HTTP 429 de YouTube. Para dejar **cualquier PC Linux**
funcionando igual:

```bash
bash scripts/setup-desktop.sh
```

Esto instala:

- `yt-dlp` reciente en `~/.local/bin/yt-dlp` (ruta que la app prefiere),
- `deno` (descifra la firma `n` de YouTube) + symlink en `~/.local/bin/deno`.

Además, **inicia sesión en YouTube en tu navegador** (Firefox/Chrome/Chromium/Brave/Vivaldi).
La app pasa esas cookies a `yt-dlp` automáticamente. Para forzar un navegador concreto:

```bash
export ICARIA_COOKIES_BROWSER=chrome   # o firefox, chromium, brave, vivaldi
```

Luego, desde una terminal nueva:

```bash
pnpm install
pnpm tauri dev
```
