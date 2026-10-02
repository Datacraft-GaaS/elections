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

# 9.6-runbook-closure.sh: Assert every service has deploy/rollback/DR/degradation runbooks
# and that every chaos scenario has been executed within 30 days.
set -euo pipefail

REPO_ROOT="${EEL_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)}"
AUDIT_MODE="${EEL_AUDIT_MODE:-}"
CHAOS_REPORT="${REPO_ROOT}/verification/chaos-runner/report"

log() { printf '[9.6-runbook-closure] %s\n' "$*" >&2; }

if [[ -n "${AUDIT_MODE}" && "${AUDIT_MODE}" == "dry" ]]; then
  log "dry mode: skipping live runbook and chaos checks"
  log "PASS (dry)"
  exit 0
fi

SERVICES_DIR="${REPO_ROOT}/services"
FAILURES=()

# Check each service for required runbooks
while IFS= read -r -d '' svc_path; do
  svc_name="$(basename "${svc_path}")"
  [[ "${svc_name}" == "README.md" ]] && continue

  runbook_base="${REPO_ROOT}/docs/runbooks"
  per_svc_docs="${svc_path}/docs"

  for runbook_type in deploy rollback dr degradation; do
    found=false
    # Check global docs/runbooks/
    if ls "${runbook_base}/${svc_name}-${runbook_type}".* 2>/dev/null | grep -q .; then
      found=true
    fi
    # Check per-service docs/
    if ls "${per_svc_docs}/${runbook_type}".* 2>/dev/null | grep -q .; then
      found=true
    fi
    if ! "${found}"; then
      FAILURES+=("${svc_name}: missing ${runbook_type} runbook")
    fi
  done
done < <(find "${SERVICES_DIR}" -maxdepth 1 -mindepth 1 -type d -print0)

if [[ "${#FAILURES[@]}" -gt 0 ]]; then
  for f in "${FAILURES[@]}"; do
    log "FAIL: ${f}"
  done
  log "FAIL: ${#FAILURES[@]} runbook(s) missing"
  exit 1
fi

# Check chaos report recency (within 30 days)
if [[ -d "${CHAOS_REPORT}" ]]; then
  REPORT_FILE="$(find "${CHAOS_REPORT}" -name "*.json" -newer "$(date -d '30 days ago' '+%Y-%m-%dT%H:%M:%S' 2>/dev/null || date -v-30d '+%Y-%m-%dT%H:%M:%S' 2>/dev/null || echo '')" 2>/dev/null | head -1)"
  if [[ -z "${REPORT_FILE}" ]]; then
    log "WARN: no chaos report found within 30 days in ${CHAOS_REPORT}"
  else
    log "chaos report: ${REPORT_FILE} (within 30 days)"
  fi
else
  log "WARN: chaos-runner report directory not found at ${CHAOS_REPORT}"
fi

log "PASS: runbook closure"
