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

# analyze.sh — Run flutter analyze on one or all Flutter apps.
# Usage: scripts/mobile/analyze.sh [app|all] [--fatal-infos]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
TARGET="${1:-all}"; shift || true
EXTRA_ARGS="${*:-}"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$REPO_ROOT"
run_analyze_for() {
  local app_path="$1"
  _log_info "Analyzing $app_path..."
  [[ -f "$app_path/pubspec.yaml" ]] && (cd "$app_path" && flutter analyze --fatal-infos $EXTRA_ARGS) || _log_warn "No pubspec.yaml at $app_path"
}
case "$TARGET" in
  app)
    [[ -z "${2:-}" ]] && { _log_error "Usage: scripts/mobile/analyze.sh app <app-name>"; exit 1; }
    app_path=$(get_app_path "$2")
    [[ -z "$app_path" ]] && { _log_error "Unknown app: $2"; exit 1; }
    run_analyze_for "$app_path"
    ;;
  all)
    local_failed=0
    while IFS='|' read -r app_name app_path _ _ _; do
      [[ "$app_name" =~ ^# ]] && continue
      [[ -z "$app_path" ]] && continue
      run_analyze_for "$app_path" || { _log_error "Analysis failed for $app_name"; local_failed=1; }
    done < "$APP_REGISTRY"
    exit $local_failed
    ;;
  *) _log_error "Usage: scripts/mobile/analyze.sh [app|all] [--fatal-infos]"; exit 1 ;;
esac
_log_info "Analysis complete"
