<!--
Target: GitHub issue, zed-industries/zed
Suggested title: gpui: Window::request_autoscroll is ignored by scrollable divs
Template: .github/ISSUE_TEMPLATE/10_bug_report.yml (Reproduction steps, Current vs. Expected behavior, Zed version and system specs)
Related: #10889 (merged 2024, added request_autoscroll with List as the only consumer). #65623 (open issue) and #65740 (open PR) are about List's autoscroll retry and a11y nodes, which this change does not touch. #64236 (closed) is also List-related. No open issue found for divs.
Note: this changes desktop Zed (editors and markdown inside scrollable divs start honoring their requests); try it in a Zed build before filing.
Rewrite in your own words before posting (Zed AI policy).
-->

### Reproduction steps

An element that requests autoscroll while prepainting, inside a scrollable div:

```rust
div()
    .id("outer")
    .w(px(100.))
    .h(px(100.))
    .overflow_y_scroll()
    .track_scroll(&scroll_handle)
    .child(div().h(px(300.)))
    .child(
        canvas(
            move |bounds, window, _| {
                if request.take() {
                    window.request_autoscroll(bounds);
                }
            },
            |_, _, _, _| {},
        )
        .w_full()
        .h(px(20.)),
    )
```

1. Draw the window. The canvas sits at y = 300, below the 100px viewport.
2. Set `request` and redraw twice.
3. Read `scroll_handle.offset()`.

### Current vs. Expected behavior

Current: the offset stays `(0, 0)`. The request is dropped.

Expected: the div scrolls just far enough to show the bounds, so the offset ends at `(0, -220)` and the canvas ends at the bottom edge. The docs for `Window::request_autoscroll` say "containing elements will attempt to scroll", but only `List` does.

On a phone this is a focused text field that never scrolls into view when the software keyboard opens, so the user types behind the keyboard. Each app has to move its own `ScrollHandle`. Zed itself calls `request_autoscroll` from `EditorElement` and `MarkdownElement`, so an editor in a scrollable div would also keep its cursor visible.

I have a fix with a test on my fork: https://github.com/Bombatomica64/zed/tree/gpui-autoscroll. The div applies the offset on the next frame, instead of prepainting its children twice, and re-requests whatever it couldn't reveal so nested scrollers each do their part. Happy to open a PR if you want it.

### Zed version and system specs

Confirmed on Zed main at 14119f8d87 with `#[gpui::test]` (gpui only). Seen first in my own Android app built on gpui-pre 0.3.9 through gpui-mobile, on a OnePlus CPH2581 with Android 16.

<!-- fill in: OS of the machine that ran the test; whether you checked editor/markdown in a scrollable div in a Zed build -->
