# What the forks carry on top of upstream

The lab builds against one GPUI snapshot (`gpui-pre` 0.3.8) and three forks,
each pinned by `rev` in `Cargo.toml`. Every fix lives on its own branch,
based on upstream, so it can be proposed upstream on its own; a combined
branch is what the lab pins.

| Fork | Upstream | Branch the lab pins |
|---|---|---|
| [Bombatomica64/gpui-mobile](https://github.com/Bombatomica64/gpui-mobile) | [longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) `main` | `main` (upstream `main` + the branches below, merged) |
| [Bombatomica64/gpui-kit](https://github.com/Bombatomica64/gpui-kit) | [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit) tag `v0.7.1` | `mobile-lab` (`v0.7.1` + the commits below) |
| [Bombatomica64/gpui-pre](https://github.com/Bombatomica64/gpui-pre) | `gpui-pre` 0.3.8 from crates.io (a snapshot of [zed-industries/zed](https://github.com/zed-industries/zed) `crates/gpui`) | `mobile-lab` (branch `0.3.8` + the branches below) |

`gpui-pre` replaces the crates.io release through `[patch.crates-io]`;
`gpui-kit`, `gpui-fps` and `gpui-pre-mobile` are git dependencies.

Fixes from earlier versions of this file that are now upstream: the
gpui-pre 0.3.7 bump (gpui-mobile #21), Slider touch drags and the gpui-fps
Android font (gpui-kit #3313, in v0.7.1).

## gpui-mobile

### Bump gpui-pre to 0.3.8 — branch `bump-gpui-pre-0.3.8`

GPUI Kit 0.7.1 pins `gpui-pre =0.3.8`; gpui-mobile pinned `=0.3.7`, so the
two could not be combined. `RequestFrameOptions` gained `signal_at` and
`signal_source`; the Android window fills them with their defaults, as the
iOS frame callback already did. The example moves to Kit v0.7.1.

### Use the system clipboard — branch `fix/android`, [#24](https://github.com/longbridge/gpui-mobile/pull/24)

`AndroidClipboard` was an in-process string, so copy never reached
Android's clipboard and text copied in other apps could not be pasted. With
the `clipboard` feature it now goes through `ClipboardManager`, falling back
to the local copy when the call fails.

### Only tear down the surface a destroy names — branch `fix/android`, [#24](https://github.com/longbridge/gpui-mobile/pull/24)

On the host-driven entry point, a recreated Activity's new surface can
arrive before the old Activity's `surfaceDestroyed`; the render thread then
tore down the new surface (black screen). `surface_destroyed_for` only tears
down the surface that is attached.

### Expose GPUI's accessibility tree to TalkBack — branch `android-accessibility`, [#25](https://github.com/longbridge/gpui-mobile/pull/25)

The Android window ignored GPUI's AccessKit tree, so TalkBack saw one opaque
surface. The tree goes to `accesskit_android`'s `InjectingAdapter`.
[Bounds](docs/demos/android-a11y-bounds.png),
[recording](docs/demos/android-a11y-talkback.gif).

### Keep focus when the user hides the keyboard — branch `android-ime-dismiss`

Hiding the IME with back or its Done action injected an `escape` keystroke,
which apps also use for navigation: hiding the keyboard in a dialog closed
the dialog. The keyboard is now hidden and focus left alone, as with an
EditText. `show_soft_keyboard` / `hide_soft_keyboard` are implemented, so
`Window::request_virtual_keyboard` works (Kit's Input uses it below).

### Give the host-driven path the Activity's AssetManager — branch `android-host-appearance-emoji`

Without `android-activity` there was no AssetManager, so the window never
followed night mode (and `Platform::window_appearance` always said Dark),
and the bundled CBDT emoji font could not be read (Android 13+'s COLRv1
emoji font cannot be drawn by swash). `set_host_activity` now keeps the
Activity's AssetManager; the host path syncs night mode on first open,
re-attach and resume, and through the new `host::configuration_changed()`
for Activities that handle `uiMode` themselves.

## GPUI Kit

### Open a context menu with a long press — [#3393](https://github.com/longbridge/gpui-kit/pull/3393) (issue [#3392](https://github.com/longbridge/gpui-kit/issues/3392))

`ContextMenu` only opened on the right mouse button, which a finger does not
have. A `LongPressEvent` now opens it at the press position; an `Input`
inside the trigger keeps its own long-press selection.
[Before](docs/demos/pr-longpress-before.gif),
[after](docs/demos/pr-longpress-after.gif).
On `mobile-lab` this is the 0.7.x version of the change; the PR is against
`main`, whose context menu has since been reworked.

### Ask for the virtual keyboard when a focused input is tapped — branch `input-tap-requests-keyboard`

After the user hides the keyboard, the input keeps focus, and a later tap
does not change focus, so nothing asked for the keyboard again. A touch
press on a focused input calls `Window::request_virtual_keyboard`.

### Ask scroll containers to reveal a focused input — branch `input-reveal-on-focus`

When the IME opens, `adjustResize` shrinks the viewport, but nothing scrolled
the focused input back into view. A focused input calls
`Window::request_autoscroll` for its text area (the caret line in a
textarea) when it gains focus or the viewport size changes. Needs the GPUI
autoscroll fix below to work in plain scroll containers.

## GPUI (gpui-pre 0.3.8)

Each was checked against Zed `main`: none of the three is fixed there. The
touch-drag patch applies to Zed `main` as is; the files the other two touch
are unchanged there.

### Let touch drag elements that only handle mouse drags — branch `touch-drag`

A finger drag only reached an element as a `TouchDragEvent` it claimed
itself, otherwise it scrolled; everything built on `on_drag` (Resizable,
Dock splitters and tabs, DataTable column resize/move, List reorder) was
dead on touch. An `on_drag` element now marks a touch drag starting on it;
once the touch leaves the slop across the axes its scroll containers can
scroll (or after an unclaimed long press) it becomes a drag, replayed
through the mouse drag path, so `on_drag_move`, `on_drop` and drag-over
styles work unchanged. Handles drag directly; rows of a scrolling list
scroll, and drag after a long press or a sideways move. Mouse input is
unchanged.

### Reveal a descendant's autoscroll request in scrollable divs — branch `autoscroll`

`Window::request_autoscroll` was only honored by `List`. A scrollable div
now takes its descendants' request and scrolls by the smallest amount that
reveals it (applied on the next frame), passing what it cannot reveal to
outer scroll containers.

### Don't remap touch pans onto a container's other axis — branch `touch-axis`

A vertical-only container turned a horizontal pan into vertical scrolling
unless it set `restrict_scroll_to_axis()` — a mouse-wheel convenience that
is wrong for a finger. Scrolls dispatched from a touch gesture (including
fling momentum) are no longer remapped; wheel input is unchanged.

## Still handled in the host app

These belong to the embedding Activity, not to the framework:

- The `SurfaceView`, lifecycle, touch/key forwarding and the IME
  `InputConnection` proxy (`LabActivity`, `src/host.rs`), as gpui-mobile's
  host entry point expects.
- Back navigation: back arrives as `escape`, which the shell binds to
  closing the open screen.
