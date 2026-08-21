#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PASS=0
FAIL=0

run_check() {
  local lang="$1"
  local script="${SCRIPT_DIR}/${lang}/generate.sh"
  if [[ ! -f "$script" ]]; then
    echo "FAIL [${lang}]: generate.sh not found at ${script}"
    FAIL=$((FAIL + 1))
    return
  fi
  if bash "$script" --check; then
    echo "PASS [${lang}]"
    PASS=$((PASS + 1))
  else
    echo "FAIL [${lang}]: generate.sh --check exited non-zero"
    FAIL=$((FAIL + 1))
  fi
}

run_check dart
run_check ts
run_check py
run_check rs

echo ""
echo "Results: ${PASS} passed, ${FAIL} failed"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
