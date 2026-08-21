# ADR 0010: Edge & DDoS

Status: accepted
Date: 2026-08-21

## Context

5-10M concurrent public viewers on election night. State-actor and partisan adversaries will attempt L3/L4 volumetric floods, L7 application-layer floods, credential stuffing on admin/observer/agent portals, slow-loris, DNS amplification, and Certificate Transparency-visible impersonation. 95% of load must be absorbed before origin sees a packet; the remaining 5% must survive.

## Decision

- **Multi-ISP peering inside Kenya**: agreements with Safaricom, Airtel, Jamii Telecommunications, iColo, Konza / Safaricom DC. Multi-provider anycast + BGP. Upstream scrubbing agreements formalised before the freeze.
- **Origin shield per site**: **nginx + Varnish** (or Apache Traffic Server) in front of every service. Long-TTL for signed snapshots (ADR-0011 Tier 1); short-TTL for live surfaces.
- **Static-first**: everything that can be pre-rendered is — candidate lists, boundary GeoJSON, voter-ed content, gazette, results snapshots. Rendered on cadence, cosign-signed, published to MinIO, cached at every edge.
- **Protocols**: HTTP/3 (QUIC) at the edge for mobile users on lossy Kenyan networks; HTTP/2 fallback; HTTP/1.1 blocked except for known agents.
- **WAF**: **Coraza** (OSS OWASP ModSec-compatible, Apache 2.0) in front of every ingress. Rules tuned per service, not default CRS.
- **Bot mitigation**: **Anubis** (MIT proof-of-work challenge under load) as escalation. Escalation ladder — normal → JS challenge → PoW → mTLS-only for authenticated surfaces. Alternatives evaluated: `go-away`, Curiefense.
- **Rate limiting**: per-IP + per-token + per-API-key + per-endpoint, sliding window, at the Envoy Gateway. Adaptive tightening under load.
- **Slow-loris / low-and-slow**: strict connection timeouts, header timeouts, minimum request rates enforced at nginx.
- **DNS**: **DNSSEC** signed, **CAA** records set, multi-provider authoritative DNS (at least two Kenyan providers + a fallback), continuously monitored for hijack.
- **TLS**: TLS 1.3 only, **HSTS preload**, **DANE**, **ECH** where clients support it. Ciphers: PQ hybrid (ML-KEM + X25519), classical fallback X25519-only.
- **Certificate Transparency monitoring**: alert on any cert issued for eel.local or lookalike domains.
- **Brand / typosquat monitoring**: continuous on public search + social; misinformation vector.
- **Anti-clone response playbook**: quick DNS/TLS provisioning for legitimate additions; takedown workflow with Kenyan registrars for impersonation.

## Consequences

Positive: 95%+ of load absorbed at edge; per-request cost at origin drops by 10-100x; DDoS attempts terminate outside the mesh.

Negative: multi-ISP peering is material budget + engineering time; Coraza tuning is ongoing operational work; PoW challenges add mild UX friction under attack (acceptable trade-off).

## Alternatives rejected

- Cloudflare / Fastly for critical path: sovereignty risk + rug-pull risk on a national election. Kenyan-hosted edge is a non-negotiable constraint.
- CRS default WAF ruleset: high false-positive rate on legitimate election-day traffic.
- Commercial bot-mitigation (DataDome, etc.): non-OSS + vendor lock-in.

## Related

ADR-0011 (signed-snapshot pattern), ADR-0009 (edge shares load-shedding), ADR-0004 (Envoy Gateway).
