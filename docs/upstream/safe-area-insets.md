# Draft: longbridge/gpui-mobile — safe-area and keyboard insets on the host path

Not posted. Branch
[`Bombatomica64/gpui-mobile:android-host-insets`](https://github.com/Bombatomica64/gpui-mobile/tree/android-host-insets),
one commit on upstream `main` (f9fe5a7), 4 files, +95 −6. It applies to
upstream as is; on our fork `main` it only conflicts next to
`configuration_changed` (#30).

---

**Title:** android: Let host-driven apps report safe-area and keyboard insets

### What

A host that draws behind the system bars (edge to edge, which SDK 35 makes
the default) has no way to tell GPUI where the bars, the display cutout and
the software keyboard are. `AndroidWindow` already stores `SafeAreaInsets`,
but only the `android-activity` path fills it, and
`gpui_mobile::safe_area_insets()` returns zeros on Android.

This adds one entry point to `android::host`, in the style of the other host
calls:

```rust
pub fn insets_changed(host: HostId, safe_area: SafeAreaInsets, ime: f32)
```

- Physical pixels relative to the surface, from
  `View.OnApplyWindowInsetsListener` (`Type.systemBars() | Type.displayCutout()`
  and `Type.ime()`).
- The render thread stores them on that host's window. If the window does
  not exist yet, they are kept until it opens. When they change, the window
  fires its resize callback so GPUI re-lays out, because an inset change does
  not always come with a resize.
- `safe_area_insets()` returns the active Android window's insets (from
  either entry point), and the attached host's keyboard feeds
  `keyboard_height()`. Apps then use the same two calls on iOS and Android.

A host that keeps its surface between the bars does not call it, and
everything stays zero, as today.

### Host side (Java)

```java
WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
ViewCompat.setOnApplyWindowInsetsListener(surfaceView, (view, insets) -> {
    Insets bars = insets.getInsets(
            WindowInsetsCompat.Type.systemBars() | WindowInsetsCompat.Type.displayCutout());
    int ime = insets.isVisible(WindowInsetsCompat.Type.ime())
            ? insets.getInsets(WindowInsetsCompat.Type.ime()).bottom : 0;
    nativeInsets(hostId, bars.top, bars.bottom, bars.left, bars.right, ime);
    return insets;
});
```

and in Rust:

```rust
host::insets_changed(host, SafeAreaInsets { top, bottom, left, right }, ime);
```

An app pads by them:

```rust
let (top, bottom, left, right) = gpui_mobile::safe_area_insets();
let bottom = bottom.max(gpui_mobile::keyboard_height());
div().size_full().pt(px(top)).pb(px(bottom)).pl(px(left)).pr(px(right))
```

### On a device

OnePlus CPH2581 (Android 16, punch-hole cutout), in the
[GPUI Mobile Lab](https://github.com/Bombatomica64/Gpui-android). The app shell
pads by the insets and paints its title bar color underneath them. The
Diagnostics row shows the values GPUI gets, in logical pixels.

| Portrait | Landscape (cutout on the left) | Keyboard open |
| --- | --- | --- |
| ![portrait](../demos/safe-area-portrait.png) | ![landscape](../demos/safe-area-landscape.png) | ![keyboard](../demos/safe-area-keyboard.png) |

### Not in this PR

- A per-`Window` API. GPUI has no safe-area concept, and with one active
  window per host (#26) the global accessor is enough to start with.
- Animating with the keyboard (`WindowInsetsAnimation`). The inset arrives
  once the keyboard has settled.

Thanks for looking at this.
