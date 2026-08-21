# SPIRE SVID mass rotation without outage

## Purpose

Rotate SVIDs for all eel workloads in a coordinated manner, ensuring new SVIDs are issued and accepted before old ones expire. No authentication failures should occur during the rotation. This is used for scheduled key rotation, upstream CA changes, or post-incident re-keying.

## Prerequisites

- SPIRE server is healthy (`spire-server healthcheck`).
- All SPIRE agents are connected and healthy.
- Workloads are configured to use the SPIRE Workload API (not pinned certs).
- An ITSM change ticket open as `<TICKET>`.

## Commands

### Step 1: verify SPIRE server and agent health

```bash
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server healthcheck
kubectl get pods -n eel-system -l app=spire-agent
# all agent pods must be Running
```

### Step 2: check current SVID TTL and expiry distribution

```bash
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server entry show -all \
  | grep -E "SPIFFE ID|TTL|Entry ID"
# note the current TTL (default is typically 3600s)
```

### Step 3: reduce SVID TTL to force rapid rotation

```bash
# update the server config to reduce TTL (adjust path as needed):
kubectl edit configmap spire-server-config -n eel-system
# change: default_svid_ttl = "1h" -> default_svid_ttl = "5m"
# save and restart the server:
kubectl rollout restart deployment/spire-server -n eel-system
kubectl rollout status deployment/spire-server -n eel-system
```

### Step 4: monitor workloads for SVID pickup

```bash
# watch SPIRE agent renewal logs:
kubectl logs -n eel-system -l app=spire-agent -f --tail=20 \
  | grep -i "svid renewed\|svid expired\|rotation"
# SVIDs should renew automatically within the new TTL window
```

### Step 5: verify no auth failures during transition

```bash
# monitor service error rates during rotation:
for svc in bvr-svc bvi-svc rts-svc abis-svc transparency-svc; do
  echo -n "$svc auth errors: "
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:9090/metrics \
    | grep 'eel_auth_errors_total'
done
# count must not increase during the rotation window
```

### Step 6: restore original TTL after rotation completes

```bash
kubectl edit configmap spire-server-config -n eel-system
# restore: default_svid_ttl = "1h"
kubectl rollout restart deployment/spire-server -n eel-system
kubectl rollout status deployment/spire-server -n eel-system
```

### Step 7: confirm all SVIDs are renewed

```bash
# check that no SVID expiry warnings remain:
kubectl logs -n eel-system -l app=spire-agent --tail=50 \
  | grep -i "expir\|warn" | tail -10
# output should be empty or show only renewal success messages
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.spire-svid-mass-rotation.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. No auth errors in any service during or after rotation.
2. SPIRE agent logs show `svid renewed` for all workloads.
3. `spire-server entry show -all` shows expiry times updated to reflect the new TTL.
4. Transparency log contains the mass-rotation leaf.

## Rollback procedure

If workloads fail to pick up new SVIDs (e.g., Workload API socket errors), restart the SPIRE agent DaemonSet:

```bash
kubectl rollout restart daemonset/spire-agent -n eel-system
```

If auth failures are widespread, increase TTL immediately and page the Security team. SVIDs do not auto-expire during a server restart; workloads will continue using existing SVIDs until their natural expiry.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Site Reliability Lead (on-call via PagerDuty rotation)

## Estimated duration

20 to 40 minutes for full rotation at reduced TTL, plus 10 minutes to restore original TTL.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-cert-expiry-15min.yaml` and `chaos/scenarios/eel-svid-revocation.yaml`
