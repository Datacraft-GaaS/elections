# transparency-svc

Source of truth: `services/transparency-svc/` (read directly). Cross-checked against `docs/architecture/services/transparency.md` and `services/transparency-svc/TRACEABILITY.yaml`. Where the prose docs and the code disagree, this document follows the code and flags the discrepancy.

## 1. Overview

transparency-svc operates an append-only, Merkle-tree transparency log ("T1 election transparency log") over platform event streams, per ADR-0017 / work package WP-2.1. Events consumed from Kafka (or appended directly via gRPC) are hashed (BLAKE3), sequenced into a single append-only log in PostgreSQL, and covered by Signed Tree Heads (STHs) produced at 1 s / 60 s / 5 min cadences. Independent witnesses cosign STHs; split-view detection and a cross-check CLI certify witness consistency. A separate HSM signer service (WP-2.2, ADR-0005) provides PKCS#11-backed ECDSA-P384 signing with M-of-N custodian ceremonies. Supporting crates cover reproducibility bundles (M14) and campaign-finance transparency (M22, stub).

- Runtime: Rust (tokio + axum + tonic + sqlx), one Django admin skeleton, one Astro site skeleton.
- Entrypoints (binaries): `crates/tlog-api/src/main.rs`, `crates/tlog-ingest/src/main.rs`, `crates/tlog-sequencer/src/main.rs`, `crates/tlog-witness/src/main.rs`, `crates/hsm-signer/src/main.rs`, CLI `crates/repro-bundle/src/main.rs` (`eel-repro`), CLI `crates/tlog-admin/src/main.rs` (`tlog-admin`).
- Listen ports:
  - tlog-api: `0.0.0.0:8080` REST (`BIND_ADDR`, default in `crates/tlog-api/src/main.rs:25`)
  - tlog-witness: `0.0.0.0:8082` HTTP (`BIND_ADDR`, default in `crates/tlog-witness/src/main.rs:36`)
  - hsm-signer: `0.0.0.0:8443` gRPC + `0.0.0.0:8081` health HTTP (`crates/hsm-signer/src/main.rs:42,52`)
  - tlog-sequencer / tlog-ingest: no listeners (background daemons)

## 2. Ownership & Boundaries

From `docs/architecture/services/transparency.md`: owns modules M14 (reproducibility-bundle) and M22 (campaign-finance). The service-level `TRACEABILITY.yaml` lists `work_package: WP-2.1` and crates `tlog-domain, tlog-sequencer, tlog-ingest, tlog-witness, tlog-api, tlog-admin`; its `modules:` field is empty (M14/M22 mapping not recorded there — see Known deferrals).

Data ownership rules:

- Owns the canonical append-only leaf store, tree tiles, STHs, witness registry/cosignatures, ingest offsets, and domain registry (see section 5). Rows are never mutated or deleted after append.
- Does not own source events; it records fingerprints of events produced by other services (leaf = hash of canonical event JSON + producer SPIFFE ID + timestamp + sequence, `crates/tlog-domain/src/leaf.rs:15` `LeafFormat::hash`).

Dependencies:

| Direction | Service | Basis |
|---|---|---|
| Upstream | rts-svc | Architecture doc: Form 34 publications trigger log appends (not visible in code — ingest subscribes to generic domain topics instead) |
| Upstream | Kafka topics `eel.voter-events`, `eel.results-events`, `eel.custody-events`, `eel.credential-events`, `eel.meta-events` | `crates/tlog-ingest/src/main.rs:24-30` |
| Downstream | portal-svc / public Astro site | Astro README links to tlog-api endpoints |
| Internal | hsm-signer / PKCS#11 | STH signing uses the `eel-sth` PKCS#11 key via softHSM in dev (`crates/tlog-sequencer/src/signer.rs`); network-HSM path exercised at deployment |

Events declared in `TRACEABILITY.yaml` as emitted: `eel.transparency.leaf_appended.v1`, `eel.transparency.tree_head_signed.v1`, `eel.transparency.witness_cosigned.v1`, `eel.transparency.split_view_detected.v1`. No Kafka producer code exists in the crates (not verified as wired); `deploy/service.yaml` declares the same four produce topics plus `*.tamper`.

Events consumed: any topic routed by `crates/tlog-ingest/src/topic_router.rs:6` `topic_to_domain` (prefixes `eel.results*`/`results*`, `eel.custody*`, `eel.credential*`, `eel.meta*`; everything else defaults to voter-events), plus direct gRPC appends.

