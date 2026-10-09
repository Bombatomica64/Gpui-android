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
        let mut loaded = Loaded::default();
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            let stamp = modified(&table_path);
            if stamp.is_none() || stamp == applied {
                continue;
            }
            // adb may still be writing the table: retry on the next tick.
            let Ok(patch) = std::fs::read(&table_path)
                .map_err(anyhow::Error::from)
                .and_then(|json| Ok(serde_json::from_slice(&json)?))
            else {
                continue;
            };
            applied = stamp;
            match loaded.apply(patch) {
                Ok(entries) => {
                    log::info!("hot: patch applied ({entries} hot closures)");
                    cx.update(|cx| cx.refresh_windows());
                }
                Err(err) => log::error!("hot: patch failed: {err:#}"),
            }
        }
    })
    .detach();
}

/// The patches loaded so far: their device path and load bias.
#[derive(Default)]
struct Loaded(Vec<(String, u64)>);

impl Loaded {
    /// `patch` is `{ table, earlier: [{ lib, map: [[old, new]] }] }` from
    /// tools/hotpatch. Views created while an earlier patch was active call
    /// that patch's hot closures; Subsecond only redirects the app's own, so
    /// those entries are rebased here to where the earlier patch was loaded.
    fn apply(&mut self, mut patch: serde_json::Value) -> anyhow::Result<usize> {
        let mut table: subsecond::JumpTable = serde_json::from_value(patch["table"].take())?;
        // apply_patch adds this to every key.
        let slide = (subsecond::aslr_reference() as u64).wrapping_sub(table.aslr_reference);
        for earlier in patch["earlier"].as_array().into_iter().flatten() {
            let lib = earlier["lib"].as_str().unwrap_or_default();
            let Some(&(_, bias)) = self.0.iter().find(|(l, _)| l == lib) else {
                continue;
            };
            for pair in earlier["map"].as_array().into_iter().flatten() {
                if let (Some(old), Some(new)) = (pair[0].as_u64(), pair[1].as_u64()) {
                    table.map.insert(bias.wrapping_add(old).wrapping_sub(slide), new);
                }
            }
        }
        let entries = table.map.len();
        let lib = table.lib.to_string_lossy().into_owned();
        let before = patch_biases();
        // SAFETY: the table comes from tools/hotpatch, built against this process's
        // aslr_reference; we are between frames, so no patched code is running.
        unsafe { subsecond::apply_patch(table) }?;
        if let Some(bias) = patch_biases().into_iter().find(|b| !before.contains(b)) {
            self.0.push((lib, bias));
        }
        Ok(entries)
    }
}

/// Load biases of the patches Subsecond loaded (all named `/subsecond-patch`).
fn patch_biases() -> Vec<u64> {
    unsafe extern "C" fn each(
        info: *mut libc::dl_phdr_info,
        _: libc::size_t,
        out: *mut libc::c_void,
    ) -> libc::c_int {
        // SAFETY: called by dl_iterate_phdr with a valid entry and our Vec.
        unsafe {
            let info = &*info;
            if !info.dlpi_name.is_null()
                && std::ffi::CStr::from_ptr(info.dlpi_name).to_bytes() == b"/subsecond-patch"
            {
                (*(out as *mut Vec<u64>)).push(info.dlpi_addr as u64);
            }
        }
        0
    }
    let mut out = Vec::new();
    // SAFETY: `each` only reads the entry and writes to `out`.
    unsafe { libc::dl_iterate_phdr(Some(each), &mut out as *mut Vec<u64> as *mut libc::c_void) };
    out
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
