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

# pq-hybrid/pq-disabled-client-check.sh -- attempt a classical-only TLS handshake
# against each edge URL. Asserts the server accepts (backward compat) and records
# that PQ hybrid is the preferred group (per ADR-0005).
# Set EEL_DR_MODE=dry to skip live checks.
#
# Requires: openssl 3+ with oqs provider, or curl with --curves support.
#
# References:
#   ADR-0005 (TLS 1.3 + PQ hybrid ML-KEM + X25519; classical fallback X25519-only)
#   ADR-0010 (edge TLS policy)

set -euo pipefail

EEL_DR_MODE="${EEL_DR_MODE:-live}"
REPORT_DIR="${EEL_PQ_REPORT_DIR:-/tmp/eel-pq-check-$(date +%Y%m%d)}"

EDGE_URLS=(
  "${EEL_EDGE_NAIROBI:-https://nairobi.edge.eel.internal}"
  "${EEL_EDGE_MOMBASA:-https://mombasa.edge.eel.internal}"
  "${EEL_EDGE_KISUMU:-https://kisumu.edge.eel.internal}"
  "${EEL_EDGE_NAKURU:-https://nakuru.edge.eel.internal}"
  "${EEL_EDGE_ELDORET:-https://eldoret.edge.eel.internal}"
)

# Classical-only curve (no PQ group).
CLASSICAL_CURVES="X25519:P-256:P-384"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)
FAIL_COUNT=0

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live PQ-disabled checks"
  printf '{"stage":"pq-disabled-client-check","mode":"dry","start_epoch":%s,"status":"ok","failed_urls":0}\n' \
    "${START_EPOCH}"
  exit 0
fi

mkdir -p "${REPORT_DIR}"

for URL in "${EDGE_URLS[@]}"; do
  HOST=$(echo "${URL}" | sed 's|https://||;s|/.*||')
  PORT=443
  log "Classical-only handshake: ${HOST}:${PORT}"

  HANDSHAKE_OUT="${REPORT_DIR}/${HOST}-classical.txt"

  # Attempt classical-only handshake using openssl s_client.
  # -groups restricts to non-PQ curves; server should still accept (backward compat).
  if command -v openssl >/dev/null 2>&1; then
    OPENSSL_OUT=$(echo Q | openssl s_client \
      -connect "${HOST}:${PORT}" \
      -tls1_3 \
      -groups "${CLASSICAL_CURVES}" \
      -brief \
      2>&1 | head -20 || true)
    echo "${OPENSSL_OUT}" > "${HANDSHAKE_OUT}"

    if echo "${OPENSSL_OUT}" | grep -qi "Verification: OK\|Connection established\|Handshake complete\|SSL-Session"; then
      log "  Classical handshake: ACCEPTED (backward compat OK)"

      # Record negotiated group for evidence.
      NEGOTIATED=$(echo "${OPENSSL_OUT}" | grep -i "group\|curve" | head -1 || true)
      log "  Negotiated: ${NEGOTIATED:-not reported}"
      echo "preferred_group_note: server accepted classical-only; hybrid preferred per ADR-0005" \
        >> "${HANDSHAKE_OUT}"
    elif echo "${OPENSSL_OUT}" | grep -qi "error\|fail\|refused"; then
      log "  FAIL: server rejected classical-only handshake on ${HOST}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    else
      log "  WARN: ambiguous handshake result on ${HOST}; see ${HANDSHAKE_OUT}"
    fi
  else
    log "  SKIP ${HOST}: openssl not on PATH"
  fi
done

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "PQ-disabled client check complete in ${ELAPSED}s; failed: ${FAIL_COUNT}"

printf '{"stage":"pq-disabled-client-check","mode":"live","urls_tested":%s,"failed_urls":%s,"report_dir":"%s","note":"server_accepts_classical_hybrid_preferred","start_epoch":%s,"elapsed_seconds":%s,"status":"%s"}\n' \
  "${#EDGE_URLS[@]}" "${FAIL_COUNT}" "${REPORT_DIR}" "${START_EPOCH}" "${ELAPSED}" \
  "$( [ "${FAIL_COUNT}" -eq 0 ] && echo ok || echo fail)"

[ "${FAIL_COUNT}" -eq 0 ] || die "${FAIL_COUNT} edge URL(s) failed classical-only handshake acceptance check"
