# input: Ask scroll containers to reveal a focused input

- Branch: [`upstream/input-reveal-on-focus`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-reveal-on-focus), one commit on upstream `main` (0bbd9870)
- Diff: 2 files, +136 −2 (`input/base/element.rs` +134 including a 60-line test, `state.rs` +4)
- Depends on, to be useful:
  - **Zed PR "gpui: Reveal a descendant's autoscroll request in scrollable divs"** ([draft](../../upstream-zed/autoscroll.md)), released in a `gpui-pre` snapshot that Kit pins. Without it only `List` honours the request, so in the usual `overflow_y_scroll()` form this PR does nothing. It's safe but inert.
  - On Android, a viewport that shrinks when the IME opens (gpui-mobile's host with `adjustResize`, as today).
- **Its own test fails on Kit `main` today**: `focused_input_asks_its_scroll_container_to_reveal_it` uses an `overflow_y_scroll` div, and stock gpui-pre 0.3.8 divs ignore the request, so `assert!(scroll_handle.offset().y < px(0.))` fails (the test was written against our fork's patched gpui-pre; not re-run there for this prep). Once Kit pins a gpui-pre with the autoscroll change, it should pass unchanged; re-run it then. (Another option: test it inside a `List`, which honours the request today. But that wouldn't test the common case.)
- **Send after the GPUI autoscroll change is in a gpui-pre release.** Sending it earlier would ask reviewers to accept code whose main effect they can't see.
- Desktop impact: an input inside a `List` now scrolls into view when it gains focus (e.g. by Tab), or when the window is resized while it has focus. Worth stating; it's arguably the right behaviour.
- Recording: **missing.** Follow-up: a long form in a scrolling div, tapping a field near the bottom. Before: the keyboard covers it. After: the page scrolls it above the keyboard. This needs the GPUI autoscroll change, so it can only be captured on our fork build.

---

## Description

When an on-screen keyboard opens, the window viewport shrinks, but nothing scrolls the focused input back into view, so the keyboard covers the field being typed into. Apps have to read the input's bounds and move their own scroll handle.

The input element now calls `Window::request_autoscroll` while prepainting when it's focused and has either just gained focus or the viewport has changed size since its last request:

- The state remembers the viewport it last revealed at, so the request isn't repeated every frame, and the user can still scroll away from a focused field.
- A single-line input asks for all of itself. A multi-line one asks for the caret's line (or its top 120 px when it has no caret), so a tall textarea doesn't drag the page to its bottom.
- The request has an 8 px margin for the field's padding and border.

`List` already honours these requests. Plain `overflow_scroll` divs do with GPUI's change (zed-industries/zed#TBD).

No public API changes.

## Screenshot

*(Missing; see above.)*

## How to Test

- New test `focused_input_asks_its_scroll_container_to_reveal_it`. `cargo test -p gpui-base --lib input`: **1 failed** (`focused_input_asks_its_scroll_container_to_reveal_it`, needs GPUI autoscroll; see above), 337 passed
- `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`: clean
- `cargo test -p gpui-kit --features test-support --test input --test input_focus`: input 179 passed, input_focus 4 passed

## Checklist

- [x] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [x] Reviewed the changes in this PR and confirmed AI generated code (If any) is accurate.
- [ ] Passed `cargo run` for story tests related to the changes.
- [ ] Tested macOS, Windows and Linux platforms performance (if the change is platform-specific)

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
