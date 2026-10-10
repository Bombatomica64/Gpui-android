# input: Tell the platform's keyboard how to assist each input

- Branch: [`upstream/input-text-input-configuration`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-text-input-configuration), one commit on upstream `main` (0bbd9870)
- Diff: 1 file, `crates/base/src/input/base/state.rs`, +120 (about 50 of them a unit test)
- Depends on: nothing. `TextInputConfiguration` and `EntityInputHandler::text_input_configuration` are already in gpui-pre 0.3.8.
- **Send before gpui-mobile `android-text-input-configuration`** (see that draft): GPUI's default configuration turns all assistance off, so the Android side needs Kit to provide one first.
- Where it shows today: **gpui_web** (`gpui-pre-web` mirrors the configuration onto its hidden IME element as `autocomplete`, `autocorrect`, `autocapitalize` and `enterkeyhint`). On Android it shows once the gpui-mobile PR lands. Desktop platforms ignore it.
- Recording: the Android result with both PRs: [Search key, Enter](../../demos/android-ime-actions.png), [password keyboard](../../demos/android-password-keyboard.png) (its "after" also includes the later purpose change). Nothing from the web; a follow-up if reviewers want it (e.g. the wasm story on a phone browser showing Done vs. a line break).
- Note for the user: `input_action`, `autocorrect` and `autocapitalize` are new public API, so the PR needs the `## Public API` section (done below).

---

## Description

GPUI asks the focused input handler for a `TextInputConfiguration`: the software keyboard's enter-key action, autocorrect, suggestions and autocapitalization. `InputState` doesn't provide one, so every Kit input gets GPUI's default, which is no assistance and no action hint. On the web that means no `enterkeyhint` and autocorrect off everywhere. On Android (with gpui-mobile's matching change) it means single-line inputs get a newline key that does nothing, and password fields can't opt out of suggestions.

`InputBaseState` now implements `EntityInputHandler::text_input_configuration`:

- **Enter key:** `Done` for a single-line input (and a multi-line one that submits on enter), `Enter` (a line break) for a multi-line one.
- **Plain text:** autocorrect, suggestions and sentence capitalization.
- **Masked inputs, inputs with a mask pattern, and code editors:** none of these.

New builders override the defaults, like React Native's `returnKeyType`, `autoCorrect` and `autoCapitalize`. Pressing the action key still emits `InputEvent::PressEnter`.

## Public API

### gpui-base (re-exported for `InputState` / `TextareaState`)

- `InputBaseState::input_action(self, action: gpui::TextInputAction) -> Self`: the action a software keyboard shows on its enter key (Search, Send, Next…). Default `Done` for single-line, `Enter` for multi-line.
- `InputBaseState::autocorrect(self, autocorrect: bool) -> Self`: whether the platform may autocorrect and suggest words. Default `true` for plain text, `false` for masked, mask-pattern and code-editor inputs.
- `InputBaseState::autocapitalize(self, autocapitalize: gpui::Autocapitalize) -> Self`: how a software keyboard capitalizes. Default `Sentences` for plain text, `None` otherwise.
- `InputBaseState::text_input_configuration(&self) -> gpui::TextInputConfiguration`: the resulting configuration, also returned to GPUI through `EntityInputHandler`.

No existing signatures change.

## Screenshot

Android (OnePlus CPH2581, Android 16, SwiftKey), with gpui-mobile's matching change. Left: `input_action(TextInputAction::Search)` shows the Search key. Right: after a single-line field's Done key, the keyboard is hidden and the field keeps focus.

![ime actions](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/2b4f67a23bf34ff879b04931229c73026e8dd347/docs/demos/android-ime-actions.png)

## How to Test

- New unit test `text_input_configuration_follows_the_input` (plain, masked, textarea, and an `input_action(Search)` + `autocapitalize(None)` override): `cargo test -p gpui-base --lib input`: 338 passed (incl. the new `text_input_configuration_follows_the_input`)
- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 179 passed, input_focus 4 passed
- On Android with gpui-mobile's change: `input_action(Search/Send/Next)` show those keys; a plain single-line field gets Done with autocorrect and sentence caps; `autocorrect(false)` gives no suggestions; the action key emits `PressEnter`.

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes. (Not run: no desktop session here. Desktop platforms don't read the configuration.)
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific) (No effect on desktop; the web and Android read it.)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
