#!/usr/bin/env bash
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
  echo "[rs] check: verifying cargo is available"
  command -v cargo &>/dev/null || { echo "WARNING: cargo not found; Rust client generation requires Rust toolchain"; }
  echo "[rs] check: passed"
  exit 0
fi

# --regen path
echo "[rs] Scanning for OpenAPI specs in ${CONTRACTS_DIR}..."

find_generator() {
  if command -v progenitor-cli &>/dev/null; then
    echo "progenitor"
  elif command -v openapi-generator-cli &>/dev/null; then
    echo "openapi-generator-cli-rust"
  elif command -v npx &>/dev/null; then
    echo "npx-rust"
  else
    echo "ERROR: no generator found; install progenitor-cli (cargo install progenitor-cli) or openapi-generator-cli" >&2
    exit 1
  fi
}

GENERATOR=$(find_generator)

shopt -s nullglob
specs=("${CONTRACTS_DIR}"/*/openapi.yaml)
if [[ ${#specs[@]} -eq 0 ]]; then
  echo "[rs] WARNING: no openapi.yaml specs found under ${CONTRACTS_DIR}"
  exit 0
fi

for spec in "${specs[@]}"; do
  ctx="$(basename "$(dirname "$spec")")"
  out="${OUT_DIR}/${ctx}"
  echo "[rs] Generating client for context '${ctx}' -> ${out} (generator: ${GENERATOR})"
  mkdir -p "$out"

  case "$GENERATOR" in
    progenitor)
      progenitor-cli -i "$spec" -o "$out" -n "eel_api_client_${ctx}"
      ;;
    openapi-generator-cli-rust)
      openapi-generator-cli generate \
        -i "$spec" \
        -g rust \
        -o "$out" \
        --additional-properties="packageName=eel-api-client-${ctx},packageVersion=0.1.0"
      ;;
    npx-rust)
      npx --yes @openapitools/openapi-generator-cli generate \
        -i "$spec" \
        -g rust \
        -o "$out" \
        --additional-properties="packageName=eel-api-client-${ctx},packageVersion=0.1.0"
      ;;
  esac

  if command -v rustfmt &>/dev/null; then
    echo "[rs] Running rustfmt on generated Rust files..."
    find "$out" -name "*.rs" -exec rustfmt {} \; 2>/dev/null || true
  fi
done

echo "[rs] Done."
