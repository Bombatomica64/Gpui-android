---
name: gpui-mobile
description: Build, run and debug Android apps written in Rust with GPUI, using the Bombatomica64 forks (gpui-pre, gpui-kit `mobile-lab`, gpui-mobile `main`) and the GPUI Mobile Lab as the reference app. Covers starting a new app (Cargo pins, the Java host Activity, manifest), the dev loop (hot patching with Subsecond, fast library reload, full APK builds, CI), platform packages (share, pickers, permissions, notifications, deep links…), keyboards and TalkBack, and testing on a shared phone. Use this whenever someone works on the Gpui-android lab or gpui-mobile, wants a Rust/GPUI app on a phone, asks about hot reload or hot patching of Rust on Android, adds a screen, calls a gpui_mobile package, wires JNI or a host Activity, or debugs an APK, even if they don't say "skill" or name the forks.
---

# GPUI on Android (Bombatomica64 stack)

GPUI is Zed's Rust UI framework. This stack runs it on Android:

| Layer | Crate | Where |
|---|---|---|
| GPUI | `gpui-pre` 0.3.8 + touch/IME patches | `Bombatomica64/gpui-pre` branch `mobile-lab`, via `[patch.crates-io]` |
| Components | `gpui-kit` 0.7.1 (`gpui-component`) | `Bombatomica64/gpui-kit` branch `mobile-lab` |
| Platform | `gpui-pre-mobile` (`use gpui_mobile::…`) | `Bombatomica64/gpui-mobile` `main` |
| Reference app | GPUI Mobile Lab | `Bombatomica64/Gpui-android` |

The app is a Rust `cdylib`, loaded by a small Java host Activity that owns a
`SurfaceView` and forwards lifecycle, touch, keys and IME events to Rust. One
GPUI render thread (`gpui-main`) lives for the whole process, so app state
survives Activity recreation.

The lab is both the demo and the template: when in doubt, copy what it does.
Its `MOBILE_PATCHES.md` says which fork branch carries which fix and which
revs are pinned; read it before changing a pin.

## Pick your task

- **Start a new app, or wire an existing Rust crate into Android** →
  [references/new-app.md](references/new-app.md) (Cargo pins, host Activity,
  manifest, `build.sh`).
- **Make the edit-to-phone loop fast** →
  [references/hot-reload.md](references/hot-reload.md) (hot patching with
  Subsecond, library reload, limits).
- **Call a device API** (share, clipboard, pickers, camera, permissions,
  notifications, deep links, location, audio…) →
  [references/platform-packages.md](references/platform-packages.md).
- **Text fields, keyboards, Back, TalkBack** →
  [references/text-input-a11y.md](references/text-input-a11y.md).
- **Something renders wrong or crashes** →
  [references/gotchas.md](references/gotchas.md) first; most problems seen so
  far are listed there.

## The dev loop

Choose the cheapest step that covers your change. Measured on a OnePlus
(Android 16) reached over an ssh adb tunnel; over USB the push steps are
several times faster.

| Change | Tool | Time to screen | State kept |
|---|---|---|---|
| Render code in the app crate (text, colours, layout, handlers built in `render`) | `hotpatch watch` | ~5–6 s (1.2–1.6 s compile) | yes: screen, input, view state |
| Anything else in the app crate, struct changes, gpui/Kit/gpui-mobile changes, new deps | `hotpatch reload` | ~15 s, no Gradle, no reinstall | only the open screen |
| Java, manifest, assets, new packages' Java helpers | `./build.sh android --debug`, then install | Gradle + install | no |
| Release APK | CI (`.github/workflows/android.yml`, artifact `gpui-mobile-lab-apk`) | 4–6 min | — |

Hot patching and reload live in `tools/hotpatch` and `src/hot.rs`. They were
built in the lab's hot-patching work (PRs #30/#51 and the `dev-reload`
branch); if they are missing from your checkout, those haven't merged yet,
so say so instead of guessing at commands.

Local builds: debug arm64 builds fit on a small server (clean build ~5 min,
~2 GB peak per rustc). Release builds with LTO need far more memory; on a
shared or small machine, push and let CI build the release APK instead of
building it locally. Wait on CI with a background waiter, not a sleep loop.

