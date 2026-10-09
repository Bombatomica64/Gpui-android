# A new GPUI Android app

The fastest route is to copy the lab (`Bombatomica64/Gpui-android`), rename
the package, and delete screens you don't need. This file lists what each
piece does so you can also wire an existing crate.

## Contents
- Toolchain
- Cargo.toml
- Rust entry points
- Java host Activity
- Manifest and Gradle
- build.sh, CI, install
- Adding a screen

## Toolchain

Rust 1.98+ (edition 2024), NDK 27.2.12479018, compileSdk 35, **targetSdk
34**, minSdk 26, JDK 17.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
sdkmanager "platform-tools" "platforms;android-35" "build-tools;35.0.0" "ndk;27.2.12479018"
export ANDROID_HOME=$HOME/Android/Sdk
```

targetSdk stays at 34 because 35 forces edge-to-edge, after which
`adjustResize` stops shrinking the window for the keyboard and safe-area
insets aren't wired yet (see gotchas.md).

## Cargo.toml

Copy the lab's current pins from its `Cargo.toml` rather than the example
revs below, which age. Shape:

```toml
[lib]
name = "my_app"
crate-type = ["cdylib"]

[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.8", default-features = false }
gpui-kit = { git = "https://github.com/Bombatomica64/gpui-kit", rev = "<rev>", default-features = false, features = ["component", "assets"] }
gpui-mobile = { package = "gpui-pre-mobile", git = "https://github.com/Bombatomica64/gpui-mobile", rev = "<rev>", default-features = false, features = ["clipboard", "share"] }
log = "0.4"

[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.15"
jni = "0.22"
ndk = { version = "0.9", features = ["nativewindow"] }
android-activity = { version = "0.6", features = ["native-activity"] }

[patch.crates-io]
gpui-pre = { git = "https://github.com/Bombatomica64/gpui-pre", rev = "<rev>" }

[profile.dev]
panic = "abort"
opt-level = 1
debug = "line-tables-only"

[profile.dev.package."*"]
opt-level = 2

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 4
strip = true
panic = "abort"
```

- `cargo tree -d` must show one version of every `gpui-pre-*` crate. Two
  versions compile but give two GPUIs that can't see each other's types.
- `android-activity` is linked even though the host drives the app. Define a
  stub `android_main` (see the lab's `src/lib.rs`) to satisfy the symbol.
- gpui-mobile packages are opt-in features named after the package; see
  platform-packages.md.

## Rust entry points

`src/lib.rs`:
- `launch(cx: &mut App)`: runs once on the render thread. Call
  `gpui_kit::init(cx)`, your own init, then `cx.activate(true)` (and
  `hot::init(cx)` if you use hot patching).
- `open_window(cx)`: `gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| cx.new(|cx| MyRoot::new(window, cx)))`.
- Initialise `android_logger` with your log tag, and install a panic hook
  that logs the message and backtrace (the lab's is a good copy).

`src/host.rs` (`#[cfg(target_os = "android")]`): JNI exports named
`Java_<package_with_underscores>_<Activity>_native*`. They only queue
commands for the render thread and return. The `gpui_mobile::android::host`
API:

| Call | When |
|---|---|
| `jni::set_host_activity(env, &activity)` | first thing in every `onCreate`, recreation included |
| `host::on_open_window(\|_, cx\| open_window(cx))` | before the first surface |
| `host::start_with_assets(gpui_kit::assets::AllAssets, launch)` | every `onCreate`; only the first call starts the render thread |
| `host::surface_created(host, NativeWindow::from_surface(env, surface), scale)` | `surfaceCreated` and `surfaceChanged` (a repeat is a resize) |
| `host::surface_destroyed(host)` | `surfaceDestroyed`; blocks up to 2 s until the renderer lets go |
| `host::resumed(host)` / `host::paused(host)` | `onResume` / `onPause` |
| `host::host_destroyed(host)` | `onDestroy`, only when `isFinishing()` |
| `host::configuration_changed()` | `onConfigurationChanged` |
| `host::motion_event(action, index, &[Pointer { id, x, y }]) -> bool` | touch listener; physical px relative to the SurfaceView |
| `host::key(code, action, meta)` | `dispatchKeyEvent` |
| `host::ime_event(session, kind, text, start, end)` | from the IME proxy; kinds: 0 composing, 1 commit, 2 delete surrounding, 3 delete code points, 4 done, 5 keyboard hidden by Back, 6 action key (`IME_ACTION_*` in `start`) |

`HostId` is a `u64` the host picks. Keep it stable across recreation: the
lab keeps a static counter, saves the id in `onSaveInstanceState`, and
restores it in `onCreate`. A new id means a new GPUI window.

The lab's JNI exports take `EnvUnowned<'local>` and run
`env.with_env(|env| -> jni::errors::Result<()> { … }).resolve::<LogErrorAndDefault>()`.
Copy that pattern.

## Java host Activity

Copy `LabActivity.java` and rename. What it must do:
- Load the library in `onCreate` (debug: prefer `files/dev/lib<name>.so` for
  `hotpatch reload`).
- Own a `SurfaceView`; forward its callbacks and touch events.
- `dispatchKeyEvent`: let volume keys through, send Back to Rust (it arrives
  as `escape`), send printable keys to the IME proxy while it has focus.
- **IME contract.** Rust calls these public methods on the Activity:
  `gpuiShowKeyboardWithInputType(int inputType, int imeOptions, long session)`,
  `gpuiHideKeyboard(long session)`, `gpuiResetComposition(long session)`.
  The `InputProxy` (a 1×1 invisible `EditText` with an
  `InputConnectionWrapper`) turns keyboard input into `nativeIme` calls.
  Copy it as is.
- `onNewIntent`: `setIntent(intent)`, then forward the data URI to
  `deeplink::handle_link` and any `screen` extra.
- `onConfigurationChanged`: call the native `configuration_changed`.

## Manifest and Gradle

Activity attributes the lab relies on:

```xml
android:configChanges="orientation|screenSize|screenLayout|smallestScreenSize|keyboardHidden|keyboard|navigation|uiMode|density"
android:windowSoftInputMode="adjustResize|stateHidden"
android:launchMode="singleTask"
android:exported="true"
```

Plus `<uses-feature android:glEsVersion="0x00030000" android:required="true"/>`
(Vulkan is preferred but optional), and whatever permissions, `<queries>`,
FileProvider and helper Activities your packages need
(platform-packages.md).

Gradle (`android/app/build.gradle.kts`): `abiFilters` arm64-v8a (+ x86_64
for emulators), `sourceSets.main.java.srcDir("build/generated/gpui-helpers")`,
AndroidX deps for the packages you use, and
`packaging.jniLibs.keepDebugSymbols` for your `.so` if you want symbols in
debug builds. The lab signs release with the debug key so CI APKs install
without a keystore; use a real key for anything published.

Bundle `assets/fonts/NotoColorEmoji.ttf` (CBDT, OFL licence) or emoji render
as boxes on Android 13+.

## build.sh, CI, install

```sh
./build.sh android --debug                     # arm64, local dev
./build.sh android --release --abi all         # what CI runs
LAB_FEATURES=demo-pr3329 ./build.sh android --debug
```

It runs `cargo ndk -t <abi> --platform 26 -o android/app/src/main/jniLibs build`,
deletes gpui-mobile's own `.so` (it's linked statically), copies the
gpui-mobile Java helpers (`Gpui*.java`, found via `cargo metadata`) into
`android/app/build/generated/gpui-helpers`, and runs Gradle. The APK lands in
`dist/<name>-<version>-<profile>.apk`.

The helper list in `build.sh` is hard-coded: enabling a package whose Java
helper isn't in the loop compiles fine, and the package's calls then return
a class-not-found error at run time. Add the helper's name.

Avoid `--install` on a shared phone; it calls `adb` directly. Install inside
a `phone session` instead.

CI: `.github/workflows/android.yml` builds on every push and PR to `main`
and uploads the `gpui-mobile-lab-apk` artifact; `v*` tags also publish a
release. Fetch it with `gh run download <run-id> -n gpui-mobile-lab-apk`.

## Adding a screen (lab)

1. `src/screens/foo.rs`: `pub struct FooScreen` with
   `pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self` and a
   render (via `hot_render!` if the app uses hot patching).
2. `mod foo;` and `screen!("Foo", foo::FooScreen)` in `src/screens/mod.rs`
   (third argument `true` if it owns its scroll container).
3. Lab tools also go in `TOOLS`; component demos get a row in
   `COMPONENT_MATRIX.md` whose "Demo screen" equals the title.
4. Open it directly with `--es screen "'Foo'"`.
