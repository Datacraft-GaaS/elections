#!/usr/bin/env bash
# smoke.sh — Cold-launch app, assert process alive + crash-free, report PASS/FAIL.
# Usage: scripts/mobile/smoke.sh <APP> --device <serial>
#
# Environment:
#   EEL_TELEMETRY_URL  Optional telemetry/session endpoint probed after launch
#                      (e.g. http://localhost:9090/healthz with
#                      `adb reverse tcp:9090 tcp:9090`). When unset, the
#                      telemetry step is reported as SKIP, not faked.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
source "${SCRIPT_DIR}/lib/device.sh"

APP="${1:-}"; shift || true
DEVICE=""
while [[ $# -gt 0 ]]; do
  case "$1" in --device) DEVICE="$2"; shift 2 ;; *) shift ;; esac
done
[[ -z "$APP" || -z "$DEVICE" ]] && { _log_error "Usage: scripts/mobile/smoke.sh <APP> --device <serial>"; exit 1; }

eel_require_device "$DEVICE" || exit 1

_log_info "Smoke testing $APP on $DEVICE..."
pkg=$(eel_resolve_pkg "$DEVICE" "$APP") || exit 1
_log_info "Resolved package: $pkg"

eel_logcat_clear "$DEVICE"
ms=$(eel_launch "$DEVICE" "$pkg")
if [[ -n "$ms" ]]; then
  _log_info "Cold start TotalTime: ${ms}ms"
  if [[ "$ms" -gt 2000 ]]; then
    _log_warn "Cold start ${ms}ms exceeds 2000ms budget (plan §7) — WARN only on unknown hardware"
  fi
else
  _log_warn "Launch timing unavailable (monkey fallback); skipping cold-start budget check"
fi

# Wait up to 30 s for the process to be alive.
pid=""
for _ in $(seq 1 30); do
  pid=$(eel_pid "$DEVICE" "$pkg")
  [[ -n "$pid" ]] && break
  sleep 1
done
if [[ -z "$pid" ]]; then
  _log_error "FAIL: $pkg has no running process 30s after launch"
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"no-process\"}"
  exit 1
fi
_log_info "Process alive (pid $pid)"

# Crash check: any FATAL EXCEPTION since launch fails the smoke test.
crashlog="$(mktemp)"
if eel_logcat_crashcheck "$DEVICE" "$pkg" "$crashlog"; then
  _log_error "FAIL: crash detected after launch (see $crashlog)"
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"crash\"}"
  exit 1
fi
rm -f "$crashlog"

# Telemetry/session probe — only when an endpoint is configured.
telemetry="SKIP"
if [[ -n "${EEL_TELEMETRY_URL:-}" ]]; then
  for _ in $(seq 1 30); do
    if curl -sf --max-time 2 "$EEL_TELEMETRY_URL" >/dev/null 2>&1; then telemetry="PASS"; break; fi
    sleep 1
  done
  if [[ "$telemetry" != "PASS" ]]; then
    _log_error "FAIL: no telemetry session at $EEL_TELEMETRY_URL within 30s"
    echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"no-telemetry\"}"
    exit 1
  fi
  _log_info "Telemetry session detected"
else
  _log_warn "EEL_TELEMETRY_URL unset — telemetry step SKIP (not probed, not faked)"
fi

_log_info "PASS: $APP alive and crash-free on $DEVICE"
echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"PASS\",\"telemetry\":\"$telemetry\",\"cold_start_ms\":\"${ms:-unknown}\"}"
exit 0
