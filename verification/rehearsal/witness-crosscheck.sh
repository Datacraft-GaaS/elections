#!/usr/bin/env bash
# Witness cross-check for the election-day rehearsal.
# Pings all 5 witness channels and asserts they agree on the latest Signed Tree Head (STH).
# Fails (exit 1) if any channel disagrees or is unreachable.
#
# Environment variables (all required in live mode):
#   EEL_WITNESS_GOV_WEB_URL     -- government official web witness endpoint
#   EEL_WITNESS_GITHUB_URL      -- GitHub mirror witness endpoint
#   EEL_WITNESS_IPFS_KE_URL     -- IPFS-KE witness endpoint
#   EEL_WITNESS_GAZETTE_URL     -- Kenya Gazette witness endpoint
#   EEL_WITNESS_KBC_URL         -- KBC witness endpoint
#
# In dry mode (EEL_REHEARSAL_MODE=dry) the script logs and exits 0.
set -euo pipefail

DRY="${EEL_REHEARSAL_MODE:-live}"
log() { echo "[witness-crosscheck] $*" >&2; }
die() { echo "[witness-crosscheck] FAIL: $*" >&2; exit 1; }

if [ "${DRY}" = "dry" ]; then
  log "dry mode: skipping witness channel pings"
  exit 0
fi

# Channels and their env var names.
declare -A CHANNELS=(
  [gov-web]="${EEL_WITNESS_GOV_WEB_URL:-}"
  [github-mirror]="${EEL_WITNESS_GITHUB_URL:-}"
  [ipfs-ke]="${EEL_WITNESS_IPFS_KE_URL:-}"
  [gazette]="${EEL_WITNESS_GAZETTE_URL:-}"
  [kbc]="${EEL_WITNESS_KBC_URL:-}"
)

fetch_sth() {
  local name="$1"
  local url="$2"
  if [ -z "${url}" ]; then
    die "channel '${name}': URL not configured (set EEL_WITNESS_${name^^}_URL)"
  fi
  # Expects JSON with a "tree_head_hash" field.
  curl -fsSL --max-time 30 "${url}/sth" \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['tree_head_hash'])" \
    2>/dev/null \
    || die "channel '${name}': failed to fetch STH from ${url}/sth"
}

log "fetching STH from all 5 witness channels"

REFERENCE_HASH=""
AGREE=0
DISAGREE=0

for channel in "${!CHANNELS[@]}"; do
  url="${CHANNELS[$channel]}"
  hash="$(fetch_sth "${channel}" "${url}")"
  log "channel ${channel}: STH=${hash}"

  if [ -z "${REFERENCE_HASH}" ]; then
    REFERENCE_HASH="${hash}"
    AGREE=1
  elif [ "${hash}" = "${REFERENCE_HASH}" ]; then
    AGREE=$((AGREE + 1))
  else
    log "MISMATCH: channel ${channel} has STH=${hash}, expected=${REFERENCE_HASH}"
    DISAGREE=$((DISAGREE + 1))
  fi
done

log "agreement=${AGREE}/5 disagreements=${DISAGREE}"

if [ "${DISAGREE}" -gt 0 ]; then
  die "${DISAGREE} witness channel(s) disagree on the latest STH"
fi

if [ "${AGREE}" -lt 5 ]; then
  die "only ${AGREE}/5 channels responded; all 5 required"
fi

log "all 5 witness channels agree on STH=${REFERENCE_HASH}"
exit 0
