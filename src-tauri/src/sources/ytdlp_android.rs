//! Puente JNI hacia `YtDlpBridge` (Kotlin) → yt-dlp embebido con Chaquopy.
//!
//! Red de seguridad exclusiva de Android: cuando Invidious/Piped/InnerTube
//! fallan (a diferencia de desktop, en Android no hay binario de yt-dlp),
//! se le pide a yt-dlp — corriendo vía Python embebido en la app — que
//! resuelva la URL de audio, igual que hace `race_stream` en desktop.
//!
//! `YtDlpBridge.registerContext()` (llamado una vez desde `MainActivity.
//! onCreate`) invoca `registerContext` acá abajo, que guarda una referencia
//! global a la clase Kotlin + la JavaVM. De ahí en adelante, cualquier hilo
//! de Rust puede adjuntarse a la JVM y llamar al método estático
//! `getAudioUrl(videoId): String` sin depender del classloader del hilo
//! nativo (que no puede resolver clases de la app por sí solo).

use jni::objects::{GlobalRef, JClass, JValue};
use jni::sys::jclass;
use jni::{JNIEnv, JavaVM};
use std::sync::OnceLock;

static BRIDGE: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_togrowagencia_icaria_YtDlpBridge_registerContext(
    env: JNIEnv,
    class: jclass,
) {
    let class = unsafe { JClass::from_raw(class) };
    let Ok(vm) = env.get_java_vm() else { return };
    let Ok(global) = env.new_global_ref(class) else { return };
    let _ = BRIDGE.set((vm, global));
}

/// Le pide a yt-dlp (vía el bridge de Kotlin/Python) la URL de audio de un
/// video. Devuelve `None` si el bridge no se registró todavía, o si yt-dlp
/// no pudo resolverlo — nunca falla "fuerte", para no romper el resto de la
/// cadena de fallback si esta pieza no está disponible.
pub async fn resolve(video_id: &str) -> Option<String> {
    let video_id = video_id.to_string();
    tokio::task::spawn_blocking(move || resolve_blocking(&video_id))
        .await
        .ok()
        .flatten()
}

fn resolve_blocking(video_id: &str) -> Option<String> {
    let (vm, class_ref) = BRIDGE.get()?;
    let mut guard = vm.attach_current_thread().ok()?;
    let env: &mut JNIEnv = &mut guard;

    let jvideo_id = env.new_string(video_id).ok()?;
    let class = unsafe { JClass::from_raw(class_ref.as_obj().as_raw()) };
    let result = env
        .call_static_method(
            class,
            "getAudioUrl",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::from(&jvideo_id)],
        )
        .ok()?;

    let jobj = result.l().ok()?;
    if jobj.is_null() {
        return None;
    }
    let jstring = jni::objects::JString::from(jobj);
    let s: String = env.get_string(&jstring).ok()?.into();
    if s.is_empty() { None } else { Some(s) }
}
