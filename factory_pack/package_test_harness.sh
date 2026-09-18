#!/usr/bin/env bash
# package_test_harness.sh — Build the test harness APK (debug: run-as report
# retrieval requires a debuggable build), optionally clean intermediates, and
# create a slim, checksummed zip of the factory pack for distribution.
#
# Usage:
#   scripts/mobile/package_test_harness.sh [--clean] [--zip] [--include-src]
#
#   --clean        Remove Flutter build intermediates after copying the APK.
#   --zip          Create factory_test_harness_package.zip in the pack dir.
#   --include-src  Also include the test_harness source (Android + shared
#                  sources only; desktop/web/iOS platform dirs excluded).
#
# Works from both layouts:
#   repo:  scripts/mobile/package_test_harness.sh
#   pack:  factory_pack/package_test_harness.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -d "${SCRIPT_DIR}/test_harness" ]]; then
  PACK_DIR="$SCRIPT_DIR"                    # pack layout
else
  PACK_DIR="${SCRIPT_DIR}/../factory_pack"  # repo layout
fi
TEST_HARNESS_DIR="${PACK_DIR}/test_harness"
APK_SOURCE="${TEST_HARNESS_DIR}/build/app/outputs/flutter-apk/app-debug.apk"
APK_DEST="${PACK_DIR}/factory_test_harness.apk"
ZIP_NAME="factory_test_harness_package.zip"

CLEAN_INTERMEDIATES=false
CREATE_ZIP=false
INCLUDE_SRC=false

while [[ $# -gt 0 ]]; do
  case "$1" in
  --clean) CLEAN_INTERMEDIATES=true; shift ;;
  --zip) CREATE_ZIP=true; shift ;;
  --include-src) INCLUDE_SRC=true; shift ;;
  *)
    echo "Unknown option: $1"
    echo "Usage: $0 [--clean] [--zip] [--include-src]"
    exit 1
    ;;
  esac
done

_log_info() { echo "[INFO] $*"; }
_log_error() { echo "[ERROR] $*" >&2; }

cd "${TEST_HARNESS_DIR}"

_log_info "Getting Flutter dependencies..."
flutter pub get

_log_info "Building DEBUG harness APK (debug keeps run-as report retrieval working)..."
flutter build apk --debug

if [[ ! -f "$APK_SOURCE" ]]; then
  _log_error "APK not found at expected location: $APK_SOURCE"
  exit 1
fi

_log_info "Copying APK to factory pack..."
cp "$APK_SOURCE" "$APK_DEST"
_log_info "APK copied to: $APK_DEST"

if [[ "$CLEAN_INTERMEDIATES" = true ]]; then
  _log_info "Removing Flutter build intermediates to reduce size..."
  flutter clean
  _log_info "Intermediates cleaned."
fi

if [[ "$CREATE_ZIP" = true ]]; then
  cd "${PACK_DIR}"

  INCLUDES=(
    "README.md"
    "INSTALLER_TESTER_GUIDE.md"
    "factory_guidance.md"
    "compile_apks.md"
    "SHA256SUMS"
    # Debug APKs (drills + test-harness flows need debuggable builds)
    "bvi-kit-debug.apk"
    "bvr-kit-debug.apk"
    "rts-kit-debug.apk"
    # Release APKs (release-path validation; debug-key signed, NOT production)
    "bvi-kit-release.apk"
    "bvr-kit-release.apk"
    "rts-kit-release.apk"
    # Test harness APK
    "factory_test_harness.apk"
    "factory_test.sh"
    "smoke.sh"
    "kill-drill.sh"
    "offline-drill.sh"
    "test_harness.sh"
    "lib/common.sh"
    "lib/device.sh"
    "antiris/"
  )

  if [[ "$INCLUDE_SRC" = true ]]; then
    INCLUDES+=("test_harness/")
  fi

  for f in "${INCLUDES[@]}"; do
    # SHA256SUMS is generated below; skip its existence check here.
    [[ "$f" == "SHA256SUMS" ]] && continue
    if [[ ! -e "$f" ]]; then
      _log_error "pack entry missing: $f (aborting zip)"
      exit 1
    fi
  done

  # Checksums over exactly what ships (excludes applied to test_harness/).
  _log_info "Generating SHA256SUMS..."
  rm -f SHA256SUMS
  {
    for f in README.md INSTALLER_TESTER_GUIDE.md factory_guidance.md compile_apks.md \
             bvi-kit-debug.apk bvr-kit-debug.apk rts-kit-debug.apk \
             bvi-kit-release.apk bvr-kit-release.apk rts-kit-release.apk \
             factory_test_harness.apk factory_test.sh smoke.sh \
             kill-drill.sh offline-drill.sh test_harness.sh \
             lib/common.sh lib/device.sh; do
      shasum -a 256 "$f"
    done
    find antiris -type f -exec shasum -a 256 {} +
    if [[ "$INCLUDE_SRC" = true ]]; then
      find test_harness -type f \
        -not -path "*/build/*" -not -path "*/.dart_tool/*" \
        -not -path "*/.idea/*" -not -path "*/.gradle/*" \
        -not -path "*/ios/*" -not -path "*/macos/*" \
        -not -path "*/windows/*" -not -path "*/linux/*" \
        -not -path "*/web/*" -not -name ".DS_Store" \
        -exec shasum -a 256 {} +
    fi
  } > SHA256SUMS

  ZIP_EXCLUDES=(
    "test_harness/build/*"
    "test_harness/.dart_tool/*"
    "test_harness/.idea/*"
    "test_harness/.gradle/*"
    "test_harness/ios/*"
    "test_harness/macos/*"
    "test_harness/windows/*"
    "test_harness/linux/*"
    "test_harness/web/*"
    "__MACOSX*"
    "*.DS_Store"
    "*/.DS_Store"
  )

  _log_info "Creating zip package: $ZIP_NAME"
  rm -f "$ZIP_NAME"
  zip -r -q "$ZIP_NAME" "${INCLUDES[@]}" -x "${ZIP_EXCLUDES[@]}"

  _log_info "Zip created: $(cd "$(dirname "$ZIP_NAME")" && pwd)/$ZIP_NAME"
  unzip -l "$ZIP_NAME" | tail -5
fi

_log_info "Package preparation complete."
exit 0
