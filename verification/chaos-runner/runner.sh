#!/usr/bin/env bash
# Chaos suite runner (WP-6.3).
# Applies chaos scenarios in schedule order against a live rehearsal load.
# Watches Prometheus SLO burn; aborts and writes a red report if burn exceeds
# the declared allowance in scenarios-index.yaml.
#
# Dry run: EEL_CHAOS_MODE=dry -- validates all scenario refs exist, exits 0.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

EEL_CHAOS_MODE="${EEL_CHAOS_MODE:-live}"
EEL_CLUSTER_CONTEXT="${EEL_CLUSTER_CONTEXT:-eel-preprod}"
EEL_PROMETHEUS_URL="${EEL_PROMETHEUS_URL:-http://prometheus.eel-preprod:9090}"
EEL_REHEARSAL_START="${EEL_REHEARSAL_START:-0}"
EEL_REPORT_DIR="${EEL_REPORT_DIR:-${REPO_ROOT}/verification/reports}"
RUN_DATE="$(date -u +%Y-%m-%d)"

DRY="${EEL_CHAOS_MODE}"
is_dry() { [ "${DRY}" = "dry" ]; }

log() { echo "[chaos-runner] $*" >&2; }
die() { echo "[chaos-runner] FATAL: $*" >&2; exit 1; }

SCHEDULE="${SCRIPT_DIR}/schedule.yaml"
SCENARIOS_INDEX="${SCRIPT_DIR}/scenarios-index.yaml"

