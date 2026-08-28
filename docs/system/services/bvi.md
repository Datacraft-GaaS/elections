# bvi-svc

Source of truth: `services/bvi-svc/` (read directly). Cross-checked against `docs/architecture/services/bvi.md`, `services/bvi-svc/TRACEABILITY.yaml`, and the service README. Code is authoritative; discrepancies flagged.

## 1. Overview

bvi-svc is the Biometric Voter Identification backend (modules M06 bvi-backend, M07 bvv). The implemented core is `bvi-api`, a Rust/Axum REST service that KIEMS kits call to open/close polling sessions, submit identification events, telemetry, and distress signals, pull per-station configuration, and revoke kit certificates; turnout aggregation and voter-station lookups are served from PostgreSQL. A Phoenix LiveView app (`ex/bvv_web`) provides the public BVV portal: voter-register lookup with anti-enumeration timing defence and anonymous objection filing. Supporting crates: `bvi-config` (env config) and `bvi-turnout` (turnout scope/rollup read logic).

- Runtime languages: Rust (axum + sqlx) and Elixir (Phoenix LiveView).
- Entrypoints: `crates/bvi-api/src/main.rs` (binary `bvi-api`); `ex/bvv_web/mix.exs` (OTP app `BvvWeb.Application`, `lib/bvv_web/application.ex`).
- Listen ports:
  - bvi-api: `0.0.0.0:8080` via `HTTP_ADDR` (`crates/bvi-api/src/main.rs:18`)
  - bvv_web: `127.0.0.1:4000` dev / port 4000 prod default (`ex/bvv_web/config/dev.exs`, `config/runtime.exs`)
  - Note: `bvi-config` defaults `BIND_ADDR` to `0.0.0.0:8088` (`crates/bvi-config/src/lib.rs:20`) but `bvi-api/main.rs` uses `HTTP_ADDR`/8080 and never calls `BviConfig` — inconsistency (see Known deferrals).

## 2. Ownership & Boundaries

From `docs/architecture/services/bvi.md` and `TRACEABILITY.yaml`: owns M06 (kit config bundles, identification events, turnout rollup, kit certificate management) and M07 (voter lookup audit, objection handling). Work package WP-3.7, gate G3. ADR notes recorded in TRACEABILITY: ADR-0013 (not event-sourced; state + audit shadow table), ADR-0015 (transactional outbox; 64 partitions for identification events), ADR-0016 (PII retention: hashed IDs in audit log), plus a signing-role decision dedicating a `StationConfig` key role separate from Sth.

Data ownership rules:

- Owns session lifecycle, identification event mirror rows, kit configs/certs, turnout rollups, objections, lookup audit, distress events, telemetry, outbox, idempotency store (section 5).
- Kafka is source of truth for identification events; PostgreSQL is a read mirror (`migrations/V3__identification_events.sql:1-3`). In the current code, events arrive over the REST API rather than a consumer.
- Station descriptive metadata is mirrored from boundary-svc; the service "never invents station metadata on its own" (`migrations/V13__polling_stations.sql:1-4`).
- Audit shadow rows are references only — no biometric refs or raw identifiers in `audit.shadow_log` entries (`crates/bvi-api/src/store.rs:266-282`).

Dependencies:

| Direction | Service | Basis |
|---|---|---|
| Upstream | bvr-svc | Architecture doc (register extracts for bundles) |
| Upstream | abis-svc | Architecture doc (match decisions; kits perform local 1:1) |
| Upstream | inventory-svc | Architecture doc (kit serial registry) |
| Upstream | boundary-svc | `V13__polling_stations.sql` sync job |
| Downstream | rts-svc, ecc-svc, portal-svc, queue-svc | Architecture doc (session-open gating, turnout feeds) |
| Internal | Debezium/Kafka outbox relay | `store.rs:337` comment: Debezium publishes outbox topics |

Events produced (verified in code, via transactional outbox consumed by Debezium):

| Topic | Emitter |
|---|---|
| `eel.bvi.distress_raised.v1` | `crates/bvi-api/src/store.rs:344-351` `emit_outbox` inside the distress insert transaction |
| `eel.bvi.kit_revoked.v1` | `crates/bvi-api/src/store.rs:450-457` (CRL distribution consumes it) |

Events listed only in the architecture doc (`eel.bvi.kit.session.opened/closed`, `eel.bvi.identification.event`, `eel.bvi.turnout.rollup`, `eel.bvi.kit.cert.revoked`, consumed `eel.bvr.register.snapshot.published`, `eel.inventory.kit.assigned`, `eel.abis.match.completed`) have no producer/consumer code in this repo (not verified as wired).

