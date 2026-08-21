#!/usr/bin/env bash
# Run SpiceDB golden tests against the configured endpoint.
# Applies all schemas then checks each test case in the YAML test files.
# Usage: SPICEDB_ENDPOINT=... SPICEDB_TOKEN=... ./golden-test.sh
set -euo pipefail

SPICEDB_ENDPOINT="${SPICEDB_ENDPOINT:-localhost:50051}"
SPICEDB_TOKEN="${SPICEDB_TOKEN:-dev-preshared-key}"

export SPICEDB_ENDPOINT SPICEDB_TOKEN

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
ZED="${REPO_ROOT}/tools/spicedb/zed"
TESTS_DIR="${REPO_ROOT}/packages/authz-schema/tests"

if [[ ! -x "$ZED" ]]; then
  echo "ERROR: SpiceDB CLI not found at $ZED" >&2
  exit 1
fi

echo "==> Applying schemas..."
bash "${SCRIPT_DIR}/schema-apply.sh"

TEST_FILES=(
  abis_golden.yaml
  ai_platform_test.yaml
  boundary_test.yaml
  bvi_golden.yaml
  bvr_golden.yaml
  crms_test.yaml
  custody_golden.yaml
  cvr_test.yaml
  ecc_test.yaml
  field_mobile_test.yaml
  gazette_test.yaml
  governance_test.yaml
  grievance_test.yaml
  iab_test.yaml
  inspection_test.yaml
  inventory_test.yaml
  party_test.yaml
  portal_test.yaml
  queue_test.yaml
  rts_golden.yaml
  support_admin_test.yaml
  training_test.yaml
  transparency_golden.yaml
  vprs_test.yaml
)

echo "==> Running permission checks..."

TOTAL=0
FAILURES=0

# Use python3 to parse YAML and emit check lines, then evaluate each.
for test_file in "${TEST_FILES[@]}"; do
  yaml_path="${TESTS_DIR}/${test_file}"
  if [[ ! -f "$yaml_path" ]]; then
    echo "SKIP  $test_file (not found)"
    continue
  fi

  # python3 reads the YAML and prints tab-separated lines:
  # <resource_type>:<resource_id>  <permission>  <subject_type>:<subject_id>  <expected>
  while IFS=$'\t' read -r resource permission subject expected; do
    TOTAL=$(( TOTAL + 1 ))
    output=$("$ZED" permission check \
      --endpoint "$SPICEDB_ENDPOINT" \
      --token "$SPICEDB_TOKEN" \
      --subject "$subject" \
      --resource "$resource" \
      --permission "$permission" 2>&1 || true)

    if echo "$output" | grep -qi "true"; then
      actual="true"
    else
      actual="false"
    fi

    if [[ "$actual" == "$expected" ]]; then
      echo "PASS  [${test_file}] ${resource} ${permission} ${subject} => ${expected}"
    else
      echo "FAIL  [${test_file}] ${resource} ${permission} ${subject} => expected ${expected}, got ${actual}" >&2
      FAILURES=$(( FAILURES + 1 ))
    fi
  done < <(python3 - "$yaml_path" <<'PYEOF'
import sys, yaml

path = sys.argv[1]
with open(path) as fh:
    doc = yaml.safe_load(fh)

# Support two shapes:
#   tests:
#     - resource: "type:id"
#       permission: "..."
#       subject: "type:id"
#       expected: true | false
#
#   checks:
#     - object: "type:id"
#       permission: "..."
#       subject: "type:id"
#       expect: "true" | "false"

entries = doc.get("tests") or doc.get("checks") or []
for entry in entries:
    resource  = entry.get("resource") or entry.get("object") or ""
    permission = entry.get("permission") or ""
    subject   = entry.get("subject") or ""
    expected  = entry.get("expected") if "expected" in entry else entry.get("expect", False)
    expected_str = "true" if str(expected).lower() in ("true", "1", "yes") else "false"
    if resource and permission and subject:
        print(f"{resource}\t{permission}\t{subject}\t{expected_str}")
PYEOF
)
done

echo ""
echo "Results: $TOTAL checks, $FAILURES failure(s)."

if (( FAILURES > 0 )); then
  exit 1
fi
