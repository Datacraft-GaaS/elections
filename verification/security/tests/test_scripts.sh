#!/usr/bin/env bash
# tests/test_scripts.sh -- bats-style tests for security harness scripts.
# Runs: bash -n on every .sh, shellcheck if available, yaml validation on every .yaml.
# Dry-runs each script with EEL_DR_MODE=dry; asserts exit 0 and JSON output.
#
# Usage: bash tests/test_scripts.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PASS=0
FAIL=0
ERRORS=""

log()  { echo "[$(date -u +%H:%M:%SZ)] $*"; }
ok()   { log "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { log "  FAIL: $*"; FAIL=$((FAIL + 1)); ERRORS="${ERRORS}\n  - $*"; }

# --- syntax check every .sh ---
log "=== bash -n syntax check ==="
while IFS= read -r -d '' SH; do
  REL="${SH#${SCRIPT_DIR}/}"
  if bash -n "${SH}" 2>/dev/null; then
    ok "bash -n: ${REL}"
  else
    fail "bash -n: ${REL}"
  fi
done < <(find "${SCRIPT_DIR}" -name "*.sh" -print0)

# --- shellcheck (optional) ---
log "=== shellcheck ==="
if command -v shellcheck >/dev/null 2>&1; then
  while IFS= read -r -d '' SH; do
    REL="${SH#${SCRIPT_DIR}/}"
    if shellcheck -S warning "${SH}" 2>/dev/null; then
      ok "shellcheck: ${REL}"
    else
      fail "shellcheck: ${REL}"
    fi
  done < <(find "${SCRIPT_DIR}" -name "*.sh" -print0)
else
  log "  shellcheck not installed; skipping"
fi

# --- YAML validation ---
log "=== YAML validation ==="
while IFS= read -r -d '' YF; do
  REL="${YF#${SCRIPT_DIR}/}"
  if python3 -c "import yaml,sys; yaml.safe_load(open('${YF}'))" 2>/dev/null; then
    ok "YAML valid: ${REL}"
  else
    fail "YAML invalid: ${REL}"
  fi
done < <(find "${SCRIPT_DIR}" \( -name "*.yaml" -o -name "*.yml" \) -print0)

# --- dry-run each top-level and sub-script ---
run_dry() {
  local label="$1"
  local script="$2"

  if [ ! -f "${SCRIPT_DIR}/${script}" ]; then
    fail "${label}: script not found: ${script}"
    return
  fi

  local out rc=0
  out=$(EEL_DR_MODE=dry EEL_TICKET=TEST-DRY bash "${SCRIPT_DIR}/${script}" 2>/dev/null) || rc=$?

  if [ "${rc}" -ne 0 ]; then
    fail "${label}: exited ${rc} in dry mode"
    return
  fi

  local status
  status=$(echo "${out}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status',''))" 2>/dev/null || echo "")

  if [ "${status}" = "ok" ] || [ "${status}" = "skipped" ]; then
    ok "${label}: exit 0, status=${status}"
  else
    fail "${label}: unexpected status '${status}' (output: ${out})"
  fi
}

log "=== dry-run tests ==="
run_dry "sast-sca-container"          "sast-sca-container.sh"
run_dry "dast/zap-baseline"           "dast/zap-baseline.sh"
run_dry "fuzz/run-corpus"             "fuzz/run-corpus.sh"
run_dry "asvs-evidence-collector"     "asvs-evidence-collector.sh"
run_dry "pq-hybrid/testssl"           "pq-hybrid/testssl.sh"
run_dry "pq-hybrid/pq-disabled"       "pq-hybrid/pq-disabled-client-check.sh"

# --- report template envsubst smoke test ---
log "=== report-template envsubst smoke test ==="
TMPL="${SCRIPT_DIR}/report-template.md.tmpl"
if [ -f "${TMPL}" ]; then
  OUT_MD=$(REPORT_DATE=2026-08-21 REPORT_TICKET=TEST REPORT_ENV=test \
    REPORT_OPERATORS=test CARGO_AUDIT_STATUS=run CARGO_AUDIT_CRITICAL=0 \
    CARGO_AUDIT_HIGH=0 CARGO_AUDIT_NOTES=pass CARGO_DENY_STATUS=run \
    CARGO_DENY_CRITICAL=0 CARGO_DENY_NOTES=pass SEMGREP_STATUS=run \
    SEMGREP_CRITICAL=0 SEMGREP_HIGH=0 SEMGREP_NOTES=pass TRIVY_STATUS=run \
    TRIVY_CRITICAL=0 TRIVY_HIGH=0 TRIVY_NOTES=pass GRYPE_STATUS=run \
    GRYPE_CRITICAL=0 GRYPE_HIGH=0 GRYPE_NOTES=pass BANDIT_STATUS=run \
    BANDIT_HIGH=0 BANDIT_NOTES=pass SOBELOW_STATUS=skipped SOBELOW_CRITICAL=0 \
    SOBELOW_NOTES=pass SAST_GATE=PASS ZAP_TARGET=https://test ZAP_HIGH_ALERTS=0 \
    ZAP_REPORT_PATH=/tmp/test NUCLEI_FINDINGS=0 NUCLEI_CUSTOM_RESULTS=none \
    FUZZ_TABLE="| test | 0 | 0% | 0 |" FUZZ_TOTAL_CRASHES=0 \
    PENTEST_OPEN_ITEMS="| none |" ASVS_SERVICES_COUNT=0 ASVS_L3_GAPS=0 \
    ASVS_EVIDENCE_FILE=/dev/null ASVS_GATE=PASS PQ_TABLE="| test | yes | yes | yes |" \
    SIGNOFF_SEC=test SIGNOFF_OPS=test SIGNOFF_PENTEST=test \
    envsubst < "${TMPL}")
  if echo "${OUT_MD}" | grep -q "Security Verification Report"; then
    ok "report-template.md.tmpl: envsubst renders"
  else
    fail "report-template.md.tmpl: envsubst output missing expected header"
  fi
else
  fail "report-template.md.tmpl not found"
fi

log ""
log "Results: ${PASS} passed, ${FAIL} failed"
if [ "${FAIL}" -gt 0 ]; then
  printf "Failures:%b\n" "${ERRORS}"
  exit 1
fi
exit 0
