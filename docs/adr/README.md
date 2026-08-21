# Architecture Decision Records

Numbered, immutable-after-acceptance. Amendments come as new ADRs that supersede the old, never in-place edits after acceptance. Each ADR: Context, Decision, Consequences, Alternatives, Related.

| # | Subject | Status |
|---|---|---|
| [0001](0001-monorepo-layout.md) | Monorepo with per-module vertical slices | accepted |
| [0002](0002-languages.md) | Language selection: Rust default, Python fallback, Elixir for LiveView, Dart for kits+mobile | accepted |
| [0003](0003-data-infrastructure.md) | Data infra: PostgreSQL 17 primary, Valkey, Kafka Kraft, MinIO, OpenSearch scoped, ClickHouse deferred | accepted |
| [0004](0004-platform.md) | Platform: Kubernetes 1.31 + Cilium + SPIFFE/SPIRE + ArgoCD + Harbor + Ubuntu 24.04 | accepted |
| [0005](0005-crypto-hsm.md) | Crypto & HSM: PKCS#11/KMIP, TLS 1.3 + PQ hybrid, M-of-N key ceremony | accepted |
| [0006](0006-build-supply-chain.md) | Build & supply chain: Nix flakes, Gitea + Woodpecker + Nexus, cosign, SLSA L3 | accepted |
| [0007](0007-authn-authz-secrets.md) | AuthN + AuthZ + Secrets: Keycloak + SpiceDB + OpenBao + SPIFFE/SPIRE + OPA/Kyverno + BFF | accepted |
| [0008](0008-portal-stack.md) | Portal stack: Astro + Django + LiveView + Tauri + Flutter (Path B) | accepted |
| [0009](0009-resilience-posture.md) | Resilience: degradation modes, circuit breakers, backpressure, idempotency, load shedding | accepted |
| [0010](0010-edge-ddos.md) | Edge & DDoS: multi-ISP peering, Coraza WAF, Anubis PoW, rate limits, DNSSEC + CAA | accepted |
| [0011](0011-public-api-surface.md) | Public API: 5-tier, signed everything, CBOR + JSON, RFC 9457 errors | accepted |
| [0012](0012-data-topology.md) | Data topology: 3 T1 clusters per site (voter/consensus/audit), single primary + partitioning | accepted |
| [0013](0013-event-sourcing-boundaries.md) | Event sourcing scope: only M08/M09/M10, M20, M63, M51; rest state + audit shadow | accepted |
| [0014](0014-schema-evolution.md) | Schema evolution: expand-migrate-contract, zero-downtime, 30-day pre-election freeze | accepted |
| [0015](0015-cross-service-consistency.md) | Cross-service consistency: outbox + Debezium + Kafka + CloudEvents envelope + signing | accepted |
| [0016](0016-pii-retention.md) | PII classification & retention: 5 tiers, soft-delete + scheduled purge, rotating backup keys | accepted |
| [0017](0017-transparency-log.md) | Transparency log: single monolithic Rust service, HSM-signed STHs at 1s/60s/5min | accepted |
| [0018](0018-write-path-performance.md) | Write path performance: 50k/sec sustained per cluster, P99 25ms end-to-end, 5x load buffer | accepted |
