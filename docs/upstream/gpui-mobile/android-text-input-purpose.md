# android: Pick the keyboard from what the field holds

- Branch: [`upstream/android-text-input-purpose`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-purpose), one commit on top of `upstream/android-text-input-configuration`
- Diff (own commit): 1 file, `src/android/input_type.rs`, about +64 −6
- **Blocked.** It reads `TextInputConfiguration::purpose`, which exists only in our gpui-pre fork. It needs the Zed PR ([../../upstream-zed/text-input-purpose.md](../../upstream-zed/text-input-purpose.md)) merged and released in a `gpui-pre` snapshot, and gpui-mobile bumped to that snapshot. Until then the branch does not build against crates.io `gpui-pre`. It was not compiled for this prep; it builds on our fork's `main`, which patches gpui-pre.
- Also depends on: android-text-input-configuration (and so android-ime-dismiss).
- Recording: [password keyboard before/after](../../demos/android-password-keyboard.png), [amount keyboard](../../demos/android-amount-keyboard.png).

---

*(Draft. Fill in once `purpose` is in a gpui-pre release.)*

GPUI's `TextInputConfiguration` gains `purpose: TextInputPurpose` (the HTML `inputmode` values plus `Password` and `NumericPassword`). On Android this picks the `EditorInfo` class and variation:

- `Password` → `TYPE_CLASS_TEXT | TYPE_TEXT_VARIATION_PASSWORD`, `NumericPassword` → `TYPE_CLASS_NUMBER | TYPE_NUMBER_VARIATION_PASSWORD`, both with `IME_FLAG_NO_PERSONALIZED_LEARNING`, so the keyboard neither suggests nor learns the password.
- `Numeric`, `Decimal`, `Phone`, `Email`, `Url` → the matching class or variation; `Text` and `Search` stay plain text.
- A keyboard type an app asked for through `show_keyboard_with_type` still wins over the purpose.

### Screenshots

OnePlus CPH2581, Android 16, SwiftKey: a masked input before (suggests "Hunter2") and after (`inputType=0x81`, no suggestion strip); a decimal-mask amount field gets the number pad.

![password keyboard](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/android-password-keyboard.png)
![amount keyboard](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/android-amount-keyboard.png)

### Testing

- On device with our forks: a masked input asks for `inputType=0x81` with `IME_FLAG_NO_PERSONALIZED_LEARNING`, and SwiftKey shows no suggestion strip. The phone mask gets `TYPE_CLASS_PHONE`, the amount mask number|decimal, a PIN (masked + digit mask) a numeric password.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
