# Hot patching and fast reload

`cargo gpui` on the host, plus the `gpui-hot` crate in the app, both in the
standalone repo [Bombatomica64/cargo-gpui](https://github.com/Bombatomica64/cargo-gpui); gpui-mobile's
`GpuiLibraryLoader` loads a pushed library in the host Activity. All of it
is debug-only: release builds compile the hooks to nothing.

- **Hot patching** with [Subsecond](https://docs.rs/subsecond) (Dioxus). On
  save, `cargo gpui` recompiles only the app crate, links a small patch
  library against the running process, and pushes it; the app swaps function
  pointers between frames and acknowledges the patch. The screen, typed text
  and view state stay.
- **Reload**: rebuilds the app library, pushes it into the app's private
  directory (a ~1.4 MB zstd delta after the first push) and restarts the app
  with the intent extras it remembered, e.g. the open screen. No Gradle, no
  reinstall. Use it for the changes hot patching can't do.

`cargo gpui dev` does both: patch on every save, reload when a patch can't
be built or applied, and on `r` + Enter. Background, measurements and the
go/no-go are in `docs/hot-patching-plan.md` (sections A0–A3); usage is in
the [cargo-gpui README](https://github.com/Bombatomica64/cargo-gpui#readme).

## Contents
- One-time setup
- Daily loop
- Adopting it in a new app
- What a patch can and can't change
- Troubleshooting

## One-time setup

Prerequisites: `cargo-ndk`, the `aarch64-linux-android` target, an Android
SDK at `$ANDROID_HOME` (default `~/Android/Sdk`) with an NDK
(`$ANDROID_NDK_HOME`, else the newest under `$ANDROID_HOME/ndk`), and `zstd`
and `unzip` on the host. The tool assumes a Linux x86_64 host and an arm64
phone.

```sh
cargo install --git https://github.com/Bombatomica64/cargo-gpui cargo-gpui
cargo gpui apk                            # base debug APK → dist/<name>-<version>-debug.apk
cargo gpui install                        # install it, ready for delta reloads
```

`apk` builds the arm64 debug library with extra flags for the app crate only
(`-Csave-temps`, `-Clink-dead-code`, `--no-gc-sections`, `-Clto=off`, and
the tool itself as linker shim), records the rustc and linker commands under
`target/gpui-dev/`, strips the packaged `.so`, and runs Gradle. `-Clto=off`
matters: crate-local ThinLTO was 5 of the 6 seconds of each recompile.
`cargo gpui build` does the same without Gradle and prints where the time
went.

On a machine with the shared `phone` lock, set `CARGO_GPUI_ADB="phone adb"`
and run the device commands inside `phone session -- …`.

## Daily loop

```sh
cargo gpui dev      # inside `phone session` if the machine has the lock
```

- It starts with a reload, then polls `src/` every 50 ms. Each save prints
  `cargo gpui: patch N: x/y objects changed, … ms (rustc, stub, link,
  table+strip, push)`, then `patch N applied T ms after the save` once the
  app acknowledges it. Measured through an ssh adb tunnel: 3.3–3.6 s.
- Compile errors print as cargo would; fix and save again.
- When a patch fails to link, or the app reports it couldn't apply it (the
  tool then prints the app's recent `hot:` and crash log lines), `dev`
  reloads instead.
- Press `r` + Enter to reload, e.g. after changing a struct's layout. A
  reload with a delta took 8.3 s through the tunnel.

`cargo gpui reload` and `cargo gpui watch` (patches only, app already
running) run the two halves on their own. `CARGO_GPUI_OFFLINE=<hex
aslr_reference> cargo gpui watch` builds patches without a phone.

The host only loads `files/dev/…` when the APK is debuggable, so a release
build ignores it. `cargo gpui install` removes `files/dev/`; by hand:
`run-as <package> rm -r files/dev`.

Use a full `./build.sh android --debug` + install only for Java, manifest,
asset or Gradle changes.

## Adopting it in a new app

1. In the app's Cargo.toml (the app crate must be the `cdylib`: only this
   "tip" crate is patchable):
   ```toml
   [package.metadata.gpui]
   android-package = "com.example.app"
   android-activity = ".MainActivity"

   [dependencies]
   gpui-hot = { git = "https://github.com/Bombatomica64/cargo-gpui", rev = "…" }
   ```
2. Call `gpui_hot::init(cx)` once in `launch`, after your own init. It
   returns at once in release builds; in debug it writes the app's ASLR
   reference to `files/dev/aslr` and starts the patch poller.
3. Put `#[gpui_hot::hot]` on each view's `impl Render`. GPUI calls `render`
   through a vtable, which a patch can't redirect, so a single hook in the
   frame loop patches nothing; the attribute runs the body through
   `gpui_hot::call`:
   ```rust
   #[gpui_hot::hot]
   impl Render for ButtonsScreen {
       fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
           …
       }
   }
   ```
4. Optional: `gpui_hot::remember("screen", Some(title))` when a screen opens
   (`None` when it closes); a reload passes it back as `--es screen <title>`.
5. In the host Activity, replace the static `System.loadLibrary` with
   `GpuiLibraryLoader.load(this, "<lib name>")` at the start of `onCreate`
   (a gpui-mobile helper in its example project; the lab keeps a copy in
   `android/app/src/main/java`, since `LabActivity` needs it to compile). In a debuggable build it loads
   `files/dev/lib<name>.so` when it exists, else the packaged library.
   `NativeActivity` hosts can't use it: `NativeActivity` loads the packaged
   library itself.

## What a patch can and can't change

Patched (new code runs on the next frame):
- anything reached from a `#[hot]` `render`, including closures built there
  (`on_click`, `on_mouse_down`…), helpers in the app crate, and views
  created after an earlier patch.

Not patched, use a reload (`r`):
- struct or enum layout changes, new fields;
- code in gpui, Kit, gpui-mobile or any other dependency; new dependencies;
- `cx.subscribe` / `cx.observe` callbacks registered once at creation (they
  keep the old code until the view is recreated);
- Java, manifest, assets (full build).

Side effects to expect:
- **Statics and thread-locals in the app crate reset** on the first patch:
  the patch gets its own fresh copies, and the original copies stop being
  read. Keep state that must survive in GPUI entities or globals, or in a
  dependency crate.
- Static initialisers are not re-run.
- Each patch (~10 MB) stays loaded until the app restarts. After dozens of
  patches, reload.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `did not report its aslr_reference` | The app isn't running a debug build with `gpui_hot::init`, or the package in `[package.metadata.gpui]` is wrong. |
| `did not acknowledge patch N` | The app crashed or hung applying it; the tool prints its log lines and reloads. |
| Edit shows nothing | Was it inside a `#[hot]` render path? A listener registered at creation? A static? See the lists above. |
| Values like "unknown" appear after a patch | A static in the app crate was reset. Expected; move it out of the app crate. |
| Disk fills up | `-Csave-temps` writes ~65 MB per compile; the tool deletes them per patch, but check `target/` after crashes. |
| Old library loads after installing an APK by hand | Remove `files/dev/` with `run-as`, or install with `cargo gpui install`. |
