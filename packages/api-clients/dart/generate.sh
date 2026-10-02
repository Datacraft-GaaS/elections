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
CONFIG="${SCRIPT_DIR}/openapi-generator-config.yaml"

MODE="regen"
for arg in "$@"; do
  case "$arg" in
    --check) MODE="check" ;;
    --regen) MODE="regen" ;;
  esac
done

if [[ "$MODE" == "check" ]]; then
  echo "[dart] check: config exists at ${CONFIG}"
  [[ -f "$CONFIG" ]] || { echo "ERROR: config not found at ${CONFIG}"; exit 1; }
  echo "[dart] check: passed"
  exit 0
fi

# --regen path
echo "[dart] Scanning for OpenAPI specs in ${CONTRACTS_DIR}..."

find_generator() {
  if command -v openapi-generator-cli &>/dev/null; then
    echo "openapi-generator-cli"
  elif command -v npx &>/dev/null; then
    echo "npx @openapitools/openapi-generator-cli"
  elif command -v docker &>/dev/null; then
    echo "docker run --rm -v ${REPO_ROOT}:/local openapitools/openapi-generator-cli"
  else
    echo "ERROR: no openapi-generator-cli found (install via npm, brew, or docker)" >&2
    exit 1
  fi
}

GENERATOR=$(find_generator)

shopt -s nullglob
specs=("${CONTRACTS_DIR}"/*/openapi.yaml)
if [[ ${#specs[@]} -eq 0 ]]; then
  echo "[dart] WARNING: no openapi.yaml specs found under ${CONTRACTS_DIR}"
  exit 0
fi

for spec in "${specs[@]}"; do
  ctx="$(basename "$(dirname "$spec")")"
  out="${OUT_DIR}/${ctx}"
  echo "[dart] Generating client for context '${ctx}' -> ${out}"
  mkdir -p "$out"
  # shellcheck disable=SC2086
  $GENERATOR generate \
    -i "$spec" \
    -g dart-dio \
    -o "$out" \
    -c "$CONFIG" \
    --additional-properties="pubName=eel_api_client_${ctx}"
done

if command -v dart &>/dev/null; then
  echo "[dart] Running dart format on ${OUT_DIR}..."
  dart format "$OUT_DIR"
fi

echo "[dart] Done."
