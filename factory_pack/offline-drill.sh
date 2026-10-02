#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# offline-drill.sh — Airplane-mode drill: app must stay alive and crash-free
# while offline, and recover after reconnect. Record-level dedup verification
# needs a backend hook (see EEL_DEDUP_CHECK below).
#
# Usage: scripts/mobile/offline-drill.sh <APP> --device <serial>
#
# Environment:
#   EEL_DEDUP_CHECK  Optional shell command run after reconnect to verify
#                    exactly-once delivery (e.g. query the BFF audit log).
#                    Must exit 0 on success. When unset, dedup is reported
#                    as SKIP — explicitly, never as a silent pass.
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
  _log_error "Usage: scripts/mobile/offline-drill.sh <APP> --device <serial>"
  exit 1
fi

eel_require_device "$DEVICE" || exit 1
pkg=$(eel_resolve_pkg "$DEVICE" "$APP") || exit 1
_log_info "Offline drill for $APP ($pkg) on $DEVICE..."

eel_airplane "$DEVICE" on || exit 1
_log_info "Airplane mode ON (verified)"

eel_logcat_clear "$DEVICE"
eel_launch "$DEVICE" "$pkg" >/dev/null
sleep 5
pid_offline=$(eel_pid "$DEVICE" "$pkg")
if [[ -z "$pid_offline" ]]; then
  _log_error "FAIL: app has no process while offline"
  eel_airplane "$DEVICE" off || true
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"offline-no-process\"}"
  exit 1
fi
crashlog="$(mktemp)"
if eel_logcat_crashcheck "$DEVICE" "$pkg" "$crashlog"; then
  _log_error "FAIL: app crashed while offline (see $crashlog)"
  eel_airplane "$DEVICE" off || true
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"offline-crash\"}"
  exit 1
fi
rm -f "$crashlog"
_log_info "App alive and crash-free while offline (pid $pid_offline)"

eel_airplane "$DEVICE" off || exit 1
_log_info "Network restored (verified)"
sleep 5
if [[ -z "$(eel_pid "$DEVICE" "$pkg")" ]]; then
  _log_error "FAIL: app died after reconnect"
  echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"reconnect-death\"}"
  exit 1
fi

dedup="SKIP"
if [[ -n "${EEL_DEDUP_CHECK:-}" ]]; then
  _log_info "Running dedup check: $EEL_DEDUP_CHECK"
  if eval "$EEL_DEDUP_CHECK"; then
    dedup="PASS"
  else
    _log_error "FAIL: dedup check failed"
    echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"FAIL\",\"reason\":\"dedup\"}"
    exit 1
  fi
else
  _log_warn "EEL_DEDUP_CHECK unset — record-level dedup SKIP (needs backend hook, not asserted)"
fi

_log_info "Offline drill complete for $APP (dedup: $dedup)"
echo "{\"app\":\"$APP\",\"package\":\"$pkg\",\"device\":\"$DEVICE\",\"result\":\"PASS\",\"dedup\":\"$dedup\"}"
exit 0
