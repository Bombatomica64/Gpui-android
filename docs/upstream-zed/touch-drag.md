# Draft: gpui: Let touch drag elements that only handle mouse drags

- Branch: [Bombatomica64/zed `gpui-touch-drag`](https://github.com/Bombatomica64/zed/tree/gpui-touch-drag), one commit on Zed `main` (7fb155e9a2)
- Diff: `crates/gpui/src/{gestures.rs, window.rs, elements/div.rs, elements/list.rs}`, +561 −25 (about 350 lines are tests)
- Lab fork: gpui-pre `touch-drag`, which applies to Zed `main` unchanged

> Before posting: rewrite this in your own words. Zed's AI policy asks for
> PR text written by the author. Sign the CLA, and check Zed's limit of three
> open PRs per contributor.
> This is the largest of the three and changes how touch input behaves, so
> CONTRIBUTING.md suggests starting a GitHub discussion first. The
> "Discussion first" section below is a short version for that.

---

## Summary

On a touch screen, nothing built on `on_drag` can be dragged with a finger.

A finger drag reaches an element as a drag only if the element claims a
`TouchDragEvent` on `Started`; otherwise the touch becomes a scroll.
`on_drag`, `on_drag_move` and `on_drop` listen to mouse events, so every
element built on them is dead on touch, unless it adds its own touch
handling. In GPUI apps that includes:

- split and dock resize handles,
- draggable tabs and panels,
- reorderable list rows.

In Zed that means pane splitters, tab dragging and project panel drag and
drop, on Windows touch screens (#64205), iPad/iOS (#63068) and the web build
on a tablet.

Screenshots from the GPUI Mobile Lab on Android (OnePlus CPH2581, Android 16),
each strip being before the drag, mid-drag and after:

- Resizable panels, a splitter dragged by finger:
  `docs/demos/touch-drag-resizable.png` (attach from the lab repo)
- Dock with tabs and a panel resized by finger:
  `docs/demos/touch-drag-dock.png` (attach from the lab repo)

## Change

- An element with `on_drag` registers a `TouchDragEvent` listener in the
  bubble phase. If no element in front claimed the touch, it marks the
  touch as starting on a mouse-draggable element, along with the axes its
  enclosing scroll containers can currently scroll. `Interactivity` and
  `List` record those axes while painting.
- The recognizer keeps that touch pending, so taps and claimed long presses
  behave as before. It turns the touch into a drag when:
  - the touch leaves the touch slop along an axis that no enclosing
    container scrolls, or
  - a long press fires and nobody claims it.

  A pan along a scroll axis still scrolls. So a handle or splitter drags
  directly, while a row in a scrolling list scrolls, and drags after a long
  press or a sideways move, as on iOS and Android.
- `Window` carries the drag through the mouse path: a left press at the
  start, moves with the button held, and a release. That means
  `active_drag`, `on_drag_move`, `on_drop`, drag-over styles and plain
  mouse listeners work unchanged.
- A cancelled touch clears the drag first, then releases outside the window,
  so it neither drops nor clicks.

**Desktop is unchanged:** mouse and trackpad events never go through
`TouchGestureRecognizer`. The new listener only reacts to `TouchDragEvent`,
which only touch input produces. A mouse drag-and-drop test runs alongside
the touch ones to show the mouse path is unaffected.

## Testing

New tests:

- `window::tests`:
  - `touch_drags_and_drops_an_on_drag_element`
  - `cancelled_touch_drag_ends_without_dropping`
  - `tapping_an_on_drag_element_still_clicks`
  - `touch_drag_inside_scroll_container_scrolls_along_its_axis`
  - `touch_drag_on_plain_content_scrolls`
  - `mouse_drags_and_drops_an_on_drag_element`
- `gestures::tests::mouse_draggable_touch_drags_unless_it_pans_along_a_scroll_axis`

`cargo test -p gpui --lib`: 390 passed, 0 failed. With the `allow_mouse_drag` hook disabled, three of the new touch tests fail, so `main` can't touch-drag an `on_drag` element. `cargo fmt --check` and `cargo clippy -p gpui --all-targets --all-features -- --deny warnings` (dev profile) are clean.

Manual: Android, GPUI Mobile Lab with a lab fork of GPUI (screenshots
above). Resizable and Dock work by touch.
**Recording still needed:** a short GIF of a list row being reordered
after a long press, and a check of Zed itself on a touch screen if one is
available (Windows touch laptop with #64205, or the web build on a tablet).

## Self-review

- [ ] I've reviewed my diff for quality, security, reliability, and performance.
- [ ] UI changes follow the checklist.
- [ ] Tests cover the new or changed behavior.

Release Notes:

- N/A

---

## Discussion first (for a GitHub discussion, if preferred)

> Touch input in GPUI turns every unclaimed finger drag into a scroll, so
> `on_drag`/`on_drop` elements (splitters, tabs, reorderable rows) can't be
> dragged by touch. I have a change that lets an `on_drag` element take a
> touch drag that doesn't pan along its scroll containers' axes (or after an
> unclaimed long press), replayed through the existing mouse drag path.
> Would you take a PR for this, and is replaying it as mouse events
> the approach you'd want, rather than having each element handle
> `TouchDragEvent` itself?

## Notes for the user (not part of the PR body)

- Behavior to double-check in Zed itself: the project panel scrolls on both
  axes when entries are wide, and its rows are `on_drag`. A row then drags
  only after a long press, which is probably right, but where only vertical
  scroll is possible a sideways swipe on a row starts a file drag. Say so in
  the PR, or ask in the discussion.
- `DataTable` column resize/reorder in gpui-kit still didn't work by touch in
  the lab (README "Known Android issues"). That's a Kit-side issue, not this
  patch, but don't claim table columns in the PR text without checking.
- Related open PRs in the same area: #64239 (fling-catch axis, gestures.rs),
  #64534 (keep a scroll gesture with its content; window.rs dispatch). Either
  may need a rebase of this branch if it lands first.
