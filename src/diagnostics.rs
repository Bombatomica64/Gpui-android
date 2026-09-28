//! Process-wide facts reported by the Java host, read by the Diagnostics screen.
//!
//! Only values the platform actually delivers are stored here; nothing is estimated.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

static API_LEVEL: AtomicI32 = AtomicI32::new(0);
static FOREGROUND: AtomicBool = AtomicBool::new(false);
static KEYBOARD_VISIBLE: AtomicBool = AtomicBool::new(false);
static KEYBOARD_HEIGHT_PX: AtomicI32 = AtomicI32::new(0);
static TOUCH_EVENTS: AtomicU64 = AtomicU64::new(0);
static LAST_TOUCH: Mutex<Option<TouchSample>> = Mutex::new(None);
/// Vertical scroll offset of the open screen's container, in logical pixels.
static SCREEN_SCROLL_Y: AtomicI32 = AtomicI32::new(0);

static REQUESTED_SCREEN: Mutex<Option<String>> = Mutex::new(None);
static BLUR_REQUESTED: AtomicBool = AtomicBool::new(false);
/// 0 = unknown, 1 = light, 2 = dark; bit 4 set while not yet applied to GPUI.
static NIGHT_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn set_night_mode(night: bool) {
    NIGHT_MODE.store(if night { 2 | 4 } else { 1 | 4 }, Ordering::Relaxed);
}

/// The night mode reported by the host, if it changed since the last call.
pub fn take_night_mode_change() -> Option<bool> {
    let value = NIGHT_MODE.fetch_and(!4, Ordering::Relaxed);
    (value & 4 != 0).then_some(value & 3 == 2)
}

pub fn request_blur() {
    BLUR_REQUESTED.store(true, Ordering::Relaxed);
}

pub fn take_blur_request() -> bool {
    BLUR_REQUESTED.swap(false, Ordering::Relaxed)
}

/// A screen asked for through the launch intent (`--es screen <title>`).
pub fn request_screen(screen: String) {
    *REQUESTED_SCREEN.lock().expect("poisoned") = Some(screen);
}

pub fn take_requested_screen() -> Option<String> {
    REQUESTED_SCREEN.lock().expect("poisoned").take()
}

pub fn set_screen_scroll(y: f32) {
    SCREEN_SCROLL_Y.store(y.round() as i32, Ordering::Relaxed);
}

pub fn screen_scroll() -> i32 {
    SCREEN_SCROLL_Y.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug)]
pub struct TouchSample {
    pub action: u32,
    pub pointers: usize,
    pub x: f32,
    pub y: f32,
}

pub fn set_api_level(level: i32) {
    API_LEVEL.store(level, Ordering::Relaxed);
}

pub fn api_level() -> Option<i32> {
    Some(API_LEVEL.load(Ordering::Relaxed)).filter(|level| *level > 0)
}

pub fn set_foreground(foreground: bool) {
    FOREGROUND.store(foreground, Ordering::Relaxed);
}

pub fn is_foreground() -> bool {
    FOREGROUND.load(Ordering::Relaxed)
}

pub fn set_keyboard(visible: bool, height_px: i32) {
    KEYBOARD_VISIBLE.store(visible, Ordering::Relaxed);
    KEYBOARD_HEIGHT_PX.store(height_px, Ordering::Relaxed);
}

/// Whether the software keyboard is up, and its height in physical pixels.
pub fn keyboard() -> (bool, i32) {
    (
        KEYBOARD_VISIBLE.load(Ordering::Relaxed),
        KEYBOARD_HEIGHT_PX.load(Ordering::Relaxed),
    )
}

#[cfg(target_os = "android")]
pub fn record_touch(action: u32, pointers: &[gpui_mobile::android::host::Pointer]) {
    TOUCH_EVENTS.fetch_add(1, Ordering::Relaxed);
    if let Some(first) = pointers.first() {
        *LAST_TOUCH.lock().expect("poisoned") = Some(TouchSample {
            action,
            pointers: pointers.len(),
            x: first.x,
            y: first.y,
        });
    }
}

pub fn touch_events() -> u64 {
    TOUCH_EVENTS.load(Ordering::Relaxed)
}

pub fn last_touch() -> Option<TouchSample> {
    *LAST_TOUCH.lock().expect("poisoned")
}
