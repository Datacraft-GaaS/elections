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

# run-audit.sh: Orchestrate the completeness audit (WP-6.7, sections 9.1-9.9).
# Aggregates per-step results into completeness-report.json + completeness-report.md.
# Exits non-zero if any step fails.
#
# Usage:
#   bash verification/completeness/run-audit.sh
#   EEL_AUDIT_MODE=dry bash verification/completeness/run-audit.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
STEPS_DIR="${SCRIPT_DIR}/steps"
REPORT_JSON="${SCRIPT_DIR}/completeness-report.json"

AUDIT_MODE="${EEL_AUDIT_MODE:-}"

export EEL_AUDIT_MODE="${AUDIT_MODE}"
export EEL_REPO_ROOT="${REPO_ROOT}"
export EEL_FIXTURES_DIR="${SCRIPT_DIR}/fixtures/mini-audit"

log() { printf '[run-audit] %s\n' "$*" >&2; }

# Accumulate step results in a temp file (avoids subshell/array pitfalls)
RESULTS_FILE="$(mktemp /tmp/eel-audit-results.XXXXXX.json)"
trap 'rm -f "${RESULTS_FILE}"' EXIT

printf '[]' > "${RESULTS_FILE}"

run_step() {
	local step_id="$1"
	local script="$2"
	local label="$3"
	local interpreter="$4"
	log "--- ${step_id}: ${label} ---"
	local status="pass"
	local exit_code=0
	if "${interpreter}" "${script}" 2>&1; then
		status="pass"
		log "${step_id}: PASS"
	else
		exit_code=$?
		status="fail"
		log "${step_id}: FAIL (exit ${exit_code})"
	fi
	# Append to results JSON array
	python3 - <<PYEOF
import json, sys
results_path = '${RESULTS_FILE}'
data = json.load(open(results_path))
data.append({"step": "${step_id}", "label": "${label}", "status": "${status}", "exit_code": ${exit_code}})
json.dump(data, open(results_path, 'w'), indent=2)
PYEOF
	if [[ "${status}" == "fail" ]]; then
		return 1
	fi
}

OVERALL=0

run_step "9.1" "${STEPS_DIR}/9.1-stub-scan.sh"           "stub-scan"          bash    || OVERALL=$((OVERALL + 1))
run_step "9.2" "${STEPS_DIR}/9.2-surface-inventory.sh"    "surface-inventory"  bash    || OVERALL=$((OVERALL + 1))
run_step "9.3" "${STEPS_DIR}/9.3-traceability.py"         "traceability"       python3 || OVERALL=$((OVERALL + 1))
run_step "9.4" "${STEPS_DIR}/9.4-contract-closure.sh"     "contract-closure"   bash    || OVERALL=$((OVERALL + 1))
run_step "9.5" "${STEPS_DIR}/9.5-endpoint-closure.py"     "endpoint-closure"   python3 || OVERALL=$((OVERALL + 1))
run_step "9.6" "${STEPS_DIR}/9.6-runbook-closure.sh"      "runbook-closure"    bash    || OVERALL=$((OVERALL + 1))
run_step "9.7" "${STEPS_DIR}/9.7-sla-closure.py"          "sla-closure"        python3 || OVERALL=$((OVERALL + 1))
run_step "9.8" "${STEPS_DIR}/9.8-legal-closure.py"        "legal-closure"      python3 || OVERALL=$((OVERALL + 1))
run_step "9.9" "${STEPS_DIR}/9.9-report.py"               "report"             python3 || OVERALL=$((OVERALL + 1))

# Write aggregate report
python3 - <<PYEOF
import json, datetime

results_path = '${RESULTS_FILE}'
report_path = '${REPORT_JSON}'

steps = json.load(open(results_path))
passed = sum(1 for s in steps if s['status'] == 'pass')
failed = sum(1 for s in steps if s['status'] == 'fail')

report = {
    "version": "1",
    "generated_at": datetime.datetime.utcnow().isoformat() + "Z",
    "audit_mode": "${AUDIT_MODE:-live}",
    "total_steps": len(steps),
    "passed": passed,
    "failed": failed,
    "steps": steps,
}
json.dump(report, open(report_path, 'w'), indent=2)
print(f"[run-audit] report: {passed}/{len(steps)} passed, {failed} failed")
PYEOF

if [[ "${OVERALL}" -gt 0 ]]; then
	log "FAIL: ${OVERALL} step(s) failed"
	exit 1
fi

log "PASS: all 9 steps passed"
