# Reply to huacnlee's review of longbridge/gpui-mobile#30

Draft, to be posted by hand. The review (12 Oct, CHANGES_REQUESTED) was on
c62c318. The force-push that day (ec5c89d) was only a rebase onto #29. All
review points are addressed in
[fa0075b](https://github.com/longbridge/gpui-mobile/pull/30/commits/fa0075b137437b34daeaf6a0d8d4a33686423384),
pushed on top of ec5c89d as a separate commit.

---

Thanks for the review, and sorry for the silent force-push on the 12th: that
was only a rebase onto #29 and did not address anything. The fixes are in
fa0075b, on top of it:

1. **android-activity path starts Light.** Fixed by item 2. On a cold start in
   dark mode, `cx.window_appearance()` and `window.appearance()` both say Dark
   on the first frame now. Before, the window said Light.
2. **Sync in the shared code.** `AndroidPlatform::open_window` applies the night
   mode, and so does a new `AndroidPlatform::reinit_window`, which both
   `InitWindow` and the host's reattach now use instead of calling
   `init_window` directly. The call-site syncs in `host.rs` are gone. Only
   `resumed` still re-reads it, in case it changed while another app was in
   front.
3. **App-local night mode.** `uiMode` now comes from the current host Activity's
   Resources, so `AppCompatDelegate.setDefaultNightMode` counts. With the
   application context no longer needed for night mode, I dropped the global
   this PR had added. The AssetManager is fetched as on main again.
4. **Order of config changes.** It's now `host::configuration_changed(ui_mode: i32)`,
   and the host passes `newConfig.uiMode`. Nothing re-reads a configuration in
   that path. The module docs show the call.

Non-blocking:

- The platform stores the appearance it last applied, and
  `window_appearance()` just returns it, with no JNI call or `AConfiguration`.
  It has its own small lock rather than living in the platform state, because
  GPUI reads it from main-thread tasks, and `flush_main_thread_tasks()` runs
  those with the state locked. My first version stored it in the state and
  deadlocked on the first night-mode change on the android-activity path.
  The device test below caught it.
- `ConfigurationChanged` reads the night mode once and applies it to every
  window (`AndroidPlatform::set_appearance`).
- A failed `uiMode` read logs a `warn!`. The read also runs in a local frame
  now, since the render thread never returns to Java.

Tested on a OnePlus with Android 16, switching with `cmd uimode night`:

- host path (Bombatomica64/Gpui-android, Diagnostics screen): cold start in
  dark mode, a live toggle to light (the Activity handles `uiMode`), back to
  dark while in the background then resumed, and a light system with the
  Activity forced to dark the way AppCompat does it. All four show the right
  appearance.
- android-activity path (this repo's example, with a log in an
  `observe_window_appearance` callback): a cold start in dark mode logs
  `platform=Dark window=Dark` (ec5c89d logged `platform=Dark window=Light`), and
  toggling light and back fires the observer each time with both values
  agreeing.

![Cold start dark, live toggle to light, app-local dark on a light system](https://raw.githubusercontent.com/Bombatomica64/Gpui-android/main/docs/demos/android-night-mode-review.png)

One API change for hosts: `configuration_changed` now takes the `uiMode`
argument.
