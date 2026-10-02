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

# restore-kopia.sh -- Kopia snapshot restore for MinIO buckets with hash chain integrity check.
# Set EEL_DR_MODE=dry to skip live calls.
#
# References:
#   ADR-0022 (Kopia for application data-at-rest)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
KOPIA_REPO="${EEL_KOPIA_REPO:-s3://eel-kopia-backup/}"
SNAPSHOT_ID="${EEL_KOPIA_SNAPSHOT_ID:-}"
RESTORE_PATH="${EEL_KOPIA_RESTORE_PATH:-/tmp/eel-kopia-restore-$$}"
VERIFY_PERCENT="${EEL_KOPIA_VERIFY_PERCENT:-10}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live Kopia calls"
  printf '{"stage":"restore-kopia","mode":"dry","repo":"%s","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${KOPIA_REPO}" "${START_EPOCH}"
  exit 0
fi

log "Ticket: ${TICKET} | Kopia restore from ${KOPIA_REPO}"

# Step 1: connect to repository.
log "Step 1: connecting to Kopia repository"
kopia repository connect s3 \
  --bucket "$(echo "${KOPIA_REPO}" | sed 's|s3://||;s|/.*||')" \
  --override-username="eel-dr-drill" \
  --override-hostname="drill-host" \
  2>/dev/null || log "  (repository already connected)"

# Step 2: list snapshots.
log "Step 2: listing Kopia snapshots"
kopia snapshot list --json | python3 -c "
import sys, json
snaps = json.load(sys.stdin)
print(f'  {len(snaps)} snapshots found')
if snaps:
    latest = max(snaps, key=lambda s: s.get('startTime',''))
    print(f'  latest: {latest.get(\"id\",\"?\")} at {latest.get(\"startTime\",\"?\")}')
"

# Step 3: pick snapshot ID if not specified.
if [ -z "${SNAPSHOT_ID}" ]; then
  log "Step 3: selecting most recent snapshot"
  SNAPSHOT_ID=$(kopia snapshot list --json 2>/dev/null \
    | python3 -c "
import sys, json
snaps = json.load(sys.stdin)
snaps.sort(key=lambda s: s.get('startTime',''), reverse=True)
print(snaps[0]['id'] if snaps else '')
" 2>/dev/null)
  [ -n "${SNAPSHOT_ID}" ] || die "No Kopia snapshots found in repository"
fi
log "Using snapshot: ${SNAPSHOT_ID}"

# Step 4: restore snapshot to restore path.
mkdir -p "${RESTORE_PATH}"
log "Step 4: restoring snapshot ${SNAPSHOT_ID} to ${RESTORE_PATH}"
kopia snapshot restore "${SNAPSHOT_ID}" "${RESTORE_PATH}"

# Step 5: verify hash chain integrity.
log "Step 5: verifying hash chain integrity (${VERIFY_PERCENT}% sample)"
kopia snapshot verify "${SNAPSHOT_ID}" --verify-files-percent="${VERIFY_PERCENT}"

# Step 6: check output directory is non-empty.
FILE_COUNT=$(find "${RESTORE_PATH}" -type f | wc -l | tr -d ' ')
log "  restored files: ${FILE_COUNT}"
[ "${FILE_COUNT}" -gt 0 ] || die "Restore produced no files"

# Cleanup restore directory.
rm -rf "${RESTORE_PATH:?}"

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "Kopia restore drill complete in ${ELAPSED}s"

printf '{"stage":"restore-kopia","mode":"live","repo":"%s","snapshot_id":"%s","files_restored":%s,"start_epoch":%s,"elapsed_seconds":%s,"status":"ok"}\n' \
  "${KOPIA_REPO}" "${SNAPSHOT_ID}" "${FILE_COUNT}" "${START_EPOCH}" "${ELAPSED}"
