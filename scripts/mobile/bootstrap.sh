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

# bootstrap.sh — One-time dev/CI env setup for Flutter mobile apps.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"
ENV="${1:-dev}"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$REPO_ROOT"
_log_info "Bootstrapping mobile env for $ENV..."
check_flutter_version
if ! command -v flutter &>/dev/null; then
  _log_error "Flutter not found. Install from https://flutter.dev/docs/get-started/install"
  exit 1
fi
if [[ -z "${ANDROID_HOME:-}" ]]; then
  [[ -d "$HOME/Library/Android/sdk" ]] && export ANDROID_HOME="$HOME/Library/Android/sdk"
  [[ -d "/usr/local/lib/android/sdk" ]] && export ANDROID_HOME="/usr/local/lib/android/sdk"
fi
[[ -n "${ANDROID_HOME:-}" ]] && _log_info "Android SDK: $ANDROID_HOME" || _log_warn "ANDROID_HOME not set"
for tool in cosign syft fastlane; do
  command -v "$tool" &>/dev/null && _log_info "$tool: $(command -v "$tool")" || _log_warn "$tool not found"
done
[[ "$ENV" == "dev" ]] && fetch_signing_material "dev" || true
flutter doctor -v || _log_warn "flutter doctor had issues"
_log_info "Mobile bootstrap complete for $ENV"
