package com.togrowagencia.icaria

import android.content.Context
import com.chaquo.python.Python
import com.chaquo.python.android.AndroidPlatform

/**
 * Puente hacia yt-dlp embebido (vía Chaquopy). Red de seguridad para
 * Android: cuando Invidious/Piped/InnerTube no logran resolver el audio de
 * un video, el backend Rust llama a [getAudioUrl] por JNI para que yt-dlp
 * (que en desktop corre como binario externo) lo intente igual, corriendo
 * como Python embebido dentro de la app.
 */
object YtDlpBridge {
    fun initPython(context: Context) {
        if (!Python.isStarted()) {
            Python.start(AndroidPlatform(context))
        }
    }

    /** Llamado una vez desde MainActivity; le da a Rust una referencia fija a esta clase. */
    @JvmStatic
    external fun registerContext()

    @JvmStatic
    fun getAudioUrl(videoId: String): String {
        return try {
            val py = Python.getInstance()
            val module = py.getModule("resolver")
            module.callAttr("get_audio_url", videoId).toString()
        } catch (e: Exception) {
            ""
        }
    }
}
