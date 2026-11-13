# What the forks carry on top of upstream

The lab builds against one GPUI snapshot (`gpui-pre` 0.3.12) and three forks,
each pinned by `rev` in `Cargo.toml`. The forks are the source of truth: fixes
land on the branch the lab pins, and proposing them upstream is optional. The
per-fix branches named below predate the 0.3.12 rebase and stay on their old
bases; the combined branch is the current version of each fix.

| Fork | Upstream | Branch the lab pins |
|---|---|---|
| [Bombatomica64/gpui-mobile](https://github.com/Bombatomica64/gpui-mobile) | [longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) `main` | `main` (upstream `main` + the branches below, merged) |
| [Bombatomica64/gpui-kit](https://github.com/Bombatomica64/gpui-kit) | [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit) tag `v0.9.1` | `mobile-lab` (`v0.9.1` + the commits below) |
| [Bombatomica64/gpui-pre](https://github.com/Bombatomica64/gpui-pre) | `gpui-pre` 0.3.12 from crates.io (a snapshot of [zed-industries/zed](https://github.com/zed-industries/zed) `crates/gpui`) | `mobile-lab` (branch `0.3.12` + the commits below) |

Versions of these branches rebased onto upstream `main`, with draft PR
texts, are in [docs/upstream](docs/upstream/README.md) (gpui-mobile, Kit)
and [docs/upstream-zed](docs/upstream-zed/README.md) (GPUI).

`gpui-pre` replaces the crates.io release through `[patch.crates-io]`;
`gpui-kit`, `gpui-fps` and `gpui-pre-mobile` are git dependencies.

Fixes from earlier versions of this file that are now upstream: the
gpui-pre 0.3.7, 0.3.8 and 0.3.9 bumps (gpui-mobile #21, #27, #36), the system
clipboard and the surface-teardown fix (gpui-mobile #24, reworked on top of
#26's per-Activity `HostId`), the TalkBack bridge (#25, with the
exception-clearing fix folded in), the emoji font (#29) and night mode (#30)
on the host path, Slider touch drags and the gpui-fps Android font (gpui-kit
#3313, in v0.7.1), long-press context menus (gpui-kit #3393, in v0.7.2), and
the input keyboard configuration, names for the clear and password buttons,
the password role and back keeping the text (gpui-kit #3554–#3557, in
v0.8.0); tapping a focused input asks for the keyboard (gpui-kit #3623, v0.9.0
or later), a focused input asks scroll containers to reveal it (#3625), and
sheets and toasts stay inside the visible area, all in Kit v0.9.1. gpui-mobile
#45 (gpui-pre 0.3.10, Android host fixes) is merged into the fork's `main`, and
the fork carries #46's 0.3.11 bump plus 0.3.12.

Two things the merged versions changed for hosts: the TalkBack adapter goes
on each host's own Activity, so hosts call `host::set_activity(env, host,
activity)`; and night mode is read from the application context, so an
Activity that handles `uiMode` itself calls
`host::configuration_changed(newConfig.uiMode)`, and an app-local night mode
(`AppCompatDelegate`) does not count.

The commits the lab pinned before the 0.3.9 rebase are kept as tags:
`archive/pre-0.3.9/main` (gpui-mobile), `archive/v0.7.1/mobile-lab` and
`archive/v0.7.3/mobile-lab` (Kit), and
`archive/0.3.8/*` (gpui-pre). The commits pinned before the 0.3.12 / v0.9.1
rebase are `archive/0.3.9/mobile-lab` (gpui-pre) and
`archive/v0.8.0/mobile-lab` (Kit).

## gpui-mobile

### Keep focus when the IME's Done action hides the keyboard — branch `android-ime-dismiss`

Upstream (#26) stopped back from injecting an `escape` keystroke (IME event
5), but Done (event 4) still did, and apps use escape for navigation: Done
in a dialog closed the dialog. Done now hides the keyboard and keeps focus,
like back. `show_soft_keyboard` / `hide_soft_keyboard` are implemented, so
`Window::request_virtual_keyboard` works (Kit's Input uses it below).

### Give each field its own keyboard and enter key — branches `android-text-input-configuration`, `android-text-input-purpose`

The host's IME proxy always asked for a multi-line text field: password
fields showed the typed password among the keyboard's suggestions (and the
keyboard learned it), number and phone fields got the full keyboard, and
single-line fields had a newline key that did nothing. The window now
implements `set_text_input_configuration` and turns GPUI's
`TextInputConfiguration` (enter-key action, autocorrect, suggestions,
autocapitalization, and the purpose from the GPUI branch below) into
`EditorInfo` `inputType`/`imeOptions`, handed to the host's new
`gpuiShowKeyboardWithInputType` (older hosts keep `gpuiShowKeyboard`). A
field with an action is single-line; the host reports the action key as IME
event 6, which the field receives as its enter key, and Done, Go, Search and
Send then hide the keyboard. The purpose half needs the GPUI branch.
[Password keyboard before/after](docs/demos/android-password-keyboard.png),
[amount keyboard](docs/demos/android-amount-keyboard.png),
[Search key, Enter](docs/demos/android-ime-actions.png).

### Let host-driven apps report safe-area and keyboard insets — branch `android-host-insets`

The host path had no way to pass insets in, so an edge-to-edge host could
not tell GPUI where the system bars, the cutout and the keyboard are: the
Android window returned GPUI's default, zero, `PlatformWindow::insets`.
`host::insets_changed(host, SafeAreaInsets, ime)` takes them in physical
pixels. The render thread stores them on that host's window, or keeps them
until the window opens. The window implements `insets()` and
`on_insets_changed`, so apps read them through GPUI
(`Window::fully_visible_bounds`) and GPUI refreshes the window when they
change. `safe_area_insets()` and `keyboard_height()` return them too, as on
iOS. LabActivity calls it from its `OnApplyWindowInsetsListener`.
Draft upstream PR: [docs/upstream/safe-area-insets.md](docs/upstream/safe-area-insets.md).

### Back as its own event, with predictive back — branch `feat/predictive-back`

The host turned Back into `escape`, so an app could not tell "go back" from a
key, and Android 14's predictive back gesture had no way in. Hosts now report
Back with `host::back_event` (from an `OnBackAnimationCallback` on Android 14+,
an `OnBackInvokedCallback` on 13, `KEYCODE_BACK` before), as
`BackEvent::{Started, Progressed, Cancelled, Invoked}` with the gesture's
progress and edge; apps take it with `host::on_back`. `host::set_back_enabled`
calls the host's `gpuiSetBackEnabled(boolean)`, so the callback is registered
only while the app has somewhere to go back to and the system keeps Back (and
its back-to-home animation) otherwise. Without an `on_back` handler a committed
Back still reaches GPUI as `escape`. The lab's stack navigation uses it
([#27](https://github.com/Bombatomica64/Gpui-android/issues/27)).
Merged into the fork's `main` as gpui-mobile #20.

### Load a pushed debug library without reinstalling — branch `dev-hot-loader`

`GpuiLibraryLoader.load(context, name)`, a Java helper next to the others in
the example project: in a debuggable build it loads `files/dev/lib<name>.so`
when a dev tool ([cargo-gpui](https://github.com/Bombatomica64/cargo-gpui))
has pushed one, else the packaged library. Release builds always load the
packaged one. It replaces the `loadLibrary()` the lab's `LabActivity`
carried; the lab keeps a copy in `android/app/src/main/java` (`LabActivity`
needs it to compile, also in `cargo gpui apk`, which skips `build.sh`). Host-driven Activities
only (`NativeActivity` loads the packaged library itself). Proposed as
[longbridge/gpui-mobile#39](https://github.com/longbridge/gpui-mobile/pull/39),
for the question in [#34](https://github.com/longbridge/gpui-mobile/issues/34)
([text](docs/upstream/hot-reload-discussion.md)).

### Rework the JNI layer — branch `jni-refactor`

Host Activity lifetime, pending Java exceptions, UI-thread calls and class
lookup, audited in gpui-mobile issue
[#31](https://github.com/longbridge/gpui-mobile/issues/31) and proposed there
as eight small PRs. The first two are open, ported onto upstream `main`:
[#40](https://github.com/longbridge/gpui-mobile/pull/40) (the `with_env` docs,
SAFETY comments) and [#41](https://github.com/longbridge/gpui-mobile/pull/41)
(`or_clear` / `or_catch` in the core files).

### Report clickable nodes to accessibility services — branch `a11y-clickable`

accesskit_android 0.7.5 offers `ACTION_CLICK` but never calls
`setClickable`, so every node, buttons included, reported `clickable=false`
to UI Automator and to services that read the flag. The injecting adapter
marks a node clickable when it offers the click
([#39](https://github.com/Bombatomica64/Gpui-android/issues/39)).

### Make the platform packages behave — seven branches, merged as gpui-mobile #2–#8

Each branch is based on fork `main` at `4dd00ad` (two are stacked), so it
builds on the JNI rework proposed upstream in gpui-mobile issue #31. The
Platform APIs screen exercises them; the defects are #44–#50.

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
- `android-video-async-prepare` (#50): video streams too (`set_url` returns
  `None` while loading, `state()` reports `Loading`). The Video screen
  exercises it through the platform-view host.
- `android-player-completed` (gpui-mobile #22, for
  [#88](https://github.com/Bombatomica64/Gpui-android/issues/88)): a clip that
  plays to the end reports `Completed`, not `Paused`. Audio and video record
  `OnCompletionListener` under the async-prepare lock and clear it on play,
  seek, stop and a new source. `LoopMode::One` and `All` both repeat the one
  source, since a player has no playlist.

### Platform views: maps, web view controls, camera preview, Activity recreation — gpui-mobile #21, #23, #24

From the JNI audit (#26, #87). The platform-view host kept its views in the
first Activity it saw. Now `host::set_activity` calls
`GpuiPlatformView.attachToActivity`, which moves every view into the new
Activity; the views are built on a `MutableContextWrapper` that switches to it.

`platform_view_element` is a real element. It places the view, clips it to the
scroll area, and hides it when the element isn't painted. Native views draw
above GPUI, so an app hides them under a GPUI overlay with `set_visible(false)`.

What changed per package:
- **Web view:** the controls act on the platform view's own `WebView`. Before,
  they acted on a static `WebView` that nothing set. New calls: `navigate`,
  `go_forward`, `state()`, and `evaluate_javascript_for_result`, which returns
  the script's result through a native callback.
- **Maps:** the log-only Android stubs are replaced by `GpuiMapView`, an
  OpenStreetMap tile map with no SDK and no API key. Maps are drawn by a
  `MapProvider` (gpui-mobile #24). `TileMapProvider` is the default:
  OpenStreetMap, or any `{z}/{x}/{y}` tile server. Apps can plug in another
  SDK by implementing `MapProvider` and `MapController` and registering their
  view with `GpuiPlatformView.registerViewType`.
- **Camera:**
  - the preview is removed on the UI thread;
  - photos and videos are written to the app cache instead of public storage;
  - output is tagged with the display rotation;
  - the preview is cropped to fill its view instead of stretched.

The Native Views screen exercises all of this. `LabActivity` gives keys to a
focused native view (a WebView field), takes focus back on a GPUI touch, and
adds `gpuiRecreate` for the screen's "Recreate Activity" button.

The Java helpers live in gpui-mobile's example project; `build.sh` copies the
ones the lab uses from the pinned checkout.

## GPUI Kit

### Say what an input holds — branch `input-purpose`

Kit v0.8.0 has `input_action`, `autocorrect` and `autocapitalize`
([#3557](https://github.com/longbridge/gpui-kit/pull/3557)); the review
made text assistance opt-in, so autocorrect, suggestions and caps are off
unless an input asks for them. This branch adds the purpose on top:
Password for a masked input (still while revealed), Numeric/Decimal for
number and digits-only masks, and the purpose the `Input`'s `content_type`
implies (email, phone, URL, password), with `input_purpose` to override it,
like React Native's `keyboardType` and `secureTextEntry`. It needs the GPUI
branch below.

### Report a disabled input as disabled — branch `input-a11y-disabled`

Every other control marks its accessibility node disabled through
`A11yDisabledExt`; `InputBase` only styled itself, so a disabled `Input`
reached TalkBack as `enabled=true`. It now wraps its root the same way
([#39](https://github.com/Bombatomica64/Gpui-android/issues/39)).

### Resize and move table columns by touch — branch `table-touch-drag`

In a DataTable wider than the screen, a finger could neither resize nor
move a column: the resize handle's drag became a sideways scroll of the
header, and the rows' context menu, which covered the header too, took the
long press that should pick a column up (it opened an empty menu). Resize
handles are drag handles now (GPUI's `drag_handle`, below), the rows'
context menu covers only the body, and `ResizeHandle` (Resizable panels,
Dock edges) drags at once too
([#28](https://github.com/Bombatomica64/Gpui-android/issues/28)).

## GPUI (gpui-pre 0.3.12)

None of these is in 0.3.12. Kit's input reveal, now upstream, still needs the
autoscroll patch to work in plain scroll containers.

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

### Let drag handles drag at once by touch — branch `drag-handle`

On top of `touch-drag`: an `on_drag` element inside a scroll container
yields a pan along the container's axis to scrolling, which is right for
list rows and wrong for handles. A DataTable column resize handle sits in a
header that scrolls sideways, so a finger could only scroll the header.
`drag_handle()` marks an element whose touch drag starts as soon as it leaves
the slop, along any axis. Kit's table resize handles and `ResizeHandle`
(Resizable, Dock) use it
([#28](https://github.com/Bombatomica64/Gpui-android/issues/28)).

### Recognize two-finger pinches — branch `touch-pinch`

The portable recognizer ignored every touch after the first, so a phone
never produced a `PinchEvent` and `on_pinch` only worked with a trackpad. A
second touch landing while the first is pending or panning now starts a
pinch: the pan ends, and `PinchEvent`s follow at the point between the
fingers, each `delta` the change in their distance since the last step
(`zoom *= 1 + delta`). When either finger lifts the pinch ends; the other
does nothing until it lifts. Touches that claimed a drag or a long press,
and third fingers, are still ignored. No rotation yet: `PinchEvent` has no
angle ([#28](https://github.com/Bombatomica64/Gpui-android/issues/28)).

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

### Say what a text field holds — branch `text-input-purpose`

`TextInputConfiguration` could not say "password" or "number", so no
platform could pick a numeric keyboard or keep a password out of the
keyboard's suggestions. It gains `purpose: TextInputPurpose` (HTML
`inputmode` values plus Password and NumericPassword). Zed `main` has the
same struct without it.

### Report text without an element id to screen readers — branch `a11y-static-text`

Only elements with an element id and a role reach the AccessKit tree, so
plain string children, `SharedString` and `StyledText`, which is how most
headings, paragraphs and value rows are drawn, were invisible to TalkBack
([#37](https://github.com/Bombatomica64/Gpui-android/issues/37)). They now
add a `Label` leaf under their nearest accessible ancestor, unless that
ancestor already speaks for its text (a label, a text field, a button with
`aria_label`). Node ids come from the ancestor and the text's position, so
they are stable across frames. Built on `mobile-lab` (it touches only the
a11y builder and `elements/text.rs`).

### Report focus for a focused element with no node of its own — branch `a11y-focus-ancestor`

GPUI only reported focus for a focused element that had an accessibility
node. Kit's `Input` keeps focus on its editor, which has no role, and puts
the role on the frame, so the tree had no focus and every text field read
`focused=false` while editing. Focus now goes to the nearest ancestor that
has a node and a focus handle (the frame), and only without one to the
window. Tracking the editor's handle on the frame instead registers it as a
tab stop twice, which Kit reverted upstream (longbridge/gpui-kit#3253)
([#39](https://github.com/Bombatomica64/Gpui-android/issues/39)).

## Still handled in the host app

These belong to the embedding Activity, not to the framework:

- The `SurfaceView`, lifecycle, touch/key forwarding and the IME
  `InputConnection` proxy (`LabActivity`, `src/host.rs`), as gpui-mobile's
  host entry point expects.
- Back navigation: the screen stack and its transitions (`src/app.rs`,
  `src/nav.rs`), and the back callbacks in `LabActivity`. Back reaches the
  lab through gpui-mobile's `host::on_back`; the lab offers it to Kit
  overlays as `escape` first (that is how Kit closes them), then pops.
