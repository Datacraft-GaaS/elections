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

# tlog-attest-restore.sh -- submit an eel.dr.restore_completed.v1 leaf to the
# transparency log after any restore operation. Asserts inclusion proof within 60 s.
#
# Input (environment variables):
#   EEL_RESTORE_SUBJECT   -- what was restored (e.g. "namespace/eel-system")
#   EEL_RESTORE_SHA256    -- SHA-256 of the restored state (hex, 64 chars)
#   EEL_TLOG_URL          -- transparency log base URL (REST)
#   EEL_DR_MODE=dry       -- skip network calls; emit stubbed proof
#
# References:
#   ADR-0017 (transparency log, meta-events domain)
#   docs/runbooks/dr-failover-primary-to-mirror.md

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
RESTORE_SUBJECT="${EEL_RESTORE_SUBJECT:-unknown}"
RESTORE_SHA256="${EEL_RESTORE_SHA256:-}"
TLOG_URL="${EEL_TLOG_URL:-http://transparency-svc.eel-system.svc:8080}"
INCLUSION_TIMEOUT_SECONDS=60

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

test -f "${REPO_ROOT}/docs/runbooks/dr-failover-primary-to-mirror.md" \
  || die "runbook docs/runbooks/dr-failover-primary-to-mirror.md not found"

START_EPOCH=$(date +%s)
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Compute SHA-256 of restore subject if not provided.
if [ -z "${RESTORE_SHA256}" ]; then
  RESTORE_SHA256=$(echo -n "${RESTORE_SUBJECT}:${TIMESTAMP}:${TICKET}" | sha256sum | cut -c1-64)
  log "Computed restore SHA-256: ${RESTORE_SHA256}"
fi

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: emitting stubbed tlog attestation"
  STUB_LEAF_ID="dry-run-leaf-$(date +%s)"
  printf '{"stage":"tlog-attest-restore","mode":"dry","leaf_id":"%s","restore_subject":"%s","restore_sha256":"%s","inclusion_proof":"stubbed","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${STUB_LEAF_ID}" "${RESTORE_SUBJECT}" "${RESTORE_SHA256}" "${START_EPOCH}"
  exit 0
fi

log "Ticket: ${TICKET} | Submitting tlog leaf for restore of ${RESTORE_SUBJECT}"

# Build CloudEvent leaf payload (meta-events domain, eel.dr.restore_completed.v1).
LEAF_PAYLOAD=$(python3 - <<PYEOF
import json, sys

payload = {
    "specversion": "1.0",
    "type": "eel.dr.restore_completed.v1",
    "source": "urn:eel:dr-drill",
    "subject": "${RESTORE_SUBJECT}",
    "id": "${TICKET}-$(date +%s)",
    "time": "${TIMESTAMP}",
    "domain": "meta-events",
    "datacontenttype": "application/json",
    "data": {
        "ticket": "${TICKET}",
        "restore_sha256": "${RESTORE_SHA256}",
        "restore_subject": "${RESTORE_SUBJECT}",
        "drill_mode": "live"
    }
}
print(json.dumps(payload))
PYEOF
)

log "Leaf payload prepared"

# Submit leaf to transparency log REST API.
log "Step 1: submitting leaf to ${TLOG_URL}/api/v1/log/entries"
SUBMIT_RESP=$(curl -sf --max-time 30 \
  -X POST "${TLOG_URL}/api/v1/log/entries" \
  -H "Content-Type: application/json" \
  -d "${LEAF_PAYLOAD}")

LEAF_HASH=$(echo "${SUBMIT_RESP}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('leaf_hash',''))" 2>/dev/null || echo "")
[ -n "${LEAF_HASH}" ] || die "Tlog submission failed; no leaf_hash in response: ${SUBMIT_RESP}"
log "Leaf submitted: hash=${LEAF_HASH}"

# Step 2: assert inclusion proof within INCLUSION_TIMEOUT_SECONDS.
log "Step 2: polling for inclusion proof (timeout ${INCLUSION_TIMEOUT_SECONDS}s)"
DEADLINE=$(($(date +%s) + INCLUSION_TIMEOUT_SECONDS))
LEAF_ID=""
while true; do
  PROOF_RESP=$(curl -sf --max-time 10 \
    "${TLOG_URL}/api/v1/log/inclusion/${LEAF_HASH}" 2>/dev/null || echo "{}")
  PROOF_STATUS=$(echo "${PROOF_RESP}" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('status',''))" 2>/dev/null || echo "")
  if [ "${PROOF_STATUS}" = "included" ]; then
    LEAF_ID=$(echo "${PROOF_RESP}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('leaf_index',''))" 2>/dev/null || echo "")
    log "Inclusion proof confirmed: leaf_index=${LEAF_ID}"
    break
  fi
  if [ "$(date +%s)" -gt "${DEADLINE}" ]; then
    die "Inclusion proof not confirmed within ${INCLUSION_TIMEOUT_SECONDS}s"
  fi
  sleep 5
done

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "Tlog attestation complete in ${ELAPSED}s"

printf '{"stage":"tlog-attest-restore","mode":"live","leaf_id":"%s","leaf_hash":"%s","restore_subject":"%s","restore_sha256":"%s","inclusion_proof":"confirmed","start_epoch":%s,"elapsed_seconds":%s,"status":"ok"}\n' \
  "${LEAF_ID}" "${LEAF_HASH}" "${RESTORE_SUBJECT}" "${RESTORE_SHA256}" "${START_EPOCH}" "${ELAPSED}"
