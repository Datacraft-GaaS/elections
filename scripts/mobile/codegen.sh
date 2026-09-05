#!/usr/bin/env bash
# codegen.sh — Run code generation for one or all Flutter apps.
# Usage: scripts/mobile/codegen.sh [app|all]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
TARGET="${1:-all}"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$REPO_ROOT"
run_codegen_for() {
  local app_path="$1"
  _log_info "Running codegen for $app_path..."
  [[ -f "$app_path/pubspec.yaml" ]] && (cd "$app_path" && flutter pub get && dart run build_runner build --delete-conflicting-outputs) || _log_warn "No pubspec.yaml at $app_path"
}
case "$TARGET" in
  app)
    [[ -z "${2:-}" ]] && { _log_error "Usage: scripts/mobile/codegen.sh app <app-name>"; exit 1; }
    app_path=$(get_app_path "$2")
    [[ -z "$app_path" ]] && { _log_error "Unknown app: $2"; exit 1; }
    run_codegen_for "$app_path"
    ;;
  all)
    while IFS='|' read -r app_name app_path _ _ _; do
      [[ "$app_name" =~ ^# ]] && continue
      [[ -z "$app_path" ]] && continue
      run_codegen_for "$app_path"
    done < "$APP_REGISTRY"
    ;;
  *) _log_error "Usage: scripts/mobile/codegen.sh [app|all]"; exit 1 ;;
esac
_log_info "Codegen complete"
