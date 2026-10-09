# Draft: gpui: Reveal a descendant's autoscroll request in scrollable divs

- Branch: [Bombatomica64/zed `gpui-autoscroll`](https://github.com/Bombatomica64/zed/tree/gpui-autoscroll), one commit on Zed `main` (7fb155e9a2)
- Diff: `crates/gpui/src/elements/div.rs` only, +277 −25 (about 150 lines are tests, about 25 are a function moved out unchanged)
- Lab fork: gpui-pre `autoscroll`

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.
> **This one changes desktop behavior in Zed** (see "Effect on Zed" below), so
> build Zed with it and try the cases listed there before opening it.

---

## Summary

`Window::request_autoscroll` says that "containing elements will attempt to
scroll" to reveal the given bounds, but only `List` honors it. A plain
`div().overflow_y_scroll()` drops the request.

On a phone this means a focused text field never scrolls into view. When the
software keyboard opens, the window gets shorter, and a field in the lower
half of a form ends up behind the keyboard. The field requests autoscroll,
nothing scrolls, and the user types blind. Every app has to work around it
by moving its own `ScrollHandle`.

## Change

When a div scrolls on some axis, it now takes the autoscroll request that
its descendants made while prepainting:

- It moves its scroll offset by the smallest amount that brings the
  requested bounds inside its own bounds, clamped to its scroll range.
- The children are already prepainted at the old offset, and unlike `List` a
  div can't prepaint them again. So the new offset takes effect on the next
  frame, which it requests.
- Whatever it can't reveal (because it is clamped, or because an ancestor
  clips it) it requests again, shifted to where the target will be next
  frame. Outer scroll containers continue from there, so nested scrollers
  each do their part.
- A request made before the div's children prepaint (by an earlier sibling)
  isn't the div's to handle. The div holds it and passes it on untouched.

The scroll range computation moves from `clamp_scroll_position` into a
`scroll_max` helper so both paths share it. The code inside is unchanged.

Without a request, nothing changes: scroll offsets, wheel, trackpad and
scrollbar dragging all behave as before.

## Effect on Zed

Zed already calls `request_autoscroll` in two places:

- `EditorElement`, when the editor has an autoscroll request or a pending
  selection (`autoscroll_containing_element`).
- `MarkdownElement` with `AutoscrollBehavior::Propagate`, the default.

Today these are only honored when the editor or markdown sits in a `List`
(for example, the agent thread). With this change, an editor or markdown
view inside a scrollable div also keeps its cursor or selection visible
while typing or drag-selecting. That is what the API documents, but it is a
visible change. Cases to try by hand:

- An auto-height editor in a scrollable modal or panel: type past the
  bottom edge.
- Drag-select in a markdown view inside a scrollable div, past the edge.

**TODO before posting:** list which Zed views these are and attach a short
before/after video from desktop Zed.

## Testing

Three new tests in `elements::div::tests`:

- `scroll_container_reveals_a_descendants_autoscroll_request`: the target
  ends exactly at the container's bottom edge on the next frame.
- `nested_scroll_containers_each_reveal_their_share`: the inner scroller
  reveals what it can, and the outer one scrolls the rest.
- `scroll_offset_is_untouched_without_an_autoscroll_request`: offsets set
  by hand survive redraws.

`cargo test -p gpui --lib`: 386 passed, 0 failed. With the new path disabled, the two reveal tests fail, so `main` ignores the request. `cargo fmt --check` and `cargo clippy -p gpui --all-targets --all-features -- --deny warnings` (dev profile) are clean. The `editor` and `markdown` crate tests were not run (too large a build for this machine).

Manual: **not yet verified on a device.** A recording is needed: in a lab
build with this branch and *without* gpui-kit `input-reveal-on-focus`, focus
a field low in a form; the keyboard opens and the form scrolls to it.
Record before and after.

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist.
- [ ] Tests cover the new or changed behavior.

Release Notes:

- N/A *(or, if the desktop effect above is confirmed: "Fixed editors inside
  scrollable panels not scrolling to keep the cursor visible")*

---

## Notes for the user (not part of the PR body)

- Closest upstream history: #10889 ("Introduce autoscroll support for
  elements", merged 2024) added the API with `List` as its only consumer.
  No open PR or issue was found for divs.
- The one-frame lag (the request is applied on the next frame) is the
  trade-off for not prepainting twice. A reviewer may ask about it.
- The lab also carries gpui-kit `input-reveal-on-focus`, which scrolls a
  focused Kit input into view. Check how the two overlap before using the
  lab as the motivating example.
