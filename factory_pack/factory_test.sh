#!/usr/bin/env bash
# factory_test.sh — Master test script: smoke → kill-drill → offline-drill →
# test_harness, sequentially. Writes a JSON report. Stops at first failure.
# Usage: scripts/mobile/factory_test.sh <APP> --device <serial>
#
# Example:
#   scripts/mobile/factory_test.sh bvi-kit --device emulator-5554
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

APP="${1:-}"
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

if [[ -z "$APP" || -z "$DEVICE" ]]; then
  _log_error "Usage: scripts/mobile/factory_test.sh <APP> --device <serial>"
  exit 1
fi

_log_info "Starting factory test suite for $APP on $DEVICE"

# Pack layout: install the matching debug APK when shipped beside the script.
# (Repo layout has no such file; the app must already be installed there.)
if [[ -f "${SCRIPT_DIR}/${APP}-debug.apk" ]]; then
  _log_info "Installing ${APP}-debug.apk on $DEVICE..."
  # -g pre-grants runtime permissions so drills never block on dialogs.
  adb -s "$DEVICE" install -r -g "${SCRIPT_DIR}/${APP}-debug.apk"
fi

TESTS=(
  "smoke.sh"
  "kill-drill.sh"
  "offline-drill.sh"
  "test_harness.sh"
)

# Track results; index-aligned with TESTS (missing index = NOT RUN).
RESULTS=()

for test in "${TESTS[@]}"; do
  _log_info "=== Running $test for $APP on $DEVICE ==="
  if "${SCRIPT_DIR}/${test}" "$APP" --device "$DEVICE"; then
    _log_info "PASS: $test"
    RESULTS+=("PASS")
  else
    _log_error "FAIL: $test"
    RESULTS+=("FAIL")
    break
  fi
done

# Summary — default unrun entries to NOT-RUN (safe under set -u).
_log_info "=== Factory Test Suite Summary ==="
overall="PASS"
report="{\"app\":\"$APP\",\"device\":\"$DEVICE\",\"results\":{"
for i in "${!TESTS[@]}"; do
  status="${RESULTS[$i]:-NOT-RUN}"
  _log_info "${TESTS[$i]}: $status"
  [[ "$status" != "PASS" ]] && overall="FAIL"
  report="$report\"${TESTS[$i]}\":\"$status\","
done
report="${report%,}},\"result\":\"$overall\"}"
echo "$report" | tee "factory-report-${APP}-${DEVICE}.json"

if [[ "$overall" == "FAIL" ]]; then
  _log_error "Factory test suite FAILED"
  exit 1
fi
_log_info "Factory test suite PASSED"
exit 0
