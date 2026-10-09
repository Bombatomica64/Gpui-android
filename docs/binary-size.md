# Binary size: where the ~30 MB per ABI goes, and what cuts it

Status, October 2026: measured on CI. The APK went from 74.0 MB to 35.1 MB
(#56, merged). A size-tuned release profile (#60) takes the arm64 library from
29.5 to 15.6 MB and is waiting for a frame-rate check on the phone.

## How it is measured

- **Android APK** (every PR) writes the APK and per-ABI `.so` sizes, and the
  largest APK entries, to the job summary.
- **Binary size** (PRs that touch the Rust build, or by hand) builds an
  unstripped arm64 `.so`, then:
  - breaks it down by section, crate and crate family with
    `tools/size-report.py` (`llvm-nm` symbol sizes, attributed like
    cargo-bloat to the first non-std crate in the symbol's path);
  - uploads the report, the full symbol list and `cargo tree -d` as the
    `size-report-*` artifact.

  Release-profile experiments need no Cargo.toml edit:

  ```sh
  gh workflow run size.yml --ref <branch> -f label=z-fat -f opt_level=z -f lto=fat \
      [-f codegen_units=1] [-f rustflags=...] [-f toolchain=nightly -f cargo_args=-Zbuild-std=...]
  ```

cargo-bloat does not find the Android target when run under cargo-ndk, so
the report reads the symbol table itself.

## Baseline (main at d50139e)

| | MB |
|---|---:|
| APK (arm64 + x86_64) | 74.04 |
| `lib/x86_64/libgpui_mobile_lab.so`, stored | 32.46 |
| `lib/arm64-v8a/libgpui_mobile_lab.so`, stored | 29.49 |
| `assets/fonts/NotoColorEmoji.ttf` (10.59, deflated to) | 9.81 |
| `classes.dex` (4.54, deflated to) | 1.62 |

Release profile at the time: `opt-level = 3`, `lto = "thin"`,
`codegen-units = 4`, `panic = "abort"`, `strip = true`.

### arm64 `.so` by section

| Section | MB |
|---|---:|
| `.text` (code) | 20.36 |
| `.rodata` (constants, tables) | 3.07 |
| `.eh_frame` + `.eh_frame_hdr` (unwind tables) | 2.41 |
| `.rela.dyn` (relocations) | 1.86 |
| `.data.rel.ro` (vtables, pointers) | 1.62 |

### arm64 `.so` by crate family

Named symbols account for 21.8 of the 29.5 MB. The rest is anonymous
constants, unwind tables, relocations and the dynamic symbol table.

| Family | MB | Biggest crates (MB) |
|---|---:|---|
| gpui | 5.14 | gpui 4.46, taffy 0.25, scheduler 0.14, sum_tree 0.12 |
| gpui-kit components | 3.91 | gpui_base 1.77, gpui_component 1.52, markdown 0.31, html5ever 0.13 |
| images + SVG | 2.65 | exr 0.37, image_webp 0.39, zune_jpeg 0.37, image 0.31, usvg 0.30, tiff 0.16, tiny_skia 0.14 |
| general-purpose crates | 1.91 | regex (3 crates) 0.59, hashbrown 0.39, smallvec 0.23, async_task 0.12 |
| fonts + text shaping | 1.83 | harfrust 0.43, skrifa 0.28, rustybuzz 0.28, read_fonts 0.25, zeno 0.15, ttf_parser 0.14, cosmic_text 0.10 |
| wgpu + Vulkan/GL | 1.45 | wgpu_core 0.77, wgpu_hal 0.39, ash 0.09 |
| naga (shader compiler) | 1.36 | SPIR-V out 0.30, WGSL in 0.26, validator 0.26, GLSL out 0.11 |
| unnamed / C | 1.00 | |
| gpui-mobile + Android glue | 0.88 | jni 0.52, gpui_mobile 0.26 |
| std | 0.82 | |
| lab | 0.54 | |
| AccessKit | 0.14 | |

`Debug` impls alone add up to 1.18 MB.

## What each change saves

Every number is from a CI run. "`.so`" is the stripped arm64 library. "gzip"
is `gzip -9` of it, a stand-in for its compressed size in the APK.

| Change | Measured on | `.so` MB | gzip MB | Saving | Belongs in | Status |
|---|---|---:|---:|---:|---|---|
| Baseline | | 29.49 | 11.18 | | | |
| Store native libs compressed (`useLegacyPackaging`) | APK | | | **APK 74.04 → 35.06 (−38.98)** | lab; gpui-mobile example | lab #56 merged; gpui-mobile draft |
| `opt-level = "s"` | `.so` | 24.71 | 8.73 | −4.78 | lab | measured |
| `opt-level = "z"` | `.so` | 21.53 | 8.66 | −7.96 | lab | measured |
| `lto = "fat"` | `.so` | 26.63 | 10.95 | −2.86 | lab | measured |
| `codegen-units = 1` | `.so` | 27.61 | 10.85 | −1.88 | lab | measured |
| 3 + fat + cgu 1 | `.so` | 25.77 | 10.79 | −3.72 | lab | measured |
| s + fat + cgu 1 | `.so` | 20.84 | 8.52 | −8.65 | lab | fallback if `z` is slow |
| z + fat + cgu 1 | `.so` | 16.98 | 7.63 | −12.51 | lab | measured |
| **z + fat + cgu 1 + packed relocations** | `.so` | **15.65** | **7.49** | **−13.84** | lab | #60, needs a phone check |
| Packed relocations alone (`-Wl,--pack-dyn-relocs=android`) | `.so` | 27.88 | 11.00 | −1.61 | lab | in #60 |
| `-C force-unwind-tables=no` | `.so` | 28.63 | 11.04 | −0.86 (0 on top of fat LTO) | — | rejected: breaks native stack traces in tombstones |
| `-Zbuild-std` (nightly) | `.so` | 29.44 | 11.12 | −0.05 | — | rejected |
| `-Zbuild-std` + `-Cpanic=immediate-abort` (nightly) | `.so` | 28.15 | 10.80 | −1.34 (−0.17 on top of z + fat + cgu 1) | — | rejected: nightly for 1% |
| Decode only gif/jpeg/png/webp | `.so` | 28.06 | 10.71 | −1.43 (−0.81 on top of z + fat + cgu 1) | GPUI (zed) | fork branch, upstream draft |
| R8 on the Java side | APK | | | APK −1.36 (dex 4.54 → 0.61) | lab | #57 draft, needs a phone check |
| One skrifa/read-fonts (lockfile) | `.so` | 29.47 | 11.14 | −0.02 | — | not worth it |
| One png, via image 0.25.6 (lockfile) | `.so` | 29.34 | 11.15 | −0.15 | — | not worth a downgrade |
| One APK per ABI (splits or AAB) | APK | | | arm64 APK ≈ 35.06 − 11.80 = 23.3 | lab, for distribution | not done: sideload APK, and build.sh is shared with the hot-reload work |
| Twemoji COLRv0 instead of the bundled CBDT Noto (1.47 vs 10.59 MB) | APK | | | ≈ −8.3, estimated | lab; gpui-mobile example | idea; changes the emoji style, needs a phone check |

With #56 and #60 together (the APK job of #60), the APK is **27.34 MB**
instead of 74.04, and the libraries install at 15.66 MB (arm64) and 19.12 MB
(x86_64) instead of 29.49 and 32.46.

## Questions from the brief

- **Does #26's GL fallback compile in a second backend?** Yes. wgpu's
  default features build Vulkan and GLES on Android (DX12 and Metal are
  cfg'd out, and naga's HLSL/MSL writers are dead-stripped to 0 bytes). The
  GL side costs about 0.34 MB: `wgpu_hal::gles` 0.18, naga's GLSL writer
  0.11, glow 0.04, khronos-egl 0.01. Keep it: phones without a usable
  Vulkan driver fall back to it.
- **naga** is 1.36 MB because shaders are WGSL compiled at runtime. Shipping
  precompiled SPIR-V would drop the WGSL front end and most of the
  validator (about 0.5 MB). That is GPUI/wgpu work for little gain.
- **Duplicate versions** (`cargo tree -d`): skrifa/read-fonts/font-types
  (cosmic-text vs swash), png (tiny-skia vs image), jni 0.21 (accesskit) vs
  0.22, hashbrown ×3, miniz_oxide ×2, getrandom ×2. LTO already removes most
  of the overlap. Unifying the two biggest saves 0.02 and 0.15 MB.
- **Image codecs**: GPUI builds `image` with every decoder (exr, tiff, dds,
  hdr, …) and decodes through `ImageReader::with_format`, so the linker
  can't drop any of them.
- **The lab's own assets**: the bundled emoji font is the only one, 9.8 MB
  of the compressed APK. It is there because swash can't draw Android 13+'s
  COLRv1 system emoji. Some Android builds ship a CBDT
  `NotoColorEmojiLegacy.ttf`, but not all do, so it can't be relied on.
- **Upstream's profile**: gpui-mobile's root `Cargo.toml` uses `opt-level =
  3` and thin LTO, but that profile only applies to gpui-mobile's own
  builds. Its example app (`example/Cargo.toml`) uses `z`, fat LTO and
  `panic = "abort"`. A downstream app's own profile is what counts.

## Upstream drafts (not opened)

### GPUI: make the extra image decoders opt-in

Branch `image-formats` on Bombatomica64/gpui-pre (on `0.3.8`). The real
PR goes to zed-industries/zed, where the `image` features live in the
workspace `Cargo.toml`; Zed would enable the new feature for its image viewer.

> **gpui: Make image decoders beyond gif, jpeg, png and webp opt-in**
>
> `gpui` builds `image` with every decoder: bmp, dds, exr, ff, hdr, ico,
> pnm, qoi, tga, tiff, plus rayon. `img()` decodes through
> `ImageReader::with_format`, so none of them is dead code to the linker.
> On an arm64 Android build of a GPUI app that is 1.43 MB of a 29.5 MB
> library (0.81 MB with `opt-level = "z"` and fat LTO). exr alone is
> 0.37 MB.
>
> This keeps gif, jpeg, png and webp (the formats `img` handles itself) and
> moves the rest behind a new `extra-image-formats` feature. Without it, an
> image in one of those formats fails to load with an "unsupported format"
> error instead of decoding. Zed's image viewer would enable the feature.
>
> Measured with the GPUI Mobile Lab's size report (link to the CI run).

### gpui-mobile: store the example's native library compressed

Branch `example-compressed-libs` on Bombatomica64/gpui-mobile (on
longbridge `main`). It adds one line to `example/android/gradle/app/build.gradle.kts`.

> **example: Store the native library compressed in the APK**
>
> With `minSdk = 26`, the Android Gradle plugin stores `.so` files
> uncompressed (page-aligned, so they can be mapped from the APK). A GPUI
> library is mostly code and compresses well, so this makes the APK much
> larger than it needs to be: in the GPUI Mobile Lab, the arm64 library is
> 29.5 MB stored and 11.2 MB deflated. `useLegacyPackaging = true` deflates
> it; Android extracts it once at install time. Apps on Google Play would
> use an App Bundle instead, where Play handles this.
>
> | | Before | After |
> |---|---:|---:|
> | Lab APK (arm64 + x86_64) | 74.04 MB | 35.06 MB |

Not drafted, and not worth it yet: gpui-kit. Its 3.9 MB is the components
themselves; the only optional-looking part is the Markdown/HTML parser
(0.46 MB), which the lab's rich-text screens use.
