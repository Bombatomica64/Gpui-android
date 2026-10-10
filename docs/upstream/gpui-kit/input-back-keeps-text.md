# input: Keep the text on Android's back button with clean_on_escape

- Branch: [`upstream/input-back-keeps-text`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-back-keeps-text), one commit on upstream `main` (0bbd9870)
- Diff: 1 file, +5 −1
- Depends on: nothing. (Back reaches GPUI as `escape` on gpui-mobile's host path today.)
- Recording: before only, [back-clears-input.png](../../demos/back-clears-input.png). After: verified on the phone (Gpui-android#36), but not captured. **Follow-up:** a three-frame "type, back, back leaves the screen" capture.
- Reviewers may prefer a different shape, for example an opt-out, or keying it on a touch platform instead of `target_os`. Open to either.

---

## Description

gpui-mobile delivers Android's back button to GPUI as `escape`. In an input with `clean_on_escape()`, the first back hides the keyboard (the input keeps focus), and the second back **clears the text** instead of leaving the screen. Only a third back navigates. On Android, back never edits a field.

On Android, `clean_on_escape` no longer clears. Escape still dismisses the selection handles, the edit menu and an IME composition first, then propagates as before, so the app's back handling runs. Other platforms are unchanged, and the builder's doc says so.

## Screenshot

Android (OnePlus CPH2581, Android 16), the "Type here…" input of our test app has `clean_on_escape()`:

**Before**: type "Hello" (left), back hides the keyboard (middle), back again clears the text and stays on the screen (right).

![back clears input](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/back-clears-input.png)

**After**: type, back hides the keyboard, back again leaves the screen with the text untouched. (Not captured.)

## How to Test

- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-base --lib input`: 337 passed; `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 179 passed, input_focus 4 passed
- On Android, through our test app (Kit 0.7.1 plus this change).

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here; the change is Android-only.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (Android-only.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
