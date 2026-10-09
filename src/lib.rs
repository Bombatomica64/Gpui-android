//! GPUI Kit Mobile Lab: an on-device component gallery for GPUI Kit v0.7.1.

extern crate gpui_mobile;

mod app;
pub mod diagnostics;
#[cfg(target_os = "android")]
mod host;
mod matrix;
mod screens;
mod ui;

use gpui::{App, AppContext as _, WindowOptions};

pub const GPUI_KIT_VERSION: &str = "0.7.1";
pub const GPUI_VERSION: &str = "gpui-pre 0.3.8";
pub const GPUI_MOBILE_REVISION: &str = "Bombatomica64/gpui-mobile a6fc468 (platform-api-lab)";

/// Runs on the GPUI render thread once the first surface exists.
pub fn launch(cx: &mut App) {
    log::info!("launch: initializing GPUI Kit {GPUI_KIT_VERSION}");
    gpui_kit::init(cx);
    app::init(cx);
    screens::init(cx);
    cx.activate(true);
}

/// Opens the catalog in a new host Activity's window, on the GPUI render thread.
pub fn open_window(cx: &mut App) {
    let result = gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
        screens::themes::apply_saved(window, cx);
        cx.new(|cx| app::LabApp::new(window, cx))
    });
    match result {
        Ok(_) => log::info!("launch: window open, catalog is live"),
        Err(err) => log::error!("launch: open_window failed: {err:#}"),
    }
}

/// `android-activity` (pulled in by gpui-mobile) links against this symbol even
/// though the lab never starts through NativeActivity.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(_app: android_activity::AndroidApp) {
    log::error!("android_main called: this app is hosted by LabActivity, not NativeActivity");
}
