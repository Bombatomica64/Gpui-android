# android: Document the `<queries>` that `can_launch_url` needs

- Branch: [`upstream/android-package-visibility`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-package-visibility), one commit on upstream `main` (f9fe5a7)
- Diff: 3 files, +25 −1 (docs and the example manifest only)
- Depends on: nothing. This is the links half of fork branch `android-notifications-links` (Bombatomica64/Gpui-android#49).
- Recording: not needed (docs and manifest).

---

On Android 11+ (API 30), package visibility hides other apps from this one. So `url_launcher::can_launch_url("https://…")`, `can_launch_url("mailto:…")` and `maps_launcher::is_available()` return `false` even with a browser, a mail app and Maps installed, unless the host manifest declares `<queries>` for those schemes. `launch_url` itself works without them.

### Change

- The docs of `can_launch_url` and `maps_launcher::is_available` say so, with an example `<intent>` element.
- The example's `AndroidManifest.xml` declares `https`, `mailto` and `geo` in `<queries>`.

No code changes.

### Testing

- `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets`: clean.
- On device (OnePlus CPH2581, Android 16, test app with targetSdk 34): without `<queries>`, all three calls return `false`; with them, `true`.

Thanks for taking the time to review this.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
