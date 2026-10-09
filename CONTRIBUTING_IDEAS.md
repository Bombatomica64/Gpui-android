# Ideas for helping the GPUI and GPUI Kit teams

Things worth working on, based on building this Android lab against GPUI Kit
0.7.0 and 0.7.1. Current bugs and workarounds are listed in
[README.md](README.md#known-android-issues) and [MOBILE_PATCHES.md](MOBILE_PATCHES.md).

## Where GPUI / GPUI Kit fit on mobile

**Bugs fixed, verdict:** I'd still pick React Native for a typical mobile app,
and GPUI Kit for a narrower set of projects.

### What worked

- **The components are solid.** About three-quarters of the catalog worked on
  Android without changes, although Kit is built for desktop. Dialogs, sheets,
  virtualized lists and tables, charts and the Questionnaire all behaved
  correctly.
- **The API is consistent.** Stateful controls follow one pattern (a state
  entity, an element, an event enum), so after a few screens the rest were
  quick to write.
- **Rust types caught mistakes early.** Most errors showed up at compile time.
- **One renderer.** GPUI draws every pixel itself, so the app looks the same
  everywhere, and charts and custom drawing are first-class.
- **Owning the whole stack made bugs fixable.** The Slider, clipboard and
  Activity-recreation fixes were each a few dozen lines, and upstream merged
  them.

### What was painful

- **Compile loop.** Each change meant a 2–3 minute release build plus
  reinstall, and a release build needs more memory than a small build
  server has (the lab now builds its APK on GitHub Actions). React Native
  hot-reloads in about a second.
- **API churn and stale docs.** GPUI is pinned to exact pre-release snapshots,
  and several doc examples didn't match the 0.7.0 source.
- **Binary size.** Each architecture adds about 30 MB of native library
  (16 MB with a size-tuned profile), and Android stores it uncompressed in
  the APK by default.
- **You rebuild the platform yourself.** IME handling, keyboard avoidance,
  fonts, dark mode and back navigation all had to be done by hand.

### Fixable vs. structural

Fixable: the touch-drag and soft-keyboard gaps, layout overflows and the
mobile-specific hooks. What stays structural:

- **Accessibility.** A GPU-drawn UI is invisible to TalkBack unless a full
  accessibility bridge is built. A first bridge exists now (gpui-mobile
  [#25](https://github.com/longbridge/gpui-mobile/pull/25), about 250 lines on
  top of GPUI's AccessKit tree), but every control's labels and roles still
  have to be right. React Native gets this from native views.
- **Native behavior.** Autofill, spellcheck, system text-selection handles,
  share sheets and system fonts come free with native widgets. GPUI has to
  imitate each one, and the imitation drifts.
- **Ecosystem.** React Native has libraries for maps, payments, auth, camera
  and notifications. With GPUI, much of that is your job.

### Where GPUI wins

- You already have a GPUI desktop app and want one Rust codebase, UI included.
- The app is mostly custom rendering: editors, charts, trading screens, dense
  data views.
- You want the same look on every platform.
- You want no JavaScript runtime at all.

Kit's own docs position mobile as a way to embed Kit views inside a native app,
not as a full mobile framework. That matches this experience.

## Possible work items

Ordered roughly by how much they'd move the verdict above. Status as of
October 2026; "fork" means the fix is on one of the branches listed in
[MOBILE_PATCHES.md](MOBILE_PATCHES.md) but not yet proposed upstream.

1. **Accessibility bridge for Android.** Expose GPUI's element tree to
   TalkBack through `AccessibilityNodeProvider`.
   - Open: gpui-mobile [#25](https://github.com/longbridge/gpui-mobile/pull/25),
     rebased onto the one-window-per-host model of #26.
   - Next: check Kit's controls for missing labels and roles once #25 lands.
2. **Soft-keyboard and IME support.**
   - Upstream: composing text and file picking (gpui-mobile #6); back hides
     the keyboard and a tap brings it back (#26).
   - Fork: keep focus when the keyboard's Done key hides it (gpui-mobile
     `android-ime-dismiss`); ask for the keyboard when a focused input is
     tapped, and scroll a focused input into view (gpui-kit
     `input-tap-requests-keyboard`, `input-reveal-on-focus`).
   - Not started: selection handles, autofill.
3. **Touch-drag and gesture handling.**
   - Upstream: Slider drags (gpui-kit #3313), long-press context menus
     (#3393), Android scroll physics and fling (gpui-mobile #13, #14).
   - Fork: `on_drag` elements on touch, autoscroll in scrollable divs, and
     no axis remapping for touch pans (gpui-pre `touch-drag`, `autoscroll`,
     `touch-axis`). These belong in zed-industries/zed, which gpui-pre
     snapshots.
4. **Faster iteration loop.** Hot reload, a dev-mode `cdylib` swap, or a
   desktop-hosted phone-size preview. Not started. Build it as a
   standalone package first and propose it upstream once it works.
5. **Binary size.** Measured in [docs/binary-size.md](docs/binary-size.md);
   every PR's job summary shows the APK and `.so` sizes.
   - Lab: native libraries stored compressed, APK 74 → 35 MB (#56).
   - Lab, open: a size-tuned release profile (`z`, fat LTO, one codegen
     unit, packed relocations), arm64 `.so` 29.5 → 15.6 MB (#60); R8 on
     the Java side, −1.4 MB (#57). Both wait for a phone check.
   - Upstream drafts, not opened: GPUI decoders beyond gif/jpeg/png/webp
     opt-in (−1.4 MB); gpui-mobile's example APK with compressed libraries.
   - Correction: gpui-mobile's example app uses `z`, fat LTO and
     `panic=abort`, but its root profile is `3` with thin LTO, and only the
     app's own profile applies. The lab had kept `3`/thin.
6. **Docs and examples for mobile.**
   - Open: gpui-mobile issue
     [#20](https://github.com/longbridge/gpui-mobile/issues/20), the
     example's `build.bat` launches an Activity the manifest doesn't declare;
     the issue includes the fix.
   - Not started: a minimal Android quick start.
7. **Platform services.**
   - Upstream: system clipboard (gpui-mobile #24), back navigation (#26).
   - Fork: dark mode follows the system on resume and configuration change
     (gpui-mobile `android-host-night-mode`), and emoji render on the
     host-driven path (`android-host-emoji-font`). Upstream as gpui-mobile
     #30 and #29, in review.
   - Share sheet: already upstream as `packages::share::share_text`, and it
     works on the host-driven path; the Diagnostics screen has a demo
     button.
   - Not started: flags and `❤️` (VS16) still don't render with the
     bundled emoji font; flags come from the system's COLRv1
     `NotoColorEmojiFlags.ttf`.
   - Not started: safe-area insets. The plan is in
     [docs/safe-area-insets.md](docs/safe-area-insets.md).
   - Not started: system fonts.
8. **Upstream the fork fixes.** Seven branches in
   [MOBILE_PATCHES.md](MOBILE_PATCHES.md) are still fork-only: two in
   gpui-mobile, two in gpui-kit, three in gpui-pre. Propose them one at a
   time.
9. **Kit components for small screens.**
   - Upstream: Radio inside RadioGroup, Textarea rows, rich text color in
     filled bubbles, Shimmer in dark mode (gpui-kit #3328–#3331).
   - Next: touch-sized variants of desktop-centric controls.
