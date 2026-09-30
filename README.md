# GPUI Kit Mobile Lab (Android)

An on-device test gallery for [GPUI Kit](https://github.com/longbridge/gpui-kit)
**v0.7.0** on Android: a phone-sized version of Kit's story gallery. Every
user-facing component has a demo screen that exercises its states and events
(each interactive screen shows the current value and an event log), plus lab
tools for touch, scrolling, diagnostics, rendering stress and themes.

- Component coverage and per-component Android status: **[COMPONENT_MATRIX.md](COMPONENT_MATRIX.md)**
  (the app renders its catalog from this file).
- Changes made to upstream crates: **[MOBILE_PATCHES.md](MOBILE_PATCHES.md)**.
- Assessment and possible work for the GPUI / GPUI Kit teams:
  **[CONTRIBUTING_IDEAS.md](CONTRIBUTING_IDEAS.md)**.

## Tested dependency revisions

One GPUI snapshot is used throughout; `cargo tree` resolves exactly one
version of every `gpui-pre-*` crate.

| Dependency | Version / revision |
|---|---|
| GPUI Kit (`gpui-kit`, `gpui-component`) | 0.7.0 from crates.io (tag `v0.7.0`, `0c830f4d`) |
| `gpui-base` | 0.7.0, vendored with patch 4 (`vendor/gpui-base`) |
| `gpui-fps` | 0.7.0, vendored with patch 5 (`vendor/gpui-fps`) |
| GPUI (`gpui-pre`, `gpui-pre-wgpu`, …) | 0.3.7 (the snapshot GPUI Kit 0.7.0 pins) |
| gpui-mobile (`gpui-pre-mobile`) | [longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) `f379bc81a2c55e0634bfd6625263a235578986c3`, vendored with patches 1–3 (`vendor/gpui-mobile`) |
| Renderer | `wgpu` 29.0.4 (Vulkan preferred, GLES fallback) |
| Rust | 1.98.1 stable, edition 2024 |
| Android | NDK 27.2.12479018, compileSdk 35, targetSdk 34, minSdk 26, AGP 9.1.0, Gradle 9.4.1, JDK 17 |

## Build

Prerequisites (Linux or macOS):

```sh
# Rust + Android targets + cargo-ndk
curl -sSf https://sh.rustup.rs | sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk

# JDK 17 and the Android SDK/NDK (command-line tools)
sdkmanager "platform-tools" "platforms;android-35" "build-tools;35.0.0" "ndk;27.2.12479018"
export ANDROID_HOME=$HOME/Android/Sdk          # build.sh defaults to this path
```

Build the APK:

```sh
./build.sh android --release                 # arm64-v8a (phones)
./build.sh android --release --abi all       # arm64-v8a + x86_64 (phones and emulators)
./build.sh android --debug                   # unoptimized native code
./build.sh android --release --install       # also adb install + launch
```

The script compiles the Rust library with `cargo ndk` into
`android/app/src/main/jniLibs/`, then runs Gradle (`assembleRelease` or
`assembleDebug`) and copies the result to
`dist/gpui-mobile-lab-<version>-<profile>.apk`. Once the native libraries are
in `jniLibs/`, `cd android && ./gradlew assembleRelease` works on its own.

The release APK is signed with the local Android **debug** key so it can be
sideloaded without a keystore. It is a test app, not for store distribution.

## Install

```sh
adb install -r dist/gpui-mobile-lab-0.1.2-release.apk
adb shell am start -n dev.gpui.mobile.lab/.LabActivity
```

Or download the APK from the GitHub release and open it on the phone (allow
installs from unknown sources).

Logs and panics go to logcat under the tag `GPUI_MOBILE_LAB` (a panic logs its
message, location and a backtrace before the process aborts):

```sh
adb logcat -s GPUI_MOBILE_LAB AndroidRuntime DEBUG
```

Testing aid: open a screen directly from adb (the title is the "Demo screen"
column of the matrix):

```sh
adb shell am start -n dev.gpui.mobile.lab/.LabActivity --es screen "'Text Input'"
```

## Using the app

The catalog lists every component grouped by category, with version info,
Android API level, viewport size, a search field and working/partial/broken
counters (tap a counter to filter). Tap a component to open its screen; the
header's **‹ Catalog** button and Android back return to the catalog. Back
closes the keyboard, then any open overlay, then the screen; at the catalog
it sends the app to the background like a launcher Activity.

Lab tools: **Touch Lab** (tap, double-tap, long-press, drag, pinch counters),
**Scroll Stress** (100 rows, 10,000 virtual rows, nested axes, long text),
**Diagnostics** (viewport, scale, visual viewport, keyboard, focus, scroll,
foreground state, gpui-fps HUD, render cadence), **Stress Test**
(Start/Stop/Reset over 120 buttons, 400 text nodes, inputs, a 5,000-row list
and a per-frame chart) and **Themes** (System/Light/Dark and bundled Kit
themes, switched at runtime).

## Android host

`android/app/src/main/java/dev/gpui/mobile/lab/LabActivity.java` is a plain
`Activity` with a `SurfaceView`, driving gpui-mobile's host entry point
(`gpui_mobile::android::host`, see `src/host.rs`). GPUI runs on one
process-lived render thread, so the app keeps its state across surface
destruction (Home/resume) and Activity recreation. The Activity forwards
touch (`MotionEvent`), keys (back arrives in GPUI as `escape`), lifecycle,
night mode and an IME `InputConnection` proxy adapted from gpui-mobile's
`GpuiInputActivity`. `adjustResize` shrinks GPUI's viewport above the
keyboard; the app is locked to portrait.

## How statuses were verified

No physical phone or KVM-capable emulator was available while building this.
All statuses in the matrix come from running the release APK (x86_64 ABI) on
**[Redroid](https://github.com/remote-android/redroid-doc) Android 13
(API 33)** in Docker on the host kernel, configured as a 1080×2340, 420 dpi
phone (411×819 pt at 2.625×), with the stock AOSP keyboard (LatinIME).
Interactions were driven with `adb shell input` (taps, swipes, raw
`motionevent` DOWN/MOVE/UP for long-press and drags, key events and IME
typing), and verified from screenshots and the in-app event logs in logcat.

Several screens were later checked on a physical phone (360×736 pt); the
Bubble, ShimmerText and Toggle notes in the matrix come from that.

Limits of that setup:

- Rendering is CPU-based (SwiftShader Vulkan). GPUI's own draw time was
  ~0.5–2.4 ms per frame, but continuously animated frames arrived only every
  ~1.2 s, and timers ran several times slower than requested. **No
  performance number from this setup says anything about a phone GPU** — use
  Diagnostics and Stress Test on a real device.
- The arm64-v8a library in the APK was built and packaged but not executed.
- Multi-touch (pinch) and real finger physics could not be exercised with
  adb injection.

## Known Android issues

Each is detailed in the matrix; fixes applied locally are in MOBILE_PATCHES.md.

**Broken on a touch-only phone**

- `OtpInput` and `TimeField` (and the time row of a date+time `DatePicker`)
  never raise the soft keyboard: they have no text-input handler and do not
  call `request_virtual_keyboard` (which gpui-mobile does not implement
  either). Editing needs a hardware keyboard.
- Context menus (`ContextMenuExt`) open only on the right mouse button;
  long-press does nothing.
- Everything built on GPUI drag-and-drop (`on_drag`) ignores finger drags:
  Resizable dividers, Dock splitters and tab dragging, DataTable column
  resize/reorder. (Slider had the same problem; patch 4 fixes it.)

**Partially working**

- Bold and italic text render as regular weight (Android's Roboto is a
  variable font; the cosmic-text stack does not apply weight/italic
  variations). Headings are larger but not bold.
- TextView long-press selection shows handles and a Copy/Select All menu, but
  the initial selection is not reliably the pressed word.
- Popovers and submenus are not moved back into the viewport: a popover near
  the right edge and submenus are clipped. Breadcrumb, full Pagination,
  static Table, DescriptionList, Settings and long Accordion titles do not
  adapt to a 411 pt width.
- Carousel: one fling can skip several slides; its prev/next buttons render
  outside the card.
- `NumberInput` and masked number inputs open the full text keyboard; gpui-mobile
  always requests the default keyboard type.
- A disabled `Radio` inside a `RadioGroup` is still selectable; a `.rows(5)`
  Textarea renders one row tall (both not Android-specific; reported as
  longbridge/gpui-kit#3324 and #3325, fixes proposed in #3331 and #3330).
- In dark mode, a TextView inside a `Filled` chat `Bubble` renders white on
  white (TextView ignores the bubble's text color), and the default
  `ShimmerText` highlight is invisible on regular text. Both are not
  Android-specific (#3326 and #3327, fixes proposed in #3329 and #3328); the
  demos work around them.
- Kit tooltips are disabled on mobile by design; a raw GPUI `.tooltip()` did
  not show on long-press either.
- Closing a sheet while a fling is still running hands the remaining momentum
  to the page behind; a horizontal fling that reaches the end of a TabBar
  can continue as vertical page scrolling.
- After a tap, the touched menu row keeps its hover highlight.

**Worked around in this app (candidates for gpui-mobile / Kit)**

- The IME's back-to-dismiss makes gpui-mobile inject an `escape` keystroke,
  which apps also treat as navigation. The host blurs the input instead.
- Neither GPUI nor Kit scrolls a focused input above the keyboard. The shell
  does it (`LabApp::reveal_focused_input`); if a long-press both focuses a
  field and triggers that scroll, Kit's edit menu is dismissed — long-press
  again.
- GPUI converts a horizontal pan into vertical scrolling of a vertical-only
  container unless `restrict_scroll_to_axis()` is set; the lab's page
  containers set it. Kit's own scroll containers do not.
- On the host-driven entry point: night mode is not reported to GPUI (the
  host forwards `uiMode`), and gpui-mobile's bundled-emoji fallback cannot
  read APK assets (the host loads `NotoColorEmoji.ttf` itself; without it,
  emoji do not render on Android 13+).
- `window.request_animation_frame()` panics when called outside rendering
  (for example in a click handler); call it from `render`.

## Project layout

```
build.sh                  build script (cargo ndk + Gradle)
Cargo.toml                the lab crate (cdylib) and [patch.crates-io]
src/lib.rs                GPUI launch, fonts, window
src/host.rs               JNI entry points for LabActivity
src/app.rs                catalog, navigation, back, keyboard avoidance
src/matrix.rs             parses COMPONENT_MATRIX.md into the catalog
src/screens/              one module per demo screen
android/                  Gradle project (LabActivity, manifest, assets)
vendor/                   gpui-mobile, gpui-base, gpui-fps (patched, see MOBILE_PATCHES.md)
patches/                  the vendor patches as .patch files
themes/                   Kit theme JSONs bundled into the app
```

## License

AGPL-3.0 (see `LICENSE`). Vendored crates keep their own licenses: gpui-mobile
is GPL-3.0-or-later / AGPL-3.0-or-later / Apache-2.0; gpui-base and gpui-fps
are Apache-2.0. `NotoColorEmoji.ttf` is under the SIL Open Font License.
