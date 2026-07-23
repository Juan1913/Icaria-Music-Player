#!/usr/bin/env bash
#
# Setup de entorno para Icaria en escritorio (Linux).
# Deja el equipo listo para reproducir audio de YouTube igual que en un PC que ya funciona:
#   1. yt-dlp reciente en ~/.local/bin/yt-dlp   (ruta que la app prefiere)
#   2. deno (runtime JS que yt-dlp usa para descifrar la firma 'n' de YouTube)
#   3. symlink de deno en ~/.local/bin/deno      (para que la app lo halle sin importar el PATH)
#
# NOTA: además de esto, inicia sesión en YouTube en tu navegador (Firefox/Chrome/…).
#       La app pasa esas cookies a yt-dlp para evitar el "confirm you're not a bot" / HTTP 429.
#
# Uso:  bash scripts/setup-desktop.sh
set -euo pipefail

LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

echo "==> 1/3  Instalando yt-dlp más reciente en $LOCAL_BIN"
curl -L --fail -o "$LOCAL_BIN/yt-dlp" \
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp
chmod +x "$LOCAL_BIN/yt-dlp"
echo "    yt-dlp $("$LOCAL_BIN/yt-dlp" --version)"

echo "==> 2/3  Instalando deno (runtime JS)"
if [ ! -x "$HOME/.deno/bin/deno" ]; then
  curl -fsSL https://deno.land/install.sh -o /tmp/deno_install.sh
  bash /tmp/deno_install.sh -y
else
  echo "    deno ya estaba instalado"
fi
echo "    deno $("$HOME/.deno/bin/deno" --version | head -1)"

echo "==> 3/3  Enlazando deno en $LOCAL_BIN (para que la app lo encuentre)"
ln -sf "$HOME/.deno/bin/deno" "$LOCAL_BIN/deno"

echo
echo "==> Verificación rápida"
if PATH="$LOCAL_BIN:$PATH" yt-dlp -f "140/bestaudio" --get-url --no-warnings --no-playlist \
     "https://www.youtube.com/watch?v=dQw4w9WgXcQ" >/dev/null 2>&1; then
  echo "    OK: yt-dlp + deno resuelven un stream."
else
  echo "    AVISO: la prueba sin cookies falló (bot-check/429). Inicia sesión en YouTube"
  echo "    en tu navegador; la app usará esas cookies automáticamente."
fi

echo
echo "Listo. Recuerda:"
echo "  - Inicia sesión en YouTube en tu navegador (Firefox/Chrome/Chromium/Brave/Vivaldi)."
echo "  - Si $LOCAL_BIN no está en tu PATH, abre una terminal nueva antes de 'pnpm tauri dev'."
echo "  - Para forzar un navegador concreto de cookies: export ICARIA_COOKIES_BROWSER=chrome"
