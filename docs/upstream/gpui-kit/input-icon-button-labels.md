# input: Name the clear button for screen readers

- Branch: [`upstream/input-icon-button-labels`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-icon-button-labels), one commit on upstream `main` (0bbd9870)
- Diff: 3 files, +37 (one `accessibility_label`, one localized string, a 29-line UI test)
- Depends on: nothing.
- **Trimmed for upstream.** Our fork branch also named the password mask toggle. Open PR longbridge/gpui-kit#3424 (counterbeing, 2026-10-09) does exactly that ("Show password" / "Hide password", plus the sidebar caret), so that half is dropped here to avoid a duplicate. Our string keys (`Input.Show password`) differed from #3424's (`Input.Show Password`), so there's no conflict either way.
- Recording: before, a `uiautomator dump` ([a11y-text-input-baseline.xml](../../demos/a11y-text-input-baseline.xml)): every clear button is an `android.widget.Button` with empty `text` and `content-desc`. After: checked with uiautomator and TalkBack (Gpui-android#38), but no screenshot. A TalkBack screenshot would help; a follow-up.

---

## Description

The clear button that `Input`, `Select`, `Combobox` and `DatePicker` show (`.cleanable(true)`) is icon-only. Screen readers announce it as just "button" (TalkBack: "Unlabelled, button"), and accessibility-driven UI tests can't find it by name. Applications can't name it either, because they never construct it.

It is now named "Clear", localized for en, zh-CN, zh-HK and zh-TW like the other `Input` strings. This follows #3424, which does the same for the mask toggle.

## Screenshot

Android (OnePlus CPH2581, Android 16, TalkBack), Kit's Input with `cleanable(true)`.

| Before | After |
| --- | --- |
| `uiautomator dump`: `<node class="android.widget.Button" text="" content-desc="" …>` | `content-desc="Clear"`; TalkBack focuses "Clear" and activates it |

## How to Test

- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 180 passed (incl. the new `clear_button_is_named`), input_focus 4 passed
- On Android, through our test app (Kit 0.7.1 plus this change): uiautomator shows `content-desc="Clear"`, and TalkBack focuses and activates it.

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (Not platform-specific; a label only.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
