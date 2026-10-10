# android: Prepare audio streams without blocking the caller

- Branch: [`upstream/android-async-prepare`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-async-prepare), one commit on upstream `main` (f9fe5a7)
- Diff: 3 files, +159 −29 (mostly `GpuiAudio.java`)
- Depends on: nothing. (Lab issue Bombatomica64/Gpui-android#50. The fork's version used the `or_clear` helper from the #31 rework. This branch uses the `map_err` + `exception_clear` idiom already in `main`.)
- Recording: none, but there are timings measured on the device (Gpui-android#50). A recording would add little.

---

`AudioPlayer::set_url` calls `MediaPlayer.prepare()`, which waits for a stream to buffer (1.2 s over Wi-Fi on a test device). Called from a click handler, it freezes GPUI for that long. `stop()` prepares again the same way. A failed load reads as a missing duration.

### Change

- Streams use `prepareAsync()`. `state()` is `Loading` until they are ready. `play`, `pause` and `seek` made in the meantime apply once they are. A load failure is reported by `state()` as an error.
- `GpuiAudio.getState` reports the state directly, instead of Rust inferring it from position and duration.
- Local files are still prepared right away.

No signature changes (`PlayerState::Loading` already exists). `set_url`'s doc says it doesn't wait.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- `GpuiAudio.java` is the same as on our fork, whose CI builds the example APK.
- On device (OnePlus CPH2581, Android 16, our fork's `main`), an MP3 over Wi-Fi: `set_url` took 1222 ms before and 21–27 ms after. `state()` is `Loading` until it's ready, a `play()` made while loading starts playback once it is, and pause and `state()` work afterwards. The failure path (bad URL) was not checked on the device.
- Not changed: `GpuiVideoPlayer` prepares synchronously the same way. Our test app can't exercise the video player, so I left it alone.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
