#!/usr/bin/env bash
# Apply rustfmt to the workspace named by $EEL_WS.
# Formatting is best-effort: a failure here must not abort `just fmt`.

set -uo pipefail
: "${EEL_WS:?EEL_WS must be set by foreach-workspace.sh}"

cargo fmt --all --manifest-path "$EEL_WS" 2>/dev/null || true
exit 0
