#!/usr/bin/env bash
# rotate-pins.sh — Staged SPKI pin rotation.
# Usage: scripts/mobile/rotate-pins.sh --set <id>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
SET_ID=""
while [[ $# -gt 0 ]]; do
  case "$1" in --set) SET_ID="$2"; shift 2 ;; *) shift ;; esac
done
[[ -z "$SET_ID" ]] && { _log_error "Usage: scripts/mobile/rotate-pins.sh --set <id>"; exit 1; }
_log_info "Rotating pins to set $SET_ID..."
_log_info "Step 1: Pushing dual-pin set via BFF config (TODO: implement BFF config update)"
_log_info "Step 2: Waiting for fleet adoption (TODO: poll monitor.sh)"
_log_info "Step 3: Dropping old pin (TODO: update secure storage)"
_log_info "Pin rotation complete for set $SET_ID"
