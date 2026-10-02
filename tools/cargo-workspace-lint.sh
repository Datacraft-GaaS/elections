#!/usr/bin/env bash
# Clippy and rustfmt check for the workspace named by $EEL_WS.

set -uo pipefail
: "${EEL_WS:?EEL_WS must be set by foreach-workspace.sh}"

rc=0
cargo clippy --all-targets --all-features --manifest-path "$EEL_WS" -- -D warnings -q || rc=1
cargo fmt --all --check --manifest-path "$EEL_WS" || rc=1
exit "$rc"
