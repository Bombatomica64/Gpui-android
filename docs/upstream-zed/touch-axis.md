# Draft: gpui: Don't remap touch pans onto a container's other axis

- Branch: [Bombatomica64/zed `gpui-touch-axis`](https://github.com/Bombatomica64/zed/tree/gpui-touch-axis), one commit on Zed `main` (e231594b6b, 2026-11-06)
- Diff: `crates/gpui/src/elements/div.rs` only, +97 −11 (about 90 lines are tests)
- Lab fork: gpui-pre `touch-axis`

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.

---

## Summary

On a touch screen, a sideways swipe over a container that only scrolls
vertically scrolls it up and down.

`Interactivity`'s scroll handler maps a wheel's delta onto the other axis
when the container scrolls only one way. Unless
`restrict_scroll_to_axis()` is set, a vertical-only container takes `delta.x`
as `delta.y`. That makes sense for a mouse wheel, which has one axis: the wheel
over a horizontal-only strip scrolls it sideways. But touch pans reach the same handler as
`ScrollEvent`s from `TouchGestureRecognizer`, and #63553 already locks
each pan to its dominant axis. So a deliberate horizontal swipe arrives as
`(dx, 0)` and moves a vertical list by `dx`. Some ways a touch user hits it:

- Swiping a horizontal carousel or code block inside a vertical page: when
  the inner element can't scroll further (or doesn't scroll), the page jumps
  vertically instead.
- A swipe that is "mostly sideways" scrolls the list in a direction the
  finger never went.

To avoid it, every touch app had to set `restrict_scroll_to_axis()` on every
container.

## Change

- The div scroll handler skips the cross-axis remap when the event's
  `source` is `ScrollSource::Touch` (pans and the fling momentum GPUI
  synthesizes after them).

This uses `ScrollEvent::source`, added by #65805 (merged 2026-10-20). Before
that, the branch set a flag on `Window` while it dispatched touch scroll
steps; that flag and its `window.rs` hunk are gone.

**Desktop is unchanged:** mouse wheels, trackpads (they come from the
platform as `ScrollWheelEvent`, not through the touch recognizer) and
`restrict_scroll_to_axis` behave exactly as before. Only touch pans that go
through `TouchGestureRecognizer` are affected.

## Testing

Two new tests in `elements::div::tests`:

- `horizontal_touch_pan_does_not_scroll_a_vertical_container`: a horizontal
  pan leaves a vertical-only scroller at offset 0, and a vertical pan still
  scrolls it.
- `horizontal_wheel_still_scrolls_a_vertical_container`: a horizontal wheel
  delta still scrolls it vertically by the same amount, as today.

`cargo test -p gpui --lib` (2026-11-06 rebase): 436 passed, 0 failed (Zed `main` has 434). With the `source` check disabled, `horizontal_touch_pan_does_not_scroll_a_vertical_container` fails (the list moves 60px), so the bug is present on `main`. `cargo fmt --check` is clean. `cargo clippy -p gpui --all-targets --all-features -- --deny warnings` was clean on the 2026-10-16 rebase and was not re-run after the 2026-11-06 one.

Manual: **not yet verified on a device.** The lab's Scroll Stress screen
(nested axes) is the place for it: a sideways swipe over a vertical list
without `restrict_scroll_to_axis()`, before and after, recorded on the phone.
(The lab currently works around the bug with `restrict_scroll_to_axis()`;
see COMPONENT_MATRIX.md.)

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist (no visual change on desktop).
- [ ] Tests cover the new or changed behavior.

Release Notes:

- N/A

---

## Notes for the user (not part of the PR body)

- Related upstream work: #63553 (merged, locks pans to the dominant axis),
  #64239 (merged 2026-10-16, a touch that catches a fling picks its own
  axis), #65805 (merged 2026-10-20, renamed `ScrollWheelEvent` to
  `ScrollEvent` and added `ScrollEvent::source`, which this branch now
  uses). Open #64534 and #66012 change `gestures.rs`/`window.rs` and add
  fields to `ScrollEvent`. Since the 2026-11-06 rebase this branch touches
  only `div.rs`, so they don't overlap.
- The 2026-11-06 rebase replaced the `Window::dispatching_touch_scroll` flag
  with `event.source == ScrollSource::Touch`. That's the "mark the event"
  alternative an earlier version of this note suggested, and upstream has
  now built it.
