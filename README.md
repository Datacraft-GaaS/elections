# eel — Kenya IEBC IEMS (2026-2027)

Integrated Elections Management System for the Independent Electoral and Boundaries Commission of Kenya. Datacraft software prime under IEBC/OIT/01/2026-2027.

Three-year build + steady-state ops, 15-year audit retention, sovereign hosting, HSM-anchored roots of trust, reproducible builds, election-critical integrity.

## Repository shape

```
services/           18 K8s services grouping the 69 modules (ADR-0020)
modules/            69 module design/spec homes (one directory per module, clean slug)
packages/           shared cross-language libraries (Rust, Python, Elixir, Dart, TS)
platform/           K8s + mesh + identity + secrets + registry deployment config
edge/               5 regional edge cluster configs (Nairobi/Mombasa/Kisumu/Eldoret/Garissa) + WAF + rate limits
integrations/       external adapters (IPRS, KRA, MRZ, telco, CBK, FRC, KEBS, ODPC, Power BI)
operations/         ops runbook code (MDM, helpdesk, e-learning, warranty, dispatch, data migration)
compliance/         DPA toolkit, ISO 27001 evidence, Elections Tech Regs test harness, IPR handover
chaos/              Chaos Mesh scenarios rehearsed weekly/monthly
loadtest/           k6 + Gatling + custom kit simulator; 5x-peak targets
detection/          Falco + Tetragon + Sigma rules + Wazuh + canary tokens
reproducibility/    M14 make-reproduce toolkit, Nix flakes, hash manifests
air-gapped-build/   Nexus mirror config + build environment definition
environments/       dev / staging / pre-prod / prod overlays
tests/              cross-service e2e + integration
scripts/  tools/  assets/  config/
docs/               architecture, ADRs, change requests, runbooks, module index
infra/              legacy holding — being deprecated; use platform/ and edge/
```

## The 18 services

Grouping of the 69 modules per ADR-0020:

`biometric-edge-svc` · `bvr-svc` · `abis-svc` · `vprs-svc` · `bvi-svc` · `rts-svc` · `portal-svc` · `crms-svc` · `transparency-svc` · `ecc-svc` · `boundary-svc` · `queue-svc` · `inventory-svc` · `governance-svc` · `ai-platform-svc` · `field-mobile-svc` · `training-svc` · `support-admin-svc`

## The 69 modules

See `docs/modules/README.md` for the full index. Slugs match `sw/modules/M<nn>-<slug>.md` in the upstream Datacraft spec corpus (M-numbers dropped for clean slugs).

## Stack (locked)

Decided in ADRs 0001-0023. See `docs/adr/README.md` for the index and `docs/architecture/decision-log.md` for the reasoning that got us here.

- **Backend default**: Rust (axum + sqlx + tokio + utoipa + tonic).
- **Backend fallback**: Python 3.12 async (FastAPI + Pydantic v2) for ML, vendor SDKs, ops glue.
- **Real-time public surfaces**: Phoenix LiveView (Elixir + Ecto + Broadway + Oban + Rustler).
- **Admin portals**: Django 5 + HTMX + Alpine.
- **Static portals**: Astro 5 + Svelte islands.
- **Operator desktop**: Tauri 2 (Rust + web UI).
- **Kits + mobile**: Flutter/Dart (shared `libiebc-verifier` Rust FFI).
- **OLTP**: PostgreSQL 17, three T1 clusters per site (voter / consensus / audit), two IEBC DR sites.
- **Analytics**: PG read replicas + BRIN + materialised views day 1; ClickHouse deferred.
- **Search**: OpenSearch scoped to four modules; Postgres full-text elsewhere.
- **Cache**: Valkey 8.
- **Event bus**: Apache Kafka Kraft + Karapace schema registry + Debezium outbox CDC.
- **Object store**: MinIO.
- **Container platform**: Kubernetes 1.31 on Ubuntu 24.04, Cilium mesh + CNI, SPIFFE/SPIRE.
- **Identity**: Keycloak (human) + SPIFFE/SPIRE (workload) + SpiceDB (authz) + OpenBao (secrets) + OPA/Kyverno (admission).
- **Crypto**: HSM PKCS#11/KMIP, TLS 1.3 + PQ hybrid (ML-KEM + X25519 KEM; ML-DSA + ECDSA P-384 hybrid sigs; SLH-DSA for archive), FIPS 140-3 L3+.
- **Build**: Nix flakes; cargo + uv + pnpm + mix + flutter; `just` at root.
- **Forge + CI**: Gitea/Forgejo + Woodpecker + Nexus OSS (air-gapped).
- **Observability**: OpenTelemetry → Prometheus + Loki + Tempo + Grafana + Mimir + Vector.
- **Detection**: Falco + Tetragon + Wazuh + Sigma rules.
- **Edge**: nginx + Varnish + Coraza WAF + Anubis PoW; Kenyan multi-ISP peering.

## Where things live for a new contributor

- Adding a new module? Design its `docs/SPEC.md` under `modules/<slug>/docs/`; its implementation lives inside the appropriate `services/<svc>/src/`.
- Adding a new event? Follow `docs/EVENT-CONVENTIONS.md`. Schema in `packages/event-schemas/`. Topic in `platform/kafka/topics/`. Producer via `iebc-outbox-{rs,py,ex}` shared lib. Test proves idempotency.
- Adding a new API endpoint? OpenAPI 3.1 in `packages/openapi-contracts/`. Client generation lands in `packages/api-clients/<lang>/`. Follow ADR-0011 tier + signing rules.
- Adding a new external integration? Adapter under `integrations/<name>/`. Python by default (vendor SDK ecosystem). Wrap in a thin Rust facade for the consuming service.
- New ADR? Numbered `docs/adr/<n>-<slug>.md`, update `docs/adr/README.md` index.

## Status

Design phase complete. 23 ADRs committed. Change-request pack (`docs/change-requests/`) awaiting Datacraft response. Client blockers (`docs/blockers-to-client-2026-08-21.md`) awaiting IEBC + Datacraft response.

Next: first real service — `services/transparency-svc/` in Rust per ADR-0017.
