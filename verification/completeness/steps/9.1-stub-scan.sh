#!/usr/bin/env bash
# 9.1-stub-scan.sh: Run tools/completeness/scan --strict.
# Expects zero findings outside .completeness-allowlist.yaml.
set -euo pipefail

REPO_ROOT="${EEL_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)}"
AUDIT_MODE="${EEL_AUDIT_MODE:-}"
FIXTURES_DIR="${EEL_FIXTURES_DIR:-${REPO_ROOT}/verification/completeness/fixtures/mini-audit}"
BINARY="${REPO_ROOT}/target/release/eel-completeness"
ALLOWLIST="${REPO_ROOT}/.completeness-allowlist.yaml"

log() { printf '[9.1-stub-scan] %s\n' "$*" >&2; }

if [[ -n "${AUDIT_MODE}" && "${AUDIT_MODE}" == "dry" ]]; then
  log "dry mode: skipping real scan; asserting fixtures are present"
  [[ -d "${FIXTURES_DIR}" ]] || { log "fixtures dir missing: ${FIXTURES_DIR}"; exit 1; }
  log "PASS (dry)"
  exit 0
fi

if [[ ! -x "${BINARY}" ]]; then
  log "eel-completeness binary not found; build with: cargo build --release -p eel-completeness"
  exit 1
fi

ALLOWLIST_ARG=""
if [[ -f "${ALLOWLIST}" ]]; then
  ALLOWLIST_ARG="--allowlist=${ALLOWLIST}"
fi

"${BINARY}" --root "${REPO_ROOT}" ${ALLOWLIST_ARG} scan --strict --format json \
  || { log "FAIL: stub findings detected outside allowlist"; exit 1; }

log "PASS: zero stub findings"
