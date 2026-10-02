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

# restore-velero.sh -- Velero namespace restore against a synthetic backup.
# In dry mode: lists backup names and exits 0.
# In live mode: invokes velero restore create and polls for completion.
#
# References:
#   docs/runbooks/velero-restore.md
#   ADR-0022

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
TARGET_NAMESPACE="${EEL_VELERO_NAMESPACE:-eel-system}"
BACKUP_NAME="${EEL_VELERO_BACKUP:-}"
RESTORE_TIMEOUT_SECONDS="${EEL_VELERO_TIMEOUT:-600}"
KUBECONFIG="${EEL_KUBECONFIG:-${HOME}/.kube/config}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

test -f "${REPO_ROOT}/docs/runbooks/velero-restore.md" \
  || die "runbook docs/runbooks/velero-restore.md not found"

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live Velero calls"
  printf '{"stage":"restore-velero","mode":"dry","namespace":"%s","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${TARGET_NAMESPACE}" "${START_EPOCH}"
  exit 0
fi

log "Ticket: ${TICKET} | Velero restore for namespace ${TARGET_NAMESPACE}"

# Step 1: discover most recent backup if not specified.
if [ -z "${BACKUP_NAME}" ]; then
  log "Step 1: discovering most recent backup for namespace ${TARGET_NAMESPACE}"
  BACKUP_NAME=$(velero backup get --output json 2>/dev/null \
    | python3 -c "
import sys, json
items = json.load(sys.stdin).get('items', [])
ns_backups = [b for b in items
  if '${TARGET_NAMESPACE}' in b.get('spec', {}).get('includedNamespaces', [])
  and b.get('status', {}).get('phase') == 'Completed']
ns_backups.sort(key=lambda b: b.get('metadata', {}).get('creationTimestamp', ''), reverse=True)
print(ns_backups[0]['metadata']['name'] if ns_backups else '')
" 2>/dev/null)
  [ -n "${BACKUP_NAME}" ] || die "No completed Velero backup found for namespace ${TARGET_NAMESPACE}"
fi
log "Using backup: ${BACKUP_NAME}"

# Step 2: describe backup for confirmation.
log "Step 2: inspecting backup ${BACKUP_NAME}"
velero backup describe "${BACKUP_NAME}" --details

# Step 3: delete existing namespace resources for clean restore.
log "Step 3: deleting existing namespace resources in ${TARGET_NAMESPACE}"
kubectl --kubeconfig="${KUBECONFIG}" delete namespace "${TARGET_NAMESPACE}" --ignore-not-found --wait=true \
  --timeout=120s || log "  (namespace delete returned non-zero; continuing)"

# Step 4: create velero restore.
RESTORE_NAME="drill-$(echo "${BACKUP_NAME}" | cut -c1-20)-$(date +%s)"
log "Step 4: creating Velero restore ${RESTORE_NAME}"
velero restore create "${RESTORE_NAME}" \
  --from-backup "${BACKUP_NAME}" \
  --include-namespaces "${TARGET_NAMESPACE}" \
  --existing-resource-policy=update \
  --wait

# Step 5: assert completion within timeout.
log "Step 5: polling restore completion (timeout ${RESTORE_TIMEOUT_SECONDS}s)"
DEADLINE=$(($(date +%s) + RESTORE_TIMEOUT_SECONDS))
PHASE=""
while true; do
  PHASE=$(velero restore get "${RESTORE_NAME}" --output json 2>/dev/null \
    | python3 -c "import sys,json; print(json.load(sys.stdin).get('status',{}).get('phase',''))" 2>/dev/null || echo "")
  if [ "${PHASE}" = "Completed" ]; then
    log "Restore phase: Completed"
    break
  elif [ "${PHASE}" = "Failed" ] || [ "${PHASE}" = "PartiallyFailed" ]; then
    die "Velero restore ${RESTORE_NAME} ended with phase: ${PHASE}"
  fi
  if [ "$(date +%s)" -gt "${DEADLINE}" ]; then
    die "Velero restore timed out after ${RESTORE_TIMEOUT_SECONDS}s (phase: ${PHASE})"
  fi
  sleep 15
done

# Step 6: verify pods are running in restored namespace.
log "Step 6: verifying Running pods in ${TARGET_NAMESPACE}"
sleep 10
RUNNING=$(kubectl --kubeconfig="${KUBECONFIG}" get pods -n "${TARGET_NAMESPACE}" \
  --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l | tr -d ' ')
log "  Running pods: ${RUNNING}"
[ "${RUNNING}" -gt 0 ] || die "No Running pods found in ${TARGET_NAMESPACE} after restore"

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "Velero restore drill complete in ${ELAPSED}s"

printf '{"stage":"restore-velero","mode":"live","namespace":"%s","backup":"%s","restore":"%s","running_pods":%s,"start_epoch":%s,"elapsed_seconds":%s,"status":"ok"}\n' \
  "${TARGET_NAMESPACE}" "${BACKUP_NAME}" "${RESTORE_NAME}" "${RUNNING}" "${START_EPOCH}" "${ELAPSED}"
