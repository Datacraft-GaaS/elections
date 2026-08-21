# edge/mombasa

Regional edge cluster for eel serving the Coast region (Mombasa, Kilifi, Kwale, Lamu,
Tana River, Taita-Taveta). Designated failover peer for the Nairobi cluster and benefits
from submarine cable proximity for East Africa connectivity.

## Cluster identity

| Attribute | Value |
|-----------|-------|
| Cluster name | `eel-edge-mombasa` |
| Namespace | `eel-edge` |
| K8s API | `https://k8s.eel-mombasa.internal:6443` |
| Anycast VIP (public) | `185.x.x.2` |
| Internal anycast VIP | `10.200.1.2` |
| Primary upstream VIP | `10.100.0.10` |
| Mirror upstream VIP | `10.101.0.10` |
| Failover peer | `edge/nairobi` |

## Location and capacity

Mombasa colo sits within 10km of the Seacom and TEAMS submarine cable landing stations
at Mtongwe. This gives it the lowest latency to East African internet exchange points and
makes it the preferred path for traffic originating from the Indian Ocean submarine cable
ecosystem. The cluster handles Coast-region domestic traffic and absorbs Nairobi overflow
during Nairobi failover events (30% headroom reserved).

Upstream ISPs: Seacom (primary), TEAMS (secondary fiber), Safaricom national backhaul.

## Components deployed

All shared edge components deployed via `kustomization.yaml`:

- `envoy-gateway`: L7 routing, anycast VIP `10.200.1.2`, rate limit enforcement
- `varnish`: signed-snapshot caching
- `nginx-shield`: TLS offload and origin-shield
- `anubis`: PoW bot mitigation
- `coraza-waf`: WAF enforcement

## Deployment

```bash
argocd app sync eel-edge-mombasa

# Manual kustomize apply
kubectl kustomize edge/mombasa | kubectl apply --dry-run=client -f -
kubectl kustomize edge/mombasa | kubectl apply -f -

kubectl get pods -n eel-edge --context=k8s-eel-mombasa
```

## Config

| Key | Value |
|-----|-------|
| `region` | `mombasa` |
| `anycast_vip` | `10.200.1.2` |
| `upstream_primary` | `10.100.0.10:443` |
| `upstream_mirror` | `10.101.0.10:443` |

Node selector: `topology.kubernetes.io/region=mombasa`.

## Failover

Mombasa is the secondary failover peer for Nairobi. During a Nairobi BGP withdrawal,
Mombasa absorbs Nairobi's advertised traffic. The reverse is also true: if Mombasa
withdraws, Nairobi absorbs Coast traffic.

```bash
# Verify Mombasa is healthy
curl -H "Host: portal.eel.local" https://185.x.x.2/healthz

# Check Mombasa region header
curl -I https://185.x.x.2/ | grep x-eel-region
```

## Test and verify

```bash
curl -H "Host: portal.eel.local" https://185.x.x.2/healthz
# Expected: HTTP 200, body {"status":"ok","region":"mombasa"}
```

## Related docs

- `edge/README.md`: full edge overview and BGP architecture
- `docs/runbooks/edge-failover.md`: failover runbook
- `docs/adr/0019-edge-topology.md`: topology ADR
