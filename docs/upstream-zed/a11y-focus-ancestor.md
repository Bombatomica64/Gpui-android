# Draft: gpui: Report focus on the focusable ancestor of a focused element with no node

- Branch: [Bombatomica64/zed `gpui-a11y-focus-ancestor`](https://github.com/Bombatomica64/zed/tree/gpui-a11y-focus-ancestor), one commit on Zed `main` (e231594b6b, 2026-11-06)
- Diff: `crates/gpui/src/window/a11y.rs`, `crates/gpui/src/elements/div.rs`, `crates/gpui/src/window/a11y/debug.rs` (test accessor), +150 −10 (about 110 lines are tests)
- Lab fork: gpui-pre `mobile-lab` 851c9f8 (Bombatomica64/gpui-pre#2), which applied to Zed `main` unchanged. Lab issue #39, device evidence on lab PR #81.

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.

## Does it fit Zed?

Yes. The code it changes is in Zed `main` unchanged (`window/a11y.rs` is
byte-identical to the gpui-pre snapshot). Zed already notices the case: it
logs "focused element has no accessibility node… assistive technology will
announce the whole window" (`A11y::note_focus_without_node`), and the log
says to give the element an id and a role. That advice doesn't work for a
composite widget whose role belongs on an outer element. No merged or open
PR changes this.

---

## Summary

Composite widgets often keep keyboard focus on an inner element that has no
role, while the role, label and value sit on an outer frame that tracks a
focus handle of its own. A text field is the usual case: the editor div
holds focus, and the frame is the `TextInput`. GPUI only reports focus for a
focused element that has a node. So the tree reports focus on the window,
and screen readers announce the window instead of the field, even while you
type in it.

Moving the role onto the editor isn't always possible. Making the frame
track the editor's handle instead registers the handle twice as a tab stop.
GPUI Kit tried that and reverted it because it broke reverse tab order
(longbridge/gpui-kit#3253).

## Change

When the focused element has no node, whether it has no id or has an id but
no role, focus goes to its nearest ancestor that has a node **and**
registered a focus handle this frame. That frame contains the focus. Only
when there is no such ancestor does focus stay on the window, with the
existing log message. `A11yNodeBuilder::nearest_ancestor` walks the open
node stack, which during prepaint is exactly the element's accessible
ancestors.

Ancestors without a focus handle are skipped, so focus never lands on a
plain container that couldn't be focused itself.

**Desktop:** this only changes which node the AccessKit tree reports as
focused, and only when a screen reader is active and focus is on an element
without a node. Keyboard focus, tab order and the dispatch tree are
untouched.

## Testing

- `window::a11y::tests`:
  `focus_without_node_falls_back_to_focusable_ancestor` (it skips a
  non-focusable wrapper on the way up) and
  `focus_without_node_or_focusable_ancestor_stays_on_window`.
- `elements::div::tests::a11y_focus_goes_to_the_focusable_ancestor_of_a_node_less_element`
  (a real window with accessibility switched on): a `TextInput` frame with
  its own focus handle, around an editor div that holds focus. The editor
  has no id in the first case, and an id but no role in the second. Either
  way, the tree's focus is the frame. Added for Zed; the fork only had the
  builder tests.
- `cargo test -p gpui --lib`: 437 passed, 0 failed (Zed `main` has 434). With the ancestor fallback disabled, the window-level test fails (focus is the window root, not the field), so `main` has the bug. `cargo fmt --check` is clean. Clippy was not re-run.

Manual, Android (the lab): on lab PR #81, TalkBack and `uiautomator`
report the focused text field as `focused=true` while you type. Before, it
was `false`. OnePlus, Android 16, CI release APKs before and after.

**How a desktop reviewer can check it** (not done yet, no macOS/Windows
machine here):

1. Build a small GPUI example with the frame/editor shape above (or use a
   Zed view that has it, see the notes below) on `main` and on the branch.
2. Turn on VoiceOver (macOS) or Narrator (Windows) and tab into the field.
   On `main` it announces the window title. On the branch it announces the
   field's label and role.
3. Or, without listening: run `dev: dump accessibility tree` in Zed (or call
   `Window::debug_a11y_tree_json`) while the field is focused. On the branch,
   the dump's `focus` is the field's node, not the window.

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist (no visual change).
- [ ] Tests cover the new or changed behavior.

Release Notes:

- N/A

---

## Notes for the user (not part of the PR body)

- I haven't found a place in Zed itself that has this shape. Zed's editor
  has no role at all, so an editor inside a focusable, role-bearing frame is
  the case to look for. If there's none, the motivation is GPUI apps (Kit's
  `Input` is the real example), and the PR should say so plainly.
- **Overlaps #65740** (open, "Roll back accessibility nodes when a prepaint
  transaction fails"). Its checkpoint restores `nodes.focus`, and focus set
  through this fallback lives there too, so a rolled-back pass is covered.
  Both add a `#[cfg(test)] A11y::last_tree_update` accessor (same name, same
  meaning); whichever lands second drops its copy.
- If static-text lands too, the two don't interact: static text nodes never
  register focus handles.