## 3. Repository Layout

```
services/bvi-svc/
  Cargo.toml               workspace manifest
  TRACEABILITY.yaml        WP-3.7 requirement traceability (FR-M06-*, FR-M07-*, EC-SEC-10, EC-OPS-02)
  data_dictionary.yaml     Data element definitions
  crates/
    bvi-api/               REST service: routes/, models, store abstraction, idempotency, problem+json
      src/routes/          One module per contract resource (sessions, identifications, signals, config, kits, turnout, voters)
      src/idem.rs          Idempotency-Key + bearer plumbing (ADR-0011)
      src/error.rs         RFC 9457 problem+json surface
      src/store.rs         BviStore trait + PgStore implementation
      src/models.rs        Contract request/response types and validation constants
    bvi-config/            Env-driven config struct (DATABASE_URL, BIND_ADDR, EEL_ELECTION_ID)
    bvi-turnout/           TurnoutScope resolution + rollup aggregation reads (library)
  ex/
    bvv_web/               Phoenix LiveView public portal (BVV)
      lib/bvv_web/live/lookup_live.ex      Voter register lookup
      lib/bvv_web/live/objections_live.ex  Objection filing/tracking
      lib/bvv_web/objections.ex            Objections context (PubSub broadcast)
      lib/bvv_web/controllers/health_controller.ex
      test/                                LiveView/context tests
  migrations/              V1-V14 (see section 5)
  deploy/service.yaml      WP-1.9 deployment template (image, sites, strategies, replicas)
  astro/bvv/               Minimal Astro site skeleton (index/about pages only)
  loadtest/k6/bvi_load.js  k6 scenario: 18k req/s for 60 s, p99<100ms threshold
  tests/                   Empty directory
```

The README's directory description ("src/ binary entry points", "astro public portal") predates the crate layout; there is no top-level `src/`.

## 4. Capabilities & Functions

### 4.1 REST API (bvi-api)

Router: `crates/bvi-api/src/routes/mod.rs:14` `router`. All mutating endpoints require an `Idempotency-Key`; read endpoints require a bearer header presence check (section 7).

| Route | Handler | Behaviour |
|---|---|---|
| POST `/api/v1/bvi/sessions` | `routes/sessions.rs:29` `open_bvi_session` | Opens a device session; one active session per device+election day (partial unique index → 409); non-empty validation of station_code/device_serial/election_id; 201 with Location header |
| PATCH `/api/v1/bvi/sessions/{id}/close` | `routes/sessions.rs:66` `close_bvi_session` | Closes active sessions (idempotent re-close returns existing record); unknown ids 404 before replay check |
| POST `/api/v1/bvi/identifications` | `routes/identifications.rs:31` `record_identification` | Validates decision ∈ {match,no_match,fallback_used,ambiguous} (`models.rs:42`), match_score ∈ [0,1], modality ∈ {fingerprint,iris,face,nfc} (`models.rs:47`); session must exist and be active (404/422); second matched ID for same voter+election → 409 `bvi/already_identified` (EC-OPS-02) |
| POST `/api/v1/bvi/telemetry` | `routes/signals.rs:22` `submit_telemetry` | battery_pct ∈ [0,100]; stores row, replies 204 with body-less idempotency snapshot |
| POST `/api/v1/bvi/distress` | `routes/signals.rs:52` `raise_distress` | distress_type ∈ `models.rs:117` DISTRESS_TYPES (officer_duress, device_tamper, power_failure, network_failure, physical_intrusion, other); persists + emits outbox event |
| GET `/api/v1/bvi/config/{station_code}?election_id=` | `routes/config.rs:32` `get_station_config` | Latest kit_configs row for station; cacheable `max-age=300, private` + strong ETag `"cfg-v{version}-{station}"`; election_id required |
| POST `/api/v1/bvi/kits/{serial}/revoke` | `routes/kits.rs:30` `revoke_kit` | reason ∈ `models.rs:168` REVOKE_REASONS (stolen/tampered/compromised/defective/reassigned); officer_id required; flips kit_certs.revoked, force-closes active sessions of that kit, emits outbox |
| GET `/api/v1/bvi/kits/health` | `routes/kits.rs:63` `list_kits_health` | NOC dashboard feed: latest telemetry per kit + revoked flag + today's ident/error counts; online = last seen ≤90 s; cursor pagination with Link header; filters station_code/online/page_size |
| GET `/api/v1/bvi/turnout?election_id=&county_code=&constituency_code=&station_code=` | `routes/turnout.rs:27` `get_turnout` | Aggregated TurnoutSummary at most-specific scope; `Cache-Control: public, max-age=30` |
| GET `/api/v1/bvi/voters/{voter_id}/station?election_id=` | `routes/voters.rs:27` `get_voter_station` | Resolves station from latest matched identification joined to polling_stations mirror; every lookup audited (found or not) with hashed actor/IP |

