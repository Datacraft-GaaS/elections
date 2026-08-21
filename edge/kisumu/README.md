# edge/kisumu

Regional edge cluster for eel serving Nyanza and Western Kenya (Kisumu, Siaya, Homa Bay,
Migori, Kisii, Nyamira, Kakamega, Vihiga, Bungoma, Busia). Third-largest voter
concentration in the country.

## Cluster identity

| Attribute | Value |
|-----------|-------|
| Cluster name | `eel-edge-kisumu` |
| Namespace | `eel-edge` |
| K8s API | `https://k8s.eel-kisumu.internal:6443` |
| Anycast VIP (public) | `185.x.x.3` |
| Internal anycast VIP | `10.200.1.3` |
| Primary upstream VIP | `10.100.0.10` |
| Mirror upstream VIP | `10.101.0.10` |
| Failover peer | `edge/nairobi` |

## Location and capacity

Kisumu cluster is colocated at a facility on the Kisumu-Kakamega fiber corridor.
Connectivity runs via JTL (Jamii Telecommunications) metropolitan fiber as primary and
Safaricom national backhaul as secondary. The cluster serves approximately 5M registered
voters across Nyanza and Western. During election day, expect biometric identification
peaks from polling stations spread across dense lakeside and rural areas.

Upstream ISPs: JTL (primary metropolitan fiber), Safaricom (secondary), Airtel Kenya (tertiary).

## Components deployed

All shared edge components deployed via `kustomization.yaml`:

- `envoy-gateway`: L7 routing, anycast VIP `10.200.1.3`, rate limit enforcement
- `varnish`: signed-snapshot caching for election assets
- `nginx-shield`: TLS offload and origin-shield
- `anubis`: PoW bot mitigation
- `coraza-waf`: WAF rule enforcement

## Deployment

```bash
argocd app sync eel-edge-kisumu

kubectl kustomize edge/kisumu | kubectl apply --dry-run=client -f -
kubectl kustomize edge/kisumu | kubectl apply -f -

kubectl get pods -n eel-edge --context=k8s-eel-kisumu
```

## Config

| Key | Value |
|-----|-------|
| `region` | `kisumu` |
| `anycast_vip` | `10.200.1.3` |
| `upstream_primary` | `10.100.0.10:443` |
| `upstream_mirror` | `10.101.0.10:443` |

Node selector: `topology.kubernetes.io/region=kisumu`.

## Failover

Kisumu failover peer is Nairobi. If Kisumu withdraws BGP, Nairobi absorbs Western-Kenya
traffic. BGP propagation: 60-180 seconds.

## Test and verify

```bash
curl -H "Host: portal.eel.local" https://185.x.x.3/healthz
# Expected: HTTP 200, body {"status":"ok","region":"kisumu"}
```

## Related docs

- `edge/README.md`: edge layer overview and anycast architecture
- `docs/runbooks/edge-failover.md`: BGP failover runbook
- `docs/adr/0019-edge-topology.md`: topology ADR
