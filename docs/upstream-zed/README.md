# Upstreaming the GPUI fixes to Zed

The GPUI fixes that live only in my gpui-pre fork, prepared for
zed-industries/zed (`crates/gpui`, which gpui-pre snapshots). Items 3 and 8
of [CONTRIBUTING_IDEAS.md](../../CONTRIBUTING_IDEAS.md).

Nothing here has been posted upstream. Each branch is on my fork
[Bombatomica64/zed](https://github.com/Bombatomica64/zed), rebased on
2026-11-06 onto Zed `main` e231594b6b. The 2026-10-16 versions are tagged
`archive/2026-11-06/<branch>` and the 2026-10-09 ones
`archive/2026-10-09/<branch>`. `cargo test -p gpui --lib` passes on each
branch (Zed `main` has 434 tests; each row below says how many the branch
has).

Changes in Zed `main` since the 2026-10-16 rebase:

- #65805 (merged 2026-10-20) renamed `ScrollWheelEvent` to `ScrollEvent`
  (the old name is a deprecated alias) and added `ScrollEvent::source`
  (`Pointer` or `Touch`). touch-axis now uses `source` instead of its own
  flag on `Window`, so it only touches `div.rs`. touch-drag needed import
  fixes and one test pattern renamed.
- #64239 (merged 2026-10-16) lets a touch that catches a fling pick its own
  axis. It's a different bug from touch-axis, and a recent precedent for a
  touch PR in gpui.
- #63068 added an iOS backend (`gpui_ios`). Its `keyboardType` is always
  `Default`, so iOS would ignore `purpose`. The text-input-purpose draft
  should say so or map it (Email → `EmailAddress`, Url → `URL`, Phone →
  `PhonePad`, Numeric → `NumberPad`, Decimal → `DecimalPad`, Search →
  `WebSearch`, Password → `secureTextEntry`). The branch only updates the
  iOS test's struct literal.
- The accessibility code (`window/a11y.rs`, AccessKit, from #56065) hasn't
  changed since #60397 and is identical to gpui-pre 0.3.9's, so both a11y
  fixes applied unchanged. Zed's accessibility guide (`_accessibility.rs`)
  makes text opt-in through `text!` (ids from source location), which is
  the main question for a11y-static-text.

Open PRs that touch the same code:

- #64205 (Windows touch, press-and-hold; last updated 2026-09-21) changes
  the drag listener in `div.rs` that touch-drag also edits. Whichever lands
  second needs a rebase. It predates #65805, so it needs one anyway.
- #65740 (roll back a11y nodes when a prepaint transaction fails) checkpoints
  the a11y builder. static-text adds a per-node counter that its checkpoint
  would also need to restore. Both a11y branches add the same
  `#[cfg(test)] A11y::last_tree_update` accessor it adds. See the a11y
  drafts.
- #64534, #66012 (iOS trackpad momentum) and #64079 (Wayland touch) change
  `gestures.rs`/`window.rs` scroll dispatch. After the rework, touch-axis no
  longer touches those files. touch-drag's gesture changes may need small
  rebases.

| Fix | Branch | Applies to Zed `main` (2026-11-06) | Tests | Draft |
| --- | --- | --- | --- | --- |
| Touch drags `on_drag` elements | [`gpui-touch-drag`](https://github.com/Bombatomica64/zed/tree/gpui-touch-drag) | import conflicts and the `ScrollEvent` rename fixed | 441 | [touch-drag.md](touch-drag.md) |
| Scrollable divs honor `request_autoscroll` | [`gpui-autoscroll`](https://github.com/Bombatomica64/zed/tree/gpui-autoscroll) | as is | 437 | [autoscroll.md](autoscroll.md) |
| No cross-axis remap for touch pans | [`gpui-touch-axis`](https://github.com/Bombatomica64/zed/tree/gpui-touch-axis) | reworked onto `ScrollEvent::source`, `div.rs` only | 436 | [touch-axis.md](touch-axis.md) |
| `TextInputConfiguration::purpose` | [`gpui-text-input-purpose`](https://github.com/Bombatomica64/zed/tree/gpui-text-input-purpose) | as is, plus a `gpui_web` mapping | 434 (changes an existing test) | [text-input-purpose.md](text-input-purpose.md) |
| Static text in the a11y tree | [`gpui-a11y-static-text`](https://github.com/Bombatomica64/zed/tree/gpui-a11y-static-text) | as is, plus a window-level test | 438 | [a11y-static-text.md](a11y-static-text.md) |
| Focus on the focusable ancestor of a node-less element | [`gpui-a11y-focus-ancestor`](https://github.com/Bombatomica64/zed/tree/gpui-a11y-focus-ancestor) | as is, plus a window-level test | 437 | [a11y-focus-ancestor.md](a11y-focus-ancestor.md) |

The two a11y fixes come from gpui-pre `mobile-lab` (Bombatomica64/gpui-pre#1
and #2, lab issues #37 and #39). They matter on desktop as well: with
VoiceOver, Narrator or Orca on, Zed's `dev: dump accessibility tree` action
shows the difference. Each draft has steps for a desktop reviewer.

## Before opening any of them

- **Sign Zed's CLA** (<https://zed.dev/cla>). It's needed before a merge.
- **AI policy.** Zed's CONTRIBUTING.md says they don't accept PRs from
  autonomous agents, and PR descriptions and replies should come from the
  author, not a model. The drafts are notes to write from, not text to
  paste. You should also be able to explain every line of each diff in
  review.
- **Three open PRs at most** per contributor, and they suggest starting with
  one. Suggested order:
  1. touch-axis: smallest, clearly a bug, desktop-neutral, and now built
     on upstream's own `ScrollEvent::source`.
  2. a11y-focus-ancestor: small bug fix with no new API. Desktop changes only
     with a screen reader on. Say clearly that the motivating widget is
     Kit's `Input` if no Zed view has the same shape.
  3. autoscroll: makes a documented API work for divs, but changes desktop
     Zed, so try it in Zed first.
  4. touch-drag: largest. A GitHub discussion first is the safer start.
     Coordinate with #64205.
  5. a11y-static-text: changes how much of every GPUI app screen readers
     see, and goes against the guide's "text needs an id" model. Start with
     a discussion (or Zed's accessibility folks), and wait for #65740.
  6. text-input-purpose: new API, so a discussion first.
- **Build and run Zed locally** with the change, as CONTRIBUTING.md asks.
  This matters most for autoscroll (desktop behavior) and the two a11y fixes
  (check them with a screen reader on and `dev: dump accessibility tree`).
- PR titles follow `crate: Sentence-case summary` (e.g. `gpui: Lock touch
  scrolling to the dominant axis`, #63553), and the body follows
  `.github/pull_request_template.md`: Summary, Testing, Self-review, then
  `Release Notes:` with `- N/A` for changes users don't see.
