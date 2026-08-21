# edge/eldoret

Regional edge cluster for eel serving Rift Valley North, Turkana, West Pokot, Trans-Nzoia,
Uasin Gishu, Elgeyo-Marakwet, Nandi, Baringo, and Laikipia counties.

## Cluster identity

| Attribute | Value |
|-----------|-------|
| Cluster name | `eel-edge-eldoret` |
| Namespace | `eel-edge` |
| K8s API | `https://k8s.eel-eldoret.internal:6443` |
| Anycast VIP (public) | `185.x.x.4` |
| Internal anycast VIP | `10.200.1.4` |
| Primary upstream VIP | `10.100.0.10` |
| Mirror upstream VIP | `10.101.0.10` |
| Failover peer | `edge/nairobi` |

## Location and capacity

Eldoret cluster is colocated at a facility in Uasin Gishu County on the northern
Nairobi-Eldoret fiber corridor. Connectivity is provided by Safaricom national fiber
(primary) and Airtel Kenya (secondary). The cluster serves a geographically dispersed
region including semi-arid areas where connectivity can degrade; the Turkana sub-region
has partial satellite fallback coverage (handled at kit firmware level, not at this edge).

Upstream ISPs: Safaricom (primary fiber), Airtel Kenya (secondary), Jambonet (government
network backup for county offices).

## Components deployed

All shared edge components deployed via `kustomization.yaml`:

- `envoy-gateway`: L7 routing, anycast VIP `10.200.1.4`, rate limit enforcement
- `varnish`: signed-snapshot caching
- `nginx-shield`: TLS offload and origin-shield
- `anubis`: PoW bot mitigation
- `coraza-waf`: WAF enforcement

## Deployment

```bash
argocd app sync eel-edge-eldoret

kubectl kustomize edge/eldoret | kubectl apply --dry-run=client -f -
kubectl kustomize edge/eldoret | kubectl apply -f -

kubectl get pods -n eel-edge --context=k8s-eel-eldoret
```

## Config

| Key | Value |
|-----|-------|
| `region` | `eldoret` |
| `anycast_vip` | `10.200.1.4` |
| `upstream_primary` | `10.100.0.10:443` |
| `upstream_mirror` | `10.101.0.10:443` |

Node selector: `topology.kubernetes.io/region=eldoret`.

## Failover

Eldoret failover peer is Nairobi. BGP withdrawal shifts traffic to Nairobi within 60-180
seconds. Remote pastoral areas in Turkana that rely on satellite-connected kits operate in
local-only mode during any edge outage.

## Test and verify

```bash
curl -H "Host: portal.eel.local" https://185.x.x.4/healthz
# Expected: HTTP 200, body {"status":"ok","region":"eldoret"}
```

## Related docs

- `edge/README.md`: edge layer overview and anycast architecture
- `docs/runbooks/edge-failover.md`: BGP failover runbook
- `docs/adr/0019-edge-topology.md`: topology ADR
