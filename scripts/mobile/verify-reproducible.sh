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

# verify-reproducible.sh — Build an app 3x and compare SHA-256 hashes.
# Usage: scripts/mobile/verify-reproducible.sh <APP>
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
APP="${1:-}"
[[ -z "$APP" ]] && { _log_error "Usage: scripts/mobile/verify-reproducible.sh <APP>"; exit 1; }
APP_PATH=$(get_app_path "$APP")
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TEMP_DIR=$(mktemp -d); trap "rm -rf $TEMP_DIR" EXIT
_log_info "Verifying reproducible build for $APP (3 builds)..."
declare -a hashes
for i in 1 2 3; do
  _log_info "Build $i/3..."
  build_dir="$TEMP_DIR/build_$i"
  mkdir -p "$build_dir"
  FLAVOR="staging" REPO_ROOT="$REPO_ROOT" SCRIPT_DIR="$SCRIPT_DIR" bash "$SCRIPT_DIR/build.sh" "$APP" --flavor staging --target-platform android
  [[ -f "$APP_PATH/build/app/outputs/flutter-apk/app-staging-release.apk" ]] && \
    cp "$APP_PATH/build/app/outputs/flutter-apk/app-staging-release.apk" "$build_dir/app.apk" && \
    hashes+=("$(sha256sum "$build_dir/app.apk" | cut -d' ' -f1)") || { _log_error "Build $i failed"; exit 1; }
done
first="${hashes[0]}"
all_same=true
for h in "${hashes[@]:1}"; do
  [[ "$h" != "$first" ]] && { all_same=false; break; }
done
if $all_same; then
  _log_info "PASS: All 3 builds identical (SHA-256: $first)"
  exit 0
else
  _log_error "FAIL: Builds produced different artifacts"
  for i in "${!hashes[@]}"; do _log_error "  Build $((i+1)): ${hashes[$i]}"; done
  exit 1
fi
