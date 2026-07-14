/**
 * Acción de Svelte para <img> con carátulas remotas (YouTube Music, etc.).
 *
 * Los servidores de imágenes fallan de forma intermitente cuando se cargan
 * muchas a la vez (throttling). Sin manejo de error, el navegador deja el ícono
 * de imagen rota. Esta acción:
 *   1. Al primer fallo, reintenta con una variante distinta de la URL (petición
 *      nueva) a un tamaño nítido, no reducido — así no se degrada la calidad.
 *   2. Si vuelve a fallar, sustituye por un placeholder discreto.
 *
 * Uso:  <img src={url} alt="" use:imgFallback />
 */

const PLACEHOLDER =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">` +
      `<rect width="100" height="100" fill="none"/>` +
      `<text x="50" y="62" font-size="42" text-anchor="middle" fill="#9a9a9a" font-family="sans-serif">♪</text>` +
    `</svg>`
  );

/**
 * Devuelve una variante distinta de la URL para reintentar (petición nueva),
 * manteniendo una resolución nítida. null si no aplica.
 */
function retryVariant(url: string): string | null {
  // googleusercontent / lh3: "=w544-h544-l90-rj" → tamaño fijo nítido (2x display)
  if (/=w\d+-h\d+/.test(url)) return url.replace(/=w\d+-h\d+/, '=w480-h480');
  // i.ytimg: maxres/sd → hqdefault (siempre existe, 480×360)
  if (/\/(maxres|sd)default\.jpg/.test(url)) {
    return url.replace(/\/(maxres|sd)default\.jpg/, '/hqdefault.jpg');
  }
  return null;
}

export function imgFallback(node: HTMLImageElement) {
  let retried = false;

  function onError() {
    if (!retried) {
      retried = true;
      const alt = retryVariant(node.currentSrc || node.src);
      if (alt && alt !== node.src) {
        node.src = alt;
        return;
      }
    }
    node.removeEventListener('error', onError);
    node.src = PLACEHOLDER;
  }

  node.addEventListener('error', onError);
  return {
    destroy() {
      node.removeEventListener('error', onError);
    },
  };
}
