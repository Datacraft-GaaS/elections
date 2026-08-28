# inventory-svc

## 1. Overview

Election asset lifecycle service: chain-of-custody for kits, ballots, seals and consumables (M20), the ballot printing chain (M39), diaspora ballot physical logistics (M65), plus RFID ballot-box cover custody, a Django admin console, an offline Flutter custody app, and a Python RFID reader driver.

Runtimes: Rust library crates under crates/ (no binary target exists; `src/` contains only `.gitkeep` - the axum routers in `rfid-cover-custody::api` and `physical-seal-binding::api` must be nested by an out-of-repo binary); Django 5 admin app at django/inventory-admin; Flutter app apps/custody-mobile; FastAPI driver py/rfid-reader-driver. No port is bound anywhere in code; the reader driver defaults its upstream to http://localhost:8000 (py/rfid-reader-driver/src/eel_rfid_reader/config.py:11).

## 2. Ownership & Boundaries

Ground truth: docs/architecture/services/inventory.md and services/inventory-svc/TRACEABILITY.yaml (WP-4.3).

| M-ID | Module | Crate | Notes |
|---|---|---|---|
| M20 | inventory-management | crates/custody | Event-sourced custody; ADR-0013; availability class cycle-critical-99.9 |
| M39 | ballot-printing-chain | crates/printing | MES double pattern |
| M65 | diaspora-ballot-physical-logistics | crates/diaspora-logistics | MFA/KRA/airline doubles |

Additional code not assigned an M-ID in TRACEABILITY.yaml: crates/rfid-cover-custody (RFID ballot-box covers, ref docs/features/rfid-ballot-box-covers.md, ADR-0016 data class C2) and crates/physical-seal-binding (HARD-16 seal-photo binding). Django inventory-admin covers kit fleet, reserve pool, ballot lots, rfid covers surfaces.

Data owned: tables from migrations V001/V002/V003/V051 (see section 5). Upstream: boundary-svc (geofences), external MES/KRA/airlines/customs. Downstream: biometric-edge-svc, bvi-svc, rts-svc, ecc-svc, transparency-svc (Merkle roots). Events emitted/consumed per TRACEABILITY.yaml: 14 `eel.custody.*`/`eel.inventory.*`/`eel.printing.*` types emitted via transactional outbox; consumes `eel.bvi.kit_revoked.v1`, `eel.diaspora.embassy_station_registered.v1`. In code, event type constants exist in custody/src/events/mod.rs:4-8, printing/src/events.rs:3-7, diaspora-logistics/src/events.rs:4-7; the only actual publish path implemented is the custody Merkle-root outbox write (custody/src/merkle.rs:39-88).

## 3. Repository Layout

```
services/inventory-svc/
  TRACEABILITY.yaml, README.md
  crates/
    custody/               src/domain/mod.rs   # Asset, CustodyEvent, Seal, Consumable, ReservePool
                           src/events/mod.rs   # kit_dispatched..reserve_pool_updated constants + payloads
                           src/geo_rbac.rs     # GeographicScope hierarchy + check_access
                           src/merkle.rs       # MerklePublisher -> custody_outbox
                           src/rfid.rs         # KitHeartbeat -> CustodyEvent conversion
    printing/              src/domain.rs       # PrintOrder, LotData, PrintCustodyEvent, StationReconciliation
                           src/events.rs       # variance/rfid-mismatch/reconciliation alert types
    diaspora-logistics/    src/domain.rs       # BallotContainer, PreClearanceManifest, HubRoute, ArrivalRecord
                           src/events.rs       # container dispatched/received/seal_broken/reconciliation
                           src/integrations/   # airline/kra/mfa test doubles
    rfid-cover-custody/    src/api.rs          # 7-route axum router
                           src/{tag,state_machine,read_ingest,replay_defence,tamper_alert,
                                hash_chain,repository,reader,metrics,events,domain,error}.rs
                           tests/rfid_test.rs
    physical-seal-binding/ src/{api,bind,verify,seal_photo,error}.rs
                           tests/seal_test.rs
  django/inventory-admin/
    inventory_admin/settings.py, urls.py
    inventory_admin/apps/{kit_fleet,reserve_pool,ballot_lots,rfid_covers}/
    inventory_admin/apps/rfid_covers/{views.py,urls.py,templates/,tests/}
  apps/custody-mobile/     lib/src/{app.dart,screens/rfid_scan.dart,services/rfid_reader_service.dart}
  py/rfid-reader-driver/   src/eel_rfid_reader/{main,config,upstream,hmac_gen2v2,
                             adapter,models}.py, adapters/{synthetic,impinj_llrp,
                             zebra_llrp,cipherlab_rs36,llrp_common}.py, tests/
  migrations/              V001__custody_core.sql, V002__printing_chain.sql,
                           V003__diaspora_logistics.sql, V051__rfid_ballot_covers.sql
  deploy/service.yaml
  docs/CAPABILITIES.md, docs/runbooks/*.md, data_dictionary.yaml
```

