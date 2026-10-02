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

# Bats-style tests for load-5x scripts (WP-6.4).
# Run with: bash verification/load-5x/tests/test_scripts.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOAD5X_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${LOAD5X_DIR}/../.." && pwd)"

PASS=0
FAIL=0

ok() {
  echo "  ok: $1"
  PASS=$((PASS + 1))
}

not_ok() {
  echo "  FAIL: $1"
  FAIL=$((FAIL + 1))
}

echo "# load-5x script tests"

# 1. Shell script syntax checks.
echo "# 1. bash -n syntax checks"
for script in \
  "${LOAD5X_DIR}/regression-gate.sh" \
  "${LOAD5X_DIR}/kafka-100k/producer.sh"; do
  if bash -n "${script}" 2>&1; then
    ok "$(basename ${script}) passes bash -n"
  else
    not_ok "$(basename ${script}) fails bash -n"
  fi
done

# 2. k6 inspect on each .js file (skip with SKIP marker if k6 not on PATH).
echo "# 2. k6 inspect"
K6_SCRIPTS=(
  "${LOAD5X_DIR}/k6/5x/rts-submit.js"
  "${LOAD5X_DIR}/k6/5x/bvi-auth.js"
  "${LOAD5X_DIR}/k6/5x/portal-view.js"
  "${LOAD5X_DIR}/k6/5x/verifier-lookup.js"
)

if which k6 > /dev/null 2>&1; then
  for script in "${K6_SCRIPTS[@]}"; do
    if k6 inspect "${script}" > /dev/null 2>&1; then
      ok "k6 inspect: $(basename ${script})"
    else
      not_ok "k6 inspect failed: $(basename ${script})"
    fi
  done
else
  echo "  SKIP: k6 not on PATH -- skipping k6 inspect (CI should have k6 installed)"
  for script in "${K6_SCRIPTS[@]}"; do
    echo "  SKIP: $(basename ${script})"
  done
fi

# 3. k6 .js files have required thresholds block.
echo "# 3. k6 thresholds defined"
for script in "${K6_SCRIPTS[@]}"; do
  if grep -q "thresholds" "${script}"; then
    ok "$(basename ${script}) has thresholds"
  else
    not_ok "$(basename ${script}) missing thresholds"
  fi
done

# 4. k6 .js files have http_req_duration threshold.
echo "# 4. k6 http_req_duration threshold"
for script in "${K6_SCRIPTS[@]}"; do
  if grep -q "http_req_duration" "${script}"; then
    ok "$(basename ${script}) has http_req_duration threshold"
  else
    not_ok "$(basename ${script}) missing http_req_duration threshold"
  fi
done

# 5. Gatling 5xSimulation.scala syntactic sanity via grep.
echo "# 5. Gatling 5xSimulation.scala sanity checks"
SCALA="${LOAD5X_DIR}/gatling/5xSimulation.scala"
for pattern in \
  "class FiveXSimulation" \
  "extends Simulation" \
  "setUp(" \
  ".assertions(" \
  "responseTime.percentile(99)"; do
  if grep -q "${pattern}" "${SCALA}"; then
    ok "5xSimulation.scala contains: ${pattern}"
  else
    not_ok "5xSimulation.scala missing: ${pattern}"
  fi
done

# 6. 5x profile YAML is valid and has required keys.
echo "# 6. 5x profile YAML"
PROFILE="${LOAD5X_DIR}/profiles/5x.yaml"
if python3 -c "import yaml; yaml.safe_load(open('${PROFILE}'))" 2>&1; then
  ok "profiles/5x.yaml parses without error"
else
  not_ok "profiles/5x.yaml YAML parse failed"
fi

for key in scaling_factor bvi_req_per_sec rts_concurrent kafka_events_per_sec write_path_p99_ms; do
  if python3 -c "import yaml; d=yaml.safe_load(open('${PROFILE}')); assert '${key}' in d" 2>&1; then
    ok "profiles/5x.yaml has key: ${key}"
  else
    not_ok "profiles/5x.yaml missing key: ${key}"
  fi
done

# 7. regression-gate.sh dry mode exits 0.
echo "# 7. regression-gate.sh dry mode"
if EEL_5X_MODE=dry bash "${LOAD5X_DIR}/regression-gate.sh" > /dev/null 2>&1; then
  ok "regression-gate.sh exits 0 in dry mode"
else
  not_ok "regression-gate.sh exited non-zero in dry mode"
fi

# 8. Fixture report exists and is marked as fixture.
echo "# 8. fixture report"
FIXTURE="${LOAD5X_DIR}/fixtures/local-5x-report.md"
if [ -f "${FIXTURE}" ]; then
  if grep -qi "fixture" "${FIXTURE}"; then
    ok "fixture exists and is marked as fixture"
  else
    not_ok "fixture exists but not marked as fixture"
  fi
else
  not_ok "fixture not found: ${FIXTURE}"
fi

echo ""
echo "# result: ${PASS} passed, ${FAIL} failed"
[ "${FAIL}" -eq 0 ] || exit 1
