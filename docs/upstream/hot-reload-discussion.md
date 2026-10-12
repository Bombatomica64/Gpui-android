# Hot reload for gpui-mobile apps: the upstream issue

> **Status: posted 2026-10-12 as
> [longbridge/gpui-mobile#34](https://github.com/longbridge/gpui-mobile/issues/34)**
> (Discussions are off there). The text below is what was posted.

---

Hi! While building a test app on gpui-mobile we made Rust hot reload work on
Android, and we'd like to know whether you find it useful and where you'd
like it to live.

## What it is

[cargo-gpui](https://github.com/Bombatomica64/cargo-gpui) is a small dev tool
plus an app-side crate:

- **Hot patching.** On save, `cargo gpui dev` recompiles only the app crate,
  links a small patch library against the running app and pushes it. The app
  swaps in the new code between frames, and the screen, typed text and view
  state stay. It's built on [Subsecond](https://docs.rs/subsecond) from
  [Dioxus](https://github.com/DioxusLabs/dioxus); the patch builder adapts
  parts of the Dioxus CLI (`dx serve --hotpatch`), with credit and its MIT
  notice in the repo.
- **Reload without reinstalling**, for the changes a patch can't make (struct
  layout, dependencies): the tool rebuilds the library, pushes a zstd delta
  (~1.4 MB) and restarts the app on the same screen. No Gradle, no APK
  install.

Views opt in with an attribute on their `Render` impl:

```rust
#[gpui_hot::hot]
impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(format!("{}", self.count))
    }
}
```

It's debug-only: release builds compile the hooks to nothing.

## Numbers

Measured on a OnePlus phone (Android 16) reached over an ssh adb tunnel,
with a 32-screen test app:

| Step | Time |
|---|---|
| Save → change on screen (patch) | 3.3–3.6 s |
| Reload with a 1.4 MB delta, same screen | 8.3 s |

Over USB the push part (~1.5 s of each patch) should shrink; we haven't
measured that yet.

![Patches applied on the phone](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/main/docs/demos/subsecond-buttons-patches.png)

*From our first prototype: a title, a colour and a layout changed live while
the click counter kept its value (3).*

## How it's split

- **Outside gpui-mobile** (the repo above, MIT OR Apache-2.0): the
  `cargo gpui` CLI, the `gpui-hot` crate and its attribute macro. They depend
  on GPUI and Subsecond, not on gpui-mobile.
- **Inside gpui-mobile**, one small piece: a Java helper,
  `GpuiLibraryLoader`, that a host-driven Activity calls instead of
  `System.loadLibrary`. In a debuggable build it loads
  `files/dev/lib<name>.so` when a dev tool has pushed one; release builds
  always load the packaged library. It's opt-in, about 50 lines, and adds no
  Rust code or dependency. It's on our fork as branch
  [`dev-hot-loader`](https://github.com/Bombatomica64/gpui-mobile/blob/dev-hot-loader/example/android/gradle/app/src/main/java/dev/gpui/mobile/GpuiLibraryLoader.java),
  based on your `main`.

## Questions

1. Is this useful to you?
2. Where would you like it?
   - keep it as a separate project that apps opt into;
   - move the tool under longbridge;
   - or bring it into gpui-mobile itself.
3. Would you take `GpuiLibraryLoader` as a small PR? We can open it
   whenever you like.

Known limits: Android arm64 debug builds only; host-driven Activities only
for reloads (`NativeActivity` loads the packaged library itself); statics in
the app crate restart from their initial values after a patch.

Thanks for gpui-mobile!
