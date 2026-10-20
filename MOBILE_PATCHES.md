# What the forks carry on top of upstream

The lab builds against one GPUI snapshot (`gpui-pre` 0.3.9) and three forks,
each pinned by `rev` in `Cargo.toml`. Every fix lives on its own branch,
based on upstream, so it can be proposed upstream on its own; a combined
branch is what the lab pins.

| Fork | Upstream | Branch the lab pins |
|---|---|---|
| [Bombatomica64/gpui-mobile](https://github.com/Bombatomica64/gpui-mobile) | [longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) `main` | `main` (upstream `main` + the branches below, merged) |
| [Bombatomica64/gpui-kit](https://github.com/Bombatomica64/gpui-kit) | [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit) tag `v0.8.0` | `mobile-lab` (`v0.8.0` + the commits below) |
| [Bombatomica64/gpui-pre](https://github.com/Bombatomica64/gpui-pre) | `gpui-pre` 0.3.9 from crates.io (a snapshot of [zed-industries/zed](https://github.com/zed-industries/zed) `crates/gpui`) | `mobile-lab` (branch `0.3.9` + the branches below) |

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
v0.8.0).

Two things the merged versions changed for hosts: the TalkBack adapter goes
on each host's own Activity, so hosts call `host::set_activity(env, host,
activity)`; and night mode is read from the application context, so an
Activity that handles `uiMode` itself calls
`host::configuration_changed(newConfig.uiMode)`, and an app-local night mode
(`AppCompatDelegate`) does not count.

The commits the lab pinned before the 0.3.9 rebase are kept as tags:
`archive/pre-0.3.9/main` (gpui-mobile), `archive/v0.7.1/mobile-lab` and
`archive/v0.7.3/mobile-lab` (Kit), and
`archive/0.3.8/*` (gpui-pre).

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

The Java helpers live in gpui-mobile's example project; `build.sh` copies the
ones the lab uses from the pinned checkout.

## GPUI Kit

### Ask for the virtual keyboard when a focused input is tapped — branch `input-tap-requests-keyboard`

After the user hides the keyboard, the input keeps focus, and a later tap
does not change focus, so nothing asked for the keyboard again. A touch
press on a focused input calls `Window::request_virtual_keyboard`.

### Ask scroll containers to reveal a focused input — branch `input-reveal-on-focus`

When the IME opens, `adjustResize` shrinks the viewport, but nothing scrolled
the focused input back into view. A focused input calls
`Window::request_autoscroll` for its text area (the caret line in a
textarea) when it gains focus or `Window::fully_visible_bounds()` changes:
the viewport resizing under `adjustResize`, or the keyboard arriving as an
inset on an edge-to-edge window. Needs the GPUI autoscroll fix below to work
in plain scroll containers.

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

### Keep sheets and toasts out from under the system bars — branch `upstream/overlays-safe-area`

On an edge-to-edge window a sheet's title and close button sat under the
status bar, and toasts under the keyboard: both were laid out against the
whole viewport. The sheet host and the notification layer are now inset by
`Window::fully_visible_bounds()`, on top of the frame insets the sheet
already used; on desktop nothing changes. A Kit UI test covers it. Not
proposed yet: it waits for before/after captures on the phone.

## GPUI (gpui-pre 0.3.9)

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

### Say what a text field holds — branch `text-input-purpose`

`TextInputConfiguration` could not say "password" or "number", so no
platform could pick a numeric keyboard or keep a password out of the
keyboard's suggestions. It gains `purpose: TextInputPurpose` (HTML
`inputmode` values plus Password and NumericPassword). Zed `main` has the
same struct without it.

## Still handled in the host app

These belong to the embedding Activity, not to the framework:

- The `SurfaceView`, lifecycle, touch/key forwarding and the IME
  `InputConnection` proxy (`LabActivity`, `src/host.rs`), as gpui-mobile's
  host entry point expects.
- Back navigation: back arrives as `escape`, which the shell binds to
  closing the open screen.
