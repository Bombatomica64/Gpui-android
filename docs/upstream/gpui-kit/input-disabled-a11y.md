# input: Report a disabled input as disabled to assistive technology

- Branch: [`upstream/input-disabled-a11y`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-disabled-a11y), one commit on upstream `main` (48ac25b5, #3821)
- Diff: 2 files, +12 −2 (one line in `InputBase`, plus `input` in the existing `a11y_disabled` test)
- Depends on: nothing. Same fix as my fork's Bombatomica64/gpui-kit#1 (lab issue Bombatomica64/Gpui-android#39).
- Overlap checked on 2026-11-06: `main` has `A11yDisabledExt` (#3531) on every control except `InputBase`, and #3825 just added it to `SliderThumb`. The open #3818 (single-line input text for screen readers) touches the input's a11y node but not its disabled state.
- Recording: a `uiautomator dump` before/after (Gpui-android#81). No screenshot: the visible field doesn't change.

---

## Description

#3531 marks a disabled control's accessibility node disabled through `A11yDisabledExt`, but `InputBase` only styles itself (opacity). So a disabled `Input` reaches screen readers as a field that accepts text: TalkBack doesn't say "disabled", and on Android the node reports `enabled=true`.

`InputBase` now wraps its root with `.a11y_disabled(self.disabled)`, like the other controls. The `a11y_disabled` test, which renders each control disabled and enabled and reads the node it writes, now includes `InputBase`.

## Screenshot

Android (OnePlus CPH2581, Android 16), the "Disabled input" field of my test app, read with `uiautomator dump` while typing in another field:

| | class | enabled |
| --- | --- | --- |
| Before | `EditText` "Disabled input" | `true` |
| After | `EditText` "Disabled input" | `false` |

## How to Test

- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-base --features test-support --lib a11y_disabled`: 2 passed. `a_disabled_control_marks_its_node_disabled` now covers 12 controls, and fails on "input, disabled true" without the one-line change.
- `cargo test -p gpui-base --features test-support --lib input`: 424 passed
- `cargo test -p gpui-kit --features test-support --test integration input`: 223 passed, 1 failed: `input::completions::ordinary_input_schedules_no_followup_frame` (7 follow-up frames, expects 0). It fails the same way with the `InputBase` line reverted (so on `main` 48ac25b5 plus only the test change), on my machine. [Check before sending whether it's flaky or fixed on `main`; drop this line if it is.]
- On Android, through my test app (Kit 0.8.0 plus this commit), with uiautomator.

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (Not platform-specific; every platform's accessibility bridge reads the flag.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
