# eel edge layer

Five regional Kubernetes edge clusters plus shared configuration for the eel election platform.
The edge layer absorbs public traffic before it reaches the 18-service K8s core, providing
DDoS mitigation, caching, WAF enforcement, and bot challenge across 22M+ voter interactions.

## Regional clusters

| Directory | City | Geographic coverage |
|-----------|------|---------------------|
| `nairobi/` | Nairobi | Central, Nairobi Metro, Rift Valley South |
| `mombasa/` | Mombasa | Coast, Taita-Taveta |
| `kisumu/` | Kisumu | Nyanza, Western |
| `eldoret/` | Eldoret | Rift Valley North, Turkana, West Pokot, Trans-Nzoia |
| `garissa/` | Garissa | North East, Upper Eastern |

Each regional directory contains a Kustomize overlay for the `eel-edge` namespace targeting that
cluster's K8s API server. ArgoCD ApplicationSets (`argocd-appset.yaml`) deploy from the
corresponding directory path in this repo.

## Shared config directories

| Directory | Purpose |
|-----------|---------|
| `nginx/` | Origin-shield nginx configuration, upstream routing, TLS termination config |
| `varnish/` | Long-TTL signed-snapshot caching for static election assets |
| `waf-rules/` | Coraza WAF rules tuned per service, not default CRS |
| `anubis/` | Proof-of-work bot mitigation with escalation ladder |
| `rate-limits/` | Envoy Gateway sliding-window rate limits per IP, token, API key, and endpoint |

## Anycast and BGP architecture

All five regions announce the same anycast prefix. BGP route advertisements propagate from each
cluster's upstream ISPs to the global table. Voters are routed to the nearest cluster by standard
BGP shortest-path selection. Each region holds a distinct anycast VIP (185.x.x.1 through
185.x.x.5) configured in `dns-vip-configmap.yaml` inside each regional overlay.

During planned maintenance or a cluster outage, the affected region withdraws its BGP
advertisement. Traffic shifts to the next-nearest peer within 60-180 seconds depending on BGP
propagation time and the failover peer's capacity headroom. Failover peer assignments are noted
in each regional README.

Nairobi peers at KIXP (Kenya Internet Exchange Point) for low-latency domestic routing.
Mombasa benefits from Seacom and TEAMS submarine cable landing infrastructure. The other
three regions rely on upstream terrestrial fiber from Safaricom, JTL, and Airtel respectively.
Garissa carries a satellite fallback path for continuity during fiber cuts.

## Deployment

Each regional cluster is deployed via ArgoCD using the directory path as the source:

```bash
# Sync a specific region manually
argocd app sync eel-edge-nairobi

# Sync all edge apps
argocd app list | grep eel-edge | awk '{print $1}' | xargs -I{} argocd app sync {}

# Verify all edge namespaces are healthy
kubectl get pods -n eel-edge --context=k8s-eel-nairobi
kubectl get pods -n eel-edge --context=k8s-eel-mombasa
```

## Config and environment

Each regional overlay sets the following in `dns-vip-configmap.yaml`:

| Key | Example (Nairobi) | Description |
|-----|-------------------|-------------|
| `region` | `nairobi` | Cluster region label |
| `anycast_vip` | `10.200.1.1` | Internal anycast VIP |
| `primary_dr_vip` | `10.100.0.10` | Upstream primary cluster VIP |
| `mirror_dr_vip` | `10.101.0.10` | Upstream mirror cluster VIP |
| `dns_zone` | `eel.local` | Internal DNS zone |

## Dependencies

- Envoy Gateway (per-cluster) for L7 routing and rate limiting
- Varnish OSS 7.x for caching
- Coraza WAF embedded in nginx
- Anubis (golang PoW service) in each cluster
- Valkey (Redis-compatible) for rate limit state
- ArgoCD for GitOps deployment
- KIXP / ISP BGP sessions for anycast propagation

## Test and verify

```bash
# Confirm anycast VIP is advertised by a region
curl -H "Host: portal.eel.local" https://185.x.x.1/healthz

# Check WAF is enforcing (expect 403 on SQLi probe)
curl -s -o /dev/null -w "%{http_code}" \
  "https://portal.eel.local/search?q=' OR 1=1--"

# Rate limit smoke test
for i in $(seq 1 200); do
  curl -s -o /dev/null -w "%{http_code}\n" https://portal.eel.local/api/v1/ping
done | sort | uniq -c
```

## Related docs

- `docs/adr/0019-edge-topology.md`: ADR describing the five-region architecture
- Each `edge/<region>/README.md`: cluster-specific capacity, ISP, and failover details
- `edge/nginx/README.md`, `edge/varnish/README.md`, `edge/waf-rules/README.md`, `edge/anubis/README.md`, `edge/rate-limits/README.md`: per-component guides
- `docs/runbooks/edge-failover.md`: BGP failover runbook

## Limitations

- BGP propagation delay means failover is not instantaneous; plan for up to 3 minutes of degraded routing during a regional outage.
- Garissa satellite fallback introduces latency spikes above 600ms; BVI biometric matching has local-only fallback mode for this path.
- Varnish cache keys are signed; stale content cannot be served after an election result update without an explicit PURGE.
- Anubis PoW challenges break non-browser API clients; kit firmware and official apps carry the PoW bypass token.
