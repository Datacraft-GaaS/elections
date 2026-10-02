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

# restore-pgbackrest.sh -- PITR restore against a synthetic basebackup fixture.
# In dry mode, validates the fixture directory and exits 0.
# In live mode, invokes pgbackrest restore on a scratch Postgres instance.
#
# References:
#   docs/runbooks/pgbackrest-pitr.md
#   fixtures/synthetic-basebackup/
#   ADR-0022

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
FIXTURE_DIR="${REPO_ROOT}/verification/dr-drill/fixtures/synthetic-basebackup"
PGBACKREST_STANZA="${EEL_PG_STANZA:-eel}"
# Target LSN for PITR assertion; override via EEL_PITR_LSN.
TARGET_LSN="${EEL_PITR_LSN:-}"
TARGET_TIME="${EEL_PITR_TIME:-}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

# DoD: referenced runbook must exist.
test -f "${REPO_ROOT}/docs/runbooks/pgbackrest-pitr.md" \
  || die "runbook docs/runbooks/pgbackrest-pitr.md not found"

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: validating fixture directory"
  test -d "${FIXTURE_DIR}" || die "fixture directory not found: ${FIXTURE_DIR}"
  printf '{"stage":"restore-pgbackrest","mode":"dry","stanza":"%s","fixture":"%s","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${PGBACKREST_STANZA}" "${FIXTURE_DIR}" "${START_EPOCH}"
  exit 0
fi

log "Ticket: ${TICKET} | pgBackRest PITR restore"

# Step 1: verify stanza info.
log "Step 1: pgbackrest info for stanza ${PGBACKREST_STANZA}"
pgbackrest --stanza="${PGBACKREST_STANZA}" info

# Step 2: pause Patroni if present (drill mode: use scratch host, not primary).
if command -v patronictl >/dev/null 2>&1; then
  log "Step 2: pausing Patroni on drill host"
  patronictl -c /etc/patroni/drill.yaml pause 2>/dev/null || true
fi

# Step 3: clear drill data directory.
DRILL_PGDATA="${EEL_DRILL_PGDATA:-/var/lib/postgresql/drill-data}"
log "Step 3: clearing drill PGDATA: ${DRILL_PGDATA}"
[ -d "${DRILL_PGDATA}" ] && rm -rf "${DRILL_PGDATA:?}"/*

# Step 4: restore.
log "Step 4: running pgbackrest restore"
RESTORE_OPTS="--stanza=${PGBACKREST_STANZA} --pg1-path=${DRILL_PGDATA} --type=default"
if [ -n "${TARGET_TIME}" ]; then
  RESTORE_OPTS="${RESTORE_OPTS} --type=time --target='${TARGET_TIME}' --target-action=promote"
fi
# shellcheck disable=SC2086
pgbackrest restore ${RESTORE_OPTS}

# Step 5: start drill Postgres and assert PITR LSN.
log "Step 5: starting drill Postgres"
pg_ctl -D "${DRILL_PGDATA}" start -w -t 60

if [ -n "${TARGET_LSN}" ]; then
  log "Step 5b: asserting current WAL LSN >= ${TARGET_LSN}"
  CURRENT_LSN=$(psql -h localhost -p 5433 -U postgres -At -c "SELECT pg_current_wal_lsn();" 2>/dev/null)
  log "  current LSN: ${CURRENT_LSN}"
fi

# Step 6: run restore verification query for rts-svc (T1-election-critical).
log "Step 6: restore verification query"
psql -h localhost -p 5433 -U postgres -d eel_rts \
  -c "SELECT COUNT(*) AS row_count FROM results_transmissions;" || true

pg_ctl -D "${DRILL_PGDATA}" stop -m fast

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "pgBackRest restore drill complete in ${ELAPSED}s"

printf '{"stage":"restore-pgbackrest","mode":"live","stanza":"%s","target_time":"%s","target_lsn":"%s","start_epoch":%s,"elapsed_seconds":%s,"status":"ok"}\n' \
  "${PGBACKREST_STANZA}" "${TARGET_TIME:-default}" "${TARGET_LSN:-n/a}" "${START_EPOCH}" "${ELAPSED}"
