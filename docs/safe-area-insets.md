# Safe-area insets on the host-driven path: what it would take

Status: done on the fork. Steps 1–2 are gpui-mobile branch
`android-host-insets` (draft upstream PR in
[upstream/safe-area-insets.md](upstream/safe-area-insets.md)); steps 3–5 are
in the lab. Two choices differ from the plan below:

- The keyboard is passed in as a fifth value (`ime`) and feeds
  `gpui_mobile::keyboard_height()`, which iOS already fills. The shell pads
  its bottom by `max(navigation bar, keyboard)`. Shrinking the `SurfaceView`
  in Java would have rebuilt the swapchain on every keyboard toggle, and
  apps would get a different contract on Android than on iOS.
- `safe_area_insets()` reads the active window from the platform, not a
  host-path global, so the `android-activity` path gets it as well.

The original plan follows.

## Where things stand

- `AndroidWindow` already stores `SafeAreaInsets` (physical px) and has
  `safe_area_insets_logical()`. Only the `android-activity` path fills it,
  from `content_rect` on `WindowResized` / `ContentRectChanged`.
- The host-driven path (`android::host`) has no way to pass insets in, so
  they stay zero.
- `gpui_mobile::safe_area_insets()`, the cross-platform accessor, reads
  the iOS window and returns zeros on Android.
- GPUI itself has no safe-area concept: apps cannot get insets from a
  `Window`.
- The lab targets SDK 34 and is not edge-to-edge, so its `SurfaceView`
  already sits between the system bars and the real insets are zero. The
  lab stays on 34 because SDK 35 forces edge-to-edge, after which
  `adjustResize` no longer shrinks the window for the keyboard. Google Play
  requires SDK 35 for updates, so hosts will have to face this anyway.

## Steps

1. **gpui-mobile: a host entry point.** Add
   `host::insets_changed(host, top, bottom, left, right)` in physical
   pixels. It posts a command and the render thread stores the insets on that
   host's window, as the `android-activity` path does. Then refresh the
   window so views that read the insets re-render, because an inset change
   (for example the status bar hiding) does not always come with a resize.
   About 40 lines, in the style of `configuration_changed()`.
2. **gpui-mobile: make the accessor work on Android.** Make
   `gpui_mobile::safe_area_insets()` return the attached window's logical
   insets, so the same call works on iOS and Android. A per-`Window` API
   would need a GPUI change. With one active window per host (#26), the
   global accessor is enough to start with.
3. **Host (Java).** Use `View.setOnApplyWindowInsetsListener` on the
   `SurfaceView`, reading
   `insets.getInsets(Type.systemBars() | Type.displayCutout())` on API 30+
   and falling back to `getSystemWindowInset*()` plus `getDisplayCutout()`
   below that. Call `nativeInsets(hostId, …)` from it. Go edge-to-edge with
   `getWindow().setDecorFitsSystemWindows(false)` (or target SDK 35).
4. **Keyboard under edge-to-edge.** Once `adjustResize` stops resizing, the
   IME inset has to be handled explicitly: either pad the `SurfaceView` by
   `Type.ime()` in Java (no GPUI change, matches today's behaviour), or pass
   it as a fifth inset so GPUI can animate with it. The first is the smaller
   step.
5. **Lab demo.** A screen that pads its content by the insets and tints the
   areas underneath the bars. Check it on the OnePlus (punch-hole cutout) in
   portrait and landscape, with gesture and 3-button navigation.

Steps 1–2 make one small upstream PR. Steps 3–5 are lab work, needed to show
the PR working on a device.
