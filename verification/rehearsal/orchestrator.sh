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

# Election-day rehearsal orchestrator (WP-6.1).
# Stages: cluster-check, election-day-mode, load, metrics, witness, bundle, verifier, report.
# Dry-run: EEL_REHEARSAL_MODE=dry -- skips real cluster calls, validates stage hooks.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# ---------- config ----------
EEL_REHEARSAL_MODE="${EEL_REHEARSAL_MODE:-live}"
EEL_CLUSTER_CONTEXT="${EEL_CLUSTER_CONTEXT:-eel-preprod}"
EEL_PROMETHEUS_URL="${EEL_PROMETHEUS_URL:-http://prometheus.eel-preprod:9090}"
EEL_BASE_URL="${EEL_BASE_URL:-https://api.eel-preprod}"
EEL_REPORT_DIR="${EEL_REPORT_DIR:-${REPO_ROOT}/verification/reports}"
REHEARSAL_DATE="$(date -u +%Y-%m-%d)"
REPORT_OUT="${EEL_REPORT_DIR}/rehearsal-${REHEARSAL_DATE}.md"

DRY="${EEL_REHEARSAL_MODE}"
is_dry() { [ "${DRY}" = "dry" ]; }

log() { echo "[rehearsal] $*" >&2; }
die() { echo "[rehearsal] FATAL: $*" >&2; exit 1; }

# Trap to record which stage completed before any exit.
LAST_STAGE="none"
trap 'echo "[rehearsal] last_stage=${LAST_STAGE}" >&2' EXIT

stage() {
  LAST_STAGE="$1"
  log "--- stage: $1 ---"
}

# ---------- stage 1: verify cluster reachable ----------
stage "cluster-check"
if is_dry; then
  log "dry: skipping kubectl check"
else
  kubectl config use-context "${EEL_CLUSTER_CONTEXT}" \
    || die "cluster context '${EEL_CLUSTER_CONTEXT}' not found"
  kubectl -n eel-prod get nodes --no-headers \
    | grep -qE "Ready" \
    || die "cluster has no ready nodes"
  log "cluster ready"
fi

# ---------- stage 2: enable election-day-mode ----------
stage "election-day-mode"
TOGGLE="${REPO_ROOT}/platform/election-day-mode/election-day-mode-cli/toggle.sh"
if is_dry; then
  log "dry: skipping election-day-mode toggle"
elif [ -x "${TOGGLE}" ]; then
  bash "${TOGGLE}" --confirm \
    || die "election-day-mode toggle failed"
  log "election-day-mode ON"
else
  log "WARNING: toggle.sh not found at ${TOGGLE}; skipping"
fi

# ---------- stage 3: run election-day load scenario ----------
stage "load"
SCENARIO="${REPO_ROOT}/loadtest/scenarios/election-day.yaml"
[ -f "${SCENARIO}" ] || die "scenario not found: ${SCENARIO}"

if is_dry; then
  log "dry: skipping load run"
else
  log "launching k6 election-day scenario at scale 1.0"
  EEL_BASE_URL="${EEL_BASE_URL}" \
  EEL_SCALING_FACTOR=1.0 \
  EEL_KIT_COUNT=65000 \
    k6 run \
      --env EEL_BASE_URL="${EEL_BASE_URL}" \
      --env EEL_SCALING_FACTOR=1.0 \
      --env EEL_KIT_COUNT=65000 \
      "${REPO_ROOT}/loadtest/k6/rts-submit.js" &
  K6_PID=$!

  # Give load a head start before Gatling fires.
  sleep 30
  EEL_BASE_URL="${EEL_BASE_URL}" \
  EEL_SCALING_FACTOR=1.0 \
    sbt -f "${REPO_ROOT}/loadtest/gatling/build.sbt" \
      "Gatling/testOnly eel.loadtest.PortalVisitorsSimulation" &
  GATLING_PID=$!

  wait "${K6_PID}" || die "k6 scenario exited with error"
  wait "${GATLING_PID}" || die "Gatling scenario exited with error"
  log "load phase complete"
fi

# ---------- stage 4: collect Prometheus SLA metrics ----------
stage "sla-metrics"
SLA_RESULTS_FILE="${EEL_REPORT_DIR}/sla-results-${REHEARSAL_DATE}.json"
mkdir -p "${EEL_REPORT_DIR}"

