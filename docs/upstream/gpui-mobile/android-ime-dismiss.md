# android: Keep focus when the IME's Done action hides the keyboard

- Branch: [`upstream/android-ime-dismiss`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-ime-dismiss), one commit on upstream `main` (f9fe5a7)
- Diff: 3 files, +21 −19
- Depends on: nothing
- Recording: an "after" screenshot only (the "before" is the dialog closing, which wasn't captured)

---

On the host-driven path, the IME's Done action (IME event 4) still sends GPUI a synthetic `escape` keystroke. #26 stopped back (event 5) from doing that, because apps bind escape to their own actions. Done is the same case. In a dialog with a text field, pressing Done on the keyboard closes the dialog.

### Change

- Event 4 now finishes the composition, hides the keyboard, and keeps focus, the same way event 5 does. As with back, a later tap on the field brings the keyboard back.
- `show_soft_keyboard` / `hide_soft_keyboard` are implemented for the Android window, so `Window::request_virtual_keyboard` and `dismiss_virtual_keyboard` work. A focused input can then ask for the keyboard again after the user has hidden it (GPUI Kit's Input will use this).

No public API changes.

### Screenshots

OnePlus CPH2581, Android 16, host-driven test app. A dialog with an input: typing (left), then Done (right). The keyboard goes away and the dialog stays open. Without this change, Done closes the dialog.

![Done keeps the dialog open](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/ime-dismiss-dialog.png)

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings (the 8 on `main` remain); `cargo test --lib`: 46 passed.
- On the device above, with our fork's `main` (which carries this commit on top of the JNI work in #31): Done in a dialog's input hides the keyboard and keeps the dialog and focus. A tap on the field shows the keyboard again. Back behaves as before.

Test app: [Bombatomica64/Gpui-android](https://github.com/Bombatomica64/Gpui-android) (Dialogs & Sheets screen).

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