## Testing on a phone

- **Shared phone.** If the machine has a `phone` command
  (`command -v phone`), the phone is shared between agents and sessions.
  Then:
  - Never run `adb` directly. Wrap a whole test in
    `phone session [--wait SECS] -- <script>` and use `phone adb …` inside
    it. `phone status` shows who holds it.
  - Keep sessions short. Don't hold the lock while compiling or waiting
    on CI.
  - Reinstall your own APK at the start of a session, because another
    session may have replaced it.
  - Restore what you changed at the end: night mode, rotation,
    `adb reverse` rules, accessibility settings.
  - Exit 75 means the phone is busy; do other work and retry. Exit 3 means
    the tunnel is down; ask the user to reconnect it.
  
  Without `phone`, use `adb` as usual. If the setup uses a forwarded adb
  server port, export `ANDROID_ADB_SERVER_PORT`, and check the tunnel is up
  first: an adb call while it's down starts a local server on that port and
  blocks the tunnel.
- **Don't tap blind.** Before scripted taps or text input, check that the app
  is in front (`dumpsys activity activities | grep -i resumed`), and stop the
  script at the first failed step. Input sent to the wrong app does real
  damage.
- **Installing a CI APK.** CI signs with the runner's throwaway debug key, so
  `install -r` over a different build fails with
  `INSTALL_FAILED_UPDATE_INCOMPATIBLE`. Uninstall first
  (`uninstall dev.gpui.mobile.lab`), which also clears app data.
- **Open a screen directly** (cold start only):
  `am start -n dev.gpui.mobile.lab/.LabActivity --es screen "'Text Input'"`.
  To force a fresh task: add `-f 0x10008000`.
- **Logs:** `logcat -s GPUI_MOBILE_LAB AndroidRuntime DEBUG`. The panic hook
  logs message, location and backtrace before the abort.
- **Evidence:** `screencap -p` works; `screenrecord` may be blocked on some
  ROMs, so build GIFs from a screencap loop if needed.

## Rules that save hours

These come from bugs actually hit in this stack.

1. **Never block GPUI's thread or the Android UI thread on the user.**
   Pickers, permission dialogs and biometric prompts wait for the user. Call
   them from `cx.background_spawn(...)`; from GPUI's thread they return an
   error on purpose.
2. **Host wiring order matters.** In the host's `onCreate`, call
   `set_host_activity` first, then register `host::on_open_window`, then
   `host::start_with_assets(gpui_kit::assets::AllAssets, launch)`. Without
   `start_with_assets` Kit's icons are blank; without `on_open_window` before
   the first surface, the screen stays blank.
3. **Pin by rev.** The fork `main` branches move. Pin every git dependency by
   `rev`, keep exactly one version of each `gpui-pre-*` crate
   (`cargo tree -d`), and keep the `[patch.crates-io]` entry for `gpui-pre`;
   touch scrolling, drag and keyboard types only exist there.
4. **`default-features = false` on gpui-mobile** and enable only the packages
   you use. The defaults pull ~30 packages that need their own Java helpers
   and manifest entries.
5. **JNI from app code goes through `gpui_mobile::android::jni::with_env`.**
   It attaches the thread, gives a local frame, and turns a pending Java
   exception into an `Err`. Get the Activity with `jni::activity(env)` and a
   long-lived `Context` with `jni::application_context(env)`; never cache raw
   Activity pointers. Load app classes with `find_app_class`, since
   `FindClass` on a native thread can't see them.
6. **If the Activity handles `uiMode` in `configChanges`, call
   `host::configuration_changed()` from `onConfigurationChanged`**, or dark
   mode never updates.
7. **`window.request_animation_frame()` only inside `render`.** Called from a
   click handler it panics.

## Contributing back

The forks exist to carry fixes until upstream (zed-industries/zed for GPUI,
longbridge/gpui-component and longbridge/gpui-mobile) takes them. Bombatomica64
repos take PRs freely. Anything aimed at upstream should be one concern per
PR, small, with device screenshots or GIFs, and should be shown to the repo
owner before it's opened. Credit borrowed code (for example Dioxus in
`tools/hotpatch`) in headers and keep its licence file.
