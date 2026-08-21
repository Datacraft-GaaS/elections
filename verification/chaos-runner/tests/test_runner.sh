#!/usr/bin/env bash
# Bats-style tests for the chaos runner.
# Run with: bash verification/chaos-runner/tests/test_runner.sh
# EEL_CHAOS_MODE=dry validates all scenario refs without a live cluster.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHAOS_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${CHAOS_DIR}/../.." && pwd)"
RUNNER="${CHAOS_DIR}/runner.sh"
SCENARIOS_INDEX="${CHAOS_DIR}/scenarios-index.yaml"
SCHEDULE="${CHAOS_DIR}/schedule.yaml"

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

echo "# chaos runner tests"

# 1. runner.sh syntax check.
echo "# 1. bash -n syntax check"
if bash -n "${RUNNER}" 2>&1; then
  ok "runner.sh passes bash -n"
else
  not_ok "runner.sh fails bash -n"
fi

# 2. Dry mode exits 0 (all scenario refs exist).
echo "# 2. dry mode exits 0"
if EEL_CHAOS_MODE=dry bash "${RUNNER}" 2>&1; then
  ok "dry run exits 0"
else
  not_ok "dry run exited non-zero -- missing scenario files"
fi

# 3. scenarios-index.yaml references exactly 34 scenarios.
echo "# 3. scenarios-index.yaml has 34 entries"
COUNT="$(python3 -c "
import yaml
d=yaml.safe_load(open('${SCENARIOS_INDEX}'))
print(len(d.get('scenarios',[])))
")"
if [ "${COUNT}" -eq 34 ]; then
  ok "scenarios-index.yaml has 34 scenarios"
else
  not_ok "expected 34 scenarios, got ${COUNT}"
fi

# 4. Every file under chaos/scenarios/*.yaml is in scenarios-index.yaml.
echo "# 4. all chaos/scenarios/*.yaml files indexed"
INDEXED="$(python3 -c "
import yaml
d=yaml.safe_load(open('${SCENARIOS_INDEX}'))
for s in d.get('scenarios',[]):
    print(s['file'])
" | sort)"

ACTUAL="$(find "${REPO_ROOT}/chaos/scenarios" -name '*.yaml' | sed "s|${REPO_ROOT}/||" | sort)"

UNINDEXED="$(comm -23 <(echo "${ACTUAL}") <(echo "${INDEXED}"))"
if [ -z "${UNINDEXED}" ]; then
  ok "all chaos/scenarios/*.yaml files are in scenarios-index.yaml"
else
  not_ok "unindexed scenario files: ${UNINDEXED}"
fi

# 5. schedule.yaml references only files present in scenarios-index.yaml.
echo "# 5. schedule.yaml refs are all in scenarios-index.yaml"
SCHEDULE_FILES="$(python3 -c "
import yaml
d=yaml.safe_load(open('${SCHEDULE}'))
for s in d.get('schedule',[]):
    print(s['scenario'])
" | sort)"

INDEX_FILES="$(python3 -c "
import yaml
d=yaml.safe_load(open('${SCENARIOS_INDEX}'))
for s in d.get('scenarios',[]):
    print(s['file'])
" | sort)"

UNMATCHED="$(comm -23 <(echo "${SCHEDULE_FILES}") <(echo "${INDEX_FILES}"))"
if [ -z "${UNMATCHED}" ]; then
  ok "all schedule.yaml entries are in scenarios-index.yaml"
else
  not_ok "schedule.yaml references files not in index: ${UNMATCHED}"
fi

# 6. Every referenced scenario file exists on disk.
echo "# 6. all referenced scenario files exist on disk"
MISSING=0
while IFS= read -r f; do
  if [ ! -f "${REPO_ROOT}/${f}" ]; then
    not_ok "missing: ${f}"
    MISSING=$((MISSING + 1))
  fi
done < <(python3 -c "
import yaml
d=yaml.safe_load(open('${SCENARIOS_INDEX}'))
for s in d.get('scenarios',[]):
    print(s['file'])
")
if [ "${MISSING}" -eq 0 ]; then
  ok "all 34 referenced scenario files exist on disk"
fi

# 7. degradation-contract.yaml is valid YAML.
echo "# 7. degradation-contract.yaml valid YAML"
if python3 -c "import yaml; yaml.safe_load(open('${CHAOS_DIR}/degradation-contract.yaml'))" 2>&1; then
  ok "degradation-contract.yaml parses without error"
else
  not_ok "degradation-contract.yaml YAML parse failed"
fi

# 8. Fixture report exists and is marked as fixture.
echo "# 8. fixture report marked"
FIXTURE="${CHAOS_DIR}/fixtures/local-chaos-report.md"
if [ -f "${FIXTURE}" ]; then
  if grep -qi "fixture" "${FIXTURE}"; then
    ok "fixture exists and is marked as fixture"
  else
    not_ok "fixture exists but not marked as fixture"
  fi
else
  not_ok "fixture not found: ${FIXTURE}"
fi

echo ""
echo "# result: ${PASS} passed, ${FAIL} failed"
[ "${FAIL}" -eq 0 ] || exit 1
