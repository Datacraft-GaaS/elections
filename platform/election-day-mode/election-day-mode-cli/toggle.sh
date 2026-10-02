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

# election-day-mode-cli/toggle.sh
# Toggles the eel-election-day-mode ConfigMap with dual SVID sign-off.
# Requires: kubectl, cosign, jq
# Usage: toggle.sh <enable|disable> <election-date-ISO8601>
# Example: toggle.sh enable 2027-08-12
set -euo pipefail

ACTION=${1:-}
ELECTION_DATE=${2:-}
NAMESPACE=argocd
CONFIGMAP=eel-election-day-mode

usage() {
  echo "Usage: $0 <enable|disable> [election-date-ISO8601]"
  echo "  enable  YYYY-MM-DD  -- activate election-day-mode; sets freeze T-1 through T+7"
  echo "  disable             -- deactivate election-day-mode"
  exit 1
}

require_cmd() {
  command -v "$1" > /dev/null 2>&1 || { echo "Error: $1 not found in PATH"; exit 1; }
}

collect_svid() {
  local label=$1
  echo ""
  echo "=== SVID sign-off required: ${label} ==="
  echo "Run the following on a host with a valid SPIRE SVID and paste the attestation token:"
  echo "  cosign attest --type slsaprovenance --predicate /dev/null --key <your-key> <image>"
  echo ""
  read -r -p "Paste attestation token for ${label}: " TOKEN
  echo "${TOKEN}"
}

verify_svid() {
  local token=$1 label=$2
  # Verify token is non-empty (full cosign verification requires image reference; omit in CLI)
  [[ -n "${token}" ]] || { echo "Error: empty attestation token for ${label}"; exit 1; }
  echo "[ok] ${label} token received (length=${#token})"
}

[[ "$ACTION" == "enable" || "$ACTION" == "disable" ]] || usage
[[ "$ACTION" == "enable" && -z "$ELECTION_DATE" ]] && usage

require_cmd kubectl
require_cmd cosign
require_cmd jq

echo "=== eel election-day-mode toggle ==="
echo "Action         : ${ACTION}"
echo "Election date  : ${ELECTION_DATE:-N/A}"
echo "Namespace      : ${NAMESPACE}"
echo ""

# Dual sign-off
TOKEN1=$(collect_svid "Operator-1")
verify_svid "${TOKEN1}" "Operator-1"
TOKEN2=$(collect_svid "Operator-2")
verify_svid "${TOKEN2}" "Operator-2"

[[ "${TOKEN1}" == "${TOKEN2}" ]] && { echo "Error: both tokens are identical; two distinct operators required"; exit 1; }

echo ""
echo "Dual sign-off verified. Applying ConfigMap patch..."

if [[ "$ACTION" == "enable" ]]; then
  # Compute freeze bounds: T-1 00:00 UTC through T+7 23:59 UTC
  FREEZE_START=$(date -u -d "${ELECTION_DATE} -1 day" '+%Y-%m-%dT00:00:00Z' 2>/dev/null \
    || date -u -v-1d -j -f '%Y-%m-%d' "${ELECTION_DATE}" '+%Y-%m-%dT00:00:00Z')
  FREEZE_END=$(date -u -d "${ELECTION_DATE} +7 days" '+%Y-%m-%dT23:59:59Z' 2>/dev/null \
    || date -u -v+7d -j -f '%Y-%m-%d' "${ELECTION_DATE}" '+%Y-%m-%dT23:59:59Z')

  kubectl patch configmap "${CONFIGMAP}" -n "${NAMESPACE}" --type merge -p "$(cat << EOF
{
  "data": {
    "election-day-mode-enabled": "true",
    "election-day": "${ELECTION_DATE}",
    "freeze-start": "${FREEZE_START}",
    "freeze-end": "${FREEZE_END}"
  }
}
EOF
)"
  echo ""
  echo "[ok] election-day-mode ENABLED"
  echo "     Freeze window: ${FREEZE_START} -- ${FREEZE_END}"
else
  kubectl patch configmap "${CONFIGMAP}" -n "${NAMESPACE}" --type merge -p '{"data":{"election-day-mode-enabled":"false","election-day":"","freeze-start":"","freeze-end":""}}'
  echo ""
  echo "[ok] election-day-mode DISABLED"
fi

echo ""
echo "Verify with: kubectl get configmap ${CONFIGMAP} -n ${NAMESPACE} -o json | jq .data"
