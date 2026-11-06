# Upstreaming the gpui-mobile and GPUI Kit fixes

These are the fixes that live only in my forks of
[longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) and
[longbridge/gpui-kit](https://github.com/longbridge/gpui-kit), prepared to be
sent one at a time. This is item 8 of [CONTRIBUTING_IDEAS.md](../../CONTRIBUTING_IDEAS.md).
The GPUI fixes for Zed are in [../upstream-zed](../upstream-zed/README.md).

Posted so far (2026-10-16): #1–#4 as gpui-kit #3554–#3557 (merged, in v0.8.0), #20 as
gpui-mobile #39, and the first two JNI PRs from issue #31 as gpui-mobile #40
and #41. The Kit ones were rebased onto Kit `main` 319b56cf first; the old
branch tips are tagged `archive/2026-10-10/upstream/<name>`. Each fix is a minimal branch named
`upstream/<name>` on my fork
([Bombatomica64/gpui-mobile](https://github.com/Bombatomica64/gpui-mobile),
[Bombatomica64/gpui-kit](https://github.com/Bombatomica64/gpui-kit)), based on
upstream `main` as of 2026-10-10: gpui-mobile f9fe5a7 (#24), Kit 0bbd9870
(#3421). Each has a draft PR body next to this file. The drafts reuse the phone
captures in [../demos](../demos) and say where a capture is missing.

**Update 2026-11-06.** Since then #12, #15 and the safe-area branch were
posted and merged (gpui-kit #3623, #3625, #3624). Upstream moved: gpui-mobile
`main` is 7548ebc (#45: gpui-pre 0.3.10, `open_url` through the host, late
`surfaceDestroyed` ignored), Kit `main` is 48ac25b5 (v0.9.0 plus 4 commits,
gpui-pre 0.3.11). The unposted gpui-mobile branches #5–#11 and #13 were
rebased onto 7548ebc; their old tips are tagged
`archive/2026-11-06/upstream/<name>`. Three new fixes from lab issues #39
and #50 were added as #23–#25. No other open PR on either repo overlaps
them (checked: Kit #3818, which exposes a single-line input's text, touches
the same node but not its disabled state).

## Status

✅ = can be sent now · ⏳ = ready, waits for another PR · ⛔ = needs work or an upstream change first

| # | Repo | Branch | Status | Depends on | Recording | Draft |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | kit | [`upstream/input-icon-button-labels`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-icon-button-labels) | ✅ merged (v0.8.0): [#3554](https://github.com/longbridge/gpui-kit/pull/3554) | — (v0.8.0 names the mask toggle too) | before: uiautomator dump; after: no | [draft](gpui-kit/input-icon-button-labels.md) |
| 2 | kit | [`upstream/input-masked-password-role`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-masked-password-role) | ✅ merged (v0.8.0): [#3555](https://github.com/longbridge/gpui-kit/pull/3555) | — | before: uiautomator dump; after: no | [draft](gpui-kit/input-masked-password-role.md) |
| 3 | kit | [`upstream/input-back-keeps-text`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-back-keeps-text) | ✅ merged (v0.8.0): [#3556](https://github.com/longbridge/gpui-kit/pull/3556) | — | before only | [draft](gpui-kit/input-back-keeps-text.md) |
| 4 | kit | [`upstream/input-text-input-configuration`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-text-input-configuration) | ✅ merged (v0.8.0): [#3557](https://github.com/longbridge/gpui-kit/pull/3557) | — (gpui-pre 0.3.8 has the API) | yes (Android, with #13) | [draft](gpui-kit/input-text-input-configuration.md) |
| 5 | mobile | [`upstream/android-package-visibility`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-package-visibility) | ✅ (docs + manifest), rebased 2026-11-06 onto 7548ebc (clean) | — | not needed | [draft](gpui-mobile/android-package-visibility.md) |
| 6 | mobile | [`upstream/android-permission-errors`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-permission-errors) | ✅, rebased 2026-11-06 onto 7548ebc (clean) | — | measured values | [draft](gpui-mobile/android-permission-errors.md) |
| 7 | mobile | [`upstream/android-media-session`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-media-session) | ✅, rebased 2026-11-06 onto 7548ebc (clean) | — | log lines | [draft](gpui-mobile/android-media-session.md) |
| 8 | mobile | [`upstream/android-deeplink`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-deeplink) | ✅, rebased 2026-11-06 onto 7548ebc (clean) | — | log lines | [draft](gpui-mobile/android-deeplink.md) |
| 9 | mobile | [`upstream/android-notifications`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-notifications) | ✅, rebased 2026-11-06 onto 7548ebc (clean) | — | no | [draft](gpui-mobile/android-notifications.md) |
| 10 | mobile | [`upstream/android-async-prepare`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-async-prepare) | ✅, rebased 2026-11-06 onto 7548ebc (clean). Video follows as #24 | — | timings | [draft](gpui-mobile/android-async-prepare.md) |
| 11 | mobile | [`upstream/android-ime-dismiss`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-ime-dismiss) | ✅, rebased 2026-11-06 onto 7548ebc (conflict in `window.rs`: the keyboard methods now sit after #25's `a11y_*` methods). Send next: Kit #3623 (#12) is merged and does nothing on Android without it | — | after only | [draft](gpui-mobile/android-ime-dismiss.md) |
| 12 | kit | [`upstream/input-tap-requests-keyboard`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-tap-requests-keyboard) | ✅ merged (2026-10-22): [#3623](https://github.com/longbridge/gpui-kit/pull/3623) | #11 for it to work on Android | no | [draft](gpui-kit/input-tap-requests-keyboard.md) |
| 13 | mobile | [`upstream/android-text-input-configuration`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-configuration) | ⏳, rebased 2026-11-06 onto 7548ebc on top of #11 (same `window.rs` conflict, both sides kept) | #11 merged. (#4 is released: Kit v0.8.0) | yes | [draft](gpui-mobile/android-text-input-configuration.md) |
| 14 | mobile | [`upstream/android-a11y-clear-exceptions`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-a11y-clear-exceptions) | ✅ merged with #25 | pushed to #25 (7006f50) with the review fixes | not possible (code audit) | [draft](gpui-mobile/android-a11y-clear-exceptions.md) |
| 15 | kit | [`upstream/input-reveal-on-focus`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-reveal-on-focus) | ✅ merged (2026-10-23): [#3625](https://github.com/longbridge/gpui-kit/pull/3625) | The Zed autoscroll PR (not posted yet) for scrolling divs to follow | no | [draft](gpui-kit/input-reveal-on-focus.md) |
| 16 | mobile | [`upstream/android-text-input-purpose`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-purpose) | ⛔ (unchanged 2026-11-06: gpui-pre 0.3.10 and 0.3.11 still have no `purpose`; the Zed PR isn't posted. Not rebased) | #13, plus Zed `TextInputConfiguration::purpose` in a gpui-pre release; doesn't build until then | yes | [draft](gpui-mobile/android-text-input-purpose.md) |
| 17 | kit | `input-purpose` (fork branch only) | ⛔ (unchanged 2026-11-06) | #4 (merged), plus the same GPUI change; conflicts with `main` (to rebase then) | yes | — |
| 18 | mobile | `android-activity-requests`, `android-picker-files` (fork only) | ⛔ (unchanged 2026-11-06) | the JNI rework (issue #31): built on its `jni.rs`. #40/#41 are its first parts, still open | partly ([picker screenshots](../demos/platform-apis)) | — |
| 19 | mobile | `android-string-arrays` (fork only) | ⛔ (unchanged 2026-11-06) | #6, plus the JNI rework (#31): `contacts/android.rs` was rewritten there | no | — |
| 20 | mobile | [`upstream/android-hot-loader`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-hot-loader) (`GpuiLibraryLoader`) | ✅ posted: [#39](https://github.com/longbridge/gpui-mobile/pull/39), after #34 got no reply. 2026-11-06: no review yet, GitHub says mergeable, builds merged with `main` | — | phone strip from the A1 spike | [issue draft](hot-reload-discussion.md) |
| 21 | mobile | [`upstream/jni-docs`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/jni-docs) | ✅ posted: [#40](https://github.com/longbridge/gpui-mobile/pull/40). 2026-11-06: no review yet, mergeable, builds merged with `main` | — | not needed | — |
| 22 | mobile | [`upstream/jni-exceptions`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/jni-exceptions) | ✅ posted: [#41](https://github.com/longbridge/gpui-mobile/pull/41). 2026-11-06: no review yet, mergeable, builds merged with `main` | — | not needed | — |
| 23 | kit | [`upstream/input-disabled-a11y`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-disabled-a11y) | ✅ (new 2026-11-06, on Kit `main` 48ac25b5) | — | uiautomator before/after (Gpui-android#81) | [draft](gpui-kit/input-disabled-a11y.md) |
| 24 | mobile | [`upstream/android-video-async-prepare`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-video-async-prepare) | ⏳ (new 2026-11-06, on 7548ebc) | no code from #10, but the same design: send after #10 is reviewed | timings, 2 screenshots (Gpui-android#82) | [draft](gpui-mobile/android-video-async-prepare.md) |
| 25 | mobile | [`upstream/android-a11y-clickable`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-a11y-clickable) | ✅ (new 2026-11-06, on 7548ebc) | — | uiautomator before/after (Gpui-android#81) | [draft](gpui-mobile/android-a11y-clickable.md) |

Notes on the triage:

- **Not fixed upstream another way.** I checked upstream `main` and open PRs on both repos (2026-10-10). The one overlap is gpui-kit #3424 (open, by another contributor), which names the password mask toggle. My labels branch was trimmed to the clear button, so it doesn't duplicate #3424. Kit `main` already gives `content_type(Password)` the `PasswordInput` role; #2 adds it for `masked(true)` alone.
- **Split for one concern each.** Fork branch `android-callbacks` became #7 (media session) + #8 (deep links), and `android-notifications-links` became #9 (notifications) + #5 (`<queries>` docs).
- **Ported off the JNI rework.** #9, #10 and #13 used the `or_clear` helper from issue #31's rework. They now use the `map_err` + `exception_clear` idiom already in upstream `main` (one explicit clear in #13). Their Java files are byte-identical to the fork's, which the fork's CI built.
- **Kit branches were tested against Kit `main`.** My earlier tests were against v0.7.1, which the lab pins. #1 and #2 gained a UI test each, in the style of #3424.

## Checks run

Per branch, on this server with `-j4`:

- gpui-mobile: `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets`; `cargo clippy --target aarch64-linux-android` (no new warnings: the same 8 as upstream `main`); `cargo test --lib` (46 passed). Upstream's CI also runs an iOS check and the example Gradle build, which only run on PRs to `main`, so they will run when the PRs are opened.
- gpui-kit: `cargo fmt --all -- --check`; `cargo check -p gpui-base -p gpui-component`; `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`; `cargo test -p gpui-base --lib input`; `cargo test -p gpui-kit --features test-support --test input --test input_focus`. Results: fmt and clippy are clean on all six; `gpui-base` lib input tests pass (337, or 338 with #4's new test); kit `input` 179 (180 on #1 and #2 with their new tests) and `input_focus` 4 pass. The exception is #15, whose own test fails on stock gpui-pre 0.3.8 (it needs the GPUI autoscroll change). #4 needed one fix for Kit `main`: clippy's `needless_update`, because gpui-pre 0.3.8's `TextInputConfiguration` has no `purpose` field.

Not compiled: #16 and #17, which need the unreleased GPUI field.

Re-run on 2026-11-06 for the rebased and new branches, with the same
commands. Results:

- gpui-mobile (base 7548ebc): every rebased and new branch (#5–#11, #13, #24, #25), and #39, #40 and #41 each merged into `main`: fmt clean, `check --all-targets` clean, clippy has the same 9 warnings as `main` (8 before #45), `cargo test --lib` 46 passed. Each package branch also checked alone with `--no-default-features --features <package>` (#6 `permission_handler`, #7 `media_session`, #8 `deeplink`, #9 `notifications`, #10 `audio`, #24 `video_player`). #24 also: the example crate with `cargo ndk -t arm64-v8a check`, clean.
- Kit (base 48ac25b5), #23 only: our files are fmt-clean (`cargo fmt --check` flags only upstream's `text/inline_flow.rs`, with rustfmt 1.10); check and clippy `--deny warnings` clean; `gpui-base` `input` 424 passed, `a11y_disabled` 2 passed (and fails without the fix); kit `integration input` 223 passed, 1 failed (`input::completions::ordinary_input_schedules_no_followup_frame`, which fails the same way without #23).
- The Kit commands changed: `gpui-base` lib tests need `--features test-support` now (CI gets it from `cargo test --workspace`), and the kit tests are one `integration` target since #3481, so `--test input --test input_focus` became `--test integration input`.
- Not re-run: #16, #17 (blocked), and no phone this round: the device results in the new drafts come from the lab PRs (#81, #82).

## Suggested send order

As of 2026-11-06: gpui-mobile #39–#41 are open with no review. Next for
gpui-mobile: #11 (Kit #3623 already asks for the keyboard, and does nothing
on Android until #11 lands), #25 (a11y clickable), then the order below.
Kit: #23 (disabled input) can go now.


#25, #29 and #30 are merged. gpui-mobile now has #39–#41 open from me, so
send the next ones there as those land.

1. **Kit, now, one or two at a time:** #1 (clear label), #2 (masked role), #3 (back keeps text), #4 (text input configuration). They're independent of each other. #4 is the only one with new public API. #3 is the one most likely to get a design question (Android-only `cfg`).
2. **gpui-mobile, once #29/#30 are merged or reviewed:** the small independent ones first, at most two open at a time: #5 (docs), #6, #7, #8, then #9 and #10, which change behaviour (an `Err` where there was `Ok`, an async load).
3. **The keyboard stack:** #11 (ime-dismiss) in gpui-mobile, then #12 in Kit. #13 goes once #11 is merged and a Kit release contains #4. Before that, #13 would turn off suggestions in released Kit fields.
4. **With #25:** #14 (push to #25's branch with the comment in its draft, or send it alone after #25 merges).
5. **After GPUI changes reach a gpui-pre release:** #15 (autoscroll), #16 + #17 (purpose).
6. **After the JNI rework (#31) lands:** #18, #19. Rebase from the fork branches then.

## Before sending each one

- Read the draft. Like [the Zed drafts](../upstream-zed/README.md), these are notes to send from; check every claim against the diff, and edit the tone to yours.
- Re-run the checks on the current upstream `main` (rebase the `upstream/*` branch first).
- Fill the missing captures listed below if you want them; the phone wasn't used for this prep.
- PR titles follow the repos' style: `android: …` (gpui-mobile), `input: …` (Kit).

## Missing captures (follow-ups)

- #12 tap requests keyboard: hide the keyboard with back, tap the still-focused field.
- #15 reveal on focus: a long form in a scrolling div, tap a field near the bottom. Needs the fork build with the GPUI autoscroll change.
- #3 back keeps text: the "after" frames (type, back, back leaves the screen).
- #9 notifications: the notification shade, and the app opened by a tap.
- #1 / #2: a TalkBack screenshot or a new uiautomator dump (optional; the before/after values are in the drafts).
- #11 ime-dismiss: a "before" (Done closing the dialog). Optional.
