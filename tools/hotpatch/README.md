# hotpatch

Dev-only fast iteration for the lab, both from
[docs/hot-patching-plan.md](../../docs/hot-patching-plan.md):

- A1, hot patching with [Subsecond](https://docs.rs/subsecond): recompile
  the lab crate on save, link a patch library and push it to the phone,
  where `src/hot.rs` applies it while the app keeps running.
- A2, reload: push a new build of the library and restart the app on the
  same screen, with no Gradle and no reinstall.

```sh
tools/hotpatch/target/debug/hotpatch fat     # base debug APK in dist/; install it
tools/hotpatch/target/debug/hotpatch watch   # inside `phone session`, with the app running
tools/hotpatch/target/debug/hotpatch reload  # inside `phone session`: rebuild, push, restart (A2)
```

## Credits

This tool is built on the work of the [Dioxus](https://github.com/DioxusLabs/dioxus)
project (Jonathan Kelley and the Dioxus contributors):

- The app side is Dioxus's `subsecond` crate, used as a dependency.
- The patch builder follows the Dioxus CLI's hot patching (`dx serve
  --hotpatch`): recording rustc and linker arguments in a "fat" build, then
  "thin" links of the app crate against the running binary.
- `src/stub.rs` is adapted from the Dioxus CLI's
  `packages/cli/src/build/patch.rs` (`HotpatchModuleCache`,
  `create_undefined_symbol_stub`, `create_native_jump_table`), reduced to
  ELF/aarch64. The thin-link arguments in `src/main.rs` (`thin_link_args`)
  come from `packages/cli/src/build/link.rs`. Both are taken from
  DioxusLabs/dioxus at commit `f951996` (2026-10-03).

The Dioxus CLI is licensed under MIT OR Apache-2.0. Its MIT license text is in
[LICENSE-MIT-dioxus](LICENSE-MIT-dioxus).
