//! JNI entry points for `dev.gpui.mobile.lab.LabActivity`.
//!
//! The Activity owns a `SurfaceView`; GPUI runs on the process-lived render thread
//! from `gpui_mobile::android::host`, so an Activity recreation (back + relaunch,
//! process-kept task switches) re-attaches a new surface to the same `App` instead
//! of rebuilding it. Every function here only hands data over to that module; none
//! of them touch GPUI state on the Java UI thread.

use gpui_mobile::android::{host, jni as mobile_jni};
use jni::objects::{JFloatArray, JIntArray, JObject, JString};
use jni::{EnvUnowned, errors::LogErrorAndDefault};
use ndk::native_window::NativeWindow;

use crate::diagnostics;

const TAG: &str = "GPUI_MOBILE_LAB";

fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag(TAG),
    );
    // Panics on the render thread would otherwise vanish: log message, location
    // and a backtrace, then let the default policy abort the process.
    std::panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Box<dyn Any>".into());
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        log::error!("PANIC on thread '{thread}' at {location}: {payload}");
        log::error!("{}", std::backtrace::Backtrace::force_capture());
    }));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeOnCreate<'local>(
    mut env: EnvUnowned<'local>,
    _this: JObject<'local>,
    activity: JObject<'local>,
    api_level: i32,
) {
    init_logging();
    env.with_env(|env| -> jni::errors::Result<()> {
        if let Err(err) = mobile_jni::set_host_activity(env, &activity) {
            log::error!("set_host_activity failed: {err}");
        }
        Ok(())
    })
    .resolve::<LogErrorAndDefault>();
    diagnostics::set_api_level(api_level);
    log::info!("LabActivity.onCreate (API {api_level}); starting GPUI render thread");
    host::on_open_window(|_, cx| crate::open_window(cx));
    host::start_with_assets(gpui_kit::assets::AllAssets, crate::launch);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeSurfaceChanged<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    host: i64,
    surface: JObject<'local>,
    scale: f32,
) {
    // SAFETY: `env` is the JNI env of this native call and `surface` a live
    // `android.view.Surface` local reference for its duration.
    let window = unsafe { NativeWindow::from_surface(env.as_raw() as _, surface.as_raw() as _) };
    match window {
        Some(window) => {
            log::info!(
                "surface changed: {}x{} px, scale {scale}",
                window.width(),
                window.height()
            );
            host::surface_created(host as host::HostId, window, scale);
        }
        None => log::error!("ANativeWindow_fromSurface returned null"),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeOpenScreen<'local>(
    mut env: EnvUnowned<'local>,
    _this: JObject<'local>,
    screen: JString<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<()> {
        let screen = screen.try_to_string(env)?;
        log::info!("intent requested screen {screen:?}");
        diagnostics::request_screen(screen);
        Ok(())
    })
    .resolve::<LogErrorAndDefault>();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeDeepLink<'local>(
    mut env: EnvUnowned<'local>,
    _this: JObject<'local>,
    url: JString<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<()> {
        let url = url.try_to_string(env)?;
        gpui_mobile::packages::deeplink::handle_link(&url);
        Ok(())
    })
    .resolve::<LogErrorAndDefault>();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeConfigurationChanged<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
) {
    log::info!("configuration changed");
    host::configuration_changed();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeSurfaceDestroyed<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    host: i64,
) {
    log::info!("surface destroyed");
    host::surface_destroyed(host as host::HostId);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeHostDestroyed<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    host: i64,
) {
    log::info!("activity finishing; closing its window");
    host::host_destroyed(host as host::HostId);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeResumed<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    host: i64,
) {
    log::info!("activity resumed");
    diagnostics::set_foreground(true);
    host::resumed(host as host::HostId);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativePaused<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    host: i64,
) {
    log::info!("activity paused");
    diagnostics::set_foreground(false);
    host::paused(host as host::HostId);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeKeyboardInsets<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    visible: bool,
    height_px: i32,
) {
    diagnostics::set_keyboard(visible, height_px);
}

/// `ids`, `xs`, `ys` hold every pointer of the `MotionEvent` in index order,
/// in physical pixels relative to the `SurfaceView`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeMotion<'local>(
    mut env: EnvUnowned<'local>,
    _this: JObject<'local>,
    action: i32,
    action_index: i32,
    ids: JIntArray<'local>,
    xs: JFloatArray<'local>,
    ys: JFloatArray<'local>,
) -> bool {
    env.with_env(|env| -> jni::errors::Result<bool> {
        let count = ids.len(env)?;
        let mut id_buf = vec![0i32; count];
        let mut x_buf = vec![0f32; count];
        let mut y_buf = vec![0f32; count];
        ids.get_region(env, 0, &mut id_buf)?;
        xs.get_region(env, 0, &mut x_buf)?;
        ys.get_region(env, 0, &mut y_buf)?;
        let pointers: Vec<host::Pointer> = (0..count)
            .map(|i| host::Pointer {
                id: id_buf[i],
                x: x_buf[i],
                y: y_buf[i],
            })
            .collect();
        diagnostics::record_touch(action as u32, &pointers);
        Ok(host::motion_event(
            action as u32,
            action_index.max(0) as usize,
            &pointers,
        ))
    })
    .resolve::<LogErrorAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeKey<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    key_code: i32,
    action: i32,
    meta_state: i32,
) {
    host::key(key_code, action, meta_state);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_gpui_mobile_lab_LabActivity_nativeIme<'local>(
    mut env: EnvUnowned<'local>,
    _this: JObject<'local>,
    session: i64,
    kind: i32,
    text: JString<'local>,
    start: i32,
    end: i32,
) {
    env.with_env(|env| -> jni::errors::Result<()> {
        let text = if text.is_null() {
            String::new()
        } else {
            text.try_to_string(env)?
        };
        host::ime_event(
            session as u64,
            kind,
            text,
            start.max(0) as usize,
            end.max(0) as usize,
        );
        Ok(())
    })
    .resolve::<LogErrorAndDefault>();
}

/// Send the task to the background, as the launcher Activity does on back since
/// Android 12. Called when back reaches the catalog root with nothing to close.
pub fn move_task_to_back() {
    let result = mobile_jni::with_env(|env| {
        let activity = mobile_jni::activity(env)?;
        env.call_method(
            &activity,
            jni::jni_str!("moveTaskToBack"),
            jni::jni_sig!("(Z)Z"),
            &[jni::objects::JValue::Bool(true)],
        )
        .map_err(|e| {
            env.exception_clear();
            e.to_string()
        })?;
        Ok(())
    });
    if let Err(err) = result {
        log::error!("moveTaskToBack failed: {err}");
    }
}
