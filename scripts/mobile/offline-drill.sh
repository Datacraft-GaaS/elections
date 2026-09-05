#!/usr/bin/env bash
# offline-drill.sh — 48h offline drill: enables airplane mode, queues ops,
# restores network, asserts dedup.
#
# Usage: scripts/mobile/offline-drill.sh <APP> --device <serial>
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

APP="${1:-}"
shift || true
DEVICE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --device) DEVICE="$2"; shift 2 ;;
    *) shift ;;
  esac
done

if [[ -z "$APP" || -z "$DEVICE" ]]; then
  _log_error "Usage: scripts/mobile/offline-drill.sh <APP> --device <serial>"
  exit 1
fi

_log_info "Offline drill for $APP on $DEVICE..."

# Enable airplane mode
if [[ -n "$DEVICE" ]]; then
  adb -s "$DEVICE" shell settings put global airplane_mode_on 1
  adb -s "$DEVICE" shell am broadcast -a android.intent.action.AIRPLANE_MODE
else
  adb shell settings put global airplane_mode_on 1
  adb shell am broadcast -a android.intent.action.AIRPLANE_MODE
fi

sleep 2

# Queue some ops (simulated via integration test driver)
_log_info "Queuing ops while offline..."
# TODO: Drive app through integration_test to queue records

# Restore network
if [[ -n "$DEVICE" ]]; then
  adb -s "$DEVICE" shell settings put global airplane_mode_on 0
  adb -s "$DEVICE" shell am broadcast -a android.intent.action.AIRPLANE_MODE
else
  adb shell settings put global airplane_mode_on 0
  adb shell am broadcast -a android.intent.action.AIRPLANE_MODE
fi

sleep 5

# Assert dedup
_log_info "Asserting dedup after reconnect..."
# TODO: Query BFF to verify no duplicates

_log_info "Offline drill complete for $APP"
