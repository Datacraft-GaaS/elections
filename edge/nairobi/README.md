# edge/nairobi

Primary regional edge cluster for eel, serving Nairobi Metro, Central Province, and
Rift Valley South. Highest capacity of the five regions; designated primary for all
cross-region traffic experiments and election-night ops.

## Cluster identity

| Attribute | Value |
|-----------|-------|
| Cluster name | `eel-edge-nairobi` |
| Namespace | `eel-edge` |
| K8s API | `https://k8s.eel-nairobi.internal:6443` |
| Anycast VIP (public) | `185.x.x.1` |
| Internal anycast VIP | `10.200.1.1` |
| Primary upstream VIP | `10.100.0.10` |
| Mirror upstream VIP | `10.101.0.10` |
| Failover peer | `edge/mombasa` (secondary) |

## Location and capacity

Nairobi is Kenya's primary internet hub. This cluster colocates at two Nairobi data
centers (primary and warm-standby) interconnected at KIXP (Kenya Internet Exchange Point).
KIXP peering reduces round-trip latency to domestic ISPs to under 5ms for most Safaricom,
Airtel, and Telkom Kenya subscribers. The cluster is sized to absorb 60% of peak national
traffic on its own.

Upstream ISPs: Safaricom, Liquid Intelligent Technologies (primary fiber), Jambonet (government
network backup), KIXP peering for domestic route optimization.

## Components deployed

All shared edge components are deployed here via `kustomization.yaml`:

- `envoy-gateway`: L7 routing, anycast VIP binding, rate limit enforcement
- `varnish`: signed-snapshot caching for static election assets
- `nginx-shield`: origin-shield layer, TLS offload
- `anubis`: PoW bot mitigation
- `coraza-waf`: WAF rule enforcement

## Deployment

```bash
# Sync this cluster's ArgoCD application
argocd app sync eel-edge-nairobi

# Manual kustomize apply (dry-run first)
kubectl kustomize edge/nairobi | kubectl apply --dry-run=client -f -
kubectl kustomize edge/nairobi | kubectl apply -f -

# Verify pods
kubectl get pods -n eel-edge --context=k8s-eel-nairobi
```

## Config

Key settings in `dns-vip-configmap.yaml` and `envoy-gateway-patch.yaml`:

| Key | Value |
|-----|-------|
| `region` | `nairobi` |
| `anycast_vip` | `10.200.1.1` |
| `valkey_address` | `valkey.eel-edge.svc.cluster.local:6379` |
| `upstream_primary` | `10.100.0.10:443` |
| `upstream_mirror` | `10.101.0.10:443` |

Node selector patch (`node-selector-patch.yaml`) pins all edge pods to nodes labelled
`topology.kubernetes.io/region=nairobi`.

## Failover

If Nairobi withdraws its BGP advertisement, the anycast VIP `185.x.x.1` shifts to the
next-nearest peer. Mombasa is the configured failover peer and has 30% capacity headroom
reserved for this scenario. BGP propagation takes 60-180 seconds.

To force failover during planned maintenance:

```bash
# Withdraw BGP advertisement (run on the BGP-speaking router/BFD peer)
# Exact command depends on router OS; contact network team for session credentials.
# Then verify no traffic hits Nairobi:
kubectl logs -n eel-edge -l app=envoy-gateway --tail=50 --context=k8s-eel-nairobi
```

## Test and verify

```bash
# Health check via anycast VIP
curl -H "Host: portal.eel.local" https://185.x.x.1/healthz

# Confirm region label on response header
curl -I https://185.x.x.1/ | grep x-eel-region
```

## Related docs

- `edge/README.md`: edge layer overview and anycast architecture
- `edge/nginx/README.md`, `edge/varnish/README.md`: shared component guides
- `docs/runbooks/edge-failover.md`: BGP failover runbook
- `docs/adr/0019-edge-topology.md`: topology ADR
