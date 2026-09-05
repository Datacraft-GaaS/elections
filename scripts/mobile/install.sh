#!/usr/bin/env bash
# install.sh — Install a Flutter app on physical device(s).
# Usage: scripts/mobile/install.sh <APP> --device <serial|all>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"; shift || true
DEVICE="all"
while [[ $# -gt 0 ]]; do
  case "$1" in --device) DEVICE="$2"; shift 2 ;; *) shift ;; esac
done
[[ -z "$APP" ]] && { _log_error "Usage: scripts/mobile/install.sh <APP> --device <serial|all>"; exit 1; }
APP_PATH=$(get_app_path "$APP")
[[ -z "$APP_PATH" ]] && { _log_error "Unknown app: $APP"; exit 1; }
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${REPO_ROOT}/dist/mobile/${APP}"
apk=""
[[ -f "$BUILD_DIR/${APP}-staging-release.apk" ]] && apk="$BUILD_DIR/${APP}-staging-release.apk"
[[ -z "$apk" && -f "$APP_PATH/build/app/outputs/flutter-apk/app-staging-release.apk" ]] && apk="$APP_PATH/build/app/outputs/flutter-apk/app-staging-release.apk"
[[ -z "$apk" ]] && { _log_error "No APK found — run build.sh first"; exit 1; }
_log_info "Installing $apk..."
[[ "$DEVICE" == "all" ]] && adb install -r "$apk" || adb -s "$DEVICE" install -r "$apk"
_log_info "Installation complete"
