#!/usr/bin/env bash
# primary-loss.sh -- simulate primary-site loss and verify mirror promotion.
# Invokes eel-dr-failover (tools/dr-failover) to orchestrate the failover sequence.
# Set EEL_DR_MODE=dry to run without touching live infrastructure.
#
# References:
#   docs/runbooks/dr-failover-primary-to-mirror.md
#   ADR-0022

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DR_FAILOVER_BIN="${EEL_DR_FAILOVER_BIN:-"${REPO_ROOT}/tools/dr-failover/target/release/eel-dr-failover"}"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
PATRONI_TIMEOUT_SECONDS=600   # 10 min
KUBECONFIG="${EEL_KUBECONFIG:-${HOME}/.kube/config}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

# Verify referenced runbook exists (DoD check).
test -f "${REPO_ROOT}/docs/runbooks/dr-failover-primary-to-mirror.md" \
  || die "runbook docs/runbooks/dr-failover-primary-to-mirror.md not found"

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live infrastructure calls"
  printf '{"stage":"primary-loss","mode":"dry","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' "${START_EPOCH}"
  exit 0
fi

# --- live path ---

log "Ticket: ${TICKET}"
log "Using dr-failover binary: ${DR_FAILOVER_BIN}"
test -x "${DR_FAILOVER_BIN}" || die "eel-dr-failover binary not found; run: cargo build --release -p eel-dr-failover"

# Step 1: preflight -- confirms primary is unreachable.
log "Step 1: preflight check on primary site"
"${DR_FAILOVER_BIN}" preflight --site primary --ticket "${TICKET}" \
  || die "Preflight failed: primary site still responding; investigate before proceeding"

# Step 2: freeze writes on primary (best-effort, may be unreachable).
log "Step 2: freeze writes on primary"
"${DR_FAILOVER_BIN}" freeze-primary --site primary --ticket "${TICKET}" || true

# Step 3: verify mirror replication state.
log "Step 3: check mirror replication"
"${DR_FAILOVER_BIN}" check-mirror --ticket "${TICKET}"

# Step 4: promote mirror Patroni cluster.
log "Step 4: promote mirror"
"${DR_FAILOVER_BIN}" promote-mirror --ticket "${TICKET}"

# Step 5: assert Patroni promotion within PATRONI_TIMEOUT_SECONDS.
log "Step 5: waiting for Patroni leader on mirror (timeout ${PATRONI_TIMEOUT_SECONDS}s)"
DEADLINE=$(($(date +%s) + PATRONI_TIMEOUT_SECONDS))
while true; do
  LEADER=$(patronictl -c /etc/patroni/mirror.yaml list --format json 2>/dev/null \
    | python3 -c "import sys,json; members=json.load(sys.stdin); print(next((m['Member'] for m in members if m.get('Role')=='Leader'),''))" 2>/dev/null || true)
  if [ -n "${LEADER}" ]; then
    log "Patroni leader on mirror: ${LEADER}"
    break
  fi
  if [ "$(date +%s)" -gt "${DEADLINE}" ]; then
    die "Patroni promotion timed out after ${PATRONI_TIMEOUT_SECONDS}s"
  fi
  sleep 10
done

# Step 6: verify write path resumes.
log "Step 6: verify write path on mirror"
"${DR_FAILOVER_BIN}" verify-writes --site mirror --ticket "${TICKET}"

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))

log "Primary-loss drill complete in ${ELAPSED}s"
printf '{"stage":"primary-loss","mode":"live","start_epoch":%s,"elapsed_seconds":%s,"patroni_leader":"%s","status":"ok"}\n' \
  "${START_EPOCH}" "${ELAPSED}" "${LEADER:-unknown}"
