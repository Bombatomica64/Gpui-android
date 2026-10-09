# Hot patching and fast reload

Two tools, both in `tools/hotpatch` (a standalone crate with its own
workspace) plus `src/hot.rs` on the app side. Both are debug-only: release
builds compile the hooks to nothing.

- **`hotpatch watch`**: hot patching with [Subsecond](https://docs.rs/subsecond)
  (Dioxus). On save it recompiles only the app crate, links a small patch
  library against the running process, and pushes it; the app swaps
  function pointers between frames. The screen, typed text and view state
  stay.
- **`hotpatch reload`**: rebuilds the app library, pushes it into the app's
  private directory and restarts the app on the screen that was open. No
  Gradle, no reinstall. Use it for the changes hot patching can't do.

Background, measurements and the go/no-go are in
`docs/hot-patching-plan.md` (sections A0–A2).

## Contents
- One-time setup
- Daily loop
- Adopting it in a new app
- What a patch can and can't change
- Troubleshooting

## One-time setup

Prerequisites: `cargo-ndk`, the `aarch64-linux-android` target, an Android
SDK at `$ANDROID_HOME` (default `~/Android/Sdk`) with an NDK
(`$ANDROID_NDK_HOME`, else the newest under `$ANDROID_HOME/ndk`). The tool
assumes a Linux x86_64 host and an arm64 phone.

```sh
(cd tools/hotpatch && cargo build)          # → tools/hotpatch/target/debug/hotpatch
tools/hotpatch/target/debug/hotpatch fat    # base debug APK → dist/gpui-mobile-lab-<version>-debug.apk
```

`fat` builds the arm64 debug library with extra flags for the app crate only
(`-Csave-temps`, `-Clink-dead-code`, `--no-gc-sections`, `-Clto=off`, and the
tool itself as linker shim), records the rustc and linker commands under
`target/hotpatch/`, strips the packaged `.so`, and runs Gradle. It does not
install. `-Clto=off` matters: crate-local ThinLTO was 5 of the 6 seconds of
each recompile.

Install the APK (inside `phone session` if the machine has the `phone`
lock), then start the app.

## Daily loop

```sh
phone session -- tools/hotpatch/target/debug/hotpatch watch    # app must be running
```

(Without the `phone` lock, run `hotpatch watch` directly; it calls
`phone adb`, so on a machine without `phone`, adapt the `adb()` helper in
`tools/hotpatch/src/main.rs`.)

- `watch` reads the app's `hot: aslr_reference=0x… pid=N` log line, then
  polls `src/` every 50 ms. Each save prints
  `hotpatch: patch N: x/y objects changed, … ms (rustc, stub, link, table+strip, push)`,
  or `no code change`.
- `watch` only reacts to saves made **after** it started. If the edit is
  already on disk, `touch` the file once `watch` prints that it's watching.
- Patches go to `/data/local/tmp/gpui-hot/patch-<n>.so` plus `patch.json`;
  the app polls the table every 50 ms and redraws.
- A patch that fails to link prints `patch N failed`; `watch` keeps going.
  Fix the code and save again, or fall back to `reload`.
- **After any app restart, restart `watch`.** The ASLR reference is per
  process.

When the change is outside what a patch can do (see below):

```sh
phone session -- tools/hotpatch/target/debug/hotpatch reload
```

`reload` does the same build as `fat` without Gradle, pushes the stripped
library in 16 MB parts (one big push can exceed `phone adb`'s 120 s
timeout), copies it with `run-as` to `files/dev/libgpui_mobile_lab.so`
(read-only, as Android 14 requires for loaded code), force-stops the app and
restarts it with `--es screen '<last open screen>'`. It prints
`hotpatch: reloaded N MB into '<screen>' in T ms (…)`. Then start `watch`
again.

The host only loads `files/dev/…` when the APK is debuggable, so a release
build ignores it. To go back to the packaged library:
`run-as dev.gpui.mobile.lab rm files/dev/libgpui_mobile_lab.so`.

Use a full `./build.sh android --debug` + install only for Java, manifest,
asset or Gradle changes.

## Adopting it in a new app

The tool is still lab-shaped; a `cargo gpui dev` command is planned but
doesn't exist.

1. Dependencies of the app crate (must be the `cdylib`: only this "tip"
   crate is patchable):
   ```toml
   subsecond = "0.7.10"
   serde_json = "1"
   libc = "0.2"
   ```
2. Copy the lab's `src/hot.rs` and add `mod hot;`. It provides `hot::call`,
   `hot::init`, the `hot_render!` macro, and an exported `main` symbol that
   Subsecond uses as its address reference.
3. Call `hot::init(cx)` once in `launch`, after your own init. It returns
   at once in release builds; in debug it logs the ASLR line and starts the
   patch poller.
4. Route every view's render through `subsecond::call`. GPUI calls `render`
   through a vtable, which a patch can't redirect, so a single hook in the
   frame loop patches nothing. Move the body into an inherent method and let
   the macro write the `Render` impl:
   ```rust
   crate::hot_render!(ButtonsScreen);   // several types: hot_render!(A, B, C);

   impl ButtonsScreen {
       fn render_view(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
           // what used to be Render::render
       }
   }
   ```
5. In `tools/hotpatch/src/main.rs`, change the constants `TIP` (crate lib
   name), `PACKAGE` (application id) and `LOG_TAG`.
6. In the host Activity, replace the static `System.loadLibrary` with the
   lab's `loadLibrary()` from `onCreate`: if debuggable and
   `getFilesDir()/dev/lib<name>.so` exists, `System.load` it, else
   `System.loadLibrary`. Have the app log `open screen: <title>` and accept
   the `screen` extra if you want `reload` to land on the same screen.

## What a patch can and can't change

Patched (new code runs on the next frame):
- anything reached from a `render_view`, including closures built there
  (`on_click`, `on_mouse_down`…), helpers in the app crate, and views
  created after an earlier patch.

Not patched, use `reload`:
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
  patches, `reload`.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `watch` can't find the ASLR line | logcat rotated it out. Restart the app, or set `HOTPATCH_ASLR=<hex>` from an earlier log. |
| Want to test patch building without a phone | `HOTPATCH_OFFLINE=<hex> hotpatch watch` builds patches without pushing. |
| Edit shows nothing | Was it inside a `render_view` path? A listener registered at creation? A static? See the lists above. |
| Values like "unknown" appear after a patch | A static in the app crate was reset. Expected; move it out of the app crate. |
| Disk fills up | `-Csave-temps` writes ~65 MB per compile; the tool deletes them per patch, but check `target/` after crashes. |
| Old library loads after switching to a release APK | It can't (release isn't debuggable). For a debug APK, remove `files/dev/` with `run-as`. |