## 4. Capabilities & Functions

### 4.1 RFID cover custody REST API - crates/rfid-cover-custody/src/api.rs:328-338 (`router`)

| Route | Function | What it does | Citation |
|---|---|---|---|
| POST `/v1/rfid/tags` | `post_issue_tag` | Issues a cover tag (manufacturer, lot, chip family, HSM key id); persists via repository. | api.rs:133-161; tag.rs:53 `issue_tag` |
| POST `/v1/rfid/tags/{id}/pair` | `post_pair_tag` | Binds tag to a ballot-box body serial (validates hex EPC, lot pairing). | api.rs:163-205; tag.rs:102 `pair_to_box` |
| POST `/v1/rfid/reads` | `post_ingest_read` | Stub: returns 202 accepted without processing; pure logic tested separately in rfid_test.rs. | api.rs:207-215 |
| POST `/v1/rfid/tags/{id}/transition` | `post_transition` | Officer-triggered FSM step: validates transition against state machine, appends BLAKE3-hash-chained transition row, increments metrics. | api.rs:217-290; hash_chain.rs:12 `compute_this_hash` |
| GET `/v1/rfid/tags/{id}/history` | `get_tag_history` | Full hash-chained transition list. | api.rs:292-300 |
| GET `/v1/rfid/tamper-alerts` | `get_tamper_alerts` | Pending tamper alerts. | api.rs:302-310 |
| PATCH `/v1/rfid/tamper-alerts/{id}/resolve` | `patch_resolve_alert` | Resolves an alert with a note. | api.rs:312-321 |

Cover lifecycle FSM (`CoverStateMachine::allowed_transitions`, state_machine.rs:32-115): Manufactured -> Printer -> Warehouse -> Dispatched -> InTransit -> StationReceived -> StationOpened -> VotingActive -> StationClosed -> SealedReturn -> TallyArrived -> CountComplete -> StorageArchive -> Destroyed, with QUARANTINE reachable from any state on tamper (validate(), lines 121-166). Four-eyes (dual-officer SVID) required for Dispatched, StationOpened, StationClosed, Destroyed transitions (four_eyes flags lines 50,68,80,110).

Read-ingest pipeline (pure logic, read_ingest.rs:57-143 `ingest_read_pure`): verifies reader SPIFFE SVID against registry (reader.rs `ReaderRegistry::verify_svid`), checks nonce replay window (replay_defence.rs:13 REPLAY_WINDOW = 300 s), verifies Gen2v2-style HMAC challenge/response, raises tamper alerts with severity classification (tamper_alert.rs:22 `classify_severity`) and async ECC webhook dispatch (tamper_alert.rs:58 `dispatch_to_ecc`).

### 4.2 Custody domain (M20)

- Asset/custody/seal/consumable/reserve-pool aggregates mirror migration V001 (domain/mod.rs:33-98); `ReservePool::is_below_threshold` drives low-stock alerts (line 96).
- Geographic RBAC: `check_access(actor_scope, station)` over National/County/Constituency/Station prefix rules (geo_rbac.rs:20-27).
- Merkle publication: `MerklePublisher::publish_root` writes a CloudEvents envelope to the transactional outbox on topic `eel.custody.merkle_root_published.v1`; transparency-svc performs the five-channel fan-out (merkle.rs:19,39-88).
- Kit heartbeats: `process_heartbeat` converts RFID/GPS heartbeat into a custody event with empty signature pending HSM countersigning (rfid.rs:21+).
- Seal break requires two officers in the payload contract (`SealBroken.broken_by_officer_1/2`, events/mod.rs:31-38).

