#!/usr/bin/env bash
# Clippy and rustfmt check for the workspace named by $EEL_WS.

set -uo pipefail
: "${EEL_WS:?EEL_WS must be set by foreach-workspace.sh}"

rc=0
# Deny clippy::all rather than a blanket -D warnings. `missing_docs` is
# warn-level in the 222 crates that opt into it and AGENTS.md states those
# warnings are tolerated; -D warnings promotes ~2000 of them to errors and
# buries the handful of real findings. Note that `-D warnings -A missing_docs`
# does NOT work: the warnings group overrides the specific allow regardless of
# flag order, so the allow has to come from not denying the group at all.
cargo clippy --all-targets --all-features --manifest-path "$EEL_WS" -- \
  -D clippy::all -A missing_docs || rc=1
cargo fmt --all --check --manifest-path "$EEL_WS" || rc=1
exit "$rc"
