<!--
Target: GitHub Discussion, zed-industries/zed, category Feature requests (feature requests are not filed as issues)
Suggested title: gpui: Tell the platform what a text input holds (TextInputConfiguration::purpose)
Template: none for Discussions; this is a short proposal
Related: #63068 (merged, gpui_ios backend; its keyboardType is always Default), #63740 (open, gpui_web keyboard focus), #63220 / #63629 (merged, gpui_web mobile keyboard and IME mirror). No issue or PR found for keyboard type, inputmode or TextInputConfiguration. Discussions were not searched: `gh search` does not cover them and GraphQL was avoided; check the Discussions tab by hand.
Send after the three touch bugs, or not at all if the discussion says no.
Rewrite in your own words before posting (Zed AI policy).
-->

`TextInputConfiguration` (autocorrect, autocapitalize, suggestions, input action) already lets a field shape the software keyboard, but not say what it holds. A PIN field gets the full letter layout, an email field has no `@` key, and a password field gets suggestions that learn from what is typed into it.

I would like to add `purpose: TextInputPurpose` to the configuration, defaulting to `Text` so existing code is unchanged. The values follow HTML's `inputmode` (`Text`, `Search`, `Email`, `Url`, `Phone`, `Numeric`, `Decimal`) plus `Password` and `NumericPassword`, which also ask the platform to keep the text out of suggestions and learning.

Zed has two backends that can use it:

- `gpui_web` maps it onto the IME mirror's `inputmode`, and turns off spellcheck, autocomplete and autocorrect for the password kinds.
- `gpui_ios` returns `UIKeyboardType::Default` today. A mapping would be Email to `EmailAddress`, Url to `URL`, Phone to `PhonePad`, Numeric to `NumberPad`, Decimal to `DecimalPad`, Search to `WebSearch`, and the password kinds to `secureTextEntry`. I have not written this half, since I can't test on iOS.

Native desktop backends would ignore the field. I use the same field in my Android app through gpui-mobile, where it picks the number pad and the password keyboard.

I have the `gpui` and `gpui_web` half with a test on my fork, rebased on main 14119f8d87: https://github.com/Bombatomica64/zed/tree/gpui-text-input-purpose. The web half compiles for wasm32 but I have not tried it in a browser. Would you take a PR for this, and would you want the iOS mapping in the same one?
