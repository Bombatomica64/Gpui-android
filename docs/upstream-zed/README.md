# Upstreaming the GPUI fixes to Zed

The GPUI fixes that live only in our gpui-pre fork, prepared for
zed-industries/zed (`crates/gpui`, which gpui-pre snapshots). Items 3 and 8
of [CONTRIBUTING_IDEAS.md](../../CONTRIBUTING_IDEAS.md).

Nothing here has been posted upstream. Each branch is on our fork
[Bombatomica64/zed](https://github.com/Bombatomica64/zed), based on Zed
`main` 7fb155e9a2 (2026-10-09).

| Fix | Branch | Applies to Zed `main` | Draft |
| --- | --- | --- | --- |
| Touch drags `on_drag` elements | [`gpui-touch-drag`](https://github.com/Bombatomica64/zed/tree/gpui-touch-drag) | as is | [touch-drag.md](touch-drag.md) |
| Scrollable divs honor `request_autoscroll` | [`gpui-autoscroll`](https://github.com/Bombatomica64/zed/tree/gpui-autoscroll) | as is | [autoscroll.md](autoscroll.md) |
| No cross-axis remap for touch pans | [`gpui-touch-axis`](https://github.com/Bombatomica64/zed/tree/gpui-touch-axis) | one struct-field hunk moved | [touch-axis.md](touch-axis.md) |
| `TextInputConfiguration::purpose` | [`gpui-text-input-purpose`](https://github.com/Bombatomica64/zed/tree/gpui-text-input-purpose) | as is, plus a new `gpui_web` mapping | [text-input-purpose.md](text-input-purpose.md) |

## Before opening any of them

- **Sign Zed's CLA** (<https://zed.dev/cla>). It's needed before a merge.
- **AI policy.** Zed's CONTRIBUTING.md says they don't accept PRs from
  autonomous agents, and PR descriptions and replies should come from the
  author, not a model. The drafts are notes to write from, not text to
  paste. You should also be able to explain every line of each diff in
  review.
- **Three open PRs at most** per contributor, and they suggest starting with
  one. Suggested order:
  1. touch-axis: smallest, clearly a bug, desktop-neutral.
  2. autoscroll: makes a documented API work for divs, but changes desktop
     Zed, so try it in Zed first.
  3. touch-drag: largest. A GitHub discussion first is the safer start.
  4. text-input-purpose: new API, so a discussion first.
- **Build and run Zed locally** with the change, as CONTRIBUTING.md asks.
  This matters most for autoscroll, which changes desktop behavior.
- PR titles follow `crate: Sentence-case summary` (e.g. `gpui: Lock touch
  scrolling to the dominant axis`, #63553), and the body follows
  `.github/pull_request_template.md`: Summary, Testing, Self-review, then
  `Release Notes:` with `- N/A` for changes users don't see.