if is_dry; then
  log "dry: writing stub SLA results"
  echo '{"status":"dry","slas":[]}' > "${SLA_RESULTS_FILE}"
else
  log "querying Prometheus for each SLA"
  python3 "${SCRIPT_DIR}/collect-sla-metrics.py" \
    --manifest "${SCRIPT_DIR}/sla-manifest.yaml" \
    --prometheus "${EEL_PROMETHEUS_URL}" \
    --out "${SLA_RESULTS_FILE}" \
    || die "SLA metrics collection failed"

  # Check for any failing SLAs.
  FAILS="$(python3 -c "
import json,sys
d=json.load(open('${SLA_RESULTS_FILE}'))
fails=[s for s in d.get('slas',[]) if s.get('pass') is False]
for f in fails: print(f['metric'])
sys.exit(len(fails))
" 2>&1)" || {
    log "SLA FAILURES: ${FAILS}"
    die "One or more SLAs failed: ${FAILS}"
  }
  log "all SLAs pass"
fi

# ---------- stage 5: witness cross-check ----------
stage "witness-crosscheck"
WITNESS_SCRIPT="${SCRIPT_DIR}/witness-crosscheck.sh"
if is_dry; then
  log "dry: skipping witness cross-check"
elif [ -x "${WITNESS_SCRIPT}" ]; then
  bash "${WITNESS_SCRIPT}" \
    || die "witness cross-check failed -- at least one channel disagrees on latest STH"
  log "witness cross-check passed"
else
  die "witness-crosscheck.sh not found or not executable"
fi

# ---------- stage 6: rebuild reproducible bundle ----------
stage "bundle-repro"
REPRO_SCRIPT="${REPO_ROOT}/reproducibility/make-reproduce/scripts/reproduce.sh"
if is_dry; then
  log "dry: skipping bundle reproduce"
elif [ -x "${REPRO_SCRIPT}" ]; then
  bash "${REPRO_SCRIPT}" \
    || die "bundle reproduction failed"
  log "bundle reproduction passed"
else
  log "WARNING: reproduce.sh not found at ${REPRO_SCRIPT}; skipping"
fi

# ---------- stage 7: run verifier-rs on published proofs ----------
stage "verifier"
VERIFIER="${REPO_ROOT}/target/release/verifier-rs"
if is_dry; then
  log "dry: skipping verifier"
elif [ -x "${VERIFIER}" ]; then
  "${VERIFIER}" verify-all \
    --endpoint "${EEL_BASE_URL}/transparency/proofs" \
    || die "verifier-rs reported proof failures"
  log "verifier-rs passed"
else
  log "WARNING: verifier-rs binary not found; skipping"
fi

# ---------- stage 8: render report ----------
stage "report"
TEMPLATE="${SCRIPT_DIR}/report-template.md.tmpl"
[ -f "${TEMPLATE}" ] || die "report template not found: ${TEMPLATE}"

export REHEARSAL_DATE
export EEL_CLUSTER_CONTEXT
export EEL_PROMETHEUS_URL
export EEL_BASE_URL
export EEL_REHEARSAL_MODE

# Load SLA results for inline substitution.
SLA_TABLE="$(python3 -c "
import json,os
path=os.environ.get('SLA_RESULTS_FILE','')
if not path or not os.path.exists(path):
    print('| (dry run) | - | - | - |')
else:
    d=json.load(open(path))
    rows=[]
    for s in d.get('slas',[]):
        status='PASS' if s.get('pass') else 'FAIL'
        rows.append('| {metric} | {measured} | {target} | {status} |'.format(**{
            'metric':s.get('metric','?'),
            'measured':s.get('measured','?'),
            'target':s.get('target','?'),
            'status':status,
        }))
    print('\n'.join(rows))
" 2>/dev/null || echo "| (unavailable) | - | - | - |")"
export SLA_TABLE
export SLA_RESULTS_FILE

envsubst < "${TEMPLATE}" > "${REPORT_OUT}"
log "report written: ${REPORT_OUT}"

stage "done"
log "rehearsal complete. report: ${REPORT_OUT}"
