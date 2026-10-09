# Platform packages (`gpui_mobile::packages::*`)

Each package is a Cargo feature of `gpui-pre-mobile` with the package's name
(`share`, `clipboard`, `image_picker`…). Functions return
`Result<_, String>`. Package docs live in the fork at
`src/packages/<name>/mod.rs`; the lab's "Platform APIs" screen
(`src/screens/platform_apis.rs`) calls each one the way an app would and
logs results, timings and GPUI-thread stalls. Use it to check behaviour on a
device.

## Threading

- **Calls that wait for the user** (pickers, permission requests that show a
  dialog, `local_auth` biometric prompt) block until the user answers. From
  GPUI's thread or the Android UI thread they return an error ("call it from
  a background thread") instead of freezing the app. Call them like this:
  ```rust
  use gpui_mobile::packages::file_selector::{open_file, OpenFileOptions};

  cx.spawn(async move |this, cx| {
      let picked = cx
          .background_executor()
          .spawn(async move { open_file(&OpenFileOptions::default()) })
          .await; // Result<Option<SelectedFile>, String>; Ok(None) = cancelled
      this.update(cx, |this, cx| { this.picked = Some(picked); cx.notify(); })
  })
  .detach();
  ```
  Check the option types and names in the package's `mod.rs`.
- Quick calls (clipboard, launching share, `can_launch_url`, showing a
  notification) are fine inline in a click handler.
- Callbacks from Android (deep links, media session buttons) run on the main
  Looper, outside package locks. Hop back to GPUI with a channel or
  `AsyncApp` rather than touching entities from them.

## Catalog

| Package | Use | Needs |
|---|---|---|
| `clipboard` | `set_text`, `get_text() -> Option<String>`, `has_text` | — (Kit's `cx.write_to_clipboard` also reaches the system clipboard) |
| `share` | `share_text(text, Option<subject>)`; `share_uri(uri)` just shares the URI **as text**. Both return once the sheet is launched. There is no file/image sharing yet: that needs a new `share_file` in the fork (FileProvider URI as `EXTRA_STREAM` with a read grant) | — |
| `url_launcher` | `launch_url(&str) -> bool`, `can_launch_url` | `<queries>` VIEW intent per scheme (https, mailto…) for `can_launch_url` on API 30+, otherwise it says false |
| `maps_launcher` | open a geo intent; `is_available` | `geo` in `<queries>` |
| `file_selector` | SAF picker; returns files copied into the cache under their real names | background thread; `GpuiPickerActivity` |
| `image_picker` | gallery (system photo picker on API 33+, several items, optional max size) and camera | background thread; for camera: `CAMERA` permission and the FileProvider below |
| `permission_handler` | check/request permissions | background thread for requests; the permission must be in the manifest |
| `local_auth` | biometric prompt | background thread; `GpuiAuthActivity`, `androidx.biometric`, `USE_BIOMETRIC` |
| `notifications` | `initialize`, `show`, `cancel`, `cancel_all`, `take_launch_payload` | `POST_NOTIFICATIONS` granted on Android 13+ (else `show` errors); for the tap payload, `singleTask`/`singleTop` and `setIntent` in `onNewIntent` |
| `deeplink` | `set_deep_link_handler`, `handle_link` (from `onNewIntent`), `get_initial_link`, `get_latest_link` | VIEW intent-filter for your scheme |
| `calendar`, `contacts` | read/write events, read contacts | `READ_/WRITE_CALENDAR`, `READ_CONTACTS`; missing permission is an error, not an empty list |
| `location` | service enabled, last known, current position | `ACCESS_FINE/COARSE_LOCATION`; missing permission is an error |
| `audio` | MediaPlayer; streams prepare asynchronously (`set_url` returns fast) | — |
| `media_session` | system media controls and notification | `androidx.media` |
| `microphone` | recording | `RECORD_AUDIO` |
| `battery`, `connectivity`, `device_info`, `network_info`, `package_info`, `path_provider`, `shared_preferences`, `sensors`, `vibration`, `in_app_review` | device info and utilities | `ACCESS_NETWORK_STATE`/`ACCESS_WIFI_STATE` for the network ones, `VIBRATE`, a Play Store `<queries>` entry for `in_app_review` |
| `camera`, `video_player`, `webview`, `maps` | native platform views | untested in the lab; see the fork's `example/`. The video player still prepares synchronously. |

## Manifest pieces

FileProvider for camera capture and shared files:

```xml
<provider
    android:name="androidx.core.content.FileProvider"
    android:authorities="${applicationId}.gpui.fileprovider"
    android:exported="false"
    android:grantUriPermissions="true">
    <meta-data android:name="android.support.FILE_PROVIDER_PATHS"
               android:resource="@xml/gpui_file_paths" />
</provider>
```

`res/xml/gpui_file_paths.xml`:
`<paths><cache-path name="gpui_picked" path="gpui-picked/" /></paths>`

Helper Activities (non-exported, `@android:style/Theme.Translucent.NoTitleBar`):
`dev.gpui.mobile.GpuiPickerActivity`, `GpuiPermissionActivity`,
`GpuiAuthActivity`.

Packages with a `Gpui*.java` helper in the fork's
`example/android/gradle/app/src/main/java/dev/gpui/mobile/` need it compiled
into the APK: add it to the copy loop in `build.sh` when you enable the
package. `clipboard`, `deeplink` and `in_app_review` are pure Rust JNI and
need no helper (the lab's checked-in `GpuiClipboard.java` is a leftover).

## Calling Java yourself

For an API no package covers, use the JNI helpers from
`gpui_mobile::android::jni`:

```rust
use gpui_mobile::android::jni as mobile_jni;

mobile_jni::with_env(|env| {
    let activity = mobile_jni::activity(env)?;
    env.call_method(
        &activity,
        jni::jni_str!("moveTaskToBack"),
        jni::jni_sig!("(Z)Z"),
        &[jni::objects::JValue::Bool(true)],
    )
    .map_err(|e| {
        env.exception_clear();
        e.to_string()
    })?;
    Ok(())
})
```

- `with_env` attaches the thread (for good) and runs the closure in its own
  local frame. An exception still pending when the closure returns is
  cleared and becomes the `Err`. But while it's pending, every further JNI
  call in the closure fails, so clear it where it happens (as above) when
  you want to handle the failure.
- `activity(env)`: newest live host Activity as a local ref. Don't keep it
  past the closure.
- `application_context(env)`: prefer it when any `Context` will do.
- `find_app_class(env, "com.example.Foo")` (dot notation): app classes via
  the app's class loader, cached.
- `run_on_ui_thread(|env| …)`: for View APIs that must run on the Android
  UI thread.
- Use jni 0.22 (`jni_str!`, `jni_sig!`).
