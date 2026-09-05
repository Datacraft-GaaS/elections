#!/usr/bin/env bash
# smoke.sh — Cold-launch app, wait for telemetry session, report PASS/FAIL.
# Usage: scripts/mobile/smoke.sh <APP> --device <serial>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"; shift || true
DEVICE=""
while [[ $# -gt 0 ]]; do
  case "$1" in --device) DEVICE="$2"; shift 2 ;; *) shift ;; esac
done
[[ -z "$APP" || -z "$DEVICE" ]] && { _log_error "Usage: scripts/mobile/smoke.sh <APP> --device <serial>"; exit 1; }
_log_info "Smoke testing $APP on $DEVICE..."
pkg=$(adb -s "$DEVICE" shell pm list packages | grep "$APP" | cut -d: -f2)
[[ -z "$pkg" ]] && { _log_error "App $APP not installed on $DEVICE"; exit 1; }
activity=$(adb -s "$DEVICE" shell pm list packages -S | grep "$APP" | cut -d/ -f2)
adb -s "$DEVICE" shell am start -n "$pkg/$activity"
_log_info "Waiting for telemetry session event..."
session_detected=false
for i in $(seq 1 30); do
  curl -s "http://localhost:9090/healthz" &>/dev/null && { session_detected=true; break; }
  sleep 1
done
if $session_detected; then
  _log_info "PASS: Telemetry session detected"
  echo "{\"app\":\"$APP\",\"device\":\"$DEVICE\",\"result\":\"PASS\"}"
  exit 0
else
  _log_error "FAIL: No telemetry session detected within 30s"
  echo "{\"app\":\"$APP\",\"device\":\"$DEVICE\",\"result\":\"FAIL\"}"
  exit 1
fi
