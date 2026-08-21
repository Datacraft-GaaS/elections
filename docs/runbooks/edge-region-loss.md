# Edge region loss: anycast reroute

## Purpose

Respond to the loss of an edge region by withdrawing its BGP prefix, rerouting anycast traffic to healthy regions, and verifying Varnish cache and Envoy upstream health. Prevents split-brain and ensures client traffic reaches a healthy endpoint.

## Prerequisites

- BGP management access to the failed region's router (or the upstream transit provider's portal).
- SSH access to remaining edge nodes.
- `varnishadm` and `envoyadmin` available on edge nodes.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: confirm region is unreachable

```bash
# from the ops jump host:
curl -sf --max-time 5 https://edge-<REGION>.eel.io/health || echo "REGION DOWN"
# from multiple external vantage points if possible:
for vantage in 1.1.1.1 8.8.8.8; do
  dig +short @$vantage api.eel.io
done
```

### Step 2: withdraw BGP prefix for failed region

```bash
# if using ExaBGP on the edge node (adjust prefix as needed):
ssh edge-<REGION>-gw \
  'echo "withdraw route 203.0.113.0/24 next-hop self" | exabgpcli'
# if using a managed transit provider, use their API or portal to withdraw the prefix
# confirm withdrawal:
ssh core-router 'show ip bgp 203.0.113.0/24'
# entry must be absent
```

### Step 3: verify anycast failover

```bash
# DNS should resolve to a remaining region's IP within TTL:
dig +short api.eel.io
# confirm the IP belongs to a healthy region
curl -sf https://api.eel.io/health | grep '"status":"ok"'
```

### Step 4: verify Varnish cache health on remaining regions

```bash
for edge_node in edge-region-a edge-region-b; do
  ssh $edge_node 'varnishadm backend.list' | grep -v "Sick"
done
# any "Sick" backends indicate upstream issues on that edge node
```

### Step 5: check Envoy upstream health

```bash
for edge_node in edge-region-a edge-region-b; do
  ssh $edge_node \
    'curl -sf http://localhost:9901/clusters | grep -E "health_flags|cx_active"'
done
# health_flags must be /healthy/ for all critical clusters
```

### Step 6: confirm no split-brain

```bash
# all edge nodes must agree on the same active region:
for edge_node in edge-region-a edge-region-b; do
  ssh $edge_node 'cat /etc/eel/active-region'
done
# all outputs must be identical
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.edge-region-loss.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"lost_region\":\"<REGION>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `dig +short api.eel.io` returns only IPs from healthy regions.
2. `curl https://api.eel.io/health` returns HTTP 200 from a healthy region.
3. No "Sick" Varnish backends on remaining regions.
4. Envoy cluster health flags show `/healthy/` for all critical upstreams.
5. Transparency log contains the edge-region-loss leaf.

## Rollback procedure

Once the failed region recovers, restore its BGP prefix:

```bash
ssh edge-<REGION>-gw \
  'echo "announce route 203.0.113.0/24 next-hop self" | exabgpcli'
```

Monitor for 5 minutes before considering the region fully restored. Run steps 4 and 5 again to verify health.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Network Lead for BGP issues (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if multiple regions are affected

## Estimated duration

5 to 15 minutes for anycast reroute. Region recovery time varies.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-network-loss-30pct-2s-latency.yaml` and `chaos/scenarios/eel-dns-chaos.yaml`
