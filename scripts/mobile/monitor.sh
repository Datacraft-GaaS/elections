#!/usr/bin/env bash
# monitor.sh — Fleet health monitoring.
#
# Usage: scripts/mobile/monitor.sh [--env staging|production] [--watch] [--annotate]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

ENV="staging"
WATCH=false
ANNOTATE=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env) ENV="$2"; shift 2 ;;
    --watch) WATCH=true; shift ;;
    --annotate) ANNOTATE=true; shift ;;
    *) shift ;;
  esac
done

_log_info "Monitoring fleet health (env=$ENV)..."

# Query telemetry-ingest for per-app metrics
get_metric() {
  local metric="$1"
  curl -s "http://localhost:9090/metrics/$metric?env=$ENV" 2>/dev/null || echo "0"
}

# Dashboard metrics
sessions=$(get_metric "sessions")
sync_backlog=$(get_metric "sync_backlog_p95")
crash_free=$(get_metric "crash_free_rate")
attestation_failures=$(get_metric "attestation_failures_pct")

_log_info "Sessions: $sessions"
_log_info "Sync backlog p95: $sync_backlog"
_log_info "Crash-free rate: $crash_free%"
_log_info "Attestation failures: $attestation_failures%"

# Annotate Grafana
if $ANNOTATE; then
  curl -X POST "http://grafana:3000/api/annotations" \
    -H "Content-Type: application/json" \
    -d "{\"time\": $(date +%s)000, \"text\": \"eel-mobile $ENV metrics: sessions=$sessions crash_free=$crash_free%\"}" \
    2>/dev/null || _log_warn "Grafana annotation failed"
fi

# Loop for --watch
if $WATCH; then
  while true; do
    clear
    _log_info "Fleet health (env=$ENV) — $(date)"
    _log_info "Sessions: $sessions"
    _log_info "Sync backlog p95: $sync_backlog"
    _log_info "Crash-free rate: $crash_free%"
    _log_info "Attestation failures: $attestation_failures%"
    sleep 60
  done
fi
