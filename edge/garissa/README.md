# edge/garissa

Regional edge cluster for eel serving North Eastern and Upper Eastern Kenya (Garissa,
Wajir, Mandera, Isiolo, Marsabit, Samburu, and parts of Tana River). The most
geographically dispersed region with significant satellite-dependent connectivity.

## Cluster identity

| Attribute | Value |
|-----------|-------|
| Cluster name | `eel-edge-garissa` |
| Namespace | `eel-edge` |
| K8s API | `https://k8s.eel-garissa.internal:6443` |
| Anycast VIP (public) | `185.x.x.5` |
| Internal anycast VIP | `10.200.1.5` |
| Primary upstream VIP | `10.100.0.10` |
| Mirror upstream VIP | `10.101.0.10` |
| Failover peer | `edge/nairobi` |

## Location and capacity

Garissa cluster is colocated at a secure facility in Garissa town. Primary connectivity is
terrestrial fiber via Safaricom's northern corridor and government fiber (Jambonet/KENET).
A significant portion of polling stations in Wajir, Mandera, and Marsabit depend on satellite
(VSAT) links. Kit firmware handles satellite fallback autonomously; the edge cluster itself
operates on terrestrial fiber with satellite as a backup WAN path.

Upstream ISPs: Safaricom (primary fiber), Jambonet government fiber (secondary), VSAT backup
(Ku-band for cluster WAN continuity during terrestrial cuts).

## Satellite fallback

Kits in satellite-dependent areas use local-only BVI mode (biometric matching on device)
when round-trip latency exceeds 800ms or packet loss exceeds 10%. Results (RTS) are queued
locally and flushed when connectivity recovers. This mode is transparent to the edge cluster;
kit firmware negotiates it independently.

The edge cluster itself is expected to remain on terrestrial fiber. VSAT on the cluster WAN
is a last resort and introduces 600ms+ latency on all proxied requests; Anubis PoW timeouts
are extended to 30 seconds in this scenario via the `garissa` region config.

## Components deployed

All shared edge components deployed via `kustomization.yaml`:

- `envoy-gateway`: L7 routing, anycast VIP `10.200.1.5`, extended timeouts for satellite path
- `varnish`: signed-snapshot caching, longer grace TTL for satellite congestion periods
- `nginx-shield`: TLS offload and origin-shield
- `anubis`: PoW bot mitigation, extended timeout for satellite paths
- `coraza-waf`: WAF enforcement

## Deployment

```bash
argocd app sync eel-edge-garissa

kubectl kustomize edge/garissa | kubectl apply --dry-run=client -f -
kubectl kustomize edge/garissa | kubectl apply -f -

kubectl get pods -n eel-edge --context=k8s-eel-garissa
```

## Config

| Key | Value |
|-----|-------|
| `region` | `garissa` |
| `anycast_vip` | `10.200.1.5` |
| `upstream_primary` | `10.100.0.10:443` |
| `upstream_mirror` | `10.101.0.10:443` |

Node selector: `topology.kubernetes.io/region=garissa`.

## Failover

Garissa failover peer is Nairobi. BGP withdrawal shifts traffic within 60-180 seconds.
Given the satellite dependency of many polling stations in this region, kit-level local
mode is the primary continuity mechanism; the edge cluster failover is secondary.

## Test and verify

```bash
curl -H "Host: portal.eel.local" https://185.x.x.5/healthz
# Expected: HTTP 200, body {"status":"ok","region":"garissa"}
```

## Limitations

- Satellite-connected kits may see latency above 600ms even on a healthy edge cluster.
- Anubis PoW challenge-response is not usable over VSAT from a browser without the
  extended timeout config. Kit firmware carries a bypass token.
- Results queued in local-only mode may arrive in a burst after connectivity restores;
  `rts-svc` handles idempotent batch ingest.

## Related docs

- `edge/README.md`: edge layer overview and anycast architecture
- `docs/runbooks/edge-failover.md`: BGP failover runbook
- `docs/adr/0019-edge-topology.md`: topology ADR
