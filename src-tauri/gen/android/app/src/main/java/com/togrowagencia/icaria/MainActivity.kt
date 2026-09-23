package com.togrowagencia.icaria

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Servicio en primer plano: mantiene el audio con la app en segundo plano / pantalla apagada.
    PlaybackService.start(this)
    // yt-dlp embebido (red de seguridad para resolver audio de YouTube en Android).
    YtDlpBridge.initPython(this)
    YtDlpBridge.registerContext()
  }

  override fun onDestroy() {
    PlaybackService.stop(this)
    super.onDestroy()
  }
}
