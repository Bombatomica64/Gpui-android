#!/usr/bin/env bash
# Build the GPUI Kit Mobile Lab APK.
#
#   ./build.sh android [--release|--debug] [--abi arm64-v8a|x86_64|all] [--install]
#
# Defaults: --release --abi arm64-v8a. The APK is copied to dist/.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILE=release
ABIS=(arm64-v8a)
INSTALL=false
MIN_API=26

usage() {
    sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//'
}

[[ "${1:-}" == "android" ]] || { usage; exit 1; }
shift
while [[ $# -gt 0 ]]; do
    case "$1" in
        --release) PROFILE=release ;;
        --debug) PROFILE=debug ;;
        --abi)
            shift
            case "${1:-}" in
                all) ABIS=(arm64-v8a x86_64) ;;
                arm64-v8a|x86_64) ABIS=("$1") ;;
                *) echo "unknown ABI: ${1:-}" >&2; exit 1 ;;
            esac
            ;;
        --install) INSTALL=true ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage; exit 1 ;;
    esac
    shift
done

export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    ANDROID_NDK_HOME="$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | sort -V | tail -1 || true)"
    export ANDROID_NDK_HOME
fi
[[ -d "$ANDROID_HOME" ]] || { echo "Android SDK not found; set ANDROID_HOME" >&2; exit 1; }
[[ -d "$ANDROID_NDK_HOME" ]] || { echo "Android NDK not found; set ANDROID_NDK_HOME" >&2; exit 1; }
command -v cargo-ndk >/dev/null || { echo "cargo-ndk missing: cargo install cargo-ndk" >&2; exit 1; }

JNI_LIBS="$ROOT/android/app/src/main/jniLibs"
rm -rf "$JNI_LIBS"

cargo_flags=()
[[ "$PROFILE" == release ]] && cargo_flags+=(--release)
# Optional Cargo features, e.g. LAB_FEATURES=demo-pr3329 for demo builds.
[[ -n "${LAB_FEATURES:-}" ]] && cargo_flags+=(--features "$LAB_FEATURES")
ndk_targets=()
for abi in "${ABIS[@]}"; do
    ndk_targets+=(-t "$abi")
    rustup target add "$( [[ $abi == arm64-v8a ]] && echo aarch64-linux-android || echo x86_64-linux-android )" >/dev/null
done

echo "==> cargo ndk ${ABIS[*]} ($PROFILE)"
(cd "$ROOT" && cargo ndk "${ndk_targets[@]}" --platform "$MIN_API" -o "$JNI_LIBS" build "${cargo_flags[@]}")
# gpui-mobile also declares a cdylib; the lab links it statically.
find "$JNI_LIBS" \( -name "libgpui_mobile.so" -o -name "libgpui_mobile-*.so" \) -delete

echo "==> gradle assemble${PROFILE^}"
echo "sdk.dir=$ANDROID_HOME" > "$ROOT/android/local.properties"
(cd "$ROOT/android" && ./gradlew --no-daemon -q "assemble${PROFILE^}")

APK_SRC="$ROOT/android/app/build/outputs/apk/$PROFILE/app-$PROFILE.apk"
[[ -f "$APK_SRC" ]] || { echo "APK not produced at $APK_SRC" >&2; exit 1; }
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
mkdir -p "$ROOT/dist"
APK="$ROOT/dist/gpui-mobile-lab-$VERSION-$PROFILE.apk"
cp "$APK_SRC" "$APK"
echo "==> $APK ($(du -h "$APK" | cut -f1))"

if $INSTALL; then
    adb install -r "$APK"
    adb shell am start -n dev.gpui.mobile.lab/.LabActivity
    echo "==> logs: adb logcat -s GPUI_MOBILE_LAB AndroidRuntime DEBUG"
fi
