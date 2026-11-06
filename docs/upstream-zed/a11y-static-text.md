# Draft: gpui: Report text without an element id to assistive technology

- Branch: [Bombatomica64/zed `gpui-a11y-static-text`](https://github.com/Bombatomica64/zed/tree/gpui-a11y-static-text), one commit on Zed `main` (e231594b6b, 2026-11-06)
- Diff: `crates/gpui/src/elements/text.rs`, `crates/gpui/src/window/a11y.rs`, `crates/gpui/src/window/a11y/debug.rs` (test accessor), +276 −3 (about 170 lines are tests)
- Lab fork: gpui-pre `mobile-lab` 3274e9f (Bombatomica64/gpui-pre#1), which applied to Zed `main` unchanged. Lab issue #37, device evidence on lab PR #80.

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.
> This one changes how much of every GPUI app reaches screen readers, and
> goes against what Zed's accessibility guide currently says (text needs an
> id, through `text!`). Open a GitHub discussion first, or ask in the
> accessibility channel, before the PR.

## Does it fit Zed?

Zed `main` has the same accessibility code: `window/a11y.rs` and the
AccessKit integration came with #56065, were last changed by #60397, and are
byte-identical to the gpui-pre 0.3.9 snapshot the fix was written on. Nothing
merged since does what this does, and no open PR does either.

But Zed made a choice here. `crates/gpui/src/_accessibility.rs` says a node
needs an id **and** a role, and gives text its id through the `text!` macro
(id from the macro's source location). Zed uses `text!` in about 20 places.
Everything else is plain strings: `ui::Label`/`LabelLike` render their text
as a string child, so in Zed today most labels, headings and button text
never reach a screen reader, unless the parent sets `aria_label`.

This patch doesn't replace `text!`. A `Text` element keeps its own node, and
its inner string isn't reported again (there's a test). It adds a fallback
for text nobody gave an id. The reviewer may prefer to keep text opt-in and
fix `ui::Label` instead. That's the question for the discussion.

---

## Summary

Only elements with an element id and a role get an accessibility node. A
plain string child, a `SharedString` or a `StyledText` has no id, so screen
readers skip it. In GPUI apps that is most static text: headings,
paragraphs, value rows, and the text inside buttons that have no
`aria_label`. Such a button is announced as just "button".

## Change

- While accessibility is active, `&'static str`, `SharedString` and
  `StyledText` add a `Label` leaf (text in `value`, the same way `Text` does)
  under their nearest accessible ancestor
  (`A11yNodeBuilder::push_static_text`).
- Nothing is added when that ancestor already conveys its text:
  - text and value carriers: `Label`, every kind of text field, combo boxes,
    sliders, images, meters…;
  - any other non-container role that has its own label, such as a button
    with `aria_label`.

  An unnamed button or heading still gets its text as a child, which is
  where platforms take its name from. Named containers (`Window`, `Dialog`,
  `Group`, `ScrollView`, `List`…) keep their body text.
- The node id is a hash of the ancestor's node id and the text's index among
  that ancestor's static text. It doesn't depend on the text, so a value row
  whose text changes keeps its node (and screen reader focus).
- Whitespace-only runs are skipped. With accessibility off, the only cost is
  the `is_active()` check.

**Desktop:** nothing changes until a screen reader (or another AccessKit
client) activates the tree. Then the tree has more `Label` nodes, and
unnamed buttons get names.

## Testing

- `window::a11y::tests` (builder level):
  `static_text_is_reported_under_containers`,
  `static_text_skipped_where_the_parent_speaks_for_it`,
  `static_text_ids_are_stable_and_distinct`.
- `elements::text::tests::static_text_reaches_the_a11y_tree` (a real window
  with accessibility switched on): a string under a group and under an
  unnamed button is reported once, a labelled button's text isn't, and a
  `text!` is reported exactly once (its own node only). Added for Zed; the
  fork only had the builder tests.
- `cargo test -p gpui --lib`: 438 passed, 0 failed (Zed `main` has 434). With `report_static_text` disabled, `static_text_reaches_the_a11y_tree` fails (no `Label` for "Heading"), so `main` doesn't report this text. `cargo fmt --check` is clean. Clippy was not re-run.

Manual, Android (the lab): before/after `uiautomator dump` of the Text Input
screen, OnePlus, Android 16, on lab PR #80. Before: 29 nodes and no static
text. After: titles, hints and values are there, and TalkBack reads them.

**How a desktop reviewer can check it** (not done yet, I have no
macOS/Windows machine here):

1. Build Zed from this branch and from `main`.
2. Activate the tree: VoiceOver (macOS, Cmd+F5) or Accessibility Inspector,
   Narrator (Windows), Orca (Linux).
3. Open a dialog or the settings UI, run `dev: dump accessibility tree`, and
   diff the two dumps. On the branch, headings and descriptions show up as
   `Label` nodes, and buttons without `aria_label` have a `Label` child with
   their text.
4. With VoiceOver, tab through the dialog's buttons: on `main` the unnamed
   ones are read as just "button".

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist (no visual change).
- [ ] Tests cover the new or changed behavior.

Release Notes:

- Improved screen reader support: text in Zed's UI that had no accessible label is now read.

---

## Notes for the user (not part of the PR body)

- **Overlaps #65740** (open, suxiaoshao, "Roll back accessibility nodes when
  a prepaint transaction fails"). It journals what the a11y builder records
  inside `Window::transact` and undoes it when the closure fails (`List`
  retrying an autoscroll). Static text nodes go through `push_leaf`, so its
  node rollback covers them. But `static_text_counts` would also need to be
  rolled back. Otherwise the retried pass numbers its text from where the
  failed pass stopped, and those nodes get new ids on that frame. Whichever
  PR lands second has to handle this: one more stack to snapshot.
- The ids are positional. A list that inserts a row above shifts its static
  text indices, so later rows' text gets new ids. That's no worse than today
  (no nodes at all). Text that needs a stable identity should use `text!`
  with an id, as the guide already says.
- Release note: this changes what Zed exposes with a screen reader on. If
  the reviewer would rather not claim it until Zed's own labels are checked,
  use `- N/A`.
