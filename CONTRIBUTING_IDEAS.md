# Ideas for helping the GPUI and GPUI Kit teams

Things worth working on, based on building this Android lab against GPUI Kit
0.7.0. Current bugs and workarounds are listed in
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
  Activity-recreation fixes were each a few dozen lines.

### What was painful

- **Compile loop.** Each change meant a 2–3 minute release build plus
  reinstall. React Native hot-reloads in about a second.
- **API churn and stale docs.** GPUI is pinned to exact pre-release snapshots,
  and several doc examples didn't match the 0.7.0 source.
- **Binary size.** Each architecture adds about 27 MB of native library before
  any app code.
- **You rebuild the platform yourself.** IME handling, keyboard avoidance,
  fonts, dark mode and back navigation all had to be done by hand.

### Fixable vs. structural

Fixable: the touch-drag and soft-keyboard gaps, layout overflows and the
mobile-specific hooks. What stays structural:

- **Accessibility.** A GPU-drawn UI is invisible to TalkBack unless a full
  accessibility bridge is built. React Native gets this from native views.
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

Ordered roughly by how much they'd move the verdict above.

1. **Accessibility bridge for Android.** Expose GPUI's element tree to
   TalkBack through `AccessibilityNodeProvider`. This is the largest
   structural gap.
2. **Soft-keyboard and IME support.** Keyboard avoidance, composing text,
   selection handles and autofill hooks in `gpui-mobile`.
3. **Touch-drag and gesture handling.** Consistent drag, fling and
   long-press semantics for Slider, scroll views and lists.
4. **Faster iteration loop.** Hot-reload or incremental-build options, such
   as a dev-mode `cdylib` swap, or a desktop-hosted phone-size preview.
5. **Binary size.** Measure what contributes to the ~27 MB per architecture
   and look at feature flags, LTO and stripping.
6. **Docs and examples for mobile.** Update the examples that don't match
   0.7.0 and add a minimal Android quick start.
7. **Platform services.** Clipboard, share sheet, system dark mode, system
   fonts, back navigation and safe-area insets as first-class APIs.
8. **Upstream the local patches.** Each entry in
   [MOBILE_PATCHES.md](MOBILE_PATCHES.md) is a candidate PR.
9. **Kit components for small screens.** Fix layout overflows and add
   touch-sized variants of desktop-centric controls.
