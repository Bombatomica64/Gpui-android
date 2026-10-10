# android: Deliver deep links that reach the running app

- Branch: [`upstream/android-deeplink`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-deeplink), one commit on upstream `main` (f9fe5a7)
- Diff: 3 files, +24 −4
- Depends on: nothing. This is the deep-link half of fork branch `android-callbacks` (Bombatomica64/Gpui-android#48); the media-session half is its own PR.
- Recording: none. The results are log lines and return values, measured on the device (below).

---

A deep link that reaches an app that is already running is lost on the host-driven path:

- The only way in is `Java_dev_gpui_mobile_GpuiActivity_nativeOnDeepLink`, which is bound to `GpuiActivity`. A host Activity with another name has no way to deliver its `onNewIntent`.
- Even through that entry point, `notify_deep_link` calls the handler but never updates `get_latest_link()`.
- The handler runs while `CALLBACK`'s mutex is held, so a handler that calls `set_deep_link_handler` (to replace itself) deadlocks.

### Change

- New `deeplink::handle_link(url)` (Android only). It records the link for `get_latest_link()` and calls the handler. A host calls it from `onNewIntent`. `GpuiActivity`'s JNI entry point uses it too.
- The handler is taken out of the mutex while it runs, then put back unless it was replaced.
- `set_deep_link_handler`'s doc says that on Android it runs on the UI thread.

### Public API

- `gpui_mobile::packages::deeplink::handle_link(url: &str)` (`#[cfg(target_os = "android")]`): deliver a link the running app received, from a host Activity's `onNewIntent`.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- On device (OnePlus CPH2581, Android 16, our fork's `main`; the test app's Activity calls `handle_link` from `onNewIntent`): a cold start with `gpuilab://cold/1`, then `am start -d gpuilab://warm/2`.
  - Before: `get_latest_link()` stays `gpuilab://cold/1`, and the handler is never called.
  - After: the handler fires with `gpuilab://warm/2`, and `get_latest_link()` returns it.
- The re-entrancy fix (a handler replacing itself) is checked by reading the code only.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
