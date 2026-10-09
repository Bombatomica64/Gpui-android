# Draft: gpui: Don't remap touch pans onto a container's other axis

- Branch: [Bombatomica64/zed `gpui-touch-axis`](https://github.com/Bombatomica64/zed/tree/gpui-touch-axis), one commit on Zed `main` (7fb155e9a2)
- Diff: `crates/gpui/src/elements/div.rs`, `crates/gpui/src/window.rs`, +101 −10 (about 90 lines are tests)
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
`ScrollWheelEvent`s from `TouchGestureRecognizer`, and #63553 already locks
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

- `Window` sets `dispatching_touch_scroll` while it dispatches a
  `RecognizedTouchGesture::Scroll` step, momentum included.
- The div scroll handler skips the cross-axis remap while that flag is set.

The flag lives on `Window` rather than on `ScrollWheelEvent`, so the event
type and every place that constructs one stay unchanged.

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

`cargo test -p gpui --lib`: 385 passed, 0 failed (Zed `main` has 383). With the flag never set, `horizontal_touch_pan_does_not_scroll_a_vertical_container` fails (the list moves 60px), so the bug is present on `main`. `cargo fmt --check` and `cargo clippy -p gpui --all-targets --all-features -- --deny warnings` (dev profile) are clean.

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
  #64239 (open, huacnlee, a touch that catches a fling picks its own axis).
  Both change `gestures.rs` only and don't overlap this diff. #64534 (open)
  adds `momentum` to `ScrollWheelEvent` and touches `window.rs` near the same
  dispatch site, so a rebase may be needed if it lands first.
- If a reviewer would rather mark the event than the window, a
  `ScrollWheelEvent::touch: bool` (or reuse of #64534's field) is the
  alternative. It's more invasive because every backend constructs the event.
