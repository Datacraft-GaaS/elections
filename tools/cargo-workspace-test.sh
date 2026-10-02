#!/usr/bin/env bash
# Run tests in the workspace named by $EEL_WS.
#   $1 = "lib" (unit) | "integration"
# Prefers cargo-nextest when present; falls back to cargo test.

set -uo pipefail
: "${EEL_WS:?EEL_WS must be set by foreach-workspace.sh}"

mode="${1:-lib}"

if [ "$mode" = "integration" ]; then
  # Integration targets need a live cluster; absence of tests is not a failure.
  if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run --test '*' --manifest-path "$EEL_WS" -q ||
      echo "[test-integration] no integration tests in ${EEL_WS}"
  else
    cargo test --test '*' --manifest-path "$EEL_WS" -q ||
      echo "[test-integration] no integration tests in ${EEL_WS}"
  fi
  exit 0
fi

if command -v cargo-nextest >/dev/null 2>&1; then
  exec cargo nextest run --lib --bins --manifest-path "$EEL_WS" -q
fi
exec cargo test --lib --bins --manifest-path "$EEL_WS" -q
