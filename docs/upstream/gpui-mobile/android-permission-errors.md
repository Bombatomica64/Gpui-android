# android: Report a missing permission instead of an empty result

- Branch: [`upstream/android-permission-errors`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-permission-errors), one commit on upstream `main` (f9fe5a7)
- Diff: 4 files, +41 −42 (two Java helpers, doc comments in two Rust modules)
- Depends on: nothing. (Lab issue Bombatomica64/Gpui-android#46. On our fork it sits on the JNI rework from #31, but it doesn't need it.)
- Recording: none needed. The result is a return value; the before/after values measured on the device are in the Testing section (Gpui-android#46).

---

The calendar and location packages return an empty result when the app lacks the permission, so an app can't tell "no calendars" from "not allowed":

- `GpuiCalendar` catches every exception. Without `READ_CALENDAR`, `get_calendars` and `get_events` return `Ok([])`, and `create_event` fails without a reason.
- `GpuiLocation` swallows `SecurityException`. `get_last_known_position` returns `Ok(None)`, and `get_current_position` fails with the same message for a denied permission, location services being off, and a timeout.

### Change

These exceptions now reach Rust as errors that say what went wrong, as the contacts package already does. The Rust docs of `calendar::get_calendars`, `location::get_current_position` and `location::get_last_known_position` say when they fail. `get_current_position` also notes that it blocks for up to 30 s.

No signature changes. Behaviour change: calls that used to return an empty `Ok` without the permission now return `Err`.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets` and `cargo clippy --target aarch64-linux-android`: no new warnings; `cargo test --lib`: 46 passed.
- The Java helpers are the same as on our fork, whose CI builds the example APK.
- On device (OnePlus CPH2581, Android 16, our fork's `main`), without the permissions:
  - before: `get_calendars` → `Ok([])`, `get_last_known_position` → `Ok(None)`, `get_current_position` → `Err("Failed to get current position")`
  - after: `get_calendars` → `Err("java.lang.SecurityException: Permission Denial: opening provider com.android.providers.calendar.CalendarProvider2 … requires android.permission.READ_CALENDAR …")`, and both location calls → `Err("java.lang.SecurityException: Location needs ACCESS_FINE_LOCATION or ACCESS_COARSE_LOCATION")`

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
