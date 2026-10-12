# Reply to huacnlee's review of longbridge/gpui-mobile#29

Draft, to be posted by hand. All changes are in
[773e5ad](https://github.com/longbridge/gpui-mobile/pull/29/commits/773e5ade2353399a3bac9f848c8dfd4b4c9a1e19).

---

Thanks for the careful review! All three points are addressed in 773e5ad:

1. **Call order / load once.** I went with the lazy option, plus the other two.
   If `AndroidPlatform::new` finds no AssetManager, it logs a `warn!` and keeps
   its text system pending; the next `set_host_activity` that gets an
   AssetManager loads the font into it. This covers `start` running first and a
   failed `getApplicationContext()`/`getAssets()` on the first `onCreate`. A
   mutex is held across the check so a concurrent `set_host_activity` can't slip
   in between. The "What the host has to do" section in `host.rs` now says
   `set_host_activity` comes before `start`, and what happens if it doesn't.
2. **Local refs.** `host_assets` now runs inside `env.with_local_frame(2, …)`,
   so `context` and `assets` are freed even if the caller stays in native code.
3. **Pending exceptions.** `set_host_activity` is now a thin wrapper that clears
   any pending exception whenever the inner function returns `Err`, so the
   `new_global_ref` / `get_java_vm` paths behave like the asset branch.

On the suggestions:

- **Night mode:** #30 covers this. It reads `uiMode` from the application
  context's `Configuration` over JNI on the host path, so it doesn't depend on
  this PR. I'd rather keep the two PRs independent; once both are in, switching
  it to `with_asset_manager(Configuration::from_asset_manager)` is a small
  follow-up if you prefer that.
- **`to_vec()` copy:** done here, since it was small: `load_asset_bytes` now
  reads the asset into one `Vec::with_capacity(asset.length())` instead of
  mapping it and copying.

Checked with `cargo clippy --target aarch64-linux-android` (no new warnings).
