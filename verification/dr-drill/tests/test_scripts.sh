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

# tests/test_scripts.sh -- bats-style dry-run tests for dr-drill scripts.
# Runs each script with EEL_DR_MODE=dry; asserts exit 0 and JSON stage output.
# No live infrastructure required.
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

run_dry() {
  local label="$1"
  local script="$2"
  shift 2

  # Syntax check first.
  if ! bash -n "${SCRIPT_DIR}/${script}" 2>/dev/null; then
    fail "${label}: bash -n failed"
    return
  fi

  # Run in dry mode; capture output and exit code.
  local out
  local rc=0
  out=$(EEL_DR_MODE=dry EEL_TICKET=TEST-DRY bash "${SCRIPT_DIR}/${script}" "$@" 2>/dev/null) || rc=$?

  if [ "${rc}" -ne 0 ]; then
    fail "${label}: exited ${rc} in dry mode"
    return
  fi

  # Assert JSON stage output on stdout.
  local stage
  stage=$(echo "${out}" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('stage',''))" 2>/dev/null || echo "")
  if [ -z "${stage}" ]; then
    fail "${label}: no JSON stage output (got: ${out})"
    return
  fi

  local status
  status=$(echo "${out}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status',''))" 2>/dev/null || echo "")
  if [ "${status}" != "ok" ]; then
    fail "${label}: JSON status != ok (got: ${status})"
    return
  fi

  ok "${label}: exit 0, stage=${stage}, status=ok"
}

log "=== DR drill script tests (dry mode) ==="

# Test each script.
run_dry "primary-loss"       "primary-loss.sh"
run_dry "edge-loss"          "edge-loss.sh"
run_dry "restore-pgbackrest" "restore-pgbackrest.sh"
run_dry "restore-velero"     "restore-velero.sh"
run_dry "restore-kopia"      "restore-kopia.sh"
run_dry "tlog-attest-restore" "tlog-attest-restore.sh"

# YAML validity check.
log "Checking YAML files"
YAML_FILES=$(find "${SCRIPT_DIR}" -name "*.yaml" -o -name "*.yml" 2>/dev/null)
for Y in ${YAML_FILES}; do
  if python3 -c "import yaml,sys; yaml.safe_load(open('${Y}'))" 2>/dev/null; then
    ok "YAML valid: ${Y##*/}"
  else
    fail "YAML invalid: ${Y}"
  fi
done

# Fixture directory check.
log "Checking fixtures"
FIXTURE="${SCRIPT_DIR}/fixtures/synthetic-basebackup"
if [ -d "${FIXTURE}" ] && [ -f "${FIXTURE}/backup.info" ]; then
  ok "synthetic-basebackup fixture present"
else
  fail "synthetic-basebackup fixture missing or incomplete"
fi

# Referenced runbooks exist.
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
for RUNBOOK in \
  "docs/runbooks/dr-failover-primary-to-mirror.md" \
  "docs/runbooks/pgbackrest-pitr.md" \
  "docs/runbooks/velero-restore.md"; do
  if test -f "${REPO_ROOT}/${RUNBOOK}"; then
    ok "runbook exists: ${RUNBOOK}"
  else
    fail "runbook missing: ${RUNBOOK}"
  fi
done

log ""
log "Results: ${PASS} passed, ${FAIL} failed"
if [ "${FAIL}" -gt 0 ]; then
  printf "Failures:%b\n" "${ERRORS}"
  exit 1
fi
exit 0