## 3. Repository Layout

```
services/transparency-svc/
  TRACEABILITY.yaml        WP-2.1 traceability: crates list, emitted events
  Cargo.toml               workspace manifest
  src/                     empty (README describes a monolith binary here; not present)
  migrations/
    V1__initial_schema.sql Full log schema (leaves, tiles, sth, witnesses, offsets, domains)
  proto/
    tlog.proto             eel.transparency.v1 gRPC contract (TransparencyLog)
  crates/
    tlog-domain/           Domain types: Domain tags, Leaf/LeafFormat (BLAKE3), STH, proofs, witnesses
    tlog-sequencer/        Sequencer engine, PG access, proof computation, rate limiter, STH producer
    tlog-ingest/           Kafka consumer -> sequencer bridge with offset tracking
    tlog-api/              Axum REST router + tonic TransparencyLog service
    tlog-admin/            Operator CLI: checkpoint/export/verify-chain/witness-cross-check/bulk-ingest
    tlog-witness/          IAB internal witness: cosign loop, split-view detector, cosign API
    hsm-signer/            PKCS#11 signing daemon (eel.hsm.v1 gRPC), ceremonies, rate limits, metrics
    repro-bundle/          eel-repro CLI: bundle manifest types + verification (WP-4.6, M14)
    finance-transparency/  Campaign finance crate — contains only "// stub" (src/lib.rs:1)
  deploy/
    service.yaml           Deployment descriptor: images, replicas, sites, Kafka topics, SPIFFE IDs
  astro/transparency/      Public transparency portal skeleton (README only; src/pages and tests empty)
  django/finance-admin/    Django admin skeleton for M22 (README only; apps/templates/tests empty)
  docs/CAPABILITIES.md     Service capability narrative
  loadtest/                Empty directory (no k6 scenarios present)
  tests/integration_test.rs Leaf-hash and domain-parse smoke tests
```

## 4. Capabilities & Functions

### 4.1 REST API (tlog-api, port 8080)

Router: `crates/tlog-api/src/rest/mod.rs:13` `router`. All handlers return plain JSON status codes; error surface is bare StatusCode, not problem+json.

| Route | Handler | Purpose / shapes |
|---|---|---|
| GET `/api/v1/transparency/health` | `rest/health.rs:10` `get_health` | `{status:"ok", service:"tlog-api"}` |
| GET `/api/v1/transparency/sth/latest` | `rest/sth.rs:21` `get_latest_sth` | Newest STH by tree_size; `SthResponse {id, tree_size, root_hash, cadence, signed_at, signature}`; 404 when empty |
| GET `/api/v1/transparency/sth/{tree_size}` | `rest/sth.rs:46` `get_sth_by_size` | STH at an exact tree size; 404 otherwise |
| GET `/api/v1/transparency/proof/inclusion?leaf_hash=` | `rest/proofs.rs:22` `get_inclusion_proof` | Requires hex 32-byte BLAKE3 hash; resolves leaf index then rebuilds full Merkle tree (`proofs.rs:45-51`); returns `InclusionProof {leaf_index, tree_size, root_hash, hashes[]}` |
| GET `/api/v1/transparency/proof/consistency?from_size=&to_size=` | `rest/proofs.rs:61` `get_consistency_proof` | RFC 6962-style consistency proof between sizes; from=0 returns current root with empty path (`proofs.rs:76-87`) |
| GET `/api/v1/transparency/witnesses` | `rest/witnesses.rs:14` `list_witnesses` | Registered witnesses `{witness_id, witness_spiffe_id}[]` |
| POST `/api/v1/transparency/witnesses/cosign` | `rest/witnesses.rs:152` `post_cosign` | Accepts `{sth_id, witness_id, signature(hex)}`, inserts into `witness_cosigs`, always replies `{id, accepted:true}` |
| GET `/api/v1/transparency/witnesses/report?limit=` | `rest/witnesses.rs:75` `witness_report` | Cross-check payload `{sths[], cosignatures[], witnesses[]}`; limit clamped 1..1000 (default 50) |
| GET `/api/v1/transparency/domains` | `rest/sth.rs:79` `list_domains` | Static list of the five domains |
| GET `/api/v1/transparency/leaves/{index}` | `rest/leaves.rs:20` `get_leaf` | Leaf row by index `{index, domain, hash, producer_spiffe_id, source_event_id, event_time}` |

### 4.2 gRPC TransparencyLog (eel.transparency.v1, defined in `proto/tlog.proto`)

