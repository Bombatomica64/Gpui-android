# input: Ask for the virtual keyboard when an input is tapped

- Branch: [`upstream/input-tap-requests-keyboard`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-tap-requests-keyboard), one commit on upstream `main` (0bbd9870)
- Diff: 1 file, +9
- Depends on, to have an effect: **gpui-mobile `android-ime-dismiss`**, which implements `show_soft_keyboard` on Android. Without it, `request_virtual_keyboard` is a no-op there (and on desktop), so the Kit PR is safe to send first. It's clearer to reviewers once that one is in.
- Recording: **missing.** Follow-up: hide the keyboard with back, then tap the still-focused field (before: nothing; after: the keyboard returns), and tap from one input to another after hiding it.

---

## Description

On a touch screen, the user can hide the keyboard (Android's back button, or the IME's Done) while an input keeps focus, as with a native text field. A later tap on that input doesn't change focus, and a tap on another input moves focus between two text inputs. In neither case does the platform learn that the keyboard is wanted again, so it stays hidden while the user taps the field.

A touch press (`GlobalState::is_touch_press`) with the left button on an editable input (not disabled or read-only) now calls `Window::request_virtual_keyboard()`. Platforms without a virtual keyboard ignore it, and mouse clicks are unchanged.

## Screenshot

*(Missing; see above.)*

## How to Test

- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-base --lib input`: 337 passed; `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 179 passed, input_focus 4 passed
- To check on Android (our test app: Kit 0.7.1 plus this change, gpui-mobile with `show_soft_keyboard`): hide the keyboard with back, then tap the focused field; the keyboard should come back. *(The lab carries this change, but there is no recorded check of it; capture one before sending.)*

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here; touch-only path.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (Touch-only; desktop mouse clicks don't take this path.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
