#!/usr/bin/env bash
# Run an ABIS load test scenario against a local abis-api on :8080.
# Usage: ./run-locally.sh <scenario>
# Scenarios: enrolment | match-1n | match-11 | kit-enrolment | election-day
# Env overrides: ABIS_URL, ATTEST_TOKEN, K6_OPTS, SKIP_K6_INSPECT

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ABIS_URL="${ABIS_URL:-http://localhost:8080}"
ATTEST_TOKEN="${ATTEST_TOKEN:-dev-attest-placeholder}"
SCENARIO="${1:-match-11}"

if [ -z "${SKIP_K6_INSPECT:-}" ] && command -v k6 >/dev/null 2>&1; then
  echo "k6 version: $(k6 version)"
fi

run_k6() {
  local script="$1"
  if ! command -v k6 >/dev/null 2>&1; then
    echo "SKIP: k6 not found on PATH. Install k6 >= 0.46 to run: $script"
    echo "      https://k6.io/docs/getting-started/installation/"
    exit 0
  fi
  k6 run \
    --env ABIS_URL="${ABIS_URL}" \
    --env ATTEST_TOKEN="${ATTEST_TOKEN}" \
    ${K6_OPTS:-} \
    "${SCRIPT_DIR}/k6/${script}"
}

case "${SCENARIO}" in
  enrolment)
    run_k6 enrolment-throughput.js
    ;;
  match-1n)
    run_k6 match-1n.js
    ;;
  match-11)
    run_k6 match-11.js
    ;;
  kit-enrolment)
    run_k6 kit-enrolment.js
    ;;
  election-day)
    echo "Running election-day combined profile (4 scenarios in sequence)..."
    run_k6 enrolment-throughput.js
    run_k6 match-1n.js
    run_k6 match-11.js
    run_k6 kit-enrolment.js
    ;;
  *)
    echo "Unknown scenario: ${SCENARIO}"
    echo "Usage: $0 <enrolment|match-1n|match-11|kit-enrolment|election-day>"
    exit 1
    ;;
esac
