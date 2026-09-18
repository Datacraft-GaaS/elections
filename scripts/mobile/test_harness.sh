#!/usr/bin/env bash
# test_harness.sh — Install and run the pre-built test harness APK, pull the
# JSON report, fail on FAIL/ERROR statuses.
# Usage: scripts/mobile/test_harness.sh <APP> --device <serial>
#
# Works from both layouts:
#   repo:  scripts/mobile/test_harness.sh  (pack at ../factory_pack)
#   pack:  factory_pack/test_harness.sh    (APK + source beside the script)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
source "${SCRIPT_DIR}/lib/device.sh"

_app="${1:-}" # Unused; kept for interface consistency with other drills.
shift || true
DEVICE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
  --device)
    DEVICE="$2"
    shift 2
    ;;
  *) shift ;;
  esac
done

if [[ -z "$DEVICE" ]]; then
  _log_error "Usage: scripts/mobile/test_harness.sh <APP> --device <serial>"
  exit 1
fi

# Resolve the factory pack dir in either layout.
if [[ -f "${SCRIPT_DIR}/factory_test_harness.apk" ]]; then
  PACK_DIR="$SCRIPT_DIR"                    # pack layout
else
  PACK_DIR="${SCRIPT_DIR}/../../factory_pack"  # repo layout
fi
HARNESS_SRC="${PACK_DIR}/test_harness"
APK_PREBUILT="${PACK_DIR}/factory_test_harness.apk"
APK_BUILT="${HARNESS_SRC}/build/app/outputs/flutter-apk/app-debug.apk"

eel_require_device "$DEVICE" || exit 1

if [[ -f "$APK_PREBUILT" ]]; then
  _log_info "Using pre-built test harness APK: $APK_PREBUILT"
  APK_PATH="$APK_PREBUILT"
else
  _log_info "Pre-built APK not found, building debug harness APK (debug keeps run-as report retrieval working)..."
  (cd "$HARNESS_SRC" && flutter pub get && flutter build apk --debug)
  APK_PATH="$APK_BUILT"
  if [[ ! -f "$APK_PATH" ]]; then
    _log_error "APK not found at $APK_PATH after build"
    exit 1
  fi
fi

# Package name: build.gradle.kts applicationId first, manifest second.
PACKAGE_NAME=""
if [[ -f "${HARNESS_SRC}/android/app/build.gradle.kts" ]]; then
  PACKAGE_NAME=$(grep -oE 'applicationId = "[^"]+"' "${HARNESS_SRC}/android/app/build.gradle.kts" | head -1 | sed 's/.*"\([^"]*\)"/\1/' || true)
fi
if [[ -z "$PACKAGE_NAME" && -f "${HARNESS_SRC}/android/app/src/main/AndroidManifest.xml" ]]; then
  PACKAGE_NAME=$(grep -o 'package="[^"]*"' "${HARNESS_SRC}/android/app/src/main/AndroidManifest.xml" | head -1 | sed 's/package="//;s/"//' || true)
fi
if [[ -z "$PACKAGE_NAME" ]]; then
  _log_error "Could not determine harness package name (no applicationId found)"
  exit 1
fi
# Guard against stale checkouts predating the com.eel.test_harness rename.
if [[ "$PACKAGE_NAME" == com.example.* ]]; then
  _log_error "harness source still uses example package '$PACKAGE_NAME' — pull latest and rebuild"
  exit 1
fi

_log_info "Installing test harness APK ($PACKAGE_NAME) on $DEVICE..."
# -g grants all runtime permissions at install: the harness requests
# permissions on startup and would otherwise block on the grant dialog.
adb -s "$DEVICE" install -r -g "$APK_PATH"

_log_info "Launching test harness..."
eel_logcat_clear "$DEVICE"
eel_launch "$DEVICE" "$PACKAGE_NAME" >/dev/null

# Suite takes ~45 s on device (GNSS fix wait up to 30 s) plus cold start.
_log_info "Waiting for test harness to complete (120 seconds)..."
sleep 120

_log_info "Pulling test report from device..."
# getApplicationDocumentsDirectory() maps to app_flutter/ (NOT files/).
REPORT_JSON="$(adb -s "$DEVICE" shell run-as "$PACKAGE_NAME" cat app_flutter/test_report.json 2>/dev/null | tr -d '\r')" || true

if [[ -z "$REPORT_JSON" ]]; then
  crashlog="$(mktemp)"
  if eel_logcat_crashcheck "$DEVICE" "$PACKAGE_NAME" "$crashlog"; then
    _log_error "No report AND harness crashed (see $crashlog). If the APK is a release build, run-as cannot read its files — use a debug build."
  else
    _log_error "No test report retrieved. Harness may still be running (re-run with longer wait) or report path changed."
  fi
  exit 1
fi

echo "$REPORT_JSON" >"./test_report.json"
_log_info "Test report saved to ./test_report.json"

if command -v python3 >/dev/null 2>&1; then
  # Report schema: top-level "results" array, each entry has status
  # PASS|FAIL|SKIP|ERROR (see test_harness/lib/main.dart _saveReport).
  FAILED_COUNT=$(printf '%s' "$REPORT_JSON" | python3 -c "import json,sys; print(sum(1 for t in json.load(sys.stdin).get('results', []) if t.get('status') in ('FAIL','ERROR')))" 2>/dev/null || echo "0")
else
  FAILED_COUNT=$(printf '%s' "$REPORT_JSON" | grep -oE '"status"[[:space:]]*:[[:space:]]*"(FAIL|ERROR)"' | wc -l | tr -d ' ' || true)
fi
if [[ "${FAILED_COUNT:-0}" -gt 0 ]]; then
  _log_error "Test harness detected $FAILED_COUNT failed or errored tests."
  exit 1
fi
_log_info "Test harness execution complete: no FAIL/ERROR statuses."
exit 0
