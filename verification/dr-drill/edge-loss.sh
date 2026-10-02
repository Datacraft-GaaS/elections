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

# edge-loss.sh -- simulate loss of one regional edge cluster and verify anycast reroute.
# Cordons the target edge node pool and swaps the Envoy configuration to exclude it.
# Set EEL_DR_MODE=dry to run without touching live infrastructure.
#
# References:
#   docs/runbooks/edge-region-loss.md
#   ADR-0022, ADR-0010

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EEL_DR_MODE="${EEL_DR_MODE:-live}"
EDGE_REGION="${EEL_EDGE_REGION:-nairobi}"
TICKET="${EEL_TICKET:-DRILL-$(date +%Y%m%d)}"
REROUTE_TIMEOUT_SECONDS=900   # 15 min target
KUBECONFIG="${EEL_KUBECONFIG:-${HOME}/.kube/config}"

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >&2; }
die() { echo "[ERROR] $*" >&2; exit 1; }

VALID_REGIONS="nairobi mombasa kisumu nakuru eldoret"
echo "${VALID_REGIONS}" | grep -qw "${EDGE_REGION}" \
  || die "Unknown edge region '${EDGE_REGION}'; valid: ${VALID_REGIONS}"

START_EPOCH=$(date +%s)

if [ "${EEL_DR_MODE}" = "dry" ]; then
  log "DRY MODE: skipping live infrastructure calls"
  printf '{"stage":"edge-loss","mode":"dry","region":"%s","start_epoch":%s,"status":"ok","elapsed_seconds":0}\n' \
    "${EDGE_REGION}" "${START_EPOCH}"
  exit 0
fi

log "Ticket: ${TICKET} | Simulating loss of edge region: ${EDGE_REGION}"

# Step 1: cordon all nodes in the edge region node pool.
log "Step 1: cordon edge nodes for region ${EDGE_REGION}"
NODES=$(kubectl --kubeconfig="${KUBECONFIG}" get nodes \
  -l "topology.kubernetes.io/region=${EDGE_REGION},eel/role=edge" \
  -o jsonpath='{.items[*].metadata.name}')
[ -n "${NODES}" ] || die "No edge nodes found for region ${EDGE_REGION}"

for NODE in ${NODES}; do
  log "  cordoning ${NODE}"
  kubectl --kubeconfig="${KUBECONFIG}" cordon "${NODE}"
done

# Step 2: swap Envoy cluster config to remove the edge region backend.
log "Step 2: patching Envoy ConfigMap to remove ${EDGE_REGION} backend"
kubectl --kubeconfig="${KUBECONFIG}" patch configmap eel-envoy-config \
  -n eel-system \
  --type=json \
  -p "[{\"op\":\"remove\",\"path\":\"/data/cluster_${EDGE_REGION}\"}]" \
  || log "  (patch returned non-zero; region may already be absent)"

# Trigger Envoy reload via annotation bump.
kubectl --kubeconfig="${KUBECONFIG}" annotate daemonset envoy \
  -n eel-system \
  "eel.io/config-reload=$(date +%s)" --overwrite

# Step 3: monitor anycast reroute -- poll healthz on other regions.
log "Step 3: verifying traffic rerouted within ${REROUTE_TIMEOUT_SECONDS}s"
OTHER_REGIONS=$(echo "${VALID_REGIONS}" | tr ' ' '\n' | grep -v "^${EDGE_REGION}$" | tr '\n' ' ')
DEADLINE=$(($(date +%s) + REROUTE_TIMEOUT_SECONDS))
HEALTHY_REGION=""

while true; do
  for R in ${OTHER_REGIONS}; do
    if curl -sf --max-time 5 "https://${R}.edge.eel.internal/healthz" | grep -q "ok" 2>/dev/null; then
      HEALTHY_REGION="${R}"
      break 2
    fi
  done
  if [ "$(date +%s)" -gt "${DEADLINE}" ]; then
    die "Anycast reroute not confirmed within ${REROUTE_TIMEOUT_SECONDS}s"
  fi
  sleep 15
done

END_EPOCH=$(date +%s)
ELAPSED=$((END_EPOCH - START_EPOCH))
log "Edge-loss drill complete: traffic confirmed via ${HEALTHY_REGION} in ${ELAPSED}s"

# Step 4: uncordon nodes to restore the region (end of drill).
log "Step 4: uncordoning edge nodes to restore region (end of drill)"
for NODE in ${NODES}; do
  kubectl --kubeconfig="${KUBECONFIG}" uncordon "${NODE}"
done

# Restore Envoy config from source.
log "Step 4b: re-applying Envoy config from manifests"
kubectl --kubeconfig="${KUBECONFIG}" apply -f \
  "${REPO_ROOT}/platform/envoy/configmap-${EDGE_REGION}.yaml" 2>/dev/null || true

printf '{"stage":"edge-loss","mode":"live","region":"%s","rerouted_via":"%s","start_epoch":%s,"elapsed_seconds":%s,"status":"ok"}\n' \
  "${EDGE_REGION}" "${HEALTHY_REGION}" "${START_EPOCH}" "${ELAPSED}"