### 4.3 Printing chain (M39)

PrintOrder lifecycle enum Pending..Reconciled (domain.rs:5-15) with dual-officer fields `issued_by_officer_id` / `countersigned_by_commissioner_id` (lines 27-28); LotData carries serial ranges, printed/rejected counts, rfid chip-list hash and variance bookkeeping (lines 34-48); `discrepancy()` computes received - issued - unused - spoilt for StationReconciliation (lines 90-95). Alert event constants: quantity_variance, rfid_mismatch, reconciliation discrepancy, custody_event_recorded (events.rs:3-7). MES integration is a test double only (integrations/printer_mes_double.rs).

### 4.4 Diaspora logistics (M65)

BallotContainer status machine Packed->CustomsCleared->InTransit->Received->Reconciled (+Quarantined) and ContainerCustodyEvent chain sorted by `custody_chain()` (domain.rs:5-93); PreClearanceManifest with mfa_ref/kra_ref/airline_awb (lines 52-61); HubRoute routing policies diplomatic pouch/airline cargo/charter (lines 63-78); ArrivalRecord expected-vs-received discrepancy (lines 80-88). External integrations are test doubles (integrations/{airline,kra,mfa}_double.rs). Event constants: container_dispatched/received/seal_broken/reconciliation_complete (events.rs:4-7).

### 4.5 Physical seal binding (HARD-16)

Documented endpoints `POST /v1/custody/bind-seal` and `GET /v1/custody/{id}/seal-verification` exist as handler functions only, not wired into any router (api.rs header comment lines 1-6; `handle_bind_seal`:28). Binds an HSM signature over SHA-256 of seal photo bytes to a custody event; verification replays the binding (bind.rs, verify.rs).

### 4.6 Django inventory-admin

URLs (apps/rfid_covers/urls.py:7-11; root urls.py includes at project root):

| Route | View | Auth | Citation |
|---|---|---|---|
| `/admin/` | Django admin | staff | urls.py:5 |
| `rfid/box-chain/` | `box_chain_view` | login + `view_coverstatetransition` permission; replays prev_hash chain per transition for display | views.py:49-82 |
| `rfid/tamper-triage/` | `tamper_triage_view` + `_resolve_alert` (POST) | login + view/change permissions on CoverTamperAlert | views.py:90-128 |
| `rfid/reader-health/` | `reader_health_view` | login + `view_rfidcoverread`; aggregates last-read/tag counts per reader | views.py:136-156 |

