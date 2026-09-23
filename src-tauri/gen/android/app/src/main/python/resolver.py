"""Resuelve la URL de audio de un video de YouTube usando yt-dlp.

Llamado desde Kotlin (YtDlpBridge) vía Chaquopy, y desde ahí desde Rust por
JNI — es la red de seguridad de Android para los videos que Invidious/Piped/
InnerTube no logran resolver (yt-dlp se mantiene al día contra los cambios
de YouTube; en desktop ya corre como subproceso, esto es su equivalente en
Android, donde no hay binario de yt-dlp disponible).
"""

import yt_dlp


def get_audio_url(video_id: str) -> str:
    url = "https://www.youtube.com/watch?v=" + video_id
    ydl_opts = {
        "format": "bestaudio/best",
        "quiet": True,
        "no_warnings": True,
        "noplaylist": True,
        "skip_download": True,
    }
    try:
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            info = ydl.extract_info(url, download=False)
            return info.get("url") or ""
    except Exception:
        return ""
