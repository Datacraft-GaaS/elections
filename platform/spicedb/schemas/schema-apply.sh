#!/usr/bin/env bash
# Apply all EEL SpiceDB schemas to the target endpoint.
# Usage: SPICEDB_ENDPOINT=... SPICEDB_TOKEN=... ./schema-apply.sh
set -euo pipefail

SPICEDB_ENDPOINT="${SPICEDB_ENDPOINT:-localhost:50051}"
SPICEDB_TOKEN="${SPICEDB_TOKEN:-dev-preshared-key}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZED="$(cd "${SCRIPT_DIR}/../../.." && pwd)/tools/spicedb/zed"

if [[ ! -x "$ZED" ]]; then
  echo "ERROR: SpiceDB CLI not found at $ZED" >&2
  exit 1
fi

ZED_FILES=(
  abis.zed
  ai_platform.zed
  boundary.zed
  bvi.zed
  bvr.zed
  crms.zed
  custody.zed
  cvr.zed
  ecc.zed
  field_mobile.zed
  gazette.zed
  governance.zed
  grievance.zed
  hello.zed
  iab.zed
  inspection.zed
  inventory.zed
  party.zed
  portal.zed
  queue.zed
  rts.zed
  support_admin.zed
  training.zed
  transparency.zed
  vprs.zed
)

FAILURES=0

for zed_file in "${ZED_FILES[@]}"; do
  target="${SCRIPT_DIR}/${zed_file}"
  if [[ ! -f "$target" ]]; then
    echo "SKIP  $zed_file (not present in ${SCRIPT_DIR})"
    continue
  fi

  if "$ZED" schema write \
      --endpoint "$SPICEDB_ENDPOINT" \
      --token "$SPICEDB_TOKEN" \
      --schema-file "$target"; then
    echo "OK    $zed_file"
  else
    echo "FAIL  $zed_file" >&2
    FAILURES=$(( FAILURES + 1 ))
  fi
done

if (( FAILURES > 0 )); then
  echo "ERROR: $FAILURES schema(s) failed to apply." >&2
  exit 1
fi

echo "All schemas applied successfully."