### 4.2 Store layer

Trait `crates/bvi-api/src/store.rs:51` `BviStore` with production impl `PgStore` (`store.rs:108`); the trait exists so the whole router can run against an in-memory fake in tests (no MemStore fake is shipped in-repo — tests directory empty). Notable operations: `open_session` (`store.rs:178`), `record_identification` (`store.rs:232`, writes mirror row + audit.shadow_log entry), `revoke_kit` (`store.rs:399`, transactional cert flip + session closure + outbox), `kits_health` (`store.rs:475`, DISTINCT ON latest telemetry), `voter_station` (`store.rs:556`, audit insert in same tx), `idem_get`/`idem_put` (`store.rs:610-656`, 24 h TTL with lazy expiry sweep).

### 4.3 Turnout library (bvi-turnout)

- `crates/bvi-turnout/src/lib.rs:42` `resolve_scope` — most specific of station > constituency > county > national; empty strings don't narrow.
- `lib.rs:60` `turnout_pct` — two-decimal percentage, 0 when registered = 0.
- `lib.rs:165` `turnout_summary` — exact pre-aggregated row first (`turnout_rollups`), fallback to summing child station rows by code prefix.

### 4.4 BVV Phoenix portal (ex/bvv_web)

Router `ex/bvv_web/lib/bvv_web/router.ex:17` (browser pipeline with CSRF + secure headers):

| Route | Module | Purpose |
|---|---|---|
| live `/` and `/lookup` | `live/lookup_live.ex` | Voter register lookup form (national_id + OTP fields). `handle_event("lookup")` (`lookup_live.ex:26`) enforces min 300 ms + 0-50 ms jitter; `perform_lookup` (`lookup_live.ex:44`) proxies to bvi-api getVoterStation; 404 and errors return an identical-shape not-found result so enumeration is indistinguishable (EC-SEC-10) |
| live `/objections`, `/objections/new`, `/objections/:id` | `live/objections_live.ex` | Anonymous objection filing. Changeset validations in `objections/objection.ex` (types rectification/transfer_request/removal_request; justification 10..4000 chars; target_voter_id ≤64). Objector identity is a SHA-256 session-token hash or random opaque value (`objector_session_hash`, `objections_live.ex` bottom) |
| GET `/health` (JSON) | `controllers/health_controller.ex:4` `index` | `{status:"ok", service:"bvv_web"}` |

Context `objections.ex:23` `create_objection` persists and broadcasts `{:objection_filed, objection}` on PubSub topic `eel:bvv:objections`; LiveViews subscribe on connect.

### 4.5 Load test

`loadtest/k6/bvi_load.js` — constant-arrival-rate 18 000 req/s for 60 s against `POST /identifications`-shaped payloads; thresholds p99 < 100 ms and error rate < 0.1% (FR-M06-017, AC-M06-09).

## 5. Data Model

Migrations `migrations/V1..V14`:

