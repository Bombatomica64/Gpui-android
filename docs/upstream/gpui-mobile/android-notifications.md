# android: Fail visibly when notifications are blocked; open the app on tap

- Branch: [`upstream/android-notifications`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-notifications), one commit on upstream `main` (f9fe5a7)
- Diff: 3 files, +82 −2
- Depends on: nothing. This is the notifications half of fork branch `android-notifications-links` (Bombatomica64/Gpui-android#49). The fork's version used the `or_clear` helper from the #31 rework; this one uses the `map_err` + `exception_clear` idiom already in `main`.
- Recording: none. A screenshot of the notification shade, and of the app opening on a tap, would help; a follow-up.

---

Two problems with `notifications` on Android:

- Without `POST_NOTIFICATIONS` (Android 13+), or with the app's notifications turned off, `show()` returns `Ok(())` and the system silently drops the notification.
- A notification has no content intent: tapping it does nothing, and its `payload` is never used.

### Change

- `GpuiNotifications.show` throws a `SecurityException` that says why when the permission isn't granted, notifications are off for the app, or the user turned the channel off. `show()` returns it as `Err`.
- Tapping a notification opens the app through its launch intent, with the payload as an extra. New `take_launch_payload()` returns that payload once.
- Docs for `show`, `Notification::payload` and `take_launch_payload` explain the requirements (permission, `singleTask`/`singleTop` plus `setIntent` for a tap on a running app).

### Public API

- `gpui_mobile::packages::notifications::take_launch_payload() -> Result<Option<String>, String>` (`#[cfg(target_os = "android")]`): the payload of the notification whose tap opened the app, once; `None` otherwise.

Behaviour change: `show()` returns `Err` instead of `Ok(())` when the notification would not be shown.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- `GpuiNotifications.java` is the same as on our fork, whose CI builds the example APK.
- On device (OnePlus CPH2581, Android 16, test app with targetSdk 34, our fork's `main`):
  - without `POST_NOTIFICATIONS`: before, `show` → `Ok(())` and `dumpsys notification` lists nothing from the app; after, `show` → `Err("java.lang.SecurityException: POST_NOTIFICATIONS is not granted; request it first")`.
  - granted: the notification is posted. Tapping it opens the app, and `take_launch_payload()` returns `Some("lab-payload")` once, then `None`.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
