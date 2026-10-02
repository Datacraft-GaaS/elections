#!/usr/bin/env bash
# Release build of the workspace named by $EEL_WS.

set -uo pipefail
: "${EEL_WS:?EEL_WS must be set by foreach-workspace.sh}"

exec cargo build --release --workspace --manifest-path "$EEL_WS" -q
