# input: Report a masked input as a password field to screen readers

- Branch: [`upstream/input-masked-password-role`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-masked-password-role), one commit on upstream `main` (0bbd9870)
- Diff: 2 files, +23 −1 (the change, plus a 14-line UI test)
- Depends on: nothing. Upstream `main` already gives `content_type(Password | NewPassword)` the `PasswordInput` role; this covers `masked(true)` without a content type.
- Recording: before, a `uiautomator dump` ([a11y-text-input-baseline.xml](../../demos/a11y-text-input-baseline.xml)): the masked "Password" field reports `password="false"`. After: `password="true"`, checked with uiautomator (Gpui-android#39). No screenshot; a follow-up if wanted.

---

## Description

A masked `Input` (`InputState::masked(true)`) without a password `content_type` has the plain `TextInput` role. Its value is already withheld from assistive technology, but screen readers announce it as an ordinary edit box: TalkBack doesn't say "password", and Android reports `password=false` for the node.

While masked, the input now resolves its role as if its content type were `Password`, so it gets `Role::PasswordInput`. An explicit `content_type` (e.g. `EmailAddress`) or a role override still wins, and a revealed (unmasked) input keeps its normal role.

## Screenshot

Android (OnePlus CPH2581, Android 16), the masked field of Kit's Input demo:

| Before | After |
| --- | --- |
| `uiautomator dump`: `class="android.widget.EditText" … password="false"` | `password="true"` |

## How to Test

- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 180 passed (incl. the new `masked_input_is_a_password_field`), input_focus 4 passed
- On Android, through our test app (Kit 0.7.1 plus this change), with uiautomator.

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (Not platform-specific; the role is read by every platform's accessibility bridge.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