Implemented by `crates/tlog-api/src/grpc/mod.rs:20` `impl TransparencyLog for TlogGrpcService`. Mounted by the binary: `main.rs` serves REST on `BIND_ADDR` (default 8080) and gRPC on `GRPC_ADDR` (default 50051) concurrently, sharing one `ApiState`; failure of either bind aborts startup.

| RPC | Location | Behaviour |
|---|---|---|
| `AppendLeaf(LeafRequest) -> InclusionProofResponse` | `grpc/mod.rs:23` `append_leaf` | Parses domain, calls `Sequencer::append`; rate-limited appends return `rate_limited: true` with zeroed fields |
| `GetProof(ProofQuery) -> ProofResponse` | `grpc/mod.rs:68` `get_proof` | Oneof inclusion/consistency; validates 32-byte hash, size ordering, out-of-range to_size |
| `GetSth(SthQuery) -> SignedTreeHeadResponse` | `grpc/mod.rs:131` `get_sth` | Optional tree_size/cadence filters, newest match |
| `SubscribeSth(SthQuery) -> stream SignedTreeHeadResponse` | `grpc/mod.rs:142` `subscribe_sth` | Server-side poll loop at hot cadence (1 s), pushes each new STH id |

### 4.3 Sequencer engine (tlog-sequencer)

- `sequencer.rs:36` `Sequencer::append` — token-bucket admission (`rate.rs:22` `try_acquire`, 50 000 permits configured in both binaries), dedup by `(source_topic, source_event_id)` returning a stub proof for duplicates (`sequencer.rs:52-64`), leaf hash via BLAKE3 over `event_json || spiffe || time || seq`, insert via `db.rs:26` `append_leaf`, then inclusion proof against the full rebuilt tree.
- `proofs.rs:13` `load_leaf_hashes`, `proofs.rs:26` `current_root`, `proofs.rs:34` `inclusion_proof`, `proofs.rs:68` `consistency_proof`.
- `sth_producer.rs:20` `SthProducer::new(pool, signer)` + `spawn_all` — three background tasks (hot 1 s, default 60 s, batch 300 s); `produce_sth` reads latest leaf index, computes the real Merkle root, signs canonical bytes `tree_size || root_hash || cadence` **before** persisting, and stores signature + key fingerprint. Signing backend from env: PKCS#11/softHSM `eel-sth` key by default; `EEL_STH_SIGNER=dev` selects an explicit Ed25519 dev key (`crates/tlog-sequencer/src/signer.rs`).

### 4.4 Kafka ingest (tlog-ingest)

- `consumer.rs:31` `IngestConsumer::run` — manual-commit consumer group `tlog-ingest` over the five domain topics; concurrency bounded by a 1000-permit semaphore (`main.rs:22`).
- Per message: parse `eel_events_rs::Envelope`, route topic→domain (`topic_router.rs:6`), strip `ed25519:<base64>` prefix from `eel_signature` (hand-rolled base64 decoder `consumer.rs:122`), append to sequencer, persist offset via `offset_tracker.rs` `save_offset`. Rate-limited or failed appends are logged and retried implicitly (offset saved regardless — see Known deferrals).

### 4.5 Witness subsystem (tlog-witness, port 8082)

- Cosign API: `handler.rs:32` `post_cosign` mirrors the API-side endpoint (adds optional `witness_spiffe_id`).
- IAB witness loop: `iab_client.rs:37` `from_env` loads `EEL_WITNESS_SIGNING_KEY` (hex Ed25519 seed); `run` (`iab_client.rs:52`) polls `GET /api/v1/transparency/sth/latest` every 60 s, signs `concat(tree_size_le64, root_hash)` (`cosign_message`, `iab_client.rs:64`) and posts the cosignature.
- Split-view detection: `split_view.rs:20` `SplitViewDetector::observe` flags two different roots for the same tree size and logs `"SPLIT VIEW DETECTED"`. It is instantiated nowhere in the shipped binaries (library-only).

### 4.6 HSM signer (hsm-signer, gRPC 8443 / health 8081, eel.hsm.v1)

Service impl `crates/hsm-signer/src/lib.rs:115` `impl HsmSigner for HsmSignerService`:

