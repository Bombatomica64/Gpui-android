<!--
Target: GitHub issue, zed-industries/zed
Suggested title: gpui: Elements with on_drag can't be dragged by touch
Template: .github/ISSUE_TEMPLATE/10_bug_report.yml (Reproduction steps, Current vs. Expected behavior, Zed version and system specs)
Related: #63553 / #63561 / #64239 (merged touch gestures: dominant axis, long press, fling catch), #63068 (merged, gpui_ios backend), #64205 (open, Windows touch; edits the same drag listener in div.rs, so one of the two needs a rebase), #64079 (open, Wayland touch), #65895 (open, staff, touch helpers for scrollbars in ui), #13698 (open, touch input tracking issue), #65805 (merged 2026-10-20, renamed ScrollWheelEvent to ScrollEvent; the fix branch needs a rebase). No issue found for touch and on_drag.
This is the largest fix (+561 -25, about 350 lines are tests). CONTRIBUTING.md suggests a discussion first; if you prefer, file this as a Discussion instead and keep the closing question.
Rewrite in your own words before posting (Zed AI policy).
-->

### Reproduction steps

A draggable element and a drop target, both using the ordinary mouse drag API:

```rust
struct Dragged;
impl Render for Dragged {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

div().flex().gap(px(50.))
    .child(
        div().id("handle").size(px(50.))
            .on_drag(Dragged, |_, _, _, cx| cx.new(|_| Dragged)),
    )
    .child(
        div().id("target").size(px(50.))
            .on_drop(move |_: &Dragged, _, _| dropped.set(true)),
    )
```

1. Render it on a backend that delivers `TouchEvent`s (or dispatch them in a `#[gpui::test]`).
2. Touch the handle at (10, 10), move to (30, 10), then to (120, 10).
3. Release over the target, then check `cx.active_drag` and `dropped`.

### Current vs. Expected behavior

Current: no drag starts and `dropped` stays false. A finger drag only reaches an element if it claims a `TouchDragEvent`, otherwise it becomes a scroll, and `on_drag`/`on_drag_move`/`on_drop` listen to mouse events only. Splitters, draggable tabs and reorderable rows are dead on touch unless each adds its own touch handling.

Expected: the element drags like it does with a mouse, and drops on the target. A pan along an axis that an enclosing container scrolls should still scroll, as on iOS and Android.

I have a fix with tests on my fork: https://github.com/Bombatomica64/zed/tree/gpui-touch-drag. An `on_drag` element lets the recognizer turn an unclaimed touch into a drag (off the scroll axes, or after a long press), and `Window` replays it through the existing mouse path. Mouse input never goes through the recognizer. Before opening a PR, do you want touch handled this way, as replayed mouse events, or per element through `TouchDragEvent`?

### Zed version and system specs

Confirmed on Zed main at 14119f8d87 with `#[gpui::test]` (gpui only). Seen first in my own Android app built on gpui-pre 0.3.9 through gpui-mobile, on a OnePlus CPH2581 with Android 16, where resizable panels and a dock with tabs could not be dragged by finger.

<!-- fill in: OS of the machine that ran the test; whether you tried Zed on a touch screen (Windows with #64205, iPad, web on a tablet); attach docs/demos/touch-drag-resizable.png and touch-drag-dock.png from the lab repo if wanted -->
