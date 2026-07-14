# Reproducción en segundo plano (pantalla apagada) — Android

> Objetivo: garantizar que la música **siga sonando con la pantalla apagada / app en
> segundo plano**, con controles en la pantalla de bloqueo (play/pausa/siguiente).
>
> Estado: **por aplicar**. Requiere el proyecto Android generado (`pnpm tauri android
> init`) y **pruebas en dispositivo real**. El audio en background se comporta distinto
> por versión/fabricante de Android, así que hay que iterar en el teléfono.

La app ya tiene la **Media Session** (metadatos + handlers) en `PlayerBar.svelte`, que
provee los controles del sistema. Lo que falta es evitar que Android **suspenda el
proceso** cuando apagas la pantalla. Eso lo da un **servicio en primer plano**.

---

## Enfoque A — Foreground service que mantiene vivo el WebView (recomendado empezar aquí)

El `<audio>` HTML del WebView sigue sonando si el proceso no muere. Un servicio en
primer plano con notificación persistente le dice a Android "estoy reproduciendo media,
no me mates".

### 1. Permisos + servicio en `AndroidManifest.xml`

Archivo: `src-tauri/gen/android/app/src/main/AndroidManifest.xml`

Dentro de `<manifest>` (antes de `<application>`):

```xml
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE_MEDIA_PLAYBACK" />
<uses-permission android:name="android.permission.WAKE_LOCK" />
<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
```

Dentro de `<application>`:

```xml
<service
    android:name=".PlaybackService"
    android:exported="false"
    android:foregroundServiceType="mediaPlayback" />
```

### 2. Servicio Kotlin

Archivo nuevo: `src-tauri/gen/android/app/src/main/java/<tu/paquete>/PlaybackService.kt`
(el paquete es `com.togrowagencia.icaria`).

```kotlin
package com.togrowagencia.icaria

import android.app.*
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat

class PlaybackService : Service() {
    private var wakeLock: PowerManager.WakeLock? = null

    companion object {
        const val CHANNEL_ID = "icaria_playback"
        const val NOTIF_ID = 1

        fun start(ctx: Context) {
            val i = Intent(ctx, PlaybackService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) ctx.startForegroundService(i)
            else ctx.startService(i)
        }
        fun stop(ctx: Context) {
            ctx.stopService(Intent(ctx, PlaybackService::class.java))
        }
    }

    override fun onCreate() {
        super.onCreate()
        createChannel()
        val notif = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Icaria")
            .setContentText("Reproduciendo")
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setOngoing(true)
            .build()
        startForeground(NOTIF_ID, notif)

        // Mantiene la CPU despierta mientras suena (soltar en onDestroy).
        val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "icaria:playback").apply {
            setReferenceCounted(false); acquire(3 * 60 * 60 * 1000L) // 3h máx
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int = START_STICKY
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        wakeLock?.let { if (it.isHeld) it.release() }
        wakeLock = null
        super.onDestroy()
    }

    private fun createChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val ch = NotificationChannel(
                CHANNEL_ID, "Reproducción", NotificationManager.IMPORTANCE_LOW
            )
            (getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager)
                .createNotificationChannel(ch)
        }
    }
}
```

> La notificación del servicio y la de la **Media Session** (que crea el WebView) pueden
> mostrarse juntas. Para unificarlas hay que construir la notificación con el
> `MediaStyle` y el token de la MediaSession del WebView — es un extra opcional; primero
> valida que el audio **no se corta**.

### 3. Asegurar que el WebView permita audio sin gesto en background

En la `Activity` generada por Tauri (`.../MainActivity.kt` o `WryActivity`), tras crear
el WebView, conviene:

```kotlin
webView.settings.mediaPlaybackRequiresUserGesture = false
```

(Tauri suele exponer la instancia del WebView; si no es accesible directamente, se hace
vía un plugin — ver más abajo.)

### 4. Puente: iniciar/parar el servicio desde el reproductor

Necesitas llamar a `PlaybackService.start/stop` cuando la reproducción arranca/para.
Con Tauri v2 esto se hace con un **plugin Android** que exponga comandos invocables desde
JS. Esqueleto mínimo:

- Plugin Kotlin con dos comandos `startPlaybackService` / `stopPlaybackService` que
  llaman a `PlaybackService.start(activity)` / `.stop(activity)`.
- Desde el front, en `player.ts` / `PlayerBar.svelte`:

```ts
import { invoke } from '@tauri-apps/api/core';
// al empezar a reproducir:
invoke('plugin:icaria-audio|start_service').catch(() => {});
// al pausar/parar del todo:
invoke('plugin:icaria-audio|stop_service').catch(() => {});
```

En escritorio esos `invoke` fallan silenciosamente (no existe el plugin) — por eso el
`.catch(() => {})`.

---

## Enfoque B — Reproductor nativo ExoPlayer (garantía total, más trabajo)

Si tras probar el Enfoque A el WebView **sigue cortándose** en algún dispositivo, se pasa
a reproducir **fuera del WebView** con ExoPlayer dentro del servicio:

1. Añadir dependencia `androidx.media3:media3-exoplayer` + `media3-session` en el
   `build.gradle` del módulo `app`.
2. El servicio pasa a ser un `MediaSessionService` con un `ExoPlayer` real.
3. El front, en Android, **deja de usar `<audio>`** y en su lugar invoca comandos nativos:
   `play(url)`, `pause()`, `seek(ms)`, `setQueue(urls)`, y **recibe eventos** (posición,
   `ended`) por el sistema de eventos de Tauri para actualizar la UI.
4. Se mantiene la ruta actual (`resolveStream` en Rust) para obtener la URL; solo cambia
   **quién reproduce** el audio.

Esto es lo que hace Spotify y **garantiza** el playback con pantalla apagada, pero implica
sincronizar estado UI ↔ nativo. Es un proyecto en sí mismo.

---

## Orden sugerido

1. `pnpm tauri android init` (necesita JDK 17 + Android SDK/NDK — ver `README`/guía).
2. Aplicar Enfoque A (permisos + `PlaybackService.kt` + puente).
3. Probar en dispositivo real: reproducir → apagar pantalla → ¿sigue sonando 5–10 min?
4. Si algún dispositivo corta el audio → escalar a Enfoque B (ExoPlayer).

## Notas de compatibilidad

- **Android 13+**: pedir permiso `POST_NOTIFICATIONS` en runtime la primera vez.
- **Android 14+**: obligatorio declarar `foregroundServiceType="mediaPlayback"` (ya está
  arriba) y tener el permiso `FOREGROUND_SERVICE_MEDIA_PLAYBACK`.
- **Optimización de batería**: algunos fabricantes (Xiaomi, Samsung) matan servicios
  agresivamente; puede requerir que el usuario exente la app del ahorro de batería.
