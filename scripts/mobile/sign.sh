#!/usr/bin/env bash
# sign.sh — Sign Flutter app artifacts (Android: apksigner + cosign).
# Usage: scripts/mobile/sign.sh <APP>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"
[[ -z "$APP" ]] && { _log_error "Usage: scripts/mobile/sign.sh <APP>"; exit 1; }
APP_PATH=$(get_app_path "$APP")
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${REPO_ROOT}/dist/mobile/${APP}"
_log_info "Signing $APP..."
if [[ -f "$BUILD_DIR/${APP}-release.apk" ]]; then
  command -v apksigner &>/dev/null && apksigner verify --verbose "$BUILD_DIR/${APP}-release.apk" || _log_warn "apksigner verify failed"
  command -v cosign &>/dev/null && cosign sign --key env://COSIGN_PRIVATE_KEY "$BUILD_DIR/${APP}-release.apk" || _log_warn "cosign sign failed (set COSIGN_PRIVATE_KEY)"
else
  _log_warn "No APK found — run build.sh first"
fi
_log_info "Signing complete for $APP"
