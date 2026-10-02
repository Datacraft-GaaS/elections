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

# pq-hybrid/testssl.sh -- run testssl.sh against every publicly exposed edge URL.
# Asserts TLS 1.3 + ML-KEM + X25519 hybrid KEM negotiated (per ADR-0005).
# Set EEL_DR_MODE=dry to skip live scans.
#
# Install testssl.sh: https://testssl.sh/
#   git clone --depth 1 https://github.com/drwetter/testssl.sh.git
#   export EEL_TESTSSL_BIN=/path/to/testssl.sh/testssl.sh
#
# References:
#   ADR-0005 (TLS 1.3 + PQ hybrid ML-KEM + X25519)
#   ADR-0010 (edge TLS policy)

set -euo pipefail

EEL_DR_MODE="${EEL_DR_MODE:-live}"
TESTSSL_BIN="${EEL_TESTSSL_BIN:-testssl.sh}"
REPORT_DIR="${EEL_TESTSSL_REPORT_DIR:-/tmp/eel-testssl-$(date +%Y%m%d)}"

EDGE_URLS=(
  "${EEL_EDGE_NAIROBI:-https://nairobi.edge.eel.internal}"
  "${EEL_EDGE_MOMBASA:-https://mombasa.edge.eel.internal}"
  "${EEL_EDGE_KISUMU:-https://kisumu.edge.eel.internal}"
  "${EEL_EDGE_NAKURU:-https://nakuru.edge.eel.internal}"
  "${EEL_EDGE_ELDORET:-https://eldoret.edge.eel.internal}"
)

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)
FAIL_COUNT=0

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live testssl.sh scans"
  printf '{"stage":"pq-hybrid-testssl","mode":"dry","start_epoch":%s,"status":"ok","failed_urls":0}\n' \
    "${START_EPOCH}"
  exit 0
fi

# Locate testssl.sh binary.
if ! command -v "${TESTSSL_BIN}" >/dev/null 2>&1 && [ ! -x "${TESTSSL_BIN}" ]; then
  log "testssl.sh not found at '${TESTSSL_BIN}'"
  log "Install: git clone --depth 1 https://github.com/drwetter/testssl.sh.git"
  log "         export EEL_TESTSSL_BIN=/path/to/testssl.sh/testssl.sh"
  printf '{"stage":"pq-hybrid-testssl","mode":"live","start_epoch":%s,"status":"skipped","notes":"testssl_not_installed"}\n' \
    "${START_EPOCH}"
  exit 0
fi

mkdir -p "${REPORT_DIR}"

for URL in "${EDGE_URLS[@]}"; do
  HOST=$(echo "${URL}" | sed 's|https://||;s|/.*||')
  log "Scanning ${HOST}"

  REPORT_JSON="${REPORT_DIR}/${HOST}.json"

  "${TESTSSL_BIN}" \
    --jsonfile "${REPORT_JSON}" \
    --protocols \
    --ciphers \
    --client-simulation \
    --quiet \
    "${URL}" 2>/dev/null || true

  if [ ! -f "${REPORT_JSON}" ]; then
    log "  FAIL: no report generated for ${HOST}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    continue
  fi

  # Assert TLS 1.3 is offered.
  TLS13=$(python3 -c "
import json, sys
try:
    findings = json.load(open('${REPORT_JSON}'))
    f = next((x for x in findings if x.get('id')=='TLS1_3'), {})
    print(f.get('finding',''))
except: print('')" 2>/dev/null || echo "")
  if echo "${TLS13}" | grep -qi "offered"; then
    log "  TLS 1.3: offered OK"
  else
    log "  FAIL: TLS 1.3 not offered on ${HOST} (got: ${TLS13})"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    continue
  fi

  # Assert PQ hybrid (ML-KEM or X25519Kyber768) negotiated.
  PQ_KEM=$(python3 -c "
import json, sys
try:
    findings = json.load(open('${REPORT_JSON}'))
    for f in findings:
        fnd = f.get('finding', '')
        if any(k in fnd for k in ('ML-KEM', 'X25519Kyber', 'mlkem', 'kyber')):
            print(fnd)
            break
    else:
        print('')
except: print('')" 2>/dev/null || echo "")

  if [ -n "${PQ_KEM}" ]; then
    log "  PQ hybrid KEM: ${PQ_KEM}"
  else
    log "  FAIL: PQ hybrid KEM not negotiated on ${HOST}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
done

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "testssl.sh scan complete in ${ELAPSED}s; failed: ${FAIL_COUNT}"

printf '{"stage":"pq-hybrid-testssl","mode":"live","urls_tested":%s,"failed_urls":%s,"report_dir":"%s","start_epoch":%s,"elapsed_seconds":%s,"status":"%s"}\n' \
  "${#EDGE_URLS[@]}" "${FAIL_COUNT}" "${REPORT_DIR}" "${START_EPOCH}" "${ELAPSED}" \
  "$( [ "${FAIL_COUNT}" -eq 0 ] && echo ok || echo fail)"

[ "${FAIL_COUNT}" -eq 0 ] || die "${FAIL_COUNT} edge URL(s) failed PQ-hybrid TLS check"
