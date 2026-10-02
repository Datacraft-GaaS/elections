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

# deploy.sh — Deploy Flutter apps to MDM / managed Play Store.
# Usage: scripts/mobile/deploy.sh --env staging|production [--app APP|all] [--track internal|mdm]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
ENV=""; APP="all"; TRACK="internal"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --env) ENV="$2"; shift 2 ;;
    --app) APP="$2"; shift 2 ;;
    --track) TRACK="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -z "$ENV" ]] && { _log_error "Usage: scripts/mobile/deploy.sh --env staging|production"; exit 1; }
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DEPLOY_LOG="${REPO_ROOT}/operations/mdm/deployments.log"
mkdir -p "$(dirname "$DEPLOY_LOG")"
deploy_app() {
  local app="$1"
  local app_path; app_path=$(get_app_path "$app")
  [[ -z "$app_path" ]] && return
  _log_info "Deploying $app to $ENV (track=$TRACK)..."
  if [[ "$TRACK" == "internal" && -f "$app_path/build/app/outputs/bundle/stagingRelease/app-staging-release.aab" ]]; then
    command -v fastlane &>/dev/null && (cd "$app_path" && fastlane run upload_to_play_store package_path="build/app/outputs/bundle/stagingRelease/app-staging-release.aab" track="$TRACK") || _log_warn "fastlane not found"
  fi
  echo "[$(date -Iseconds)] $app $ENV $TRACK" >> "$DEPLOY_LOG"
  _log_info "Deployment recorded for $app"
}
case "$APP" in
  all) while IFS='|' read -r app_name _ _ _ _; do [[ "$app_name" =~ ^# ]] && continue; deploy_app "$app_name"; done < "$APP_REGISTRY" ;;
  *) deploy_app "$APP" ;;
esac
_log_info "Deploy complete for $ENV"
