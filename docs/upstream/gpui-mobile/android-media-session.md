# android: Deliver media-session callbacks on the main Looper

- Branch: [`upstream/android-media-session`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-media-session), one commit on upstream `main` (f9fe5a7)
- Diff: 2 files, +20 −7
- Depends on: nothing. This is the media-session half of fork branch `android-callbacks` (Bombatomica64/Gpui-android#48).
- Recording: none. The results are log lines, measured on the device (below). A screenshot of the system media controls could be added.

---

`media_session::init()` always fails when called from Rust. `MediaSessionCompat.setCallback(callback)` without a `Handler` needs a `Looper` on the calling thread, and native threads don't have one.

The action and seek handlers also run while their mutex is held, so a handler that replaces itself deadlocks.

### Change

- `GpuiMediaSession` passes `new Handler(Looper.getMainLooper())` to `setCallback`, so the callbacks arrive on the main thread.
- `notify_action` / `notify_seek` take the handler out of its mutex while it runs, then put it back unless it was replaced.
- `set_action_handler` / `set_seek_handler` document that on Android they run on the UI thread.

No public API changes.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- `GpuiMediaSession.java` is the same as on our fork, whose CI builds the example APK.
- On device (OnePlus CPH2581, Android 16, our fork's `main`): `media_session::init()` succeeds. `adb shell cmd media_session dispatch pause` and `dispatch next` reach `onPause` / `onSkipToNext` on the main thread, and the Rust handler receives `Pause`, `Next`.
- The re-entrancy fix is checked by reading the code only.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