| RPC | Location | Behaviour |
|---|---|---|
| `Sign(SignRequest)` | `lib.rs:118` `sign` | Role-keyed PKCS#11 ECDSA-P384 signing off-thread; per-role rate limits; Prometheus counters |
| `Verify(VerifyRequest)` | `lib.rs:150` `verify` | Signature check with the role's public key |
| `KeyInfo(KeyRoleRequest)` | `lib.rs:169` `key_info` | Placeholder metadata (`key_id = "eel-<role>"`, algorithm string; CKA parsing deferred) |
| `RotateKey(RotateRequest)` | `lib.rs:188` `rotate_key` | CycleKek rotations require an Approved ceremony (`assert_approved`); actual key generation deferred to WP-3.x — returns synthetic old/new key ids |
| `Ceremony(CeremonyRequest) -> stream CeremonyEvent` | `lib.rs:232` `ceremony` | Opens or subscribes to an M-of-N ceremony; streams state transitions over broadcast channels |

Supporting modules: `ceremony.rs:127` `CeremonyStore` (states Requested → AwaitingQuorum → Approved → Executing → Complete/Aborted; Ed25519 approvals verified over `ceremony_id || operation_hash` at `ceremony.rs:185` `approve`), `pool.rs:71` `RolePool::open` (vendor selection softhsm/luna7/utimaco via `EEL_HSM_VENDOR`), `rate.rs:38` `RoleLimiters` (Sth 60/s, Declaration 10/s, KitCa/CredentialCa 100/s, CodeSigning 5/s, CycleKek 1/hour per ADR-0005), `metrics.rs:18` `metrics()`.

Note: the doc comment on `lib.rs:4` claims SPIFFE SVID TLS interception "configured in main.rs"; `main.rs` serves plain TCP for dev — no TLS interceptor exists in code (contradiction).

### 4.7 Admin CLI (tlog-admin)

Dispatch in `crates/tlog-admin/src/main.rs:45`:

| Subcommand | Impl | Status |
|---|---|---|
| `checkpoint --domain <d>` | `checkpoint.rs:run` — fetches latest STH from API (domain arg validated but unused in query) | works |
| `export --to <path> [--db-url]` | `export.rs:run` — tree snapshot export | implemented |
| `verify-chain --sth-a --sth-b` | `verify.rs:run` — calls consistency-proof endpoint | works |
| `witness-cross-check` | `witness.rs:run` — fetches witness report, verifies all Ed25519 cosignatures, fails on split views | fully implemented with unit tests |
| `bulk-ingest --topic --from-offset` | `bulk_ingest.rs:run` — logs and prints "(stub)" | STUB |

### 4.8 Reproducibility bundles (repro-bundle / M14)

CLI `eel-repro` (`crates/repro-bundle/src/main.rs`): `verify <manifest.json> <dir>` recomputes SHA-256 of each component and reports mismatches; `manifest` prints a fresh unsigned skeleton. Manifest model in `manifest.rs:18` `BundleManifest` includes `canonical_bytes()` for signing (signature field excluded) — no signing caller exists yet. Participation tokens + JSON-LD inclusion proofs in `token.rs:7` `ParticipationToken`, with `token.rs:50` `InclusionProof::verify` explicitly documented as a v1 stub (returns true if audit path non-empty).

## 5. Data Model

All tables in `migrations/V1__initial_schema.sql`:

| Table | Key columns | Purpose |
|---|---|---|
| `leaves` | `index` (from `leaves_index_seq`, unique), `domain`, `hash BYTEA`, `producer_spiffe_id`, `source_topic+source_event_id` (unique pair), `producer_signature` | Append-only log entries |
| `tree_tiles` | `(level, tile_index)` PK, `hash` | Cached Merkle tile hashes (written by nothing currently — not verified in use) |
| `sth` | `tree_size`, `root_hash`, `cadence`, `signature`, `key_role`, `key_fingerprint` | Signed Tree Heads at hot/default/batch cadences |
| `sth_batch` | `parent_sth_id` → `child_sth_id` | STH chaining/batching relationships |
| `witnesses` | `witness_id` PK, `witness_spiffe_id`, `public_key_hex`, `is_internal` | Registered witness verification keys |
| `witness_cosigs` | `(sth_id, witness_id)` unique, `signature` | Witness cosignatures over STHs |
| `ingest_offsets` | `(topic, partition)` PK, `committed_offset` | Kafka offset tracking |
| `domains` | `name` PK | Seeded with voter-events, results-events, custody-events, credential-events, meta-events (`V1:88-94`) |

Store abstractions: none beyond sqlx runtime queries; state struct `ApiState {pool, sequencer}` (`crates/tlog-api/src/state.rs`). No Mem backend.

