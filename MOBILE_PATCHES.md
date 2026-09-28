# Local patches to upstream crates

The lab uses one GPUI snapshot (`gpui-pre` 0.3.7) for everything. Three
upstream crates are vendored under `vendor/` and changed as little as possible.
Each vendored crate was first committed unmodified, then each patch as its own
commit, so `git log -- vendor` shows exactly what changed. The same patches are
exported in [`patches/`](patches/) (paths relative to `vendor/`).

| # | Crate (vendored copy) | Upstream |
|---|---|---|
| 1–3 | `vendor/gpui-mobile` — `gpui-pre-mobile` 0.1.0 at `f379bc8` | https://github.com/longbridge/gpui-mobile |
| 4 | `vendor/gpui-base` — `gpui-base` 0.7.0 | https://github.com/longbridge/gpui-kit (`crates/base`) |
| 5 | `vendor/gpui-fps` — `gpui-fps` 0.7.0 | https://github.com/longbridge/gpui-kit (`crates/fps`) |

`gpui-mobile` is a path dependency; `gpui-base` and `gpui-fps` replace the
crates.io releases through `[patch.crates-io]` in `Cargo.toml`, so `gpui-kit`
and `gpui-component` 0.7.0 still come from crates.io unchanged.

## 1. gpui-mobile: build against gpui-pre 0.3.7

- **Files:** `Cargo.toml`, `src/android/window.rs`, `src/ios/window.rs`
- **Problem:** GPUI Kit 0.7.0 pins `gpui-pre =0.3.7`; gpui-mobile `main`
  pins `=0.3.6`. Cargo would build two incompatible GPUIs (`App`/`Window`
  types from different crates). In 0.3.7, `gpui-pre-wgpu`'s
  `WgpuRenderer::gpu_specs()` returns `Option<GpuSpecs>`.
- **Fix:** bump `gpui-pre` and `gpui-pre-wgpu` to `=0.3.7`; `gpu_specs` uses
  `and_then` instead of `map`. `cargo tree` then shows every `gpui-pre-*`
  crate at 0.3.7 and a single `wgpu` 29.0.4.
