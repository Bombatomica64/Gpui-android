# Reply to huacnlee's review of longbridge/gpui-mobile#30

Draft, to be posted by hand. The review (12 Oct, CHANGES_REQUESTED) was on
c62c318. The force-push later that day to
[ec5c89d](https://github.com/longbridge/gpui-mobile/pull/30/commits/ec5c89dfe3ab399bcc90a01f03cf89873cb750b2)
only rebased the PR onto main after #29 merged: `HOST_ASSETS` now gets its
AssetManager from the `HOST_APPLICATION` context this PR keeps, instead of
calling `getApplicationContext()` a second time. `host.rs` and `platform.rs` are
unchanged apart from line offsets. **None of the review points are addressed
yet.** So this reply says that and lays out the plan. It should be rewritten
once the fixes are pushed.

---

Thanks for the careful review! One thing to clear up first: the force-push on
12 Oct was only a rebase onto main after #29 merged. It now shares the
application context that #29's `HOST_ASSETS` needs, and nothing else changed.
It does not address your points yet. Sorry it looked like a response. Here is
what I plan to do for each one:

1. **android-activity path starts Light.** You're right. `window_appearance()`
   now reports the real mode, but nothing on the `InitWindow` path sets the
   window's own appearance, so the two disagree until a `ConfigChanged`. Fixed
   by item 2.
2. **Sync in the shared code.** I'll move the initial `sync_appearance` into
   `AndroidPlatform::open_window` and after `init_window`, so both entry points
   go through it, and remove the call-site copies in `host.rs` that this makes
   redundant. I'll keep the one in `resumed`.
3. **App-local night mode.** Agreed. I'll read `uiMode` from the current host
   Activity's Resources instead of the application context, so
   `AppCompatDelegate.setDefaultNightMode` is respected. `HOST_APPLICATION`
   stays only for the AssetManager.
4. **Order of config changes.** I'll change the signature to
   `host::configuration_changed(ui_mode: i32)` and have the host pass
   `newConfig.uiMode`, so nothing re-reads a configuration that may not be
   updated yet. The host docs will show the call.

On the non-blocking points, I'll do all three in the same push:

- `window_appearance()` will return the value last stored by `sync_appearance`
  instead of querying over JNI or building an `AConfiguration` on every call.
- The `ConfigurationChanged` handler will work out the appearance once and call
  `set_appearance` on each window.
- A failed `host_ui_mode` will log a `warn!` instead of quietly meaning Light.
  (Today only a failed `getApplicationContext()` in `set_host_activity` logs.)

I'll reply again with the commit once it's pushed. None of this has been
verified on a device yet. I'll test a cold start in dark mode on both paths, and
an in-app night-mode toggle, before asking for another look.