Event schemas: Kafka envelope `eel_events_rs::Envelope` (fields used: `id`, `source`, `time`, `data`, `eel_signature` — `consumer.rs:58-78`). Emitted topics are declared in `TRACEABILITY.yaml` / `deploy/service.yaml` but no producer code publishes them (not verified).

HSM side has no SQL persistence; ceremonies live in memory (`ceremony.rs:7` notes PG durability via outbox is follow-on).

## 6. Operation

Build and test (workspace at `services/transparency-svc/Cargo.toml`):

```bash
cargo build -p tlog-api -p tlog-ingest -p tlog-sequencer -p tlog-witness \
  -p hsm-signer -p tlog-admin -p repro-bundle
cargo test -p tlog-domain -p tlog-sequencer -p tlog-witness -p hsm-signer -p tlog-admin
cargo test --test integration_test   # smoke tests at services/transparency-svc/tests/
```

Run (each binary separately):

```bash
DATABASE_URL=postgres://eel:eel@localhost/eel_tlog cargo run -p tlog-api       # :8080
DATABASE_URL=... KAFKA_BROKERS=localhost:9092 cargo run -p tlog-ingest
DATABASE_URL=... cargo run -p tlog-sequencer                                   # STH cadences
DATABASE_URL=... TLOG_API_URL=http://localhost:8080 \
  EEL_WITNESS_SIGNING_KEY=<hex seed> cargo run -p tlog-witness                 # :8082
EEL_HSM_PIN=... [EEL_HSM_SLOT=0] [EEL_HSM_VENDOR=softhsm|luna7|utimaco] \
  [PKCS11_MODULE_PATH=...] cargo run -p hsm-signer                             # :8443/:8081
```

Environment variables (verified in code): `DATABASE_URL` (all DB binaries), `BIND_ADDR` (tlog-api, tlog-witness), `KAFKA_BROKERS` (ingest), `TLOG_API_URL` (witness), `EEL_WITNESS_SIGNING_KEY` (witness, required), `EEL_HSM_PIN`, `EEL_HSM_SLOT`, `EEL_HSM_VENDOR`, `PKCS11_MODULE_PATH` (hsm-signer), `RUST_LOG` (via EnvFilter in hsm-signer main).

The README's build/run instructions reference a `--bin transparency-svc` monolith, `deploy/base` kustomize overlays, and OpenAPI paths under `/v1/tlog/*` that do not exist in the tree — treat README as aspirational.

Deployment artifact: `deploy/service.yaml` declares four images (tlog-api :8080, tlog-sequencer :9090, tlog-ingest :9091, tlog-witness :8082), replica counts, primary/mirror sites, blue-green strategy, health probe path `/api/v1/transparency/health`, MinIO snapshot bucket `eel-tlog-snapshots` (15-min interval), and SPIFFE id `spiffe://eel.local/transparency-svc`. Note the sequencer/ingest ports listed there have no listener in code.

Astro portal and Django admin: skeletons with READMEs only; commands documented in their READMEs (`pnpm dev`, `uv run manage.py runserver`) cannot be verified against missing sources.

Known deferrals:

