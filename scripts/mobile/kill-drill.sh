#!/usr/bin/env bash
# kill-drill.sh — Background-kill harness: force-stop the app at each FSM
# stage, relaunch, and assert (a) a new process comes up, (b) no FATAL
# EXCEPTION in logcat. Any failed stage fails the drill.
#
# Usage: scripts/mobile/kill-drill.sh <APP> --device <serial>
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
source "${SCRIPT_DIR}/lib/device.sh"

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

eel_require_device "$DEVICE" || exit 1
pkg=$(eel_resolve_pkg "$DEVICE" "$APP") || exit 1
_log_info "Kill-drill for $APP ($pkg) on $DEVICE..."

failures=0
for state in cold integrityCheck attestation config auth roleRoute ready; do
  _log_info "--- Stage: $state ---"
  eel_logcat_clear "$DEVICE"

  eel_launch "$DEVICE" "$pkg" >/dev/null
  sleep 2
  pid_before=$(eel_pid "$DEVICE" "$pkg")
  if [[ -z "$pid_before" ]]; then
    _log_error "FAIL [$state]: app did not start (no pid)"
    failures=$((failures + 1))
    continue
  fi

  # Deterministic kill: force-stop (am kill only affects background procs).
  adb -s "$DEVICE" shell am force-stop "$pkg" >/dev/null 2>&1
  sleep 1
  if [[ -n "$(eel_pid "$DEVICE" "$pkg")" ]]; then
    _log_error "FAIL [$state]: process survived force-stop (pid $(eel_pid "$DEVICE" "$pkg"))"
    failures=$((failures + 1))
    continue
  fi

  # Relaunch and assert recovery.
  eel_launch "$DEVICE" "$pkg" >/dev/null
  pid_after=""
  for _ in $(seq 1 15); do
    pid_after=$(eel_pid "$DEVICE" "$pkg")
    [[ -n "$pid_after" ]] && break
    sleep 1
  done
  if [[ -z "$pid_after" ]]; then
    _log_error "FAIL [$state]: app did not recover after kill"
    failures=$((failures + 1))
    continue
  fi
  if [[ "$pid_after" == "$pid_before" ]]; then
    _log_error "FAIL [$state]: pid unchanged after force-stop ($pid_after) — kill did not take effect"
    failures=$((failures + 1))
    continue
  fi

  crashlog="$(mktemp)"
  if eel_logcat_crashcheck "$DEVICE" "$pkg" "$crashlog"; then
    _log_error "FAIL [$state]: crash after relaunch (see $crashlog)"
    failures=$((failures + 1))
    continue
  fi
  rm -f "$crashlog"
  _log_info "PASS [$state]: killed pid $pid_before, recovered as pid $pid_after, no crash"
done

if [[ "$failures" -gt 0 ]]; then
  _log_error "Kill-drill FAILED: $failures stage(s) failed"
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"failed_stages\":$failures}"
  exit 1
fi
_log_info "Kill-drill complete for $APP: all 7 stages recovered, no crashes"
echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"PASS\"}"
exit 0
