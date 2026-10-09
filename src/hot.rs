//! Dev-only Rust hot patching with Subsecond (step A1 of docs/hot-patching-plan.md).
//!
//! `tools/hotpatch` builds a patch library from the changed lab crate, pushes it
//! to [`DIR`] and writes a jump table next to it. A task on the GPUI thread polls
//! for that file, loads the patch between frames and refreshes every window.
//!
//! Only functions reached through [`call`] switch to new code, so each lab
//! `Render::render` wraps its body in it: GPUI calls renders through vtables
//! built when the entity was created, which a patch cannot redirect. Everything
//! is a no-op without debug assertions.

use std::{path::Path, time::Duration};

use gpui::App;

/// Where `tools/hotpatch` pushes patches (readable by the app, writable by `adb`).
const DIR: &str = "/data/local/tmp/gpui-hot";

/// Runs `f`, or its newest patched version once a patch has been applied.
#[inline(always)]
pub fn call<R>(f: impl FnMut() -> R) -> R {
    subsecond::call(f)
}

pub fn init(cx: &mut App) {
    if !cfg!(debug_assertions) {
        return;
    }
    // tools/hotpatch reads this line to link patches against this process.
    log::info!(
        "hot: aslr_reference={:#x} pid={}",
        subsecond::aslr_reference(),
        std::process::id()
    );
    cx.spawn(async move |cx| {
        let table_path = Path::new(DIR).join("patch.json");
        // A table left by an earlier process targets other addresses.
        let modified = |path: &Path| std::fs::metadata(path).and_then(|m| m.modified()).ok();
        let mut applied = modified(&table_path);
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            let stamp = modified(&table_path);
            if stamp.is_none() || stamp == applied {
                continue;
            }
            // adb may still be writing the table: retry on the next tick.
            let Ok(table) = std::fs::read(&table_path)
                .map_err(anyhow::Error::from)
                .and_then(|json| Ok(serde_json::from_slice(&json)?))
            else {
                continue;
            };
            applied = stamp;
            match apply(table) {
                Ok(entries) => {
                    log::info!("hot: patch applied ({entries} functions)");
                    cx.update(|cx| cx.refresh_windows());
                }
                Err(err) => log::error!("hot: patch failed: {err:#}"),
            }
        }
    })
    .detach();
}

fn apply(table: subsecond::JumpTable) -> anyhow::Result<usize> {
    let entries = table.map.len();
    // SAFETY: the table comes from tools/hotpatch, built against this process's
    // aslr_reference; we are between frames, so no patched code is running.
    unsafe { subsecond::apply_patch(table) }?;
    Ok(entries)
}

/// Subsecond's address reference point: it looks `main` up with `dlsym` in the
/// running process and in each patch.
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "C" fn main() {}

/// Implements `Render` for each type by calling its inherent `render_view`
/// through [`call`], so patches reach views GPUI created before the patch.
#[macro_export]
macro_rules! hot_render {
    ($($ty:ty),* $(,)?) => {$(
        impl gpui::Render for $ty {
            fn render(
                &mut self,
                window: &mut gpui::Window,
                cx: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                $crate::hot::call(|| {
                    gpui::IntoElement::into_any_element(self.render_view(window, cx))
                })
            }
        }
    )*};
}
