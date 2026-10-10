# android: Clear the Java exceptions the accessibility bridge leaves pending

- Branch: [`upstream/android-a11y-clear-exceptions`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-a11y-clear-exceptions), one commit on top of `android-accessibility` (the head of open PR #25, d7e7a73)
- Diff: 1 file, `src/android/accessibility.rs`, +31 −7
- **Depends on #25.** `accessibility.rs` only exists there. Two ways to send it:
  1. (Simplest) Push this commit to the `android-accessibility` branch, so it joins #25, and leave a short comment on #25 (draft below). It's a fix to code that #25 introduces, so it fits that review.
  2. If #25 merges first, open it as its own PR, rebased onto `main`.
- Recording: none possible. It's a code-audit fix (Bombatomica64/Gpui-android#40); no throwing case was reproduced on the device. TalkBack still works on the build.

---

### Comment for #25 (option 1)

> I added one more commit after an audit, 0eb7efc: **clear the Java exceptions the bridge leaves pending.** `accesskit_android` uses jni 0.21, whose calls return `Err` on a Java exception but leave the exception pending. The bridge never cleared it, so the next JNI call on the render thread, in gpui-mobile's jni 0.22 code, ran with an exception pending (undefined behaviour in JNI). It now describes and clears any pending exception after the host-view lookups, `AccessibilityManager.isEnabled`, and each tree update. I didn't reproduce a throwing case on the device; TalkBack still works on the build.

### PR body (option 2)

`accesskit_android`'s jni 0.21 API returns an error for a Java exception but leaves the exception pending, and the accessibility bridge never cleared it. The next JNI call on the render thread then runs with an exception pending, which JNI does not allow.

#### Change

A small `clear_exception` helper describes (logs) and clears any pending exception. It's called after `with_host_view_of` (the `getWindow` / `getDecorView` / `findViewById` lookups), after `AccessibilityManager.isEnabled`, and after each tree update (raising AccessKit events calls into Java).

#### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings over #25's head; `cargo test --lib`: 46 passed.
- Found by reading the code; no throwing case was reproduced on the device. TalkBack still reads the test app on a OnePlus CPH2581 (Android 16).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
