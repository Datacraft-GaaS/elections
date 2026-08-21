# ADR 0019: Regional edge topology

Status: accepted
Date: 2026-08-21

## Context

`sw/ARCHITECTURE.md` §3.3 (v3 stitching pass) mandates five regional edge Kubernetes clusters to reduce kit round-trip latency to 30-60ms across Kenya's geography. The RFP itself mandates only two IEBC-provided DR sites (primary + mirror), so edge topology is Datacraft-optimised, not RFP-binding, but it directly supports the RFP's 60,000-concurrent-RTS SLA and legacy-Android device latency budget (see ADR-0018).

## Decision

**Five regional edge Kubernetes clusters**, one per major geographic region:

| region | city | serves |
|---|---|---|
| Central | **Nairobi** | Nairobi County + Central + Rift Valley South |
| Coast | **Mombasa** | Mombasa + Kilifi + Kwale + Taita-Taveta + Tana River + Lamu |
| Western | **Kisumu** | Nyanza + Western |
| North Rift | **Eldoret** | Rift Valley North + Turkana + West Pokot + Trans-Nzoia |
| North East | **Garissa** | Garissa + Wajir + Mandera + Marsabit + Isiolo + Samburu |

**Edge cluster responsibilities:**

- Kit-to-backend edge termination (mTLS, SPIFFE SVID verification, request routing)
- Static asset serving for public portals (Astro tarballs, signed snapshots)
- Coraza WAF + Anubis PoW + rate limiting (ADR-0010)
- Local caching (Varnish + nginx)
- LiveView presence sharding for M11/M19 real-time surfaces
- **No stateful T1 workloads**, writes route to the nearest of the two IEBC DR sites (primary or mirror per ADR-0012)
- **No secrets storage**, edge pulls short-lived credentials from OpenBao on the DR side via mesh mTLS

**Edge cluster shape:**

- 6-node K8s cluster per region (3 control-plane + 3 worker nodes; horizontally scalable to 12+ workers on demand)
- Same platform stack as DR sites (ADR-0004): Cilium mesh, cert-manager, Envoy Gateway, ArgoCD
- Backhaul to DR sites: multi-ISP (Safaricom + Airtel + Jamii at minimum per region), IPsec + SPIFFE-authenticated
- Local Valkey for edge cache; no local Postgres
- Local Prometheus + Loki with remote write to central Grafana (via Mimir)

**Kit routing:**

- Kits use anycast DNS to reach the geographically nearest edge cluster.
- Fallback to next-nearest on edge failure (health-check driven DNS).
- Kits sign local batches, forward to edge; edge validates signature, forwards to DR (Kafka producer via mesh).
- Round-trip target: **30-60ms kit → edge → ACK** at P99.

## Consequences

Positive: sub-60ms kit ACK across Kenya; DR sites absorb only cross-region traffic + writes; failure of one edge region isolates blast radius to that region's kits.

Negative: five additional K8s clusters is real ops burden (~30 nodes total across regions); mitigated by identical stack + ArgoCD GitOps rollout templated across regions.

## Alternatives rejected

- Two DR sites only (no edge): kit RTT from Garissa or Mandera to Nairobi is 200ms+, unacceptable under 60k concurrent RTS SLA.
- Fewer, larger edges (e.g. three): coastal + far-north latency degrades.
- Public cloud CDN edges (Cloudflare, Fastly): sovereignty rejected (ADR-0010).

## Related

ADR-0004 (platform stack replicated at edge), ADR-0010 (edge WAF + DDoS), ADR-0012 (edge does not host T1), ADR-0018 (60k concurrent RTS SLA).