- **Upstream PR?** Yes — this is the routine snapshot bump the fork already
  does per GPUI release (#18 bumped to 0.3.6).

## 2. gpui-mobile: back the Android platform clipboard with ClipboardManager

- **Files:** `src/android/platform.rs`
- **Problem:** `AndroidClipboard` (what GPUI's `write_to_clipboard` /
  `read_from_clipboard` use, and therefore Kit's Copy/Cut/Paste and
  `Clipboard` component) was an in-process string. Copying in the app never
  reached Android's clipboard, and text copied in other apps could not be
  pasted.
- **Fix:** with the crate's existing `clipboard` feature, read/write through
  the existing `packages::clipboard` JNI helper (`dev.gpui.mobile.GpuiClipboard`
  → `ClipboardManager`); fall back to the local copy (with a logged warning)
  when the call fails. Verified: Kit's Copy shows Android 13's clipboard
  overlay, and the copied text reads back.
- **Upstream PR?** Yes. Small, feature-gated, reuses code already in the crate.

## 3. gpui-mobile host: only tear down the surface a destroy names

- **Files:** `src/android/host.rs`
- **Problem:** on the host-driven entry point (`android::host`), when an
  Activity is recreated in the same process (e.g. `FLAG_ACTIVITY_CLEAR_TASK`),
  the *new* Activity's `surfaceCreated/Changed` can arrive before the *old*
  Activity's `surfaceDestroyed`. The render thread attached the new surface,
  then processed the unqualified destroy and unconfigured the new surface —
  a black screen with a working app underneath.
- **Fix:** add `surface_destroyed_for(&NativeWindow)`; the render thread only
  tears down the surface that is actually attached and ignores late destroys
  of older surfaces. The blocking wait now uses a request/ack counter instead
  of a flag that a newer surface could clear. `surface_destroyed()` keeps its
  signature and old behavior for existing hosts. Verified: 5 consecutive
  in-process recreations, all late destroys ignored, rendering and touch
  intact, GPUI state preserved.
- **Upstream PR?** Yes. It fixes a real race in the recreation path that
  PR #10 introduced, and is backward compatible.

## 4. gpui-base: let Slider claim touch drags on its track

- **Files:** `src/slider.rs`
- **Problem:** GPUI 0.3.7 turns a finger drag into either a `TouchDragEvent`
  (only if an element claims it with `prevent_default` on `Started`) or into
  scrolling. `Slider` only used GPUI's mouse `on_drag`/`on_drag_move`, so on
  a touch screen the thumb could not be dragged (tapping the track worked).
- **Fix:** `SliderTrack` adds an absolutely positioned layer that claims
  touch drags starting on the track and maps `Started`/`Moved` to
  `update_value_by_position` and `Ended`/`Cancelled` to `handle_release` —
  the same pattern Kit's own scrollbar thumb uses. Range sliders pick the
  nearer thumb. Verified on horizontal, vertical and range sliders (and
  therefore ColorPicker's HSLA sliders, which reuse Slider).
- **Upstream PR?** Yes. Mouse behavior is unchanged. The same gap affects
  every drag built on GPUI `on_drag` (Resizable, Dock, DataTable column
  resize/move, List reorder); those were not patched and are reported in the
  matrix. A general fix belongs in GPUI's drag-and-drop.

## 5. gpui-fps: use Droid Sans Mono on Android

- **Files:** `src/monitor.rs`
- **Problem:** the HUD asks for the generic `monospace` family on every
  non-macOS/Windows/iOS target. Android's cosmic-text backend has no such
  alias, and neither GPUI's desktop fallbacks, so GPUI panics
  (`failed to resolve font 'monospace' or any of the fallbacks`) the moment
  the HUD renders. The crate already special-cases iOS for the same reason.
- **Fix:** on Android use `Droid Sans Mono`, which every Android ships and
  gpui-mobile loads from `/system/fonts`.
- **Upstream PR?** Yes, one `cfg`.

## Not patched, handled in the host app instead

These are real platform gaps, but the fix belongs in the embedding host (the
lab's `LabActivity` and `src/host.rs`, `src/app.rs`), so no upstream code was
changed. Each is a candidate for gpui-mobile's example host or documentation.

- **IME dismissal injects `escape`.** When the IME is hidden with back (or
  the Done key), gpui-mobile's IME bridge (event kind 4) synthesizes an
  `escape` keystroke. An app that maps escape/back to navigation then also
  navigates (e.g. hiding the keyboard in a dialog also closed the dialog).
  The lab's host blurs the focused input instead.
- **No appearance source on the host path.** Night mode is only queried on
  the NativeActivity path, so `window.appearance()` never changes. The host
  forwards `uiMode` from `onCreate`/`onConfigurationChanged` to
  `AndroidWindow::set_appearance` on the render thread.
- **Bundled emoji font on the host path.** Android 13+'s system emoji font is
  COLRv1, which swash cannot draw; gpui-mobile's CBDT fallback reads APK
  assets through `AndroidApp`, which the host path does not have. The host
  reads `assets/fonts/NotoColorEmoji.ttf` via `AAssetManager_fromJava` and
  registers it with `cx.text_system().add_fonts`.
- **Keyboard avoidance.** `adjustResize` shrinks GPUI's viewport, but neither
  GPUI nor Kit scrolls a focused input back into view (Kit's mobile guide
  leaves keyboard avoidance to the host). The lab shell scrolls the focused
  input above the keyboard once per change of focus or viewport height.
- **Axis restriction.** GPUI turns a pure horizontal pan into vertical
  scrolling of a vertical-only container unless `restrict_scroll_to_axis()`
  is set; the lab sets it on its page containers.
