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

# fuzz/run-corpus.sh -- run cargo-fuzz for each target in 24h-schedule.yaml.
# Uploads coverage to codecov-compatible endpoint.
# Exits non-zero if any new crashes are detected.
# Set EEL_DR_MODE=dry to skip live fuzzing (syntax check only).
#
# Prerequisites: cargo-fuzz installed (cargo install cargo-fuzz).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
SCHEDULE_YAML="${SCRIPT_DIR}/24h-schedule.yaml"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
COVERAGE_URL="${EEL_COVERAGE_URL:-}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

START_EPOCH=$(date +%s)
NEW_CRASHES=0

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live fuzzing"
  test -f "${SCHEDULE_YAML}" || die "24h-schedule.yaml not found: ${SCHEDULE_YAML}"
  printf '{"stage":"fuzz-run-corpus","mode":"dry","start_epoch":%s,"status":"ok","new_crashes":0}\n' \
    "${START_EPOCH}"
  exit 0
fi

command -v cargo-fuzz >/dev/null 2>&1 || die "cargo-fuzz not installed; run: cargo install cargo-fuzz"

test -f "${SCHEDULE_YAML}" || die "Schedule not found: ${SCHEDULE_YAML}"

# Parse targets from YAML using python3.
TARGETS=$(python3 - <<PYEOF
import yaml, json, sys
with open('${SCHEDULE_YAML}') as f:
    sched = yaml.safe_load(f)
for t in sched.get('targets', []):
    print(json.dumps(t))
PYEOF
)

log "Starting fuzz run; $(echo "${TARGETS}" | wc -l | tr -d ' ') target(s)"

while IFS= read -r TARGET_JSON; do
  NAME=$(echo "${TARGET_JSON}" | python3 -c "import sys,json; print(json.load(sys.stdin)['name'])")
  CRATE=$(echo "${TARGET_JSON}" | python3 -c "import sys,json; print(json.load(sys.stdin)['crate'])")
  CORPUS=$(echo "${TARGET_JSON}" | python3 -c "import sys,json; print(json.load(sys.stdin)['corpus_dir'])")
  MAX_ITER=$(echo "${TARGET_JSON}" | python3 -c "import sys,json; print(json.load(sys.stdin)['max_iterations'])")

  CRATE_PATH="${REPO_ROOT}/${CRATE}"
  CORPUS_PATH="${REPO_ROOT}/${CORPUS}"
  CRASH_DIR="${CRATE_PATH}/fuzz/artifacts/${NAME}"

  if [ ! -d "${CRATE_PATH}" ]; then
    log "  SKIP ${NAME}: crate path not found: ${CRATE_PATH}"
    continue
  fi

  mkdir -p "${CORPUS_PATH}"

  log "  Fuzzing ${NAME} (max_iterations=${MAX_ITER}) in ${CRATE}"
  cargo fuzz run "${NAME}" \
    --manifest-path "${CRATE_PATH}/Cargo.toml" \
    "${CORPUS_PATH}" \
    -- -max_total_time=$((MAX_ITER / 10000)) \
       -runs="${MAX_ITER}" \
       -artifact_prefix="${CRASH_DIR}/" \
    2>/dev/null || FUZZ_RC=$?

  FUZZ_RC="${FUZZ_RC:-0}"
  if [ "${FUZZ_RC}" -ne 0 ]; then
    CRASH_COUNT=$(find "${CRASH_DIR}" -type f -name "crash-*" 2>/dev/null | wc -l | tr -d ' ')
    if [ "${CRASH_COUNT}" -gt 0 ]; then
      log "  NEW CRASHES in ${NAME}: ${CRASH_COUNT}"
      NEW_CRASHES=$((NEW_CRASHES + CRASH_COUNT))
    fi
  fi

  # Upload coverage if endpoint configured.
  if [ -n "${COVERAGE_URL}" ]; then
    COV_FILE="${CRATE_PATH}/target/fuzz-coverage/${NAME}/coverage.profdata"
    if [ -f "${COV_FILE}" ]; then
      curl -sf --max-time 30 -F "coverage=@${COV_FILE}" \
        "${COVERAGE_URL}?target=${NAME}" >/dev/null 2>&1 || log "  (coverage upload failed for ${NAME})"
    fi
  fi

done <<< "${TARGETS}"

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "Fuzz run complete in ${ELAPSED}s; new crashes: ${NEW_CRASHES}"

printf '{"stage":"fuzz-run-corpus","mode":"live","start_epoch":%s,"elapsed_seconds":%s,"new_crashes":%s,"status":"%s"}\n' \
  "${START_EPOCH}" "${ELAPSED}" "${NEW_CRASHES}" "$( [ "${NEW_CRASHES}" -eq 0 ] && echo ok || echo fail)"

[ "${NEW_CRASHES}" -eq 0 ] || die "${NEW_CRASHES} new crash(es) detected. Review artifacts and file findings."
