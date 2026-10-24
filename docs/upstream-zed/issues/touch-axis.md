<!--
Target: GitHub issue, zed-industries/zed
Suggested title: gpui: Touch pans are remapped onto a container's other axis
Template: .github/ISSUE_TEMPLATE/10_bug_report.yml (Reproduction steps, Current vs. Expected behavior, Zed version and system specs)
Related: #63553 (merged, locks touch pans to the dominant axis; this bug follows from it), #64239 (merged, fling catch axis, gestures.rs only), #25963 (merged, added restrict_scroll_to_axis), #64534 (open, touches the same dispatch site in window.rs). #65805 (merged 2026-10-20) renamed ScrollWheelEvent to ScrollEvent, so the fix branch needs a rebase. No open issue found for this bug.
Rewrite in your own words before posting (Zed AI policy).
-->

### Reproduction steps

A container that scrolls vertically only, without `restrict_scroll_to_axis()`:

```rust
div()
    .id("list")
    .w(px(100.))
    .h(px(100.))
    .overflow_y_scroll()
    .track_scroll(&scroll_handle)
    .child(div().w_full().h(px(300.)))
```

1. Render it in a window on a backend that delivers `TouchEvent`s (in a `#[gpui::test]`, dispatch them with `window.dispatch_event`).
2. Touch at (80, 50), move to (20, 50), release. This is a purely horizontal swipe.
3. Read `scroll_handle.offset()`.

### Current vs. Expected behavior

Current: the offset is `(0, -60)`. The list scrolled vertically although the finger only moved sideways.

Expected: the offset stays `(0, 0)`. A vertical swipe over the same list should still scroll it.

The div scroll handler maps `delta.x` onto `y` when a container only scrolls one way, which suits a mouse wheel. #63553 already locks a touch pan to its dominant axis, so a sideways swipe arrives as `(dx, 0)` and moves the vertical list by `dx`. My app works around it by setting `restrict_scroll_to_axis()` on every container.

I have a fix with a test on my fork: https://github.com/Bombatomica64/zed/tree/gpui-touch-axis. `Window` marks the dispatch of a recognized touch scroll, and the div skips the remap while it is marked. Wheels and trackpads are unchanged. Happy to open a PR if you want it.

### Zed version and system specs

Confirmed on Zed main at 14119f8d87 (gpui only, I did not run Zed itself). Seen first in my own Android app built on gpui-pre 0.3.9 through gpui-mobile, on a OnePlus CPH2581 with Android 16.

<!-- fill in: OS of the machine that ran the #[gpui::test]; output of `zed: copy system specs into clipboard` if you reproduce it in Zed itself -->
