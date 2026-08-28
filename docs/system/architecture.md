# System Architecture

Deep-dive architecture reference for the eel election platform. This volume summarises the shape of the system and links every claim to the code that implements it. For decision rationale read `docs/adr/`; for per-service implementation detail read the sibling documents in `docs/system/services/`.

---

## 1. What the system does

eel is an election management platform covering the full electoral cycle: voter registration (M02/M03), biometric enrolment and deduplication (M04/M05), polling-day kit operations (M06/M07), results capture and tallying (M08-M10), grievance and command-centre operations (M15/M16/M23), transparency and public verification (M11-M14, M22, M63), and post-election governance, archives, and training. The authoritative module index is `docs/modules/README.md`; the service-to-module mapping is ADR-0020 (`docs/adr/0020-service-topology.md`).

## 2. Runtime topology

Eighteen services in `services/`, deployed to two IEBC DR sites and five regional edge clusters (ADR-0019, `docs/adr/0019-edge-topology.md`). Placement per service is recorded in ADR-0024 (`docs/adr/0024-app-placement.md`).

```
kits (Flutter)   mobile (Flutter)   portals (Astro / LiveView / Django)   desktop (Tauri)
      \                 |                     |                                /
       +----------------+----- edge: Envoy Gateway, Coraza WAF, Varnish, Anubis ---+
                                |
              services/<svc>  (Rust default; Python for ML/vendor SDKs; Elixir for LiveView)
                                |
        packages/eel-*  (auth, secrets, events, outbox, eventstore, pii, hlc,
                         resilience, observability — see docs/system/packages/shared-packages.md)
                                |
   PostgreSQL 17   Kafka + Karapace + Debezium   Valkey   MinIO   OpenSearch
                                |
   Keycloak + SPIFFE/SPIRE + SpiceDB + OpenBao + OPA/Kyverno
   HSM (PKCS#11) + transparency log (services/transparency-svc)
```

Language policy is ADR-0002 (`docs/adr/0002-languages.md`):

| Tier | Language | Rationale | Examples |
|---|---|---|---|
| Default backend | Rust | performance, memory safety | tlog-api, bvi-api, ecc-api |
| ML / vendor SDKs | Python | ecosystem | rts-icr, abis vendor adapters |
| Operator consoles | Elixir/Phoenix LiveView | realtime UI | ecc_live, bvv_web |

Per-service entrypoints, ports, and layouts are documented in each file under `docs/system/services/`.

## 3. Architectural rules

These rules are normative; each traces to an ADR and is visible in code:

1. **Business logic lives in `services/<svc>/`.** Portals render, kits consume the public API (ADR-0011). No parallel implementations of a rule. Example: Section 44A state transitions live only in `services/ecc-svc/crates/ecc-api/src/store.rs` (`insert_s44a_officer1`, `complete_s44a_officer2`) and are surfaced through both the REST API (`services/ecc-svc/crates/ecc-api/src/routes` via `router.rs`) and the operator console (`services/ecc-svc/ex/ecc_live/lib/ecc_live/live/s44a_live.ex`).
2. **Services own their data.** No cross-schema joins, no shared writes (ADR-0012, `docs/adr/0012-data-topology.md`). Each service keeps migrations under `services/<svc>/migrations/`.
3. **Cross-service state moves through the transactional outbox onto Kafka** (ADR-0015). Implementations: `packages/eel-outbox-rs`, Django `OutboxRecord` model, Elixir `packages/eel-outbox-ex`. Event authoring rules: `docs/EVENT-CONVENTIONS.md`; schemas: `packages/event-schemas/`.
4. **Only four bounded contexts are event-sourced**: rts results (M08/09/10, M51, M63) and custody (M20) (ADR-0013). Everything else is state plus an immutable audit shadow. Eventstore implementations with a documented snapshot-compaction limitation: `packages/eel-eventstore-{rs,py,ex}`.
5. **Every consensus-relevant artefact is signed and provable against the transparency log** (ADR-0017): leaves are BLAKE3 hashes of canonical events (`services/transparency-svc/crates/tlog-domain/src/leaf.rs` `LeafFormat::hash`), sequenced into an append-only Merkle log (`crates/tlog-sequencer`), covered by STHs at three cadences (`crates/tlog-sequencer/src/sth_producer.rs`), verifiable by anyone via REST/gRPC proof endpoints (`crates/tlog-api/src/rest/proofs.rs`, `src/grpc/mod.rs`).
6. **Every write endpoint is idempotent** (ADR-0011): Idempotency-Key handling e.g. `services/bvi-svc/crates/bvi-api/src/idem.rs` (`require_key`, `replay`, `snapshot`).
7. **Every service ships degradation modes** (ADR-0009, `docs/adr/0009-resilience-posture.md`); shared primitives in `packages/eel-resilience-rs` and language triplets.
8. **No identifier carries the client name.** Packages, events, topics, headers, trust domains use the `eel` prefix.

## 4. Data infrastructure

Ground truth: ADR-0003 (`docs/adr/0003-data-infrastructure.md`).

