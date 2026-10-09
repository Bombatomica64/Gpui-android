# Known problems and their fixes

Grouped by symptom. Check `MOBILE_PATCHES.md` and the lab's open issues for
anything newer.

## Blank or broken screen

| Symptom | Cause / fix |
|---|---|
| Blank screen on start | `host::on_open_window` registered after the first `surface_created`, or not at all. |
| Icons and SVGs missing | `host::start` instead of `host::start_with_assets(gpui_kit::assets::AllAssets, …)`. |
| Emoji show as boxes | Android 13+ ships a COLRv1 emoji font that swash can't draw. Bundle `assets/fonts/NotoColorEmoji.ttf` (CBDT) and call `set_host_activity` before `host::start`. Flags and ❤️ with VS16 still don't render. |
| Dark mode doesn't follow the system | The Activity lists `uiMode` in `configChanges` but doesn't call `host::configuration_changed()` from `onConfigurationChanged`. |
| Text white on white in a dark chat bubble; shimmer invisible | Kit bugs kit#3326, kit#3327; not Android-specific. |
| Bold/italic look regular | Roboto is a variable font; weights aren't applied. |
| Popover or submenu off screen | Not clamped to the viewport yet. |
| `ERROR_NATIVE_WINDOW_IN_USE_KHR` | A second Vulkan surface for the same window. gpui-mobile treats a repeated `surface_created` as a resize; don't destroy/recreate the window yourself. |

## Crashes and freezes

| Symptom | Cause / fix |
|---|---|
| Panic in `request_animation_frame` | Called outside `render` (e.g. a click handler). Call it from `render`. |
| App freezes while a picker/permission/biometric dialog is up, or returns "call it from a background thread" | Blocking package call on GPUI's thread. Use `cx.background_executor().spawn`. |
| JNI call returns `JavaException` for no visible reason | An earlier call in the same `with_env` closure left an exception pending. Clear it where it's thrown. |
| `ClassNotFound` for `dev.gpui.mobile.Gpui…` | The package's Java helper isn't in `build.sh`'s copy loop, or you used `FindClass` from a native thread instead of `find_app_class`. |
| Activity recreation loses the window | `HostId` not saved/restored across recreation. |
| UI thread stalls ~2 s on rotation/background | `surface_destroyed` blocks until the renderer releases the surface; expected, but never call GPUI synchronously from Java. |

## Layout and insets

- Safe-area insets are **not wired** on the host-driven path:
  `gpui_mobile::safe_area_insets()` returns zeros on Android and there's no
  `host::insets_changed` yet. The lab avoids the problem by targeting SDK 34
  (not edge-to-edge). Play requires SDK 35 for updates, which forces
  edge-to-edge; the plan is in the lab's `docs/safe-area-insets.md`.
- The keyboard resizes the window through `adjustResize`; Kit scrolls a
  focused input above the keyboard.
- Touch: drag, axis lock and autoscroll come from `gpui-pre` patches. If
  scrolling with a finger doesn't work, check the `[patch.crates-io]` pin.
- DataTable column resize/reorder and long-press tooltips don't work on
  touch.

## Build and install

| Symptom | Cause / fix |
|---|---|
| Release build gets OOM-killed | LTO release builds need lots of RAM. Build release in CI; locally use `--debug`. |
| Two copies of GPUI types don't match ("expected `gpui::App`, found `gpui::App`") | Two `gpui-pre` versions. `cargo tree -d`, then align revs/the patch entry. |
| `INSTALL_FAILED_UPDATE_INCOMPATIBLE` | The installed APK was signed with another key (each CI run uses a fresh debug key). Uninstall first. |
| Debug APK is huge | The library carries debug sections; `hotpatch fat` and `reload` strip it (255 → ~49 MB). Gradle repackaging in place also leaves dead space: delete the old APK before rebuilding. |
| APK contains `libgpui_mobile.so` too | gpui-mobile's own cdylib; `build.sh` deletes it since the app links it statically. |
| Wrong revision shown in diagnostics | `GPUI_MOBILE_REVISION` in the lab is a hard-coded string; update it with the pin. |

## Phone and adb

| Symptom | Cause / fix |
|---|---|
| `phone` exits 75 | Another session holds the phone. Do other work, retry with `--wait`. |
| `phone` exits 3, or adb hangs | The adb tunnel is down; ask the user to reconnect. Don't start adb yourself on the tunnel port. |
| `--es screen` ignored | Only read on a cold start. Force-stop first, or start with `-f 0x10008000`. |
| "Don't keep activities"/font scale don't recreate the Activity | Some ROMs don't; use `am start -f 0x10008000` to get a fresh task. |
| `screenrecord` fails | Blocked on some ROMs; capture with a `screencap` loop. |
