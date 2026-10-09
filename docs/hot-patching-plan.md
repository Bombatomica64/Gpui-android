# Hot patching and over-the-air updates: plan

Two separate goals, with different rules:

- **A. Development:** see a Rust change on the phone in seconds, keeping app
  state. This is debug builds only, so store policy does not apply.
- **B. Production:** ship fixes without waiting for Google Play review, as
  Expo Updates does for React Native. Store policy decides what is possible.

## The constraint for B: Google Play policy

From the [Device and Network Abuse policy](https://support.google.com/googleplay/android-developer/answer/9888379):

> an app may not download executable code (such as dex, JAR, .so files) from
> a source other than Google Play

with an exception for

> code that runs in a virtual machine or an interpreter where either provides
> indirect access to Android APIs

Interpreted languages loaded at run time (the policy names JavaScript, Python
and Lua) must still not enable policy violations.

So:

- **Native code over the air is not allowed on Play.** That includes a new
  `.so`, which is how GPUI apps are built. It would be fine for sideloaded,
  enterprise/MDM or internal builds only.
- **React Native/Expo are allowed** because JS runs in an interpreter (Hermes).
  An equivalent for GPUI needs the updatable part to be data, or to run in an
  interpreter or VM.
- **WebAssembly is not named in the policy.** A WASM module that reaches the
  device only through functions the host provides plausibly fits "a virtual
  machine … with indirect access to Android APIs". That is a judgment call to
  confirm before relying on it.
- Apple has a similar rule (App Store Review Guideline 2.5.2); not checked here.

## Track A: development hot patching

### A0. Baseline (measured 2026-10-09)

Measured on this server (12 cores, memory capped at 7 GB, `-j 6`), building
the arm64 debug lab with `cargo ndk`:

| Step | Time |
|---|---|
| Clean Rust build | 4 min 54 s. The largest compiler process peaked at about 2 GB, so debug builds fit locally (release builds don't). |
| Incremental Rust build after a UI text change | ~6 s (3 s for a comment-only change) |
| Gradle `assembleDebug` with a cold JVM | 20 s |
| Debug APK | 265 MB. The library is 255 MB, about 190 MB of it debug sections (line tables + DWARF strings/ranges); `.text` is 21 MB. |
| `llvm-strip --strip-debug` | 0.3 s, 255 MB → 49 MB |
| `adb push` of 49 MB through the ssh tunnel | ~8 s (1.3 s of transfer) |
| CI release build (two ABIs, warm cache) | 4–6 min |

Conclusions:

- Compiling Rust is not the bottleneck locally. Gradle, the huge debug APK
  and reinstalling are.
- A dev client that pushes a *stripped* `.so` (symbolising crashes locally
  against the unstripped one, as `ndk-stack` does) gets a change onto the
  phone in about 6 + 0.3 + 8 s plus a restart, ~15 s, with no Gradle and no
  reinstall. Only hot patching beats that, and keeps state.
- `build.sh` also packaged gpui-mobile's own library by mistake (137 MB in
  debug builds); fixed in #31.

### A1. Subsecond spike (2–3 days, go/no-go)

[Subsecond](https://docs.rs/subsecond) (Dioxus) is a Rust hot-patching
library:

- Calls go through a jump table. An external tool compiles only the changed
  code, links it against the running binary's addresses, and sends the
  running app a new jump table.
- It supports Android arm64. It is active only with debug assertions, so it
  costs nothing in release builds.
- Limits:
  - Only the **tip crate** (the binary/cdylib's own crate) is patched.
  - Struct layout changes are not supported.
  - Thread-locals in the tip crate reset on each patch.
  - Static initialisers are not re-run.

The spike, in order:

1. One `subsecond::call` in gpui-mobile's frame loop, around the window's
   draw. All render code in the lab crate then becomes patchable. GPUI state
   lives in `App` entities, not in tip-crate statics, so it should survive a
   patch.
2. Patch transport: Subsecond's devtools websocket through
   `adb reverse tcp:<port>`.
3. Tooling, the main risk. ThinLink (Subsecond's linker wrapper) only ships
   inside the Dioxus CLI (`dx serve --hotpatch`), and `dx` builds its own
   Android project, not our host Activity. Options:
   - drive `dx` with a custom Android template;
   - use the `dioxus-cli` crate as a library;
   - write a minimal patch builder ourselves (diff object files, link against
     the running binary's symbol table).

   The spike decides which.

Exit criteria: changing a colour, text or layout in a lab screen shows on the
phone in under 2 s with the current screen and inputs kept. If that fails,
document why and rely on A2.

### A2. Fast full reload ("GPUI Go" dev client)

Needed even if A1 works, for the changes Subsecond cannot patch: struct
changes, gpui/Kit/gpui-mobile changes, new dependencies.

- A debug host APK is built once. On start it loads the app library from its
  private directory (`System.load(path)`), falling back to the packaged one.
- `cargo gpui dev` (our CLI, below) builds the arm64 debug `.so`, pushes it
  (`adb push` + `run-as` copy), and restarts the app process. No Gradle, no
  reinstall.
- Optional: save and restore a small "dev state" (current screen, scroll) so
  a reload lands where you were.

Exit criteria: Rust change to running new code in about 15 s, per the A0
numbers.

### A3. Build speed

Measure, then apply what helps:

- **arm64 only for dev builds.**
- **Linker:** NDK clang already uses lld; try `-Z threads`/mold where
  available.
- **Caching:** `sccache`.
- **Lighter dev builds:**
  - `debug = 0` for dependencies;
  - splitting the lab crate so a screen change recompiles little;
  - the Cranelift backend for the tip crate, if it supports aarch64-android.

## Track B: production updates without a store release (deferred)

**Status: deferred.** The goal is to make a Rust app work on mobile, not to
build an update platform. On Play the only legal ways to change behaviour
without a release are to ship data (server-driven UI, or state and logic that
live on a server) or code in a VM such as WASM. Both are substantial
infrastructure. Revisit when a real app needs it; the notes below record what
was found.


### B1. Update infrastructure (`gpui-updates`)

The part of Expo Updates that is independent of what gets updated:

- A signed manifest per channel/runtime version, with staged rollout.
- Download in the background, verify the signature, swap atomically on the
  next launch.
- Automatic rollback when the new bundle crashes at start.

It first ships **data**, which is clearly allowed: themes, strings, images,
remote config, feature flags. B2/B3 reuse it for code.

### B2. Server-driven UI (data, allowed)

A declarative UI format, for example JSON or a small DSL, that describes GPUI
Kit components, layout, text and bindings to actions the app registers
natively. Screens can change without a release; behaviour stays limited to
the registered actions. Moderate effort, no policy risk.

### B3. WASM modules for UI and logic (policy to confirm)

Closest to "React Native for Rust":

- App screens and logic compile to WASM and run in an embedded runtime.
  Candidates: `wasmi`, a pure-Rust interpreter, or `wasmtime`. JIT on Android
  needs checking.
- The module returns an element tree and handles events through a typed
  interface (WIT). The host maps the tree to GPUI elements and exposes device
  APIs only through host functions.
- Large effort: the guest UI API is a framework in itself. Spike it before
  committing:
  1. render a few elements;
  2. handle input;
  3. measure frame cost under `wasmi`.

Decide between B2 and B3 after B1 and the spike. They can coexist: B2 for
content-driven screens, B3 for logic.

## A Rust "Expo", piece by piece

| Expo | Here |
|---|---|
| Expo Go / dev client | A2: debug host APK that loads a pushed `.so` |
| Fast Refresh | A1: Subsecond |
| Expo Updates | B1 (+ B2/B3) |
| `expo prebuild`, config plugins | `cargo gpui new` / `prebuild`: generate the host Activity, manifest and helpers from config. This also removes "hosts must copy Java files" |
| EAS Build | GitHub Actions (exists) |
| CLI | `cargo gpui` (`dev`, `build`, `publish`), wrapping cargo-ndk, Gradle, adb |

## Order

1. A0 baseline. Running.
2. A1 Subsecond spike: go/no-go.
3. A2 dev client. Needed either way.
4. A3 build speed, guided by A0.
5. `cargo gpui` CLI, folding in A1 and A2 as they land.
6. Track B only when a real app needs it.
