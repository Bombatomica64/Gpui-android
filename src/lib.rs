//! GPUI Kit Mobile Lab: an on-device component gallery for GPUI Kit v0.7.0.

extern crate gpui_mobile;

mod app;
pub mod diagnostics;
#[cfg(target_os = "android")]
mod host;
mod matrix;
mod screens;
mod ui;

use std::borrow::Cow;
use std::sync::OnceLock;

use gpui::{App, AppContext as _, WindowOptions};

pub const GPUI_KIT_VERSION: &str = "0.7.0";
pub const GPUI_VERSION: &str = "gpui-pre 0.3.7";
pub const GPUI_MOBILE_REVISION: &str = "f379bc8 + local patches";

/// Font files read from the APK before GPUI starts (see `host::load_bundled_fonts`).
pub static BUNDLED_FONTS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();

/// Runs on the GPUI render thread once the first surface exists.
pub fn launch(cx: &mut App) {
    log::info!("launch: initializing GPUI Kit {GPUI_KIT_VERSION}");
    if let Some(fonts) = BUNDLED_FONTS.get().filter(|fonts| !fonts.is_empty()) {
        let fonts = fonts.iter().map(|bytes| Cow::Owned(bytes.clone())).collect();
        if let Err(err) = cx.text_system().add_fonts(fonts) {
            log::error!("registering bundled fonts failed: {err:#}");
        }
    }
    gpui_kit::init(cx);
    app::init(cx);
    screens::init(cx);

    let result = gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
        screens::themes::apply_saved(window, cx);
        cx.new(|cx| app::LabApp::new(window, cx))
    });
    match result {
        Ok(_) => log::info!("launch: window open, catalog is live"),
        Err(err) => log::error!("launch: open_window failed: {err:#}"),
    }
    cx.activate(true);
}

/// `android-activity` (pulled in by gpui-mobile) links against this symbol even
/// though the lab never starts through NativeActivity.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(_app: android_activity::AndroidApp) {
    log::error!("android_main called: this app is hosted by LabActivity, not NativeActivity");
}