| Migration | Table(s) | Key columns / purpose |
|---|---|---|
| V1 | `audit.shadow_log` (+ trigger fn `audit_shadow_trigger`) | Append-only audit trail: table_name, operation, old_row/new_row JSONB |
| V2 | `kit_configs` | Per-station versioned bundles: bundle_id, voter_list_hash, geofence_hash, cert_hash, minio_key, signed_at; UNIQUE (station_code, version) |
| V3 | `identification_events` | Mirror of kit events: event_id (unique, EC-DAT-01), voter_id, station_code, kit_serial, modality CHECK, outcome CHECK, ts_hlc, gps_fix, kit_sig, honey_mode |
| V3 (ALTER) | — | Adds election_id (default EL-2027); partial unique index `identification_events_voter_election_uq` on (voter_id, election_id) WHERE outcome='matched' (EC-OPS-02) |
| V4 | `turnout_rollups`, materialized view `mv_turnout_station` | scope ∈ {station,constituency,county,national}, counts, UNIQUE (scope, scope_code); MV counts matched events per station |
| V5 | `kit_certs` | kit_serial unique, cert_pem, issued_at, revoked flags/reason/by; partial index on revoked=true |
| V6 | `objections` | obj_id (citizen-facing), objector_session_hash, target_voter_id, type/status CHECKs, resolution fields |
| V7 | `voter_lookup_audit` | actor_session_hash, target_voter_id_hash, ip_hash, result_status ∈ {found, not_found, rate_limited, error}; not erasable even on SAR (FR-M07-021) |
| V8 | `outbox` | Transactional outbox (ADR-0015): event_id, topic, partition_key, payload, signature; partial index on unpublished |
| V9 | `request_fingerprints` | Idempotency snapshots: UNIQUE (operation, idempotency_key), response_status/body, expires_at (24 h) |
| V10 | `bvi_sessions` (+ ALTER identification_events add session_id, widen outcome CHECK) | session_id unique, status ∈ {active,closed,revoked}; partial unique index: one active session per (device_serial, election_id) |
| V11 | `device_telemetry` | battery_pct 0-100 CHECK, signal_strength_dbm, temperature_c; index (device_serial, ts DESC) |
| V12 | `distress_events` | distress_type CHECK (6 values), raised_at, received_at |
| V13 | `polling_stations` | Boundary-svc mirror: station_code PK, name, constituency_code, county_code |
| V14 | ALTER kit_configs | Device-facing delivery defaults: match_threshold (0.5-1.0 CHECK, default 0.95), allowed_modalities JSONB, timeout_s (5-120 CHECK, default 30), fallback_allowed |

Store abstraction: trait-over-Pg pattern described above; no Redis or other caches despite architecture-doc claims (TurnoutRollup "Redis + PostgreSQL" — Redis code absent, not verified).

Event schemas: outbox payloads are ad-hoc JSON (`{"distress_id","station_code","device_serial","distress_type","raised_at"}` and `{"serial","reason","officer_id"}`).

## 6. Operation

Build/test:

```bash
cargo build -p bvi-api -p bvi-config -p bvi-turnout        # from services/bvi-svc
cargo test -p bvi-api -p bvi-turnout -p bvi-config         # unit tests incl. router-level handler tests
mix deps.get && mix test                                    # in services/bvi-svc/ex/bvv_web
k6 run loadtest/k6/bvi_load.js                              # requires BVI_URL env
```

Run:

```bash
DATABASE_URL=postgres://... HTTP_ADDR=0.0.0.0:8080 cargo run -p bvi-api
cd ex/bvv_web && mix phx.server                             # :4000
```

Environment variables (grep of env usage):

| Variable | Consumer | Notes |
|---|---|---|
| `DATABASE_URL` | bvi-api main (required, panics if unset); BvvWeb.Repo runtime.exs | |
| `HTTP_ADDR` | bvi-api main (`main.rs:18`), default 0.0.0.0:8080 | |
| `BIND_ADDR`, `EEL_ELECTION_ID` | bvi-config `from_env` (`lib.rs:37-55`); defaults 0.0.0.0:8088, EL-2027 | unused by the bvi-api binary |
| `RUST_LOG` | tracing EnvFilter, default info (`main.rs:11-13`) | |
| `PORT`, `SECRET_KEY_BASE`, `POOL_SIZE` | BvvWeb runtime.exs (prod, all fetch!) | PORT default 4000 |
| `bvi_api_url`, `election_id` | BvvWeb application env (`lookup_live.ex:45,84`), defaults `http://bvi-api:8080` and `elec-2026-general` | set via config files |

Deployment: `deploy/service.yaml` — image `harbor.eel.internal/eel/bvi-svc`, blue-green at primary/mirror, rolling at five edge sites, 3/3/2 replicas. No health/readiness path declared in the file.

Degradation modes (architecture doc): kits queue events offline when unreachable (device-side, not in this repo); stale turnout serving from cache — not implemented in code (rollups read straight from PG).

Known deferrals:

- No Kafka consumer for identification events; `crates/bvi-turnout/src/consumer.rs` referenced by TRACEABILITY does not exist. FR-M06-013 and FR-M06-001 are marked `status: stub` in TRACEABILITY, consistent with the tree (referenced impl paths `crates/bvi-config/src/bundle.rs`, `crates/bvi-api/src/handlers.rs`, `crates/bvi-turnout/src/honey.rs`, and test files under `crates/*/tests/` are absent).
- Rollup refresh jobs / materialized-view maintenance are not present (MV created WITH DATA, no refresher).
- Astro `bvv` portal contains only placeholder pages; the README describes a much larger SPA.
- `tests/` at the service root is empty; the hermetic in-memory store mentioned in `store.rs` docs is not shipped.

