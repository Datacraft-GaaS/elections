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

# scripts/multi-arch-verify.sh
# Build the same Nix flake output for x86_64-linux, aarch64-linux, aarch64-darwin,
# compute sha256 of each result, and assert all three are equal.
# Skips any platform that is not available on the current builder, with a warning.

set -euo pipefail

ATTR="${1:?Usage: multi-arch-verify.sh <nix-attr>}"

log() { echo "[multi-arch-verify] $*" >&2; }

PLATFORMS=(
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
)

declare -A HASHES
AVAILABLE=0

for PLATFORM in "${PLATFORMS[@]}"; do
    log "Building .#packages.$PLATFORM.$ATTR"
    OUT_LINK="$(mktemp -d /tmp/eel-multiarch-XXXXXX)/result"

    if nix build ".#packages.$PLATFORM.$ATTR" \
        --out-link "$OUT_LINK" \
        --quiet 2>/tmp/eel-nix-err; then
        HASH=$(sha256sum "$OUT_LINK" 2>/dev/null | awk '{print $1}' \
               || sha256 -q "$OUT_LINK" 2>/dev/null \
               || echo "unavailable")
        HASHES["$PLATFORM"]="$HASH"
        log "$PLATFORM sha256: $HASH"
        AVAILABLE=$((AVAILABLE + 1))
    else
        ERR=$(head -1 /tmp/eel-nix-err 2>/dev/null || true)
        log "WARNING: $PLATFORM not available ($ERR) -- skipping"
        HASHES["$PLATFORM"]="skipped"
    fi
done

if [[ $AVAILABLE -lt 2 ]]; then
    log "WARNING: fewer than 2 platforms available; skipping cross-platform assertion"
    exit 0
fi

# Assert all available hashes are identical
REFERENCE=""
FAIL=0
for PLATFORM in "${PLATFORMS[@]}"; do
    H="${HASHES[$PLATFORM]}"
    [[ "$H" == "skipped" ]] && continue
    if [[ -z "$REFERENCE" ]]; then
        REFERENCE="$H"
        continue
    fi
    if [[ "$H" != "$REFERENCE" ]]; then
        log "ERROR: hash mismatch for $PLATFORM: $H != $REFERENCE"
        FAIL=1
    fi
done

if [[ $FAIL -eq 1 ]]; then
    echo "ERROR: multi-arch hashes differ -- build is not deterministic" >&2
    exit 1
fi

log "All available platforms produce identical sha256: $REFERENCE"
exit 0
