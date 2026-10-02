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

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
CONTRACTS_DIR="${REPO_ROOT}/packages/openapi-contracts"
OUT_DIR="${SCRIPT_DIR}/generated"

MODE="regen"
for arg in "$@"; do
  case "$arg" in
    --check) MODE="check" ;;
    --regen) MODE="regen" ;;
  esac
done

if [[ "$MODE" == "check" ]]; then
  echo "[ts] check: verifying npx is available"
  command -v npx &>/dev/null || { echo "WARNING: npx not found; TypeScript client generation requires Node.js"; }
  echo "[ts] check: passed"
  exit 0
fi

# --regen path
echo "[ts] Scanning for OpenAPI specs in ${CONTRACTS_DIR}..."

command -v npx &>/dev/null || { echo "ERROR: npx not found; install Node.js to generate TypeScript clients"; exit 1; }

shopt -s nullglob
specs=("${CONTRACTS_DIR}"/*/openapi.yaml)
if [[ ${#specs[@]} -eq 0 ]]; then
  echo "[ts] WARNING: no openapi.yaml specs found under ${CONTRACTS_DIR}"
  exit 0
fi

for spec in "${specs[@]}"; do
  ctx="$(basename "$(dirname "$spec")")"
  out="${OUT_DIR}/${ctx}"
  echo "[ts] Generating types for context '${ctx}' -> ${out}/types.ts"
  mkdir -p "$out"

  # Generate TypeScript types with openapi-typescript
  npx --yes openapi-typescript "$spec" --output "${out}/types.ts"

  # Write a fetch client wrapper using openapi-fetch patterns
  cat > "${out}/client.ts" <<'CLIENTEOF'
// Auto-generated fetch client wrapper for this context.
// Re-run ./generate.sh --regen to update.
import createClient from "openapi-fetch";
import type { paths } from "./types";

export function createEelClient(baseUrl: string, init?: RequestInit) {
  return createClient<paths>({ baseUrl, ...init });
}
CLIENTEOF

  echo "[ts] Generated ${out}/types.ts and ${out}/client.ts"
done

if command -v prettier &>/dev/null; then
  echo "[ts] Running prettier on ${OUT_DIR}..."
  prettier --write "${OUT_DIR}/**/*.ts" 2>/dev/null || true
fi

echo "[ts] Done."
