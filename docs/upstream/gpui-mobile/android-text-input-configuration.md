# android: Apply the focused field's TextInputConfiguration to the IME

- Branch: [`upstream/android-text-input-configuration`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-configuration), two commits on top of `upstream/android-ime-dismiss`
- Diff (own commits): 7 files, about +310 −15, of which 156 lines are the new `input_type.rs` mapping and its tests
- Depends on:
  - **android-ime-dismiss** (uses its `keyboard_done()` to put the keyboard away after Done/Go/Search/Send). Send that first, then rebase.
  - **gpui-kit `input-text-input-configuration` merged and released first.** GPUI's default `TextInputConfiguration` turns all assistance off. Without Kit providing one, this PR would give every Kit field a keyboard with no suggestions and no auto-capitalization, which is a regression for plain text. With a Kit that provides it, plain text keeps autocorrect, suggestions and sentence caps.
  - The fork's version used the `or_clear` helper from the #31 rework; this one clears the `NoSuchMethodError` explicitly.
- Recording: [password suggestions, before](../../demos/password-suggestions.png); [password keyboard before/after](../../demos/android-password-keyboard.png) (the "after" was taken with the purpose PR on top, see below); [Search key, Enter](../../demos/android-ime-actions.png).

---

The host's IME proxy always asks for a multi-line text field with the keyboard's default assistance, whatever the focused field is:

- single-line inputs show a newline key that does nothing (a single-line field drops `"\n"`, so `PressEnter` never fires)
- an app can't ask for Search / Send / Next on the enter key
- an app can't turn autocorrect, suggestions or auto-capitalization off. A password field shows the typed password in the keyboard's suggestion strip, and the keyboard learns it.

GPUI already forwards each field's `TextInputConfiguration` (enter-key action, autocorrect, suggestions, autocapitalize) through `PlatformWindow::set_text_input_configuration`. The Android window didn't implement it.

### Change

- `AndroidPlatformWindow::set_text_input_configuration` stores the configuration. If focus moves straight from one field to another while the keyboard is up, the keyboard restarts with the new configuration.
- New `android/input_type.rs` maps it to `EditorInfo.inputType` / `imeOptions`. A field with an action (Done, Go, Search, Send, Next, Previous) is single-line; `Enter`/`Unspecified` keeps it multi-line. Autocapitalize, autocorrect and suggestions map to the `TYPE_TEXT_FLAG_*` bits. `IME_FLAG_NO_EXTRACT_UI` stays.
- The host gets them through a new optional Java method, `gpuiShowKeyboardWithInputType(int inputType, int imeOptions, long session)`. A host without it keeps `gpuiShowKeyboard(int, long)`, so existing hosts behave as before.
- With the new method, the proxy reports the action key as IME event 6 (`start` = the `IME_ACTION_*`). The field receives it as an Enter key-down/up, as with a hardware keyboard, so `PressEnter` fires. Done, Go, Search and Send then hide the keyboard and keep focus (as Done does after the previous PR); Next and Previous keep it up.
- Second commit: a hardware Enter on a single-line field triggers the same action (a single-line `TextView` would otherwise turn it into a focus move).

### Public API

No Rust API changes. Host contract: an optional `gpuiShowKeyboardWithInputType(int, int, long)` on the host Activity (documented in `android::host`; `GpuiInputActivity` implements it), and IME event kind 6 for `host::ime`.

### Screenshots

OnePlus CPH2581, Android 16, SwiftKey.

Before: a masked input asks for `inputType=0x2c001` (TEXT | MULTI_LINE | AUTO_CORRECT | CAP_SENTENCES), so SwiftKey suggests and learns the password ("Hunter2").

![password suggestions before](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/password-suggestions.png)

Before/after for the password field. The "after" build also carries the purpose follow-up, which adds the password variation; with this PR alone the field asks for `NO_SUGGESTIONS` and no autocorrect. That build was not screenshotted on its own.

![password keyboard](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/android-password-keyboard.png)

Left: a field with `input_action(Search)` gets the Search key. Right: after a single-line field's Done key, the keyboard is hidden and the field keeps focus.

![ime actions](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/android-ime-actions.png)

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` (which builds the `input_type` unit tests) and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- `GpuiInputActivity.java` is the same as on our fork, whose CI builds the example APK.
- On device (our fork's `main` with GPUI Kit's matching change): `input_action(Search/Send/Next)` show those keys; a plain single-line field gets Done with autocorrect and sentence caps; `autocorrect(false)` gives no suggestions. The keyboard's action key and an injected hardware Enter both reach the field as `PressEnter`. Enter still inserts a newline in a textarea.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
