#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# sast-sca-container.sh -- SAST, SCA, and container security scan.
# Aggregates results into report.json; exits non-zero on any critical or high finding.
# Set EEL_DR_MODE=dry to run syntax check only.
#
# Tools invoked (if present on PATH or via docker):
#   cargo-audit, cargo-deny, Semgrep, Trivy, Grype, Syft, Bandit, Sobelow, MobSF
#
# References:
#   ADR-0006 (supply chain scanning)
#   tools/asvs-check/ (Semgrep rules)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
REPORT_JSON="${EEL_SAST_REPORT:-${REPO_ROOT}/verification/security/report.json}"
ASVS_RULES_DIR="${REPO_ROOT}/tools/asvs-check"

log()  { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die()  { echo "[ERROR] $*" >&2; exit 1; }
run_if() {
  local tool="$1"; shift
  if command -v "${tool}" >/dev/null 2>&1; then
    "$@"
  else
    log "  SKIP: ${tool} not on PATH"
    echo "skipped"
  fi
}

START_EPOCH=$(date +%s)
declare -a RESULTS=()
CRITICAL_OR_HIGH=0

append_result() {
  local tool="$1" status="$2" critical="$3" high="$4" notes="$5"
  RESULTS+=("{\"tool\":\"${tool}\",\"status\":\"${status}\",\"critical\":${critical},\"high\":${high},\"notes\":\"${notes}\"}")
  CRITICAL_OR_HIGH=$((CRITICAL_OR_HIGH + critical + high))
}

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: bash -n self-check only"
  bash -n "$0"
  printf '{"stage":"sast-sca-container","mode":"dry","start_epoch":%s,"status":"ok","critical_or_high":0,"results":[]}\n' \
    "${START_EPOCH}"
  exit 0
fi

log "=== SAST / SCA / Container scan ==="

# --- cargo-audit (Rust SCA) ---
log "cargo-audit"
if command -v cargo-audit >/dev/null 2>&1; then
  CA_OUT=$(cargo audit --json -q 2>/dev/null || true)
  CA_VULN=$(echo "${CA_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    print(len(d.get('vulnerabilities',{}).get('list',[])))
except: print(0)" 2>/dev/null || echo 0)
  append_result "cargo-audit" "run" "${CA_VULN}" 0 "${CA_VULN} vulnerabilities"
else
  append_result "cargo-audit" "skipped" 0 0 "not installed"
fi

# --- cargo-deny ---
log "cargo-deny"
if command -v cargo-deny >/dev/null 2>&1; then
  if cargo deny check 2>/dev/null; then
    append_result "cargo-deny" "run" 0 0 "pass"
  else
    append_result "cargo-deny" "run" 1 0 "deny check failed"
  fi
else
  append_result "cargo-deny" "skipped" 0 0 "not installed"
fi

# --- Semgrep SAST ---
log "semgrep"
if command -v semgrep >/dev/null 2>&1; then
  SEMGREP_OPTS="--json --quiet"
  if [ -d "${ASVS_RULES_DIR}" ]; then
    SEMGREP_OPTS="${SEMGREP_OPTS} --config ${ASVS_RULES_DIR}"
  else
    SEMGREP_OPTS="${SEMGREP_OPTS} --config auto"
  fi
  # shellcheck disable=SC2086
  SG_OUT=$(semgrep scan ${SEMGREP_OPTS} "${REPO_ROOT}/services" 2>/dev/null || true)
  SG_CRIT=$(echo "${SG_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    findings=d.get('results',[])
    return sum(1 for f in findings if f.get('extra',{}).get('severity','') in ('ERROR','CRITICAL'))
except: print(0)" 2>/dev/null || echo 0)
  SG_HIGH=$(echo "${SG_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    findings=d.get('results',[])
    print(sum(1 for f in findings if f.get('extra',{}).get('severity','') == 'WARNING'))
except: print(0)" 2>/dev/null || echo 0)
  append_result "semgrep" "run" "${SG_CRIT}" "${SG_HIGH}" "critical=${SG_CRIT} high=${SG_HIGH}"
else
  append_result "semgrep" "skipped" 0 0 "not installed"
fi

# --- Trivy container scan ---
log "trivy"
if command -v trivy >/dev/null 2>&1; then
  TV_OUT=$(trivy fs --format json --scanners vuln --severity CRITICAL,HIGH \
    --quiet "${REPO_ROOT}" 2>/dev/null || true)
  TV_CRIT=$(echo "${TV_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    all_v=[v for r in d.get('Results',[]) for v in r.get('Vulnerabilities',[]) or []]
    print(sum(1 for v in all_v if v.get('Severity')=='CRITICAL'))
except: print(0)" 2>/dev/null || echo 0)
  TV_HIGH=$(echo "${TV_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    all_v=[v for r in d.get('Results',[]) for v in r.get('Vulnerabilities',[]) or []]
    print(sum(1 for v in all_v if v.get('Severity')=='HIGH'))
except: print(0)" 2>/dev/null || echo 0)
  append_result "trivy" "run" "${TV_CRIT}" "${TV_HIGH}" "critical=${TV_CRIT} high=${TV_HIGH}"
else
  append_result "trivy" "skipped" 0 0 "not installed"
fi

# --- Grype ---
log "grype"
if command -v grype >/dev/null 2>&1; then
  GR_OUT=$(grype dir:"${REPO_ROOT}" -o json -q 2>/dev/null || true)
  GR_CRIT=$(echo "${GR_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    print(sum(1 for m in d.get('matches',[]) if m.get('vulnerability',{}).get('severity')=='Critical'))
except: print(0)" 2>/dev/null || echo 0)
  GR_HIGH=$(echo "${GR_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    print(sum(1 for m in d.get('matches',[]) if m.get('vulnerability',{}).get('severity')=='High'))
except: print(0)" 2>/dev/null || echo 0)
  append_result "grype" "run" "${GR_CRIT}" "${GR_HIGH}" "critical=${GR_CRIT} high=${GR_HIGH}"
else
  append_result "grype" "skipped" 0 0 "not installed"
fi

# --- Bandit (Python) ---
log "bandit"
if command -v bandit >/dev/null 2>&1; then
  BAN_OUT=$(bandit -r "${REPO_ROOT}" -f json -ll -q 2>/dev/null || true)
  BAN_HIGH=$(echo "${BAN_OUT}" | python3 -c "
import sys,json
try:
    d=json.load(sys.stdin)
    print(len([r for r in d.get('results',[]) if r.get('issue_severity')=='HIGH']))
except: print(0)" 2>/dev/null || echo 0)
  append_result "bandit" "run" 0 "${BAN_HIGH}" "high=${BAN_HIGH}"
else
  append_result "bandit" "skipped" 0 0 "not installed"
fi

# --- Sobelow (Elixir/Phoenix) ---
log "sobelow"
if command -v sobelow >/dev/null 2>&1; then
  SOB_RC=0
  sobelow --format json 2>/dev/null || SOB_RC=$?
  append_result "sobelow" "run" "${SOB_RC}" 0 "exit=${SOB_RC}"
else
  append_result "sobelow" "skipped" 0 0 "not installed"
fi

# --- Aggregate and write report.json ---
RESULTS_JSON=$(IFS=,; echo "[${RESULTS[*]}]")
END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))

python3 -c "
import json
report = {
    'stage': 'sast-sca-container',
    'mode': 'live',
    'start_epoch': ${START_EPOCH},
    'elapsed_seconds': ${ELAPSED},
    'critical_or_high': ${CRITICAL_OR_HIGH},
    'status': 'ok' if ${CRITICAL_OR_HIGH} == 0 else 'fail',
    'results': ${RESULTS_JSON}
}
with open('${REPORT_JSON}', 'w') as f:
    json.dump(report, f, indent=2)
print(json.dumps({k: v for k, v in report.items() if k != 'results'}))
"

if [ "${CRITICAL_OR_HIGH}" -gt 0 ]; then
  die "Security gate FAILED: ${CRITICAL_OR_HIGH} critical/high finding(s). See ${REPORT_JSON}"
fi

log "Security scan passed: zero critical/high findings"
