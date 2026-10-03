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
# abis-report-publisher's `pgp` feature carries a deliberate compile_error! that
# fires unless the rpgp dependency is present, so --all-features fails that
# crate by design. It is a member of the root workspace, so key the exemption on
# the crate rather than on a workspace path.
feature_flag="--all-features"
if find "$(dirname "$EEL_WS")" -maxdepth 4 -type d -name abis-report-publisher 2>/dev/null | grep -q .; then
  feature_flag=""
fi

# shellcheck disable=SC2086  # feature_flag is intentionally empty for abis-svc
cargo clippy --all-targets $feature_flag --manifest-path "$EEL_WS" -- \
  -D clippy::all -A missing_docs || rc=1
cargo fmt --all --check --manifest-path "$EEL_WS" || rc=1

exit "$rc"