Django admin registration for kit fleet, reserve pool, ballot lots, rfid covers models (apps/*/admin.py).

### 4.7 Custody-mobile Flutter app

Offline-oriented RFID/GPS custody scan screen (screens/rfid_scan.dart) behind an abstract `RfidReaderService` interface (services/rfid_reader_service.dart:61-81) with a `SyntheticReaderService` implementation (line 85). Tests: test/rfid_reader_service_test.dart, test/rfid_scan_test.dart. No persistence/sync implementation found in-repo (TRACEABILITY AC-M20-06 describes a 100k-event offline queue - not present).

### 4.8 RFID reader driver - py/rfid-reader-driver (FastAPI)

| Route | Function | Behaviour | Citation |
|---|---|---|---|
| POST `/reader/register` | `register_reader` | Registers reader SVID/metadata in memory. | main.py:101-124 |
| POST `/reader/{reader_id}/read` | `submit_read` | Single-tag read; forwards upstream. | main.py:126-160 |
| POST `/reader/{reader_id}/batch-read` | `batch_read` | Burst up to 1000 reads. | main.py:162-200 |
| POST `/reader/{reader_id}/challenge` | `issue_challenge` | Gen2v2 HMAC challenge for a tag EPC via adapter. | main.py:202-222 |
| GET `/reader/{reader_id}/status` | `reader_status` | Health, last-read ts, tag count, adapter type. | main.py:224-245 |
| GET `/health` | `health` | Status + adapter name. | main.py:247-250 |

Adapters selected by `EEL_RFID_ADAPTER`: synthetic (default), impinj (LLRP), zebra (LLRP), cipherlab RS36 (Bluetooth address) (main.rs build_adapter equivalent: main.py:46-73). Upstream posts to `EEL_RFID_UPSTREAM_URL`.

## 5. Data Model

| Migration | Tables | Purpose |
|---|---|---|
| V001__custody_core.sql | `assets` (type/status CHECKs), `custody_events` (from/to custodian, HLC ts, signature, seq), `seals` (affix/break audit, signature), `consumables` (lot qty/thresholds), `reserve_pools` (scope/count/alert threshold), `merkle_checkpoints` (10-min windows), `kit_heartbeats` (GPS/battery), `assets_audit`, `custody_outbox` | Append-only custody ledger; partial index for unpublished outbox rows. |
| V002__printing_chain.sql | `print_orders`, `lot_data`, `print_custody_events`, `station_reconciliations`, `print_orders_audit`, `printing_outbox` | Dual-signed print orders through station reconciliation with variance alerts. |
| V003__diaspora_logistics.sql | `ballot_containers`, `container_custody_events`, `preclearance_manifests`, `hub_routes`, `arrival_records`, `ballot_containers_audit`, `diaspora_logistics_outbox` | Diaspora container logistics and clearance. |
| V051__rfid_ballot_covers.sql | `eel_rfid_cover_tags`, `eel_rfid_cover_reads` (HMAC challenge/response, geo, reader signature), `eel_cover_state_transitions` (prev_hash/this_hash chain, unique tag+hash index), `eel_cover_tamper_alerts` (severity, ECC dispatch state) | Tamper-evident cover custody; append-only enforced by triggers `eel_rfid_append_only_guard` blocking UPDATE/DELETE on transitions and alerts (lines 117-141). |

Stores: sqlx PgPool (rfid-cover-custody AppState api.rs:32-34; merkle publisher), Django ORM (sqlite fallback or DATABASE_URL), reader-driver in-memory dicts. Events produced: see section 2; consumed: none implemented in code.

## 6. Operation

Build/test:

```bash
cd services/inventory-svc
cargo test --workspace                        # crate tests incl. rfid_test.rs, seal_test.rs
cd django/inventory-admin && pytest           # rfid_covers view/admin tests
cd apps/custody-mobile && flutter test
cd py/rfid-reader-driver && pytest tests      # uv-managed venv
```

Run: no service binary exists in-repo; Django admin via `python manage.py runserver`; reader driver via uvicorn (`uvicorn eel_rfid_reader.main:app`). Ports are not fixed in code for any component.

Env vars (reader driver, config.py:11-19): `EEL_RFID_UPSTREAM_URL` (default http://localhost:8000), `EEL_RFID_ADAPTER` (synthetic default), `EEL_RFID_HSM_KEY_ID` (dev-key-1), `EEL_RFID_SYNTHETIC_TAMPER_PROB`, `EEL_RFID_SVID_HEADER` (X-Spiffe-Svid), plus per-hardware `EEL_IMPINJ_HOST`, `EEL_ZEBRA_HOST`, `EEL_CIPHERLAB_ADDR`. Django settings: `SECRET_KEY`, `DEBUG`, `ALLOWED_HOSTS`, `DATABASE_URL` (settings.py:6-8,54-68; DEBUG defaults true).

Deploy: deploy/service.yaml - image harbor.eel.internal/eel/inventory-svc, blue-green primary/mirror, rolling edges. Only one manifest; nothing deploys the Django admin, mobile app, or driver.

Degradation/offline: documented modes in docs/runbooks/inventory-svc-degradation.md; the offline mobile queue described in TRACEABILITY acceptance criteria is not implemented in code.

## 7. Safeguards & Security

Auth posture per surface:

- RFID REST API: no authentication middleware on the router itself (api.rs:328-338). Reader identity is verified inside the ingest pipeline by SPIFFE SVID validation against a reader registry (read_ingest.rs:63; reader.rs), and HMAC challenge failures return UNAUTHORIZED-class errors (api.rs error mapping lines 112-115) - but the wired `/reads` endpoint bypasses all of this as a stub.
- FSM transitions: four-eyes dual-officer SVID requirement enforced for sensitive states (state_machine.rs:158-163); terminal states locked except QUARANTINE (lines 127-135).
- Django admin: session auth + per-model permissions with raise_exception on every view (views.py:49-51,91,113,137); CSRF middleware enabled (settings.py:26).
- Tag secrets referenced by HSM key id rather than material (api.rs DTO line 45; V051 column tag_secret_hsm_key_id).

Idempotency/integrity:

- BLAKE3 hash chain over transitions; silent rewrites break the chain (hash_chain.rs:12,37; V051 comments lines 87-88); unique index prevents duplicate chain entries.
- Replay defence on reads: nonce dedup within 5-minute window per tag (replay_defence.rs:13,55-72); production Valkey SET NX EX adapter noted as pending.
- Custody events carry transferor/transferee signatures and HLC timestamps (V001 lines 18-29, FR-M20-007); seal breaks require dual officers.
- Append-only enforcement: DB trigger rejects UPDATE/DELETE on transitions and alerts (V051:117-141).

PII handling: domain contains asset/logistics identifiers, no citizen PII. Audit logging: `assets_audit` table with actor/purpose/action (V001:86-94); tracing logs throughout read_ingest (info/warn/error including HMAC failure warnings at read_ingest.rs:76-81).

Validation: strict DB CHECK constraints on states/severities/exit reasons; hex validation of EPCs at API boundary (api.rs:176-185); Pydantic models in the driver; error taxonomy maps domain errors to 409/422/401 statuses (api.rs:101-127).

Tests verifying safeguards: crates/rfid-cover-custody/tests/rfid_test.rs (FSM, ingest, replay, chain), physical-seal-binding/tests/seal_test.rs, custody/tests/custody_chain_test.rs, printing/tests/printing_test.rs, diaspora-logistics/tests/diaspora_test.rs, driver tests (test_hmac_gen2v2.py, test_upstream.py, test_adapters.py), Django tests (test_views.py, test_admin.py).

### Known deferrals (verified against code)

- `POST /v1/rfid/reads` accepts and discards reads (api.rs:207-215) - the full ingest path exists only as pure logic exercised by tests.
- Seal-binding routes are unwired handler functions (physical-seal-binding/src/api.rs:1-6 comment "wire these routes into the inventory-svc Axum router").
- No top-level binary/router composes the crates; `src/` is empty.
- Offline mobile custody queue (AC-M20-06) absent from apps/custody-mobile.
- Django settings lists `inventory_admin.apps.diaspora_shipments` in INSTALLED_APPS but that app directory does not exist - Django startup would fail with the committed tree.
- Chain verification mismatch: the Django box-chain view recomputes links as SHA-256 of the previous this_hash string (views.py:37) while the Rust writer uses BLAKE3 over the transition struct (hash_chain.rs:12) - the admin's chain_valid flag cannot reflect the real chain integrity as written.
- Merkle root publication is the only implemented event emission; the other twelve documented event types have no producer code.

## 8. Observability

Metrics: Prometheus counters registered for cover reads, tamper alerts (by severity), state transitions (from_to label), and HMAC failures (metrics.rs:12-56; incremented from read_ingest.rs and api.rs:283). Health: reader driver GET /health (main.py:247); Django has no health endpoint; no /health route in the RFID router. Logging: tracing structured logs in Rust (read_ingest.rs:75-86,113-119; tamper_alert.rs error-level on critical alerts); Python logging in the driver and Django defaults. ECC dispatch telemetry via webhook result logs (tamper_alert.rs:58-93). Runbooks: docs/runbooks/{inventory-svc-degradation,ballot-discrepancy-response,diaspora-container-seal-break}.md.
