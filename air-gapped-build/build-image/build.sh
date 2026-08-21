#!/usr/bin/env bash
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
