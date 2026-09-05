#!/usr/bin/env bash
# sbom.sh — Generate SBOM (SPDX + CycloneDX) for a Flutter app.
# Usage: scripts/mobile/sbom.sh <APP>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"
[[ -z "$APP" ]] && { _log_error "Usage: scripts/mobile/sbom.sh <APP>"; exit 1; }
APP_PATH=$(get_app_path "$APP")
[[ -z "$APP_PATH" ]] && { _log_error "Unknown app: $APP"; exit 1; }
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SBOM_DIR="${REPO_ROOT}/dist/mobile/${APP}/sbom"
mkdir -p "$SBOM_DIR"
_log_info "Generating SBOM for $APP..."
if command -v syft &>/dev/null; then
  syft "$APP_PATH" -o spdx-json > "$SBOM_DIR/spdx.json"
  syft "$APP_PATH" -o cyclonedx-json > "$SBOM_DIR/cyclonedx.json"
  _log_info "SBOM generated: $SBOM_DIR"
else
  _log_warn "syft not found — skipping SBOM generation (brew install syft)"
fi
