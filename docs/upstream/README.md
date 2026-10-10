# Upstreaming the gpui-mobile and GPUI Kit fixes

These are the fixes that live only in our forks of
[longbridge/gpui-mobile](https://github.com/longbridge/gpui-mobile) and
[longbridge/gpui-kit](https://github.com/longbridge/gpui-kit), prepared to be
sent one at a time. This is item 8 of [CONTRIBUTING_IDEAS.md](../../CONTRIBUTING_IDEAS.md).
The GPUI fixes for Zed are in [../upstream-zed](../upstream-zed/README.md).

Nothing here has been posted upstream. Each fix is a minimal branch named
`upstream/<name>` on our fork
([Bombatomica64/gpui-mobile](https://github.com/Bombatomica64/gpui-mobile),
[Bombatomica64/gpui-kit](https://github.com/Bombatomica64/gpui-kit)), based on
upstream `main` as of 2026-10-10: gpui-mobile f9fe5a7 (#24), Kit 0bbd9870
(#3421). Each has a draft PR body next to this file. The drafts reuse the phone
captures in [../demos](../demos) and say where a capture is missing.

## Status

✅ = can be sent now · ⏳ = ready, waits for another PR · ⛔ = needs work or an upstream change first

| # | Repo | Branch | Status | Depends on | Recording | Draft |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | kit | [`upstream/input-icon-button-labels`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-icon-button-labels) | ✅ (trimmed to the clear button) | — (the mask toggle half is in open #3424) | before: uiautomator dump; after: no | [draft](gpui-kit/input-icon-button-labels.md) |
| 2 | kit | [`upstream/input-masked-password-role`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-masked-password-role) | ✅ | — | before: uiautomator dump; after: no | [draft](gpui-kit/input-masked-password-role.md) |
| 3 | kit | [`upstream/input-back-keeps-text`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-back-keeps-text) | ✅ | — | before only | [draft](gpui-kit/input-back-keeps-text.md) |
| 4 | kit | [`upstream/input-text-input-configuration`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-text-input-configuration) | ✅ | — (gpui-pre 0.3.8 has the API) | yes (Android, with #13) | [draft](gpui-kit/input-text-input-configuration.md) |
| 5 | mobile | [`upstream/android-package-visibility`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-package-visibility) | ✅ (docs + manifest) | — | not needed | [draft](gpui-mobile/android-package-visibility.md) |
| 6 | mobile | [`upstream/android-permission-errors`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-permission-errors) | ✅ | — | measured values | [draft](gpui-mobile/android-permission-errors.md) |
| 7 | mobile | [`upstream/android-media-session`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-media-session) | ✅ | — | log lines | [draft](gpui-mobile/android-media-session.md) |
| 8 | mobile | [`upstream/android-deeplink`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-deeplink) | ✅ | — | log lines | [draft](gpui-mobile/android-deeplink.md) |
| 9 | mobile | [`upstream/android-notifications`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-notifications) | ✅ | — | no | [draft](gpui-mobile/android-notifications.md) |
| 10 | mobile | [`upstream/android-async-prepare`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-async-prepare) | ✅ | — | timings | [draft](gpui-mobile/android-async-prepare.md) |
| 11 | mobile | [`upstream/android-ime-dismiss`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-ime-dismiss) | ✅ | — | after only | [draft](gpui-mobile/android-ime-dismiss.md) |
| 12 | kit | [`upstream/input-tap-requests-keyboard`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-tap-requests-keyboard) | ⏳ | #11 (otherwise it's a no-op on Android) | no | [draft](gpui-kit/input-tap-requests-keyboard.md) |
| 13 | mobile | [`upstream/android-text-input-configuration`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-configuration) | ⏳ | #11 merged, #4 released in Kit | yes | [draft](gpui-mobile/android-text-input-configuration.md) |
| 14 | mobile | [`upstream/android-a11y-clear-exceptions`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-a11y-clear-exceptions) | ⏳ | open #25 (push it there, or send after #25 merges) | not possible (code audit) | [draft](gpui-mobile/android-a11y-clear-exceptions.md) |
| 15 | kit | [`upstream/input-reveal-on-focus`](https://github.com/Bombatomica64/gpui-kit/tree/upstream/input-reveal-on-focus) | ⏳ | Zed autoscroll PR in a gpui-pre release (its own test fails until then) | no | [draft](gpui-kit/input-reveal-on-focus.md) |
| 16 | mobile | [`upstream/android-text-input-purpose`](https://github.com/Bombatomica64/gpui-mobile/tree/upstream/android-text-input-purpose) | ⛔ | #13, plus Zed `TextInputConfiguration::purpose` in a gpui-pre release; doesn't build until then | yes | [draft](gpui-mobile/android-text-input-purpose.md) |
| 17 | kit | `input-purpose` (fork branch only) | ⛔ | #4, plus the same GPUI change; conflicts with `main` (to rebase then) | yes | — |
| 18 | mobile | `android-activity-requests`, `android-picker-files` (fork only) | ⛔ | the JNI rework (issue #31): built on its `jni.rs` | partly ([picker screenshots](../demos/platform-apis)) | — |
| 19 | mobile | `android-string-arrays` (fork only) | ⛔ | #6, plus the JNI rework (#31): `contacts/android.rs` was rewritten there | no | — |

Notes on the triage:

- **Not fixed upstream another way.** I checked upstream `main` and open PRs on both repos (2026-10-10). The one overlap is gpui-kit #3424 (open, by another contributor), which names the password mask toggle. Our labels branch was trimmed to the clear button, so it doesn't duplicate #3424. Kit `main` already gives `content_type(Password)` the `PasswordInput` role; #2 adds it for `masked(true)` alone.
- **Split for one concern each.** Fork branch `android-callbacks` became #7 (media session) + #8 (deep links), and `android-notifications-links` became #9 (notifications) + #5 (`<queries>` docs).
- **Ported off the JNI rework.** #9, #10 and #13 used the `or_clear` helper from issue #31's rework. They now use the `map_err` + `exception_clear` idiom already in upstream `main` (one explicit clear in #13). Their Java files are byte-identical to the fork's, which the fork's CI built.
- **Kit branches were tested against Kit `main`.** Our earlier tests were against v0.7.1, which the lab pins. #1 and #2 gained a UI test each, in the style of #3424.

## Checks run

Per branch, on this server with `-j4`:

- gpui-mobile: `cargo fmt --all -- --check`; `cargo check --target aarch64-linux-android --all-targets`; `cargo clippy --target aarch64-linux-android` (no new warnings: the same 8 as upstream `main`); `cargo test --lib` (46 passed). Upstream's CI also runs an iOS check and the example Gradle build, which only run on PRs to `main`, so they will run when the PRs are opened.
- gpui-kit: `cargo fmt --all -- --check`; `cargo check -p gpui-base -p gpui-component`; `cargo clippy -p gpui-base -p gpui-component -p gpui-kit --features gpui-kit/test-support --tests -- --deny warnings`; `cargo test -p gpui-base --lib input`; `cargo test -p gpui-kit --features test-support --test input --test input_focus`. Results: fmt and clippy are clean on all six; `gpui-base` lib input tests pass (337, or 338 with #4's new test); kit `input` 179 (180 on #1 and #2 with their new tests) and `input_focus` 4 pass. The exception is #15, whose own test fails on stock gpui-pre 0.3.8 (it needs the GPUI autoscroll change). #4 needed one fix for Kit `main`: clippy's `needless_update`, because gpui-pre 0.3.8's `TextInputConfiguration` has no `purpose` field.

Not compiled: #16 and #17, which need the unreleased GPUI field.

## Suggested send order

The open gpui-mobile PRs (#25 TalkBack, #29 emoji font, #30 night mode) are
still waiting for review, so don't pile more on that repo yet. GPUI Kit
merges quickly, so start there.

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
