# Draft: gpui: Say what a text field holds in TextInputConfiguration

- Branch: [Bombatomica64/zed `gpui-text-input-purpose`](https://github.com/Bombatomica64/zed/tree/gpui-text-input-purpose), one commit on Zed `main` (7fb155e9a2)
- Diff: `crates/gpui/src/platform.rs`, `crates/gpui/src/input.rs` (test), `crates/gpui_web/src/ime_mirror.rs`, +68 −7
- Lab fork: gpui-pre `text-input-purpose` (the `gpui` half; the `gpui_web` half is new for Zed)

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.
> This one adds API, so CONTRIBUTING.md asks for a GitHub discussion first.
> Send it after the three touch fixes, or not at all if the discussion says
> no.

## Does it fit Zed?

Yes, with the `gpui_web` part. `TextInputConfiguration` (`autocorrect`,
`autocapitalize`, `suggestions`, `input_action`) is already in Zed `main`,
and `gpui_web`'s IME mirror maps it onto the hidden textarea's attributes.
A `purpose` field on its own would have no consumer in Zed, so this branch
also maps it in `gpui_web`. iOS (#63068, open) is the other obvious
consumer (`UIKeyboardType`, `secureTextEntry`).

---

## Summary

A text field can't tell the platform what it holds, so a software keyboard
always shows the full text layout. A PIN field gets letters, an email field
gets no `@` key, and a password field gets suggestions that learn from what
is typed into it.

## Change

- `TextInputConfiguration` gains `purpose: TextInputPurpose`. Its values
  follow HTML's `inputmode` (text, search, email, url, tel, numeric,
  decimal), plus `Password` and `NumericPassword`. It defaults to `Text`,
  so existing configurations are unchanged.
- `gpui_web` sets the IME mirror's `inputmode` from it whenever the virtual
  keyboard is enabled; the `inputmode="none"` suppression on touch devices
  is unchanged. It writes the attribute only when the purpose changes,
  since mutating the focused element can restart the IME connection.
- A textarea can't be a password field, so for the two password kinds
  `gpui_web` also turns suggestions, spellcheck and autocorrect off,
  whatever the rest of the configuration asks.

**Desktop is unchanged:** the native backends ignore the field, and on the
web the default purpose maps to `inputmode="text"`, which is what the mirror
already set.

## Testing

- `input::tests::text_input_configuration_and_focus_state_are_forwarded_on_change`
  now sets `purpose` and checks that a purpose change is forwarded to the
  platform.
- `cargo test -p gpui --lib`: 383 passed, 0 failed. `cargo fmt --check` and `cargo clippy -p gpui --all-targets --all-features -- --deny warnings` (dev profile) are clean.
- `cargo check -p gpui_web --target wasm32-unknown-unknown`: clean, with the same `-Zbuild-std` and atomics flags as Zed's `check_wasm` CI job (run on the pinned 1.98.1 toolchain with `RUSTC_BOOTSTRAP=1`, since there's no nightly on this machine).

Manual: the Android lab uses the `gpui` half through gpui-mobile. Existing
screenshots in the lab repo: `docs/demos/android-password-keyboard.png`
(password keyboard, before and after), `docs/demos/android-amount-keyboard.png`
(an amount field's numeric keyboard) and `docs/demos/password-suggestions.png`
(baseline: suggestions shown over a password field).
**Still needed:** the web half has not been tried in a browser. Open the web
example on an Android phone and check that a numeric field gets the number
pad.

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist.
- [ ] Tests cover the new or changed behavior.

Release Notes:

- N/A
