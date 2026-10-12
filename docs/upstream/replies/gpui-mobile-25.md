# Reply to huacnlee's review of longbridge/gpui-mobile#25

Draft, to be posted by hand. The branch was rebased onto `main` (it conflicted
with #29 in `jni.rs`) and force-pushed; the fixes are separate commits on top,
head [31d79d8](https://github.com/longbridge/gpui-mobile/pull/25/commits/31d79d8f51ac71eb58bad9fa433e17758002cf96).

---

Thanks for the careful review! I rebased onto `main` (it conflicted with #29)
and added one commit per point:

1. **Wrong Activity after Back** (2b761c1). New `host::set_activity(env, host,
   activity)` does what `set_host_activity` does and also remembers the Activity
   per `HostId`. Each window knows its host, so the adapter goes on that host's
   own Activity. Hosts that only call `set_host_activity` keep the old behaviour,
   and the "What the host has to do" docs now point to `set_activity`.
2. **`isEnabled()` race** (not fully fixed, see below). I don't think
   gpui-mobile can close it on its own. The runnable that raises the events is
   posted by `InjectingAdapter::update_if_active` inside `accesskit_android`,
   and nothing runs between it and `requestSendAccessibilityEvent`. A
   `Looper`-side check could still lose to a queued `MSG_SET_STATE`.
   `accesskit_android` 0.9 fixes this the way you describe: `QueuedEvents::raise`
   checks `isEnabled()` on the UI thread. But 0.9 needs `accesskit` 0.25, and
   gpui-pre 0.3.8 is on 0.24. What changed here:
   - 7006f50 clears the Java exceptions that `accesskit_android`'s jni 0.21
     leaves pending.
   - 31d79d8 makes the GPUI-thread check cheaper and documents the remaining
     window in the module docs.

   The real fix is either a backported `accesskit_android` 0.7.x, which I can
   ask AccessKit for, or moving to 0.9 once gpui-pre is on `accesskit` 0.25.
   Would you merge with this documented, or wait for one of those?
3. **`BRIDGE` not cleared** (d0bea7c). `HostDestroyed` and `Drop for
   AndroidWindow` now drop the bridge, and with it the adapter, when it belongs
   to that window. The closed window's callbacks are not called.
4. **Existing AccessibilityDelegate** (34f8c96). Before creating the adapter,
   `attach` reads `getAccessibilityDelegate()`. If the delegate is not an earlier
   AccessKit one (whose removal is already queued ahead), it logs and skips the
   view.
5. **No retry** (ea97c6d). `init` now attaches again when there is no adapter.
   Since 9418278 it also does so when the window's Activity has changed.
6. **Attach before `setContentView`** (9418278). `set_host_activity` no longer
   touches the adapter. It is attached whenever the window becomes active: on
   resume, and again (as a cheap no-op if nothing changed) when its surface
   arrives.

Non-blocking:

- **`isEnabled()` under the lock** (31d79d8): the `AccessibilityManager` and
  the `JavaVM` are cached once, and the check runs before `BRIDGE` is taken.
  Nothing takes `BRIDGE` on the UI thread any more. An
  `AccessibilityStateChangeListener` would need a Java class, which this crate
  doesn't ship, so I kept the per-frame call. It is a synchronized field read.
- **Two jni versions**: the second one stays as long as we use
  `accesskit_android`, because `InjectingAdapter::new` takes its jni 0.21
  `JNIEnv`. The VM is no longer rebuilt per attach (31d79d8).
- **Hover before `hit_test`** (6d443bd): moved.

Checked: `cargo check` and `cargo clippy --target aarch64-linux-android` (no
warnings in the touched files), `cargo fmt --check`, and `cargo test --lib`
(46 passed). On a device: pending — I will add TalkBack results (toggling it off mid-draw, Back from a second screen) here before posting.
