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

# scripts/replay-tally.sh
# Replay bundle events through replay-tally-rs, derive Merkle root,
# compare against the bundle-declared root.

set -euo pipefail

BUNDLE_ARG="${1:?Usage: replay-tally.sh <bundle-path> <report-dir>}"
REPORT_DIR="${2:?Usage: replay-tally.sh <bundle-path> <report-dir>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/../bin/replay-tally-rs/target/release/replay-tally-rs"
WORK_DIR="$(mktemp -d /tmp/eel-tally-XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT

mkdir -p "$REPORT_DIR"

log() { echo "[replay-tally] $*" >&2; }

if [[ ! -x "$BINARY" ]]; then
    echo "ERROR: binary not found at $BINARY -- run: cd bin/replay-tally-rs && cargo build --release" >&2
    exit 1
fi

# Extract bundle
BUNDLE_DIR="$WORK_DIR/bundle"
mkdir -p "$BUNDLE_DIR"
if [[ -d "$BUNDLE_ARG" ]]; then
    cp -r "$BUNDLE_ARG/." "$BUNDLE_DIR/"
elif file "$BUNDLE_ARG" | grep -q "tar\|gzip"; then
    tar -xf "$BUNDLE_ARG" -C "$BUNDLE_DIR"
else
    cp "$BUNDLE_ARG" "$BUNDLE_DIR/"
fi

EVENTS_FILE="$BUNDLE_DIR/events.jsonl"
MANIFEST_FILE="$BUNDLE_DIR/manifest.json"

if [[ ! -f "$EVENTS_FILE" ]]; then
    echo "ERROR: events.jsonl not found in bundle" >&2
    exit 1
fi

DECLARED_ROOT=""
if [[ -f "$MANIFEST_FILE" ]]; then
    DECLARED_ROOT=$(jq -r '.tally_root // empty' "$MANIFEST_FILE" 2>/dev/null || true)
fi

EVENT_COUNT=$(wc -l < "$EVENTS_FILE")
log "Replaying $EVENT_COUNT events"

RESULT=$("$BINARY" "$EVENTS_FILE")
COMPUTED_ROOT=$(echo "$RESULT" | jq -r '.merkle_root')

log "Computed root: $COMPUTED_ROOT"
log "Declared root: ${DECLARED_ROOT:-<not declared>}"

MATCH="unknown"
if [[ -n "$DECLARED_ROOT" ]]; then
    if [[ "$COMPUTED_ROOT" == "$DECLARED_ROOT" ]]; then
        MATCH="ok"
        log "Roots match."
    else
        MATCH="mismatch"
        log "ROOT MISMATCH: computed=$COMPUTED_ROOT declared=$DECLARED_ROOT"
    fi
fi

REPORT="$REPORT_DIR/replay-tally-$(date +%Y%m%dT%H%M%S).json"
jq -n \
    --arg sv "1" \
    --arg bundle "$(basename "$BUNDLE_ARG")" \
    --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson ec "$EVENT_COUNT" \
    --arg cr "$COMPUTED_ROOT" \
    --arg dr "${DECLARED_ROOT:-}" \
    --arg match "$MATCH" \
    '{schema_version:$sv,bundle:$bundle,timestamp:$ts,event_count:$ec,computed_root:$cr,declared_root:$dr,match:$match}' \
    > "$REPORT"

log "Report: $REPORT"
cat "$REPORT"

if [[ "$MATCH" == "mismatch" ]]; then
    exit 1
fi
