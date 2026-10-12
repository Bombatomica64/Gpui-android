# Text input, keyboards, Back and TalkBack

## Keyboards come from the field's configuration

The patched `gpui-pre` adds `TextInputPurpose` (what the field holds) next to
`TextInputAction` and `Autocapitalize`. gpui-mobile turns them into Android
`EditorInfo` input types and IME options for the host's
`gpuiShowKeyboardWithInputType`. `TextInputPurpose` doesn't exist in Zed's
GPUI or crates.io 0.3.8, so this needs the `[patch.crates-io]` pin.

Kit's `InputState` builders (`fn(self, …) -> Self`):

| Want | Code | Keyboard |
|---|---|---|
| Password | `.masked(true)` + `Input::new(&state).mask_toggle()` | no suggestions or learning, even while shown |
| PIN | `.masked(true).mask_pattern("9999")` | digit pad, hidden |
| Amount | `.mask_pattern(MaskPattern::Number { separator: Some(','), fraction: Some(2) })` | decimal pad |
| Phone | `.input_purpose(TextInputPurpose::Phone)` | phone pad |
| Email | `Input::new(&state).content_type(InputContentType::EmailAddress)` | email keyboard |
| Search / Send key | `.input_action(TextInputAction::Search)` / `::Send` | action key |
| Username | `.autocorrect(false).autocapitalize(Autocapitalize::None).input_action(TextInputAction::Next)` | |
| Anything else | `.input_purpose(TextInputPurpose::…)` | explicit override |

Defaults: single-line fields get `Done`, textareas `Enter`; autocorrect and
sentence capitalisation are on for plain text and off for masked or
patterned input. A `NumberInput` without a number mask opens the full
keyboard unless you set `.input_purpose(TextInputPurpose::Decimal)`.

## Enter and actions

Subscribe to the field:

```rust
cx.subscribe_in(&state, window, |this, state, event: &InputEvent, window, cx| match event {
    InputEvent::PressEnter { .. } => this.submit(window, cx),
    _ => {}
}).detach();
```

The keyboard's action key and a hardware Enter both arrive as `PressEnter`.
Done, Go, Search and Send also hide the keyboard; Next and Previous don't.
Enter in a textarea inserts a newline.

## Back

Back reaches GPUI as `escape`. With the keyboard up, the first Back only
hides it. With Kit's `mobile-lab` branch, a field with `clean_on_escape()`
keeps its text and Back navigates instead of clearing. If the README and
behaviour disagree, trust the device.

Known gap: `OtpInput` and `TimeField` never raise the soft keyboard.

## TalkBack

gpui-mobile exposes GPUI's AccessKit tree to TalkBack (via
`accesskit_android`); no app setup needed.

- Masked inputs have the password role; TalkBack never reads their value.
- Kit's Clear (`.cleanable(true)`) and password toggle buttons are labelled.
  **Icon-only buttons you make need a label**, or TalkBack reads nothing
  useful.
- Not yet reachable: section titles and help text (needs a GPUI/Kit change);
  fields aren't reported as focused/disabled; no autofill.

Testing TalkBack with the shared phone: hold one `phone session` only while
actually testing, ask the user to switch TalkBack on, and ask them to switch
it off afterwards. Gestures under TalkBack differ (double-tap to activate),
so scripted taps won't behave as usual.
