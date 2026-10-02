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

# test_orchestrator.sh: Bats-style tests for run-audit.sh in dry mode.
# Asserts each step exits 0 and expected JSON is produced.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPLETENESS_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${COMPLETENESS_DIR}/../.." && pwd)"
RUN_AUDIT="${COMPLETENESS_DIR}/run-audit.sh"
REPORT_JSON="${COMPLETENESS_DIR}/completeness-report.json"

PASS=0
FAIL=0

ok()  { printf '[PASS] %s\n' "$*"; PASS=$((PASS + 1)); }
fail(){ printf '[FAIL] %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }

# --- Prerequisite: bash -n on run-audit.sh ---
if bash -n "${RUN_AUDIT}"; then
  ok "bash -n: run-audit.sh"
else
  fail "bash -n: run-audit.sh"
fi

# --- bash -n on all step shell scripts ---
while IFS= read -r -d '' f; do
  if bash -n "${f}" 2>/dev/null; then
    ok "bash -n: ${f##*/}"
  else
    fail "bash -n: ${f##*/}"
  fi
done < <(find "${COMPLETENESS_DIR}/steps" -name "*.sh" -print0)

# --- python -m py_compile on all step Python scripts ---
while IFS= read -r -d '' f; do
  if python3 -m py_compile "${f}" 2>/dev/null; then
    ok "py_compile: ${f##*/}"
  else
    fail "py_compile: ${f##*/}"
  fi
done < <(find "${COMPLETENESS_DIR}/steps" -name "*.py" -print0)

# --- YAML validity on fixtures ---
while IFS= read -r -d '' f; do
  if python3 -c "import yaml; yaml.safe_load(open('${f}'))" 2>/dev/null; then
    ok "yaml: ${f##*/}"
  else
    fail "yaml: ${f##*/}"
  fi
done < <(find "${COMPLETENESS_DIR}/fixtures" -name "*.yaml" -print0)

# --- Main: EEL_AUDIT_MODE=dry run returns exit 0 ---
rm -f "${REPORT_JSON}"
if EEL_AUDIT_MODE=dry EEL_REPO_ROOT="${REPO_ROOT}" bash "${RUN_AUDIT}" >/dev/null 2>&1; then
  ok "run-audit.sh --dry exits 0"
else
  fail "run-audit.sh --dry exited non-zero"
fi

# --- Report JSON exists after dry run ---
if [[ -f "${REPORT_JSON}" ]]; then
  ok "completeness-report.json created"
else
  fail "completeness-report.json not found after dry run"
fi

# --- Report JSON is valid and has expected fields ---
_check_report() {
  python3 -c "
import json, sys
path = sys.argv[1]
try:
    data = json.load(open(path))
    assert 'steps' in data, \"missing 'steps'\"
    assert 'passed' in data, \"missing 'passed'\"
    assert data.get('audit_mode') in ('dry', 'live'), f\"unexpected audit_mode: {data.get('audit_mode')}\"
    print('[PASS] completeness-report.json structure valid')
    sys.exit(0)
except Exception as e:
    print(f'[FAIL] completeness-report.json invalid: {e}', file=sys.stderr)
    sys.exit(1)
" "${REPORT_JSON}"
}
if _check_report; then
  PASS=$((PASS + 1))
else
  FAIL=$((FAIL + 1))
fi

# --- Attestation JSON exists ---
ATTEST_JSON="${COMPLETENESS_DIR}/completeness-attestation.json"
if [[ -f "${ATTEST_JSON}" ]]; then
  ok "completeness-attestation.json created"
else
  fail "completeness-attestation.json not found after dry run"
fi

echo ""
if [[ "${FAIL}" -eq 0 ]]; then
  echo "All tests passed (${PASS} pass, 0 fail)"
  exit 0
else
  echo "FAIL: ${FAIL} test(s) failed, ${PASS} passed" >&2
  exit 1
fi
