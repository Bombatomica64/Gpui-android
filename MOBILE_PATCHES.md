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
gpui-pre 0.3.7 and 0.3.8 bumps (gpui-mobile #21, #27), the system clipboard
and the surface-teardown fix (gpui-mobile #24, reworked on top of #26's
per-Activity `HostId`), Slider touch drags and the gpui-fps Android font
(gpui-kit #3313, in v0.7.1).

## gpui-mobile

### Expose GPUI's accessibility tree to TalkBack — branch `android-accessibility`, [#25](https://github.com/longbridge/gpui-mobile/pull/25)

The Android window ignored GPUI's AccessKit tree, so TalkBack saw one opaque
surface. The tree goes to `accesskit_android`'s `InjectingAdapter`. Since
upstream gave each host Activity its own window (#26), the screen reader
follows whichever window is active.
[Bounds](docs/demos/android-a11y-bounds.png),
[recording](docs/demos/android-a11y-talkback.gif).

### Keep focus when the IME's Done action hides the keyboard — branch `android-ime-dismiss`

Upstream (#26) stopped back from injecting an `escape` keystroke (IME event
5), but Done (event 4) still did, and apps use escape for navigation: Done
in a dialog closed the dialog. Done now hides the keyboard and keeps focus,
like back. `show_soft_keyboard` / `hide_soft_keyboard` are implemented, so
`Window::request_virtual_keyboard` works (Kit's Input uses it below).

### Follow the system night mode on the host path — branch `android-host-night-mode`, [#30](https://github.com/longbridge/gpui-mobile/pull/30)

Without `android-activity`, night mode was never read, so the window stayed
Light (and `Platform::window_appearance` always said Dark). It is now read
from the application context's configuration, which the first
`set_host_activity` keeps, and applied to each window when it opens,
re-attaches and resumes. An Activity that handles `uiMode` itself must call
the new `host::configuration_changed()`, which updates every host's window.

### Load the bundled emoji font on the host path — branch `android-host-emoji-font`, [#29](https://github.com/longbridge/gpui-mobile/pull/29)

Android 13+'s COLRv1 emoji font cannot be drawn by swash, and the bundled
CBDT fallback was only read through `android-activity`'s AssetManager, so
emoji were empty boxes. The first `set_host_activity` keeps the
application's AssetManager for the life of the process.

Both were one branch, `android-host-appearance-emoji`, until an adversarial
review: night mode came from whichever Activity registered last, and the
public `asset_manager()` could return a dangling pointer.

### Platform packages (Android) — branch `platform-api-lab`

The lab pins `platform-api-lab`, which merges seven branches, each based on
fork `main` (two are stacked). The Platform APIs screen exercises them; the
defects are #44–#50.

- `android-activity-requests` (#44): pickers, permission requests and the
  biometric prompt no longer wait forever on a static latch when their helper
  Activity goes away. Calls that wait for the user fail on GPUI's or the UI
  thread instead of freezing it; call them from a background thread.
- `android-picker-files` (#45, on top of the previous): pickers return files
  copied into the cache with their display name; camera capture works (the
  host declares a FileProvider); photo picker on API 33+; size limits apply.
- `android-permission-errors` (#46): a missing calendar or location
  permission is an error, not an empty result.
- `android-string-arrays` (#47, on top of the previous): calendar and
  contacts cross JNI as `String[]`, so `|` and newlines survive.
- `android-callbacks` (#48): `deeplink::handle_link` for a host's
  `onNewIntent`; handlers run outside their lock; media-session callbacks
  on the main Looper.
- `android-notifications-links` (#49): `show` fails when notifications are
  blocked; a tap opens the app with `take_launch_payload`; `<queries>`
  documented for `can_launch_url`.
- `android-async-prepare` (#50): audio streams prepare asynchronously.

The Java helpers live in gpui-mobile's example project; `build.sh` copies the
ones the lab uses from the pinned checkout.

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
