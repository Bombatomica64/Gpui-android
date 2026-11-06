# android: Report clickable on accessibility nodes that offer a click

- Branch: [`upstream/android-a11y-clickable`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-a11y-clickable), one commit on upstream `main` (7548ebc, #45)
- Diff: 1 file, +39 −5 (`src/android/accesskit_adapter.rs`, the injecting adapter #25 added)
- Depends on: nothing. Same fix as my fork's Bombatomica64/gpui-mobile#15 (lab issue Bombatomica64/Gpui-android#39).
- Recording: a `uiautomator dump` before/after (Gpui-android#81). TalkBack itself was not used.

---

accesskit_android 0.7.5 adds `ACTION_CLICK` to a node that can be clicked (or focused) but never calls `setClickable`. So every node, buttons included, reports `clickable=false` to UI Automator and to accessibility services that read the flag rather than the action list (Switch Access, test tools). TalkBack activates by the action, so its double-tap was not affected.

### Change

- After accesskit builds an `AccessibilityNodeInfo`, the adapter marks it clickable when its action list has `ACTION_CLICK` (`0x10`).
- If a JNI call fails, the node is left as accesskit built it and the exception is cleared, so it can't escape through `createAccessibilityNodeInfo`.
- The file's header lists this next to the other changes from the crates.io adapter.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed, as on `main`.
- On device (OnePlus CPH2581, Android 16), my test app's Text Input screen, `uiautomator dump` with release builds before and after:

| class | label | clickable before | clickable after |
| --- | --- | --- | --- |
| Button | Catalog | false | true |
| Button | Clear | false | true |
| EditText | Type here… (not focused) | false | true |
| EditText | the focused field | false | false |

  An unfocused EditText is clickable because accesskit offers Click on focusable nodes, which matches a native EditText. The focused field stays `false` because accesskit drops Click and Focus once the node is focused.
- Not checked: TalkBack double-tap after the change (it reads the action, which didn't change).

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
