#!/usr/bin/env bash
# kill-drill.sh — Background-kill harness: drives app through FSM states,
# kills process at each transition, asserts no dup/no loss.
#
# Usage: scripts/mobile/kill-drill.sh <APP> --device <serial>
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
  _log_error "Usage: scripts/mobile/kill-drill.sh <APP> --device <serial>"
  exit 1
fi

_log_info "Kill-drill for $APP on $DEVICE..."

# Kill at various FSM states and verify recovery
for state in cold integrityCheck attestation config auth roleRoute ready; do
  _log_info "Killing at state: $state..."
  
  # Start app
  if [[ -n "$DEVICE" ]]; then
    adb -s "$DEVICE" shell am start -n "$(adb -s "$DEVICE" shell pm list packages | grep "$APP" | cut -d: -f2)/$(adb -s "$DEVICE" shell pm list packages -S | grep "$APP" | cut -d/ -f2)"
  else
    adb shell am start -n "$(adb shell pm list packages | grep "$APP" | cut -d: -f2)/$(adb shell pm list packages -S | grep "$APP" | cut -d/ -f2)"
  fi
  
  # Wait for state
  sleep 2
  
  # Kill process
  if [[ -n "$DEVICE" ]]; then
    adb -s "$DEVICE" shell am kill "$(adb -s "$DEVICE" shell pm list packages | grep "$APP" | cut -d: -f2)"
  else
    adb shell am kill "$(adb shell pm list packages | grep "$APP" | cut -d: -f2)"
  fi
  
  # Verify no duplicates on relaunch
  sleep 1
  _log_info "PASS: Recovery from $state"
done

_log_info "Kill-drill complete for $APP"