| System | Role | Where configured |
|---|---|---|
| PostgreSQL 17 (two clusters: T1 voter/consensus/audit, T2 shared) | system of record | `services/*/migrations/*.sql`, connection strings via `DATABASE_URL` |
| Kafka + Karapace schema registry | all inter-service state movement | topic naming `eel.<service>`; registry schemas in `packages/event-schemas/` |
| Debezium | CDC into the pipeline | platform manifests under `platform/` |
| Valkey | cache / rate-limit counters | per-service config |
| MinIO | object store (evidence packages, register bundles, BagIt archives) | portal/archive services |
| OpenSearch | search over grievances, reports | search-backed services |

Data classification tiers and PII field classes are specified in `docs/architecture/cross-cutting/data-tiers.md` and ADR-0016; enforcement lives in the `packages/eel-pii-{rs,py,ex}` packages (redaction, audit-gated reveal, dump-forbidden semantics).

## 5. Identity, authorization, and trust

Summarised here; full detail with citations in `docs/system/security.md`.

- **Workload identity**: SPIFFE/SPIRE SVIDs; mTLS mandatory on the mesh. SPIRE entries under `platform/spire/`.
- **User identity**: Keycloak OIDC; JWT verification in `packages/eel-auth-rs/src/jwks.rs`.
- **Authorization**: SpiceDB with schema in `packages/authz-schema/`; checked on every write path.
- **Secrets**: OpenBao with accessors in `packages/eel-secrets-rs` and language siblings.
- **Policy**: OPA/Kyverno admission and data policies under `platform/`.

## 6. Election-day critical path

From `docs/architecture/cross-cutting/event-flow.md`, anchored to implementations:

1. Kit opens session → `bvi-svc` (`services/bvi-svc/crates/bvi-api/src/routes/sessions.rs` `open_bvi_session`, idempotent per ADR-0011).
2. Biometric identification → `biometric-edge-svc` → `abis-svc` match pipeline (`services/abis-svc/crates/abis-api/src/routes/match_routes.rs`).
3. Voter station lookup → audit-logged, timing-defended (`services/bvi-svc/crates/bvi-api/src/routes/voters.rs` `get_voter_station`; `services/bvi-svc/ex/bvv_web/lib/bvv_web/live/lookup_live.ex`).
4. Form 34A capture → `rts-svc` ICR validation and signing (`services/rts-svc/py/rts-kit`, `py/rts-icr`).
5. Publication → transparency log append and inclusion proof (`services/transparency-svc/crates/tlog-ingest/src/main.rs`, `crates/tlog-api/src/grpc/mod.rs` `append_leaf`, `get_proof`).
6. Public visibility → `portal-svc` and the Astro public site (`services/portal-svc`, `packages/eel-web-astro`).
7. Continuous transparency: STH production at 1s/60s/5min cadences (`crates/tlog-sequencer/src/sth_producer.rs`), witness cosigning (`crates/tlog-witness`), split-view detection.

Command-and-control overlays this path via `ecc-svc` (telemetry heatmaps, escalations, Section 44A dual-officer verification, grievance intake) — see `docs/system/services/ecc.md`.

## 7. Cross-cutting mechanisms

| Mechanism | ADR | Implementation anchors |
|---|---|---|
| Transactional outbox | ADR-0015 | `packages/eel-outbox-rs`, Django `OutboxRecord`, `packages/eel-outbox-ex` |
| Idempotency keys | ADR-0011 | `services/bvi-svc/crates/bvi-api/src/idem.rs`, fingerprint snapshots in `services/ecc-svc/crates/ecc-api/src/store.rs` |
| High-load clock (HLC) | — | `packages/eel-hlc-{rs,py,ex}` |
| Resilience (retry, bulkhead, load shed) | ADR-0009 | `packages/eel-resilience-{rs,py,ex}` |
| Observability | ADR-0004 | `packages/eel-observability-rs`, tracing-subscriber JSON in every Rust binary's `main.rs` |
| Traceability markers | WP-0.5 | `packages/eel-trace-rs`, audited by `tools/completeness` step 9.3 (`verification/completeness/steps/9.3-traceability.py`) |
| Reproducible builds | ADR-0006 | `reproducibility/make-reproduce/scripts/reproduce.sh`, `services/transparency-svc/crates/repro-bundle` |
| Schema evolution | ADR-0014 | Karapace registry, `packages/event-schemas/` |

## 8. Document map

| Volume | Contents |
|---|---|
| [`architecture.md`](architecture.md) (this file) | system shape, rules, data flow |
| [`security.md`](security.md) | identity, authz, crypto, PII, edge defences, threat model, known weaknesses |
| [`operations.md`](operations.md) | builds, tests, deployment, configuration, DR, CI gates |
| [`packages/shared-packages.md`](packages/shared-packages.md) | all shared libraries with API surfaces and consumers |
| [`tools-reference.md`](tools-reference.md) | developer/security tooling under `tools/` |
| [`services/*.md`](services/) | one implementation-referenced page per service (18 files) |

Where these volumes conflict with older prose under `docs/architecture/`, this set follows the code and flags the discrepancy; known doc-vs-code gaps are listed at the end of each affected page and consolidated in `docs/system/README.md`.
