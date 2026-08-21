#!/usr/bin/env bash
# Bats-style tests for the rehearsal orchestrator.
# Run with: bash verification/rehearsal/tests/test_orchestrator.sh
# Requires EEL_REHEARSAL_MODE=dry; no cluster needed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REHEARSAL_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ORCHESTRATOR="${REHEARSAL_DIR}/orchestrator.sh"

PASS=0
FAIL=0

ok() {
  echo "  ok: $1"
  PASS=$((PASS + 1))
}

not_ok() {
  echo "  FAIL: $1"
  FAIL=$((FAIL + 1))
}

run_stage_check() {
  local desc="$1"
  local expected_stage="$2"
  local output
  output="$(EEL_REHEARSAL_MODE=dry bash "${ORCHESTRATOR}" 2>&1 || true)"
  if echo "${output}" | grep -q "last_stage=${expected_stage}"; then
    ok "${desc}: last_stage=${expected_stage}"
  else
    not_ok "${desc}: expected last_stage=${expected_stage}, got: $(echo "${output}" | grep 'last_stage' | tail -1)"
  fi
}

echo "# rehearsal orchestrator tests (dry mode)"

# Test: orchestrator is syntactically valid.
echo "# 1. bash -n syntax check"
if bash -n "${ORCHESTRATOR}" 2>&1; then
  ok "orchestrator.sh passes bash -n"
else
  not_ok "orchestrator.sh fails bash -n"
fi

# Test: dry run completes and last_stage is 'done'.
echo "# 2. dry run completes"
OUTPUT="$(EEL_REHEARSAL_MODE=dry bash "${ORCHESTRATOR}" 2>&1 || true)"
if echo "${OUTPUT}" | grep -q "last_stage=done"; then
  ok "dry run: last_stage=done"
else
  not_ok "dry run: expected last_stage=done. output: ${OUTPUT}"
fi

# Test: each expected stage is logged.
echo "# 3. stage hooks called"
for stage in cluster-check election-day-mode load sla-metrics witness-crosscheck bundle-repro verifier report done; do
  if echo "${OUTPUT}" | grep -q "stage: ${stage}"; then
    ok "stage '${stage}' logged"
  else
    not_ok "stage '${stage}' not found in output"
  fi
done

# Test: sla-manifest.yaml is valid YAML.
echo "# 4. sla-manifest.yaml valid YAML"
SLA_MANIFEST="${REHEARSAL_DIR}/sla-manifest.yaml"
if python3 -c "import yaml; yaml.safe_load(open('${SLA_MANIFEST}'))" 2>&1; then
  ok "sla-manifest.yaml parses without error"
else
  not_ok "sla-manifest.yaml YAML parse failed"
fi

# Test: sla-manifest.yaml has at least 16 SLA rows.
echo "# 5. sla-manifest.yaml row count"
SLA_COUNT="$(python3 -c "
import yaml
d=yaml.safe_load(open('${SLA_MANIFEST}'))
print(len(d.get('slas',[])))
")"
if [ "${SLA_COUNT}" -ge 16 ]; then
  ok "sla-manifest.yaml has ${SLA_COUNT} SLAs (>= 16 required)"
else
  not_ok "sla-manifest.yaml has only ${SLA_COUNT} SLAs"
fi

# Test: witness-crosscheck.sh exits 0 in dry mode.
echo "# 6. witness-crosscheck.sh dry mode"
WITNESS_SCRIPT="${REHEARSAL_DIR}/witness-crosscheck.sh"
if EEL_REHEARSAL_MODE=dry bash "${WITNESS_SCRIPT}" 2>&1; then
  ok "witness-crosscheck.sh exits 0 in dry mode"
else
  not_ok "witness-crosscheck.sh exited non-zero in dry mode"
fi

# Test: report template exists and has required sections.
echo "# 7. report template sections"
TEMPLATE="${REHEARSAL_DIR}/report-template.md.tmpl"
for section in "Cluster Snapshot" "Load Profile" "SLA Table" "Witness Cross-Check" "Bundle Repro" "Verifier Output" "Incidents" "Sign-Off"; do
  if grep -q "${section}" "${TEMPLATE}"; then
    ok "template has section '${section}'"
  else
    not_ok "template missing section '${section}'"
  fi
done

# Test: fixture report exists and is marked as fixture.
echo "# 8. fixture report marked"
FIXTURE="${REHEARSAL_DIR}/fixtures/local-rehearsal-report.md"
if [ -f "${FIXTURE}" ]; then
  if grep -qi "fixture" "${FIXTURE}"; then
    ok "fixture report exists and is marked as fixture"
  else
    not_ok "fixture report exists but is not marked as fixture"
  fi
else
  not_ok "fixture report not found: ${FIXTURE}"
fi

echo ""
echo "# result: ${PASS} passed, ${FAIL} failed"
[ "${FAIL}" -eq 0 ] || exit 1
