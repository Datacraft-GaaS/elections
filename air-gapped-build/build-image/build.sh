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

# air-gapped-build/build-image/build.sh
# Offline nix build with Nexus proxy env vars set.
# Usage: build.sh <nix-attr>
# Example: build.sh packages.x86_64-linux.eel-vprs-svc

set -euo pipefail

ATTR="${1:?Usage: build.sh <nix-attr>}"

NEXUS_URL="${NEXUS_URL:-http://nexus.internal:8081}"

log() { echo "[build] $*" >&2; }

log "Offline nix build: .#$ATTR"
log "Nexus: $NEXUS_URL"

# Export Nexus proxy env vars for cargo, pip, npm, etc. so any build hooks
# that try to fetch use the local mirror.
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export CARGO_HTTP_CAINFO="${NEXUS_CA:-}"
export PIP_INDEX_URL="$NEXUS_URL/repository/eel-pypi-proxy/simple/"
export NPM_CONFIG_REGISTRY="$NEXUS_URL/repository/eel-npm-proxy/"
export NIX_SUBSTITUTERS="$NEXUS_URL/repository/eel-nix-cache-proxy/"

nix build ".#$ATTR" \
    --offline \
    --substituters "$NIX_SUBSTITUTERS" \
    --trusted-substituters "$NIX_SUBSTITUTERS" \
    --option accept-flake-config false \
    "$@"

OUT=$(nix path-info ".#$ATTR")
log "Build output: $OUT"
echo "$OUT"
