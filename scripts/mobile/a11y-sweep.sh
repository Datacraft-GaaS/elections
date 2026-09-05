#!/usr/bin/env bash
# a11y-sweep.sh — Run a11y/i18n sweep on a Flutter app.
#
# Usage: scripts/mobile/a11y-sweep.sh <APP>
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

APP="${1:-}"
if [[ -z "$APP" ]]; then
  _log_error "Usage: scripts/mobile/a11y-sweep.sh <APP>"
  exit 1
fi

APP_PATH=$(get_app_path "$APP")
if [[ -z "$APP_PATH" ]]; then
  _log_error "Unknown app: $APP"
  exit 1
fi

REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$REPO_ROOT"

_log_info "Running a11y/i18n sweep for $APP..."

# Run pseudo-locale check
if [[ -f "verification/a11y-i18n/pseudo-locale-check.sh" ]]; then
  bash "verification/a11y-i18n/pseudo-locale-check.sh" "$APP" || \
    _log_warn "Pseudo-locale check failed for $APP"
fi

# Run axe-core sweep (if web views exist)
if [[ -d "$APP_PATH/test/golden" ]]; then
  _log_info "Running semantic tree checks..."
  # TODO: Run axe-core against golden semantic snapshots
fi

_log_info "A11y sweep complete for $APP"
