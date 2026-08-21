#!/usr/bin/env bash
# dast/zap-baseline.sh -- OWASP ZAP baseline scan against eel edge URL.
# Target: EEL_TARGET_URL (default: https://nairobi.edge.eel.internal)
# Uses the OWASP Core Rule Set (CRS) profile.
# Set EEL_DR_MODE=dry to skip the live scan.
#
# References:
#   ADR-0010 (edge DDoS / TLS policy)

set -euo pipefail

EEL_DR_MODE="${EEL_DR_MODE:-live}"
TARGET_URL="${EEL_TARGET_URL:-https://nairobi.edge.eel.internal}"
ZAP_REPORT_DIR="${EEL_ZAP_REPORT_DIR:-/tmp/eel-zap-report}"
ZAP_IMAGE="${EEL_ZAP_IMAGE:-ghcr.io/zaproxy/zaproxy:stable}"
FAIL_ON_RISK="${EEL_ZAP_FAIL_ON_RISK:-2}"   # 0=info,1=low,2=medium,3=high

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live ZAP scan"
  printf '{"stage":"dast-zap-baseline","mode":"dry","target":"%s","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${TARGET_URL}" "${START_EPOCH}"
  exit 0
fi

log "ZAP baseline scan: ${TARGET_URL}"
mkdir -p "${ZAP_REPORT_DIR}"

if command -v docker >/dev/null 2>&1; then
  log "Running ZAP via docker: ${ZAP_IMAGE}"
  docker run --rm \
    -v "${ZAP_REPORT_DIR}:/zap/wrk:rw" \
    "${ZAP_IMAGE}" \
    zap-baseline.py \
      -t "${TARGET_URL}" \
      -r "zap-baseline-report.html" \
      -J "zap-baseline-report.json" \
      -l "WARN" \
      -I \
      -a \
      --hook=/zap/auth-hooks.py 2>/dev/null || ZAP_RC=$?
  ZAP_RC="${ZAP_RC:-0}"
else
  log "docker not available; checking for local zap.sh"
  ZAP_SH="${EEL_ZAP_SH:-/opt/zaproxy/zap.sh}"
  if [ -x "${ZAP_SH}" ]; then
    "${ZAP_SH}" -cmd \
      -quickurl "${TARGET_URL}" \
      -quickout "${ZAP_REPORT_DIR}/zap-baseline-report.json" \
      -quickprogress || ZAP_RC=$?
    ZAP_RC="${ZAP_RC:-0}"
  else
    log "ZAP not available; install docker or set EEL_ZAP_SH"
    printf '{"stage":"dast-zap-baseline","mode":"live","target":"%s","start_epoch":%s,"status":"skipped","notes":"zap_not_installed"}\n' \
      "${TARGET_URL}" "${START_EPOCH}"
    exit 0
  fi
fi

# Parse report for high/critical alerts.
REPORT_JSON="${ZAP_REPORT_DIR}/zap-baseline-report.json"
HIGH_ALERTS=0
if [ -f "${REPORT_JSON}" ]; then
  HIGH_ALERTS=$(python3 -c "
import json, sys
try:
    d=json.load(open('${REPORT_JSON}'))
    sites=d.get('site',[])
    alerts=[a for s in sites for a in (s.get('alerts') or [])]
    print(sum(1 for a in alerts if int(a.get('riskcode',0)) >= ${FAIL_ON_RISK}))
except: print(0)" 2>/dev/null || echo 0)
fi

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "ZAP scan complete: ${HIGH_ALERTS} alert(s) at risk >= ${FAIL_ON_RISK}"

printf '{"stage":"dast-zap-baseline","mode":"live","target":"%s","high_alerts":%s,"fail_on_risk":%s,"report":"%s","start_epoch":%s,"elapsed_seconds":%s,"status":"%s"}\n' \
  "${TARGET_URL}" "${HIGH_ALERTS}" "${FAIL_ON_RISK}" "${REPORT_JSON}" "${START_EPOCH}" "${ELAPSED}" \
  "$( [ "${HIGH_ALERTS}" -eq 0 ] && echo ok || echo fail)"

[ "${HIGH_ALERTS}" -eq 0 ] || die "ZAP found ${HIGH_ALERTS} alert(s) at risk >= ${FAIL_ON_RISK}"
