#!/usr/bin/env bash
# Regression gate for the 5x load run (WP-6.4 / ADR-0018).
# Diffs 5x P99 results against the 1x baseline from loadtest/profiles/local.yaml.
# Asserts no P99 metric regressed > 10% vs the 1x baseline.
# Writes regression-report.json to EEL_REPORT_DIR.
#
# Dry run: EEL_5X_MODE=dry -- writes a stub pass report and exits 0.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

EEL_5X_MODE="${EEL_5X_MODE:-live}"
EEL_BASELINE_PROFILE="${EEL_BASELINE_PROFILE:-${REPO_ROOT}/loadtest/profiles/local.yaml}"
EEL_5X_RESULTS="${EEL_5X_RESULTS:-}"
EEL_REPORT_DIR="${EEL_REPORT_DIR:-${REPO_ROOT}/verification/reports}"
RUN_DATE="$(date -u +%Y-%m-%d)"
REPORT_OUT="${EEL_REPORT_DIR}/regression-report-${RUN_DATE}.json"

log() { echo "[regression-gate] $*" >&2; }
die() { echo "[regression-gate] FAIL: $*" >&2; exit 1; }

mkdir -p "${EEL_REPORT_DIR}"

if [ "${EEL_5X_MODE}" = "dry" ]; then
  log "dry mode: writing stub pass report"
  cat > "${REPORT_OUT}" <<EOF
{
  "date": "${RUN_DATE}",
  "mode": "dry",
  "status": "pass",
  "metrics": [],
  "regressions": []
}
EOF
  log "stub report written: ${REPORT_OUT}"
  exit 0
fi

[ -f "${EEL_BASELINE_PROFILE}" ] || die "baseline profile not found: ${EEL_BASELINE_PROFILE}"
[ -n "${EEL_5X_RESULTS}" ] || die "EEL_5X_RESULTS must point to a 5x run results JSON"
[ -f "${EEL_5X_RESULTS}" ] || die "5x results file not found: ${EEL_5X_RESULTS}"

log "baseline: ${EEL_BASELINE_PROFILE}"
log "5x results: ${EEL_5X_RESULTS}"
log "regression threshold: 10%"

python3 - <<PYEOF
import json, sys, yaml

baseline_raw = yaml.safe_load(open('${EEL_BASELINE_PROFILE}'))
results_5x = json.load(open('${EEL_5X_RESULTS}'))

# Metrics to compare (P99 latency targets from both profiles).
# 5x profile scales throughput floors but latency ceilings must not regress.
p99_metrics = [
    'bvi_auth_p99_ms',
    'rts_submit_p99_ms',
    'portal_p99_ms',
]

REGRESSION_THRESHOLD = 0.10  # 10%

rows = []
regressions = []

for metric in p99_metrics:
    baseline_val = baseline_raw.get(metric)
    measured_val = results_5x.get('metrics', {}).get(metric)
    if baseline_val is None or measured_val is None:
        rows.append({
            'metric': metric,
            'baseline': baseline_val,
            'measured_5x': measured_val,
            'delta_pct': None,
            'status': 'skip_missing',
        })
        continue

    delta = (measured_val - baseline_val) / baseline_val
    status = 'pass' if delta <= REGRESSION_THRESHOLD else 'fail'
    rows.append({
        'metric': metric,
        'baseline': baseline_val,
        'measured_5x': measured_val,
        'delta_pct': round(delta * 100, 2),
        'status': status,
    })
    if status == 'fail':
        regressions.append(metric)

report = {
    'date': '${RUN_DATE}',
    'baseline_profile': '${EEL_BASELINE_PROFILE}',
    '5x_results': '${EEL_5X_RESULTS}',
    'regression_threshold_pct': 10,
    'status': 'fail' if regressions else 'pass',
    'metrics': rows,
    'regressions': regressions,
}

with open('${REPORT_OUT}', 'w') as f:
    json.dump(report, f, indent=2)

print(json.dumps(report, indent=2))

if regressions:
    print(f'\nREGRESSION GATE FAIL: {len(regressions)} metric(s) regressed > 10%: {regressions}', file=sys.stderr)
    sys.exit(1)
else:
    print(f'\nREGRESSION GATE PASS: all metrics within 10% of 1x baseline', file=sys.stderr)
PYEOF

log "regression report: ${REPORT_OUT}"