## 7. Safeguards & Security

Verified in code:

- Idempotency (ADR-0011): mandatory UUID `Idempotency-Key` header validated by `crates/bvi-api/src/idem.rs:19` `require_key`; success-response replay via `idem.rs:31` `replay` (duplicates always answer 200 per contract; error outcomes never snapshotted); snapshots persisted in `request_fingerprints` with 24 h TTL (`idem.rs:51` `snapshot`, lazy sweep in `store.rs:616`). Applied to every mutating route.
- Authentication posture: tiered — `idem.rs:81` `require_bearer` enforces a syntactically valid `Authorization: Bearer <non-empty>` header on tier 2/3 reads (kits/health, turnout, voter station); actual JWT validation happens at the Envoy gateway per the doc comment. Internal mTLS is terminated at the sidecar (doc comment, `lib.rs` header) — no SVID checks inside the app itself.
- Anti-enumeration timing defence (EC-SEC-10, FR-M07-005): minimum 300 ms response plus 0-50 ms uniform jitter in `ex/bvv_web/lib/bvv_web/live/lookup_live.ex:26` (`@min_response_ms 300`); hit/miss responses share identical payload shape (`perform_lookup` vs `not_found_result`).
- PII minimisation: audit rows carry SHA-256 hashes of authorization header, forwarded IP, and target voter id (`crates/bvi-api/src/routes/voters.rs:57` `hash_header`, `store.rs:594` `sha256_hex`); audit rows are non-erasable (FR-M07-021, migration V7 comment).
- Audit logging: every identification insert also writes a reference-only `audit.shadow_log` row (`store.rs:267-282`); DB-level trigger function available for all tables (`V1__init.sql:5-14`); every voter lookup audited found-or-not (`store.rs:586-598`).
- Validation rules: decision/modality/distress-type/reason allow-lists (`models.rs:42,47,117,168`), numeric range checks (match_score, battery_pct, threshold/timeout enforced again by DB CHECKs in V14), non-empty field checks, election_id required on scoped reads.
- Error surface: RFC 9457 `application/problem+json` documents with typed slugs (`common/*`, `bvi/already_identified`, `bvi/invalid_session_state`) built by `crates/bvi-api/src/error.rs:106` `IntoResponse for ApiError`; base URI `https://errors.eel.internal`.
- Double-identification protection: DB-enforced partial unique index mapped to a specific 409 problem type (`store.rs:257-261`).
- Portal anonymity: objector identity stored only as a hash (`objections_live.ex` `objector_session_hash`); CSRF protection and secure browser headers on all browser routes (`router.ex` pipelines).
- Outbox durability: distress alerts survive Kafka outage via transactional outbox (`store.rs:316-362`).

Not implemented in code: request rate limiting (only the k6 threshold implies it elsewhere), encryption at rest, signature verification of kit-submitted events (the `kit_sig` column exists but nothing validates it), OTP verification in the BVV lookup form (field collected but ignored).

Tests verifying safeguards:

- Problem+json rendering: `crates/bvi-api/src/error.rs:128` `renders_problem_json`.
- Config requirements/defaults: `crates/bvi-config/src/lib.rs` tests.
- Turnout scope/pct math: `crates/bvi-turnout/src/lib.rs:199-245` (`most_specific_scope_wins`, `pct_math`, ...).
- Objection changeset + LiveView behaviour: `ex/bvv_web/test/bvv_web/objections_test.exs`, `test/bvv_web/live/objections_live_test.exs`.
- Timing-oracle and revocation behaviours referenced in TRACEABILITY (`timing_oracle_test.rs`, `revocation_test.rs`, `kit_swap_test.rs`, `honey_mode_test.rs`) do not exist in the tree — their safeguards are verified only by the LiveView/DB constraints cited above.

## 8. Observability

- Logging: `tracing_subscriber` JSON logs with EnvFilter in bvi-api (`main.rs:10-16`); notable events `bvi-api starting`, `bvi-api HTTP listening`. Elixir side uses `Logger.warning/info` for lookup failures, dispatches, objection rejections.
- Metrics: none in either app (no Prometheus/Telemetry exporters found).
- Health: bvv_web GET `/health` JSON; bvi-api has no dedicated health endpoint (README mentions `/api/v1/bvi/health` returning 503 until register cached — not implemented).
- Tracing/OpenTelemetry: not present.