- RESOLVED: STH rows now carry real signatures from the configured backend (PKCS#11/softHSM `eel-sth` key, or explicit dev Ed25519 via `EEL_STH_SIGNER=dev`) — see `crates/tlog-sequencer/src/signer.rs`. Production trust still requires exercising the network-HSM key + M-of-N ceremony at deployment.
- `finance-transparency` crate is a literal stub (`src/lib.rs:1`); M22 has no implementation.
- `tlog-admin bulk-ingest` is a stub.
- Participation-token inclusion proof verification is a v1 stub (`repro-bundle/src/token.rs:50`).
- gRPC TransparencyLog service built but not served by the tlog-api binary (`main.rs:22` mounts REST only).
- `tree_tiles`, `sth_batch` tables written by nothing in-repo.
- Ingest saves offsets even when appends fail/rate-limit (`consumer.rs:93-102` before `save_offset`), so failed events can be skipped until offset reset — flagged as risk.
- Split-view detector library unused by any binary; detection today happens only in `tlog-admin witness-cross-check`.
- `loadtest/` directory is empty despite README referencing `k6 run sth-publication.js`.

## 7. Safeguards & Security

Verified in code:

- Rate limiting (append path): token-bucket semaphore `crates/tlog-sequencer/src/rate.rs:22` `try_acquire`; saturation surfaces as `rate_limited: true` on gRPC AppendLeaf (`grpc/mod.rs:50-57`). HTTP REST callers receive no explicit 429 mapping (the doc comment mentions "respond 429" but no handler maps it — gap).
- Per-role HSM rate limits: `crates/hsm-signer/src/rate.rs:65` `check` returns retry-after seconds; `lib.rs:93` `check_rate` converts to `RESOURCE_EXHAUSTED` with `retry-after` metadata and increments `eel_hsm_rate_limited_total`.
- Ceremony quorum control: M-of-N approval with Ed25519 signature verification over `ceremony_id || operation_hash` (`crates/hsm-signer/src/ceremony.rs:185` `approve`); terminal-state guards; abort restricted to pre-quorum states (`ceremony.rs:264` `abort`); RotateKey gated on Approved state (`ceremony.rs:338` `assert_approved`).
- Deduplication / replay resistance: unique `(source_topic, source_event_id)` on `leaves` (migration line 18) enforced by `db.rs:44` `leaf_exists` check plus DB constraint.
- Witness protocol integrity: cosignature message is `concat(tree_size_le64, root_hash_bytes)`; verified independently by `tlog-admin` cross-check (`tlog-admin/src/witness.rs` `verify_cosignature`, `split_views`) and unit-tested in `iab_client.rs` tests and `witness.rs` tests.
- Input validation: 32-byte BLAKE3 hash checks on both REST (`rest/proofs.rs:27-30`) and gRPC (`grpc/mod.rs:77-81`); consistency-size sanity checks (`grpc/mod.rs:101-104`, `rest/proofs.rs:66-75`).
- Audit logging: structured `tracing` info/warn/error events throughout (`leaf appended`, `cosigned STH`, `SPLIT VIEW DETECTED`, `sign failed`); no dedicated audit table in this service.

Not present in code (despite docs claiming them): authentication on tlog-api REST/gRPC (no bearer/SVID checks anywhere in tlog-api), problem+json error documents, idempotency keys, timing-defence (constant-time compare / response jitter), encryption at rest. The hsm-signer doc comment claims SVID-based mTLS enforcement, but no TLS interceptor is configured (`main.rs:50-55` serves plain TCP in the shipped code) — treat as unimplemented.

Tests verifying safeguards:

- Rate limiter + ceremony state machine: `crates/hsm-signer/tests/hsm_signer_tests.rs` (limiter tests lines ~41-64; ceremony open/approve/abort/quorum tests lines ~86-172; SoftHSM round-trip tests skip gracefully without SoftHSMv2, lines ~176-209).
- Cosign message format and signature validity: `crates/tlog-witness/src/iab_client.rs:105-204` (`cosign_message_is_le_size_then_root_bytes`, `signature_verifies_under_witness_key`, end-to-end `cosign_latest_posts_verifiable_signature`).
- Cross-check logic incl. tampered roots, malformed signatures, split views: `crates/tlog-admin/src/witness.rs` `mod tests`.
- Problem-json rendering: n/a (not implemented). Domain/leaf hashing: `tests/integration_test.rs:9-27`.

## 8. Observability

- Logging: `tracing_subscriber::fmt::init()` in tlog-api/ingest/sequencer/witness mains; JSON logging with EnvFilter (`hsm_signer=info`) in hsm-signer (`main.rs:26-29`).
- Metrics (hsm-signer only, Prometheus via `prometheus` crate): `eel_hsm_sign_total`, `eel_hsm_sign_error_total`, `eel_hsm_ceremony_started_total`, `eel_hsm_ceremony_complete_total`, `eel_hsm_ceremony_aborted_total`, `eel_hsm_rate_limited_total`, `eel_hsm_active_ceremonies` (`crates/hsm-signer/src/metrics.rs:19-61`). No HTTP metrics endpoint is mounted (counters registered but unexposed — gap). telemetry-query style metric registries do not exist elsewhere in this service.
- Health: GET `/api/v1/transparency/health` (`rest/health.rs:10`); hsm-signer `/healthz` and `/readyz` (`main.rs:43-45`).
- Notable log events: `tlog-api listening`, `tlog-ingest subscribed`, `leaf appended` (with leaf_index), `rate limit exceeded, will retry`, `STH produced` (cadence + tree_size), `cosigned STH`, `SPLIT VIEW DETECTED`, `ceremony opened`, `key rotation initiated`.
- Tracing/OpenTelemetry: not present (no OTel SDK references found).