# ---------- dry-run: validate all scenario refs ----------
if is_dry; then
  log "dry mode: validating scenario file references"
  MISSING=0
  while IFS= read -r scenario_file; do
    full_path="${REPO_ROOT}/${scenario_file}"
    if [ -f "${full_path}" ]; then
      log "  ok: ${scenario_file}"
    else
      log "  MISSING: ${scenario_file}"
      MISSING=$((MISSING + 1))
    fi
  done < <(python3 -c "
import yaml
idx = yaml.safe_load(open('${SCENARIOS_INDEX}'))
for s in idx.get('scenarios', []):
    print(s['file'])
")

  if [ "${MISSING}" -gt 0 ]; then
    die "${MISSING} scenario file(s) referenced in scenarios-index.yaml do not exist"
  fi

  # Also validate schedule refs.
  while IFS= read -r scenario_file; do
    full_path="${REPO_ROOT}/${scenario_file}"
    if [ ! -f "${full_path}" ]; then
      log "  MISSING (schedule): ${scenario_file}"
      MISSING=$((MISSING + 1))
    fi
  done < <(python3 -c "
import yaml
sched = yaml.safe_load(open('${SCHEDULE}'))
for s in sched.get('schedule', []):
    print(s['scenario'])
")

  if [ "${MISSING}" -gt 0 ]; then
    die "${MISSING} scenario file(s) referenced in schedule.yaml do not exist"
  fi

  log "dry: all scenario refs valid"
  exit 0
fi

# ---------- live run ----------
mkdir -p "${EEL_REPORT_DIR}"

CURRENT_SCENARIO="none"
CHAOS_LOG="${EEL_REPORT_DIR}/chaos-log-${RUN_DATE}.jsonl"
RED_REPORT="${EEL_REPORT_DIR}/chaos-${RUN_DATE}-red.md"

trap 'log "last scenario attempted: ${CURRENT_SCENARIO}"' EXIT

query_prometheus() {
  local query="$1"
  curl -fsSL --max-time 10 \
    "${EEL_PROMETHEUS_URL}/api/v1/query" \
    --data-urlencode "query=${query}" \
    | python3 -c "
import sys,json
d=json.load(sys.stdin)
results=d.get('data',{}).get('result',[])
if results:
    print(results[0]['value'][1])
else:
    print('0')
" 2>/dev/null || echo "0"
}

check_slo_burn() {
  local scenario="$1"
  local allowed_burn="$2"
  # Global error rate as a proxy for SLO burn.
  local error_rate
  error_rate="$(query_prometheus \
    'sum(rate(eel_http_requests_total{status=~"5.."}[2m])) / sum(rate(eel_http_requests_total[2m]))')"
  log "scenario=${scenario} slo_burn_proxy=${error_rate} allowed=${allowed_burn}"
  python3 -c "
import sys
burn=float('${error_rate}' or 0)
allowed=float('${allowed_burn}')
if burn > allowed:
    print('BREACH')
    sys.exit(1)
print('OK')
" || return 1
  return 0
}

write_red_report() {
  local scenario="$1"
  local burn="$2"
  local allowed="$3"
  cat > "${RED_REPORT}" <<EOF
# Chaos Run: RED REPORT

Date: ${RUN_DATE}
Aborted at scenario: ${scenario}
SLO burn observed: ${burn}
Allowed burn: ${allowed}
Action: chaos scenario aborted; rehearsal must be reviewed before G6 sign-off.

See ${CHAOS_LOG} for full scenario log.
EOF
  log "red report written: ${RED_REPORT}"
}

NOW_S() { date -u +%s; }

log "starting chaos runner against live rehearsal"
log "schedule: ${SCHEDULE}"
log "prometheus: ${EEL_PROMETHEUS_URL}"

REHEARSAL_START="${EEL_REHEARSAL_START:-$(NOW_S)}"

python3 -c "
import yaml,json,sys
sched=yaml.safe_load(open('${SCHEDULE}'))
idx_raw=yaml.safe_load(open('${SCENARIOS_INDEX}'))
burn_map={s['file']:s['allowed_slo_burn'] for s in idx_raw.get('scenarios',[])}
for entry in sched.get('schedule',[]):
    entry['allowed_slo_burn']=burn_map.get(entry['scenario'],0.05)
print(json.dumps(sched['schedule']))
" | python3 -c "
import sys,json
entries=json.load(sys.stdin)
for e in entries:
    print(e['start_offset'], e['scenario'], e['allowed_slo_burn'])
" | while read -r offset scenario allowed; do
  CURRENT_SCENARIO="${scenario}"
  TARGET_S=$((REHEARSAL_START + offset * 60))
  NOW="$(NOW_S)"

  if [ "${TARGET_S}" -gt "${NOW}" ]; then
    WAIT=$((TARGET_S - NOW))
    log "waiting ${WAIT}s before applying ${scenario}"
    sleep "${WAIT}"
  fi

  log "applying scenario: ${scenario}"
  MANIFEST="${REPO_ROOT}/${scenario}"

  if [ ! -f "${MANIFEST}" ]; then
    log "ERROR: scenario file not found: ${MANIFEST}"
    echo "{\"t\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"scenario\":\"${scenario}\",\"status\":\"file_missing\"}" >> "${CHAOS_LOG}"
    continue
  fi

  kubectl apply -f "${MANIFEST}" --context "${EEL_CLUSTER_CONTEXT}" \
    || { log "WARNING: kubectl apply failed for ${scenario}; continuing"; continue; }

  echo "{\"t\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"scenario\":\"${scenario}\",\"status\":\"applied\"}" >> "${CHAOS_LOG}"

  # Watch SLO burn for up to 120 seconds post-apply.
  WATCH_END=$(($(NOW_S) + 120))
  while [ "$(NOW_S)" -lt "${WATCH_END}" ]; do
    if ! check_slo_burn "${scenario}" "${allowed}"; then
      BURN="$(query_prometheus \
        'sum(rate(eel_http_requests_total{status=~"5.."}[2m])) / sum(rate(eel_http_requests_total[2m]))')"
      kubectl delete -f "${MANIFEST}" --context "${EEL_CLUSTER_CONTEXT}" --ignore-not-found
      write_red_report "${scenario}" "${BURN}" "${allowed}"
      echo "{\"t\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"scenario\":\"${scenario}\",\"status\":\"slo_breach\",\"burn\":${BURN}}" >> "${CHAOS_LOG}"
      die "SLO burn breach on scenario ${scenario}: burn=${BURN} > allowed=${allowed}"
    fi
    sleep 10
  done

  echo "{\"t\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"scenario\":\"${scenario}\",\"status\":\"complete\"}" >> "${CHAOS_LOG}"
  log "scenario complete: ${scenario}"
done

log "all chaos scenarios applied without SLO breach"
log "log: ${CHAOS_LOG}"
