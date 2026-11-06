# android: Prepare video streams without blocking the caller

- Branch: [`upstream/android-video-async-prepare`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-video-async-prepare), one commit on upstream `main` (7548ebc, #45)
- Diff: 5 files, +374 −90 (+249 −74 of it in `GpuiVideoPlayer.java`; the example's video screen +26 −6)
- Depends on: no code from #10 (`upstream/android-async-prepare`, audio), but it copies its design (`Loading` state, calls recorded while loading). Send it after #10 is reviewed, so the design is discussed once. Same fix as my fork's Bombatomica64/gpui-mobile#16 (lab issue Bombatomica64/Gpui-android#50). The fork's version used the `or_clear` helper from the #31 rework; this branch uses the `map_err` + `exception_clear` idiom already in `main`.
- Why not on #10's branch: different files (Java helper, package, example screen), and this one changes a public signature (`set_url`), so it is easier to review and revert alone.
- Recording: device timings and two screenshots ([ready](../../demos/android-video-async-ready.png), [Play during loading](../../demos/android-video-async-play-while-loading.png)), from the lab's Video screen (Gpui-android#82).

---

`VideoPlayer::set_url` calls `MediaPlayer.prepare()`, which waits for the stream to buffer, so a call from GPUI's thread freezes the UI for as long (over a second on network URLs). This is the video half of the audio change in #<audio PR>, with the same design.

### Change

- `set_url` returns `Result<Option<VideoInfo>, String>`. For a stream (http, https, rtsp) it returns `None` at once.
- A single worker thread resets the player, sets the source and starts `prepareAsync()`. The async prepare alone was not enough: on the device, `reset()` of a prepared player took 41–51 ms and a first `setDataSource(https)` up to 54 ms, so `set_url` still took 50–70 ms.
- Local sources (and `set_file_path`) wait for the worker, so they still return prepared, with `Some`.
- `dispose` releases the player on the worker, after any source change still in flight.
- New `VideoPlayer::state()`. It reports `Loading` until the stream is ready, and an `Err` with the reason if loading fails. `VideoPlayerState` now matches audio's `PlayerState` (`Idle, Loading, Ready, Playing, Paused, Completed`). The old enum wasn't used anywhere, but this breaks code that names its variants.
- `play`, `pause`, `seek` and `set_speed` made while loading are recorded and applied when the player is ready. All of that pending state is under one lock. `set_url` replaces it before its worker job resets the player, and the prepared callback applies it while holding the lock, so a late callback from the previous source never touches the reset player.
- A surface attached while loading only gets `setSurface`. The old re-seek and the debug log called `getDuration`/`seekTo`, which move a preparing `MediaPlayer` into its Error state. The prepared callback seeks instead, which draws the first frame. The getters return 0/false while preparing.
- iOS: `set_url` still waits for the item (wrapped in `Some`). `state()` is derived from the `AVPlayerItem` status and the rate.
- The example's video screen handles `None` and picks up duration and size from its polling loop.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets`, `--no-default-features --features video_player`, and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed, as on `main`. The example crate: `cargo check --target aarch64-linux-android`.
- `GpuiVideoPlayer.java`, `mod.rs`, `ios.rs` and the example screen are byte-identical to my fork's, whose CI built the example APK. Only `android.rs` differs (the JNI error idiom).
- On device (OnePlus CPH2581, Android 16), through my test app's Video screen, all calls on GPUI's thread, with a 10 s 640×360 https MP4:

| | before the worker | this PR |
| --- | --- | --- |
| `set_url`, fresh player | 14–69 ms | **0 ms** |
| `set_url`, player already prepared | 50–68 ms | **0 ms** |
| state right after | `Loading` | `Loading` |
| ready | `Ready` at +225–345 ms | `Ready` at +225–236 ms |
| Play pressed during `Loading` | `Playing` at +394 ms | `Playing` at +269 ms |

  The first frame is drawn when the stream is ready. Closing the screen disposes the platform view and the player. The worst GPUI-thread stall after a load was 0 ms.
- iOS: type-checked only (`cargo check --target aarch64-apple-ios --features video_player,camera`, on my fork). `video_player` without `camera` already failed on iOS before this change (`platform_view.rs` refers to `packages::camera`).
- Not covered: the load-error path (`state()` returning `Err`) on a device. At the end of a clip `state()` reports `Paused` rather than `Completed`; that check (`position >= duration`) is the same one audio uses, and I didn't change it here.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
