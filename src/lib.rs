//! GPUI Kit Mobile Lab: an on-device component gallery for GPUI Kit v0.7.0.

extern crate gpui_mobile;

pub mod diagnostics;
#[cfg(target_os = "android")]
mod host;

use gpui::{App, AppContext as _, ParentElement as _, Render, Styled as _, WindowOptions, div};

struct Hello;

impl Render for Hello {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        div().size_full().child("GPUI Kit Mobile Lab")
    }
}

/// Runs on the GPUI render thread once the first surface exists.
pub fn launch(cx: &mut App) {
    gpui_kit::init(cx);
    let result = gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Hello));
    if let Err(err) = result {
        log::error!("open_window failed: {err:#}");
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
