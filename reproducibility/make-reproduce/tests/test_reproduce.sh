#!/usr/bin/env bash
# tests/test_reproduce.sh
# Bash test suite for the reproducibility pipeline using the mini-bundle fixture.
# Each stage is tested independently. Requires: jq, the replay-tally-rs binary.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/.."
FIXTURE="$REPO_ROOT/fixtures/mini-bundle"
BINARY="$REPO_ROOT/bin/replay-tally-rs/target/release/replay-tally-rs"
REPORT_DIR="$(mktemp -d /tmp/eel-test-repro-XXXXXX)"
trap 'rm -rf "$REPORT_DIR"' EXIT

PASS=0
FAIL=0

pass() { echo "[PASS] $1"; PASS=$((PASS + 1)); }
fail() { echo "[FAIL] $1"; FAIL=$((FAIL + 1)); }

# ---------------------------------------------------------------------------
# Stage 1: fixture integrity -- manifest is valid JSON
# ---------------------------------------------------------------------------
if jq '.' "$FIXTURE/manifest.json" > /dev/null 2>&1; then
    pass "manifest.json is valid JSON"
else
    fail "manifest.json failed JSON parse"
fi

# ---------------------------------------------------------------------------
# Stage 2: manifest contains required fields
# ---------------------------------------------------------------------------
SV=$(jq -r '.schema_version' "$FIXTURE/manifest.json")
TALLY_ROOT=$(jq -r '.tally_root' "$FIXTURE/manifest.json")
ART_COUNT=$(jq '.artefacts | length' "$FIXTURE/manifest.json")

if [[ "$SV" == "1" ]]; then
    pass "manifest schema_version is 1"
else
    fail "manifest schema_version wrong: $SV"
fi

if [[ ${#TALLY_ROOT} -eq 64 ]]; then
    pass "manifest tally_root is 64-char hex"
else
    fail "manifest tally_root wrong length: ${#TALLY_ROOT}"
fi

if [[ "$ART_COUNT" -ge 1 ]]; then
    pass "manifest has at least one artefact"
else
    fail "manifest has no artefacts"
fi

# ---------------------------------------------------------------------------
# Stage 3: events file is valid JSON Lines
# ---------------------------------------------------------------------------
INVALID=0
while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if ! echo "$line" | jq '.' > /dev/null 2>&1; then
        INVALID=$((INVALID + 1))
    fi
done < "$FIXTURE/events.jsonl"

if [[ $INVALID -eq 0 ]]; then
    pass "events.jsonl: all lines are valid JSON"
else
    fail "events.jsonl: $INVALID invalid line(s)"
fi

# ---------------------------------------------------------------------------
# Stage 4: replay-tally-rs binary exists and produces JSON output
# ---------------------------------------------------------------------------
if [[ ! -x "$BINARY" ]]; then
    fail "replay-tally-rs binary not found at $BINARY"
else
    RESULT=$("$BINARY" "$FIXTURE/events.jsonl")
    if echo "$RESULT" | jq '.' > /dev/null 2>&1; then
        pass "replay-tally-rs produces valid JSON"
    else
        fail "replay-tally-rs output not valid JSON: $RESULT"
    fi

    # Stage 5: computed root matches fixture-declared root
    COMPUTED=$(echo "$RESULT" | jq -r '.merkle_root')
    if [[ "$COMPUTED" == "$TALLY_ROOT" ]]; then
        pass "computed tally root matches fixture declared root"
    else
        fail "root mismatch: computed=$COMPUTED declared=$TALLY_ROOT"
    fi

    # Stage 6: output schema has expected keys
    HAS_EC=$(echo "$RESULT" | jq 'has("event_count")')
    HAS_MR=$(echo "$RESULT" | jq 'has("merkle_root")')
    if [[ "$HAS_EC" == "true" && "$HAS_MR" == "true" ]]; then
        pass "replay output has event_count and merkle_root keys"
    else
        fail "replay output missing keys: $RESULT"
    fi
fi

# ---------------------------------------------------------------------------
# Stage 7: replay-tally.sh exits 0 on matching fixture
# ---------------------------------------------------------------------------
if [[ -x "$REPO_ROOT/scripts/replay-tally.sh" ]]; then
    if bash "$REPO_ROOT/scripts/replay-tally.sh" "$FIXTURE" "$REPORT_DIR" > /dev/null 2>&1; then
        pass "replay-tally.sh exits 0 on matching fixture"
    else
        fail "replay-tally.sh non-zero exit on matching fixture"
    fi
else
    fail "scripts/replay-tally.sh not executable"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]] && exit 0 || exit 1
