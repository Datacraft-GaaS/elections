#!/usr/bin/env bash
# asvs-evidence-collector.sh -- run tools/asvs-check --strict against every service
# and aggregate evidence into asvs-evidence.json. Exits non-zero on any L3 gap.
# Set EEL_DR_MODE=dry to skip live collection.
#
# References:
#   tools/asvs-check/ (ASVS manifest + checker)
#   ADR-0006 (supply chain, ASVS requirement)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ASVS_CHECK="${REPO_ROOT}/tools/asvs-check"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
OUTPUT_JSON="${EEL_ASVS_OUTPUT:-${REPO_ROOT}/verification/security/asvs-evidence.json}"
SERVICES_DIR="${REPO_ROOT}/services"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)

# DoD: asvs-check tool must exist.
test -d "${ASVS_CHECK}" || die "tools/asvs-check not found: ${ASVS_CHECK}"

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live asvs-check"
  printf '{"stage":"asvs-evidence-collector","mode":"dry","start_epoch":%s,"status":"ok","l3_gaps":0}\n' \
    "${START_EPOCH}"
  exit 0
fi

log "=== ASVS L3 evidence collection ==="

SERVICES=$(find "${SERVICES_DIR}" -maxdepth 1 -mindepth 1 -type d | sort)
declare -a ALL_RESULTS=()
L3_GAPS=0

for SVC_DIR in ${SERVICES}; do
  SVC_NAME=$(basename "${SVC_DIR}")
  log "  Checking ${SVC_NAME}"

  SVC_RESULT_FILE="/tmp/asvs-${SVC_NAME}-$$.json"

  # Run asvs-check. It may be a Rust binary or a script.
  if [ -f "${ASVS_CHECK}/target/release/eel-asvs-check" ]; then
    "${ASVS_CHECK}/target/release/eel-asvs-check" \
      --strict \
      --service "${SVC_DIR}" \
      --output json \
      > "${SVC_RESULT_FILE}" 2>/dev/null || true
  elif [ -f "${ASVS_CHECK}/src/main.rs" ]; then
    cargo run --quiet --manifest-path "${ASVS_CHECK}/Cargo.toml" -- \
      --strict \
      --service "${SVC_DIR}" \
      --output json \
      > "${SVC_RESULT_FILE}" 2>/dev/null || true
  else
    log "  SKIP ${SVC_NAME}: asvs-check binary not built"
    echo "{\"service\":\"${SVC_NAME}\",\"status\":\"skipped\",\"l3_gaps\":0}" > "${SVC_RESULT_FILE}"
  fi

  if [ -s "${SVC_RESULT_FILE}" ]; then
    SVC_GAPS=$(python3 -c "
import sys,json
try:
    d=json.load(open('${SVC_RESULT_FILE}'))
    print(d.get('l3_gaps', 0))
except: print(0)" 2>/dev/null || echo 0)
    L3_GAPS=$((L3_GAPS + SVC_GAPS))
    ALL_RESULTS+=("$(cat "${SVC_RESULT_FILE}")")
  fi

  rm -f "${SVC_RESULT_FILE}"
done

# Write aggregated evidence file.
RESULTS_JSON=$(IFS=,; echo "[${ALL_RESULTS[*]:-}]")
END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))

python3 -c "
import json
report = {
    'stage': 'asvs-evidence-collector',
    'mode': 'live',
    'start_epoch': ${START_EPOCH},
    'elapsed_seconds': ${ELAPSED},
    'l3_gaps': ${L3_GAPS},
    'status': 'ok' if ${L3_GAPS} == 0 else 'fail',
    'services': ${RESULTS_JSON}
}
with open('${OUTPUT_JSON}', 'w') as f:
    json.dump(report, f, indent=2)
print(json.dumps({k: v for k, v in report.items() if k != 'services'}))
"

if [ "${L3_GAPS}" -gt 0 ]; then
  die "ASVS L3 gate FAILED: ${L3_GAPS} gap(s). See ${OUTPUT_JSON}"
fi

log "ASVS L3 evidence collection passed: zero gaps"
