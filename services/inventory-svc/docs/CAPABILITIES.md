# inventory-svc capabilities and algorithms

Comprehensive capability catalogue for the inventory, ballot printing, and diaspora logistics
service. Every capability lists the algorithm, source reference (paper, standard, or
statutory authority), and the crate or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end custody pipelines
3. Event-sourced custody ledger (M20)
4. RFID and GPS ingest (M20)
5. Seals, consumables, and reserve pools (M20)
6. Geographic RBAC via SpiceDB (M20)
7. Merkle root publication (M20)
8. Ballot printing chain (M39)
9. Diaspora ballot logistics (M65)
10. Fleet mobile custody app
11. Operational excellence
12. Standards and interoperability
13. Validation harnesses
14. Data lifecycle and security
15. API surface
16. Summary

---

## 1. Modules and capabilities overview

| Module | Slug | Size | Primary crate | Notable risk |
|---|---|---|---|---|
| M20 | inventory-management | XL | crates/custody | OEM/logistics APIs; RFID lead time |
| M39 | ballot-printing-chain | L | crates/printing | printer MES API; lot certification |
| M65 | diaspora-ballot-physical-logistics | L | crates/diaspora-logistics | embassy customs; airline embargo |

Tier: T2. Availability SLO: 99.9%. RFID scan-to-record latency < 500 ms. Batch kit reconciliation < 5 min per 1000 kits.

RTO: 2 hours. RPO: 30 min.

---

## 2. End-to-end custody pipelines

### 2.1 Kit dispatch pipeline (M20)

```
Warehouse dispatch order created (admin or API)
  -> Asset state: Warehoused -> Dispatched
  -> CustodyEvent kit_dispatched: from_custodian=warehouse, to_custodian=transporter
  -> HSM signs event payload (Ed25519, PKCS#11)
  -> eel.inventory.kit_dispatched.v1 emitted
  -> RFID scan at destination: kit_received event
  -> Asset state: Dispatched -> AtStation
```

### 2.2 Ballot printing pipeline (M39)

```
PrintOrder created (printer MES integration)
  -> MES double (adapter in crates/printing/src/mes.rs): pull lot data from printer
  -> LotData.lot_certified event when printer certifies lot
  -> RFID tags applied to each ballot booklet; seal_applied event per seal
  -> eel.inventory.ballot_lot_certified.v1 emitted
  -> Ballot lot dispatched to stations via custody pipeline
```

### 2.3 Diaspora pouch pipeline (M65)

```
BallotContainer created for diaspora station
  -> PreClearanceManifest generated (airline + customs fields)
  -> crates/diaspora-logistics/src/manifest.rs: KRA e-Clearance API call
  -> pouch_dispatched event; airline tracking number recorded
  -> customs_cleared event on embassy receipt confirmation
  -> Embassy delivery: pouch_received event + HSM signature by embassy officer
```

---

## 3. Event-sourced custody ledger (M20)

### 3.1 Event sourcing

Crate: `crates/custody/src/ledger.rs`.

Every custody change is an immutable append-only event row. No UPDATE on asset state: state derived by replaying event stream. Events:

| Event type | Trigger | Payload |
|---|---|---|
| kit_dispatched | Kit shipped from warehouse | asset_id, from, to, dispatch_time, vehicle_id |
| kit_received | RFID scan at destination | asset_id, received_by, received_at, scan_method (rfid/manual) |
| seal_applied | Seal affixed to asset | asset_id, seal_id, applied_by, applied_at |
| seal_broken | Seal removed or tampered | asset_id, seal_id, broken_by, broken_at, reason |
| kit_returned | Kit returned post-election | asset_id, returned_by, returned_at |
| asset_condemned | Asset written off | asset_id, condemnation_reason, condemned_by |

### 3.2 HSM-signed events

Every event signed by the custodian's HSM key (PKCS#11, Ed25519). Signature over `SHA-256(event_cbor)`. Stored in `eel_custody_events`: event_id, asset_id, event_type, payload JSONB, custodian_id, hsm_signature, prev_event_hash.

`prev_event_hash`: SHA-256 of the previous event for the same asset_id, forming a per-asset hash chain. Tampered or reordered events are detectable by chain verification.

| Algorithm | Reference |
|---|---|
| Ed25519 HSM signing | Bernstein et al. IACR ePrint 2011 |
| SHA-256 per-asset hash chain | FIPS 180-4, NIST 2015 |
| PKCS#11 HSM interface | OASIS PKCS#11 v3.0 2020 |
| CBOR serialization | RFC 7049 (Bormann, Hoffman 2013) |
| Append-only event sourcing | Fowler "Event Sourcing" pattern 2005 |

### 3.3 Schema (migrations/V001__custody_core.sql)

`eel_assets`: asset_id (UUID7), asset_type, serial_number, batch_id, current_state (enum), registered_at.
`eel_custody_events`: event_id (UUID7), asset_id, event_type, payload JSONB, custodian_id, hsm_signature, prev_event_hash, recorded_at.
`eel_asset_current_state`: materialized read-model (one row per asset_id; state, custodian, location); rebuilt by replaying events.

---

## 4. RFID and GPS ingest (M20)

### 4.1 RFID scan handler

Crate: `crates/custody/src/rfid.rs`.

Scan arrives via mobile app (Flutter) or fixed RFID reader webhook. Handler:
1. Decode EPC Gen2 tag (ISO 18000-6C).
2. Verify tag authenticity: HMAC-SHA256 tag signature against registered tag key in `eel_rfid_keys`.
3. Lookup asset_id by tag EPC code.
4. Emit `kit_received` or `kit_dispatched` custody event as appropriate.
5. Respond within 500 ms (SLA target).

### 4.2 GPS ingest

GPS coordinates attached to each mobile-app custody event. Stored as PostGIS POINT in `eel_custody_events.location`. Cross-checked against registered station boundary (from boundary-svc GeoJSON) using PostGIS `ST_Within`. Mismatch flagged as a custody anomaly to ECC.

| Algorithm | Reference |
|---|---|
| EPC Gen2 RFID | ISO 18000-6C (GS1 EPC standard) |
| HMAC-SHA256 tag authentication | RFC 2104 (Krawczyk, Bellare, Canetti 1997) |
| PostGIS ST_Within | PostGIS spatial extension; GEOS library |

---

## 5. Seals, consumables, and reserve pools (M20)

### 5.1 Seals

`eel_seals`: seal_id, asset_id (nullable; unassigned seals), seal_type (tamper-evident/security), applied_at, applied_by, broken_at, broken_by. Seal application and breakage recorded as custody events (seal_applied, seal_broken) with HSM signature. Unbroken seal on returned kit is a reconciliation gate; broken seal triggers escalation.

### 5.2 Consumables

`eel_consumables`: consumable_id, consumable_type (ink cartridge, battery, thermal paper), batch_id, quantity, unit, station_id, updated_at. Issued via `consumable_issued` event. Nightly reconciliation checks consumable_quantity vs expected_usage from BVI identification counts.

### 5.3 Reserve pools

`eel_reserve_pools`: pool_id, asset_type, county_id, quantity_available, quantity_reserved. Reserve allocation triggered by ECC dispatch command; decremented via `reserve_allocated` event. Reserve pools sized to 10 % of deployed count per county per pre-election planning cycle.

---

## 6. Geographic RBAC via SpiceDB (M20)

Crate: `crates/custody/src/authz.rs`.

Every custody API request evaluated against SpiceDB before handler executes:

```
check(principal, relation, resource)
```

Resource hierarchy: `Nation -> Region -> County -> Constituency -> Station`. County logistics coordinator can read and write custody events for assets within their county only. National warehouse officer can read all; write all in warehoused state. SpiceDB schema in `eel_spicedb_schema.zed`.

Relation types: `can_dispatch`, `can_receive`, `can_seal`, `can_condemn`, `can_read_custody_chain`.

---

## 7. Merkle root publication (M20)

### 7.1 Rolling Merkle root

Crate: `crates/custody/src/merkle.rs`.

Every 10 minutes, a Merkle root is computed over the set of all `eel_custody_events` committed in the preceding 10-minute window:
- Leaf: `SHA-256(event_id || asset_id || event_type || hsm_signature)`.
- Binary Merkle tree (RFC 6962 Section 2 format).
- Root signed by transparency-svc HSM (key role `inventory_anchor`).

### 7.2 Five-channel publication

Root published simultaneously to 5 channels to maximize tamper-evidence:
1. transparency-svc Rekor log entry.
2. `eel.inventory.merkle_root_published.v1` CloudEvent (Kafka outbox).
3. IPFS (I13 integration; CID stored in `eel_merkle_roots.ipfs_cid`).
4. `portal-svc` inventory anchor endpoint (read by IAB dashboard).
5. KBC SFTP drop (I14 satellite backup where IPFS unavailable).

`eel_merkle_roots`: root_id, window_start, window_end, leaf_count, root_hash, publication_channels JSONB, published_at, transparency_signature.

| Algorithm | Reference |
|---|---|
| Binary Merkle tree | RFC 6962 (Laurie, Langley, Kasper 2013) |
| SHA-256 leaf hash | FIPS 180-4, NIST 2015 |

---

## 8. Ballot printing chain (M39)

### 8.1 Printer MES integration

Crate: `crates/printing/src/mes.rs`. MES double (test double for the printer MES API).

Production path: REST adapter to printer vendor MES (Manufacturing Execution System). Pulls lot data: lot_id, quantity, ballot_style, print_date, paper_batch, ink_batch, operator_id. Certification: printer operator signs `LotCertification` struct via HSM; stored in `eel_ballot_lots`.

### 8.2 Schema (migrations/V002__printing_chain.sql)

`eel_print_orders`: order_id (UUID7), election_id, ballot_style, quantity, ordered_at, ordered_by.
`eel_ballot_lots`: lot_id (UUID7), order_id, quantity, certified_at, certification_hsm_signature, paper_batch, ink_batch.
`eel_ballot_rfid_tags`: tag_id, lot_id, epc_code, applied_at, station_assignment.
`eel_station_ballot_reconciliation`: station_id, election_id, issued_count, used_count, spoilt_count, returned_count, discrepancy.

### 8.3 Station ballot reconciliation

Post-election: `issued_count` from `eel_ballot_rfid_tags` vs `used_count + spoilt_count + returned_count` from rts-svc Form34 counts. Discrepancy > 0 triggers audit escalation to governance-svc.

---

## 9. Diaspora ballot logistics (M65)

### 9.1 Pre-clearance manifests

Crate: `crates/diaspora-logistics/src/manifest.rs`.

`PreClearanceManifest`: container_id, embassy_station_id, ballot_count, ballot_style, airline, flight_number, departure_date, destination_country, customs_declaration_ref, kra_clearance_ref.

KRA e-Clearance API call (`I12 KRA`): submits manifest and receives clearance_ref. Manifest signed by HSM before submission for non-repudiation.

### 9.2 Custody event chain

Diaspora pouch custody follows same event-sourced pattern as domestic kits. Events: `pouch_created`, `pouch_dispatched`, `pouch_customs_cleared`, `pouch_received` (embassy officer HSM signature), `pouch_opened` (seal_broken equivalent at station opening).

### 9.3 Schema (migrations/V003__diaspora_logistics.sql)

`eel_ballot_containers`: container_id (UUID7), embassy_station_id, ballot_count, seal_id, state.
`eel_pre_clearance_manifests`: manifest_id, container_id, airline, kra_clearance_ref, customs_status, submitted_at.
`eel_diaspora_custody_events`: event_id, container_id, event_type, custodian_id, hsm_signature, payload JSONB, recorded_at.

---

## 10. Fleet mobile custody app

Component: `apps/` (Flutter). Deployed to Android (MDM-enrolled custody staff).

- Offline mode: custody events queued locally when connectivity unavailable; sync on reconnect.
- RFID scan via NFC reader or USB RFID bridge.
- GPS location attached automatically to each scan event.
- Barcode/QR fallback when RFID unavailable.
- HSM: software HSM (Android Keystore) used for mobile custody officer signatures in offline mode; replaced by hardware HSM signature on reconnect and event flush.
- MDM enrollment required: device registered in `eel_mdm_devices` with SPIRE SVID.

### 10.1 MDM enrollment workflow

Devices provisioned via MDM platform (Android Enterprise zero-touch enrollment):
1. Device receives MDM configuration profile including app package, SpiceDB role `custody_officer`, and county RBAC binding.
2. App installs; prompts officer to complete biometric enrollment (Android Keystore key generation bound to biometric + device unlock).
3. SPIRE SVID issued to device identity (`spiffe://eel.local/custody/<county_id>/<device_id>`).
4. Device registered in `eel_mdm_devices`: mdm_enrollment_id, device_id, county_id, officer_id, enrolled_at, spire_svid.

### 10.2 Offline sync conflict resolution

Offline custody events are timestamped with device clock (ISO 8601 with UTC offset). On reconnect, sync worker compares offline event `recorded_at` against server `received_at`. Conflict detection: if two custody events for the same asset_id arrive within 60 s of each other from different devices, a `custody_conflict` event is created and surfaced to the county logistics coordinator for manual resolution. Conflict resolution recorded as an additional custody event with `conflict_resolved` type and resolving officer's HSM signature.

---

## 11. Operational excellence

| Capability | Mechanism |
|---|---|
| RFID reader failure | Mobile manual scan fallback; audit note on event |
| HSM unavailability | Offline signature queue in `eel_pending_signatures`; re-signed on HSM recovery |
| Kit dispatch backlog | Prometheus alert when dispatch_queue > 1000; warehouse workflow review |
| Diaspora customs hold | Alert to inventory ops at 48 h; escalation to MFA liaison at 72 h |
| Reconciliation discrepancy | Automated diff report; discrepancy > 0 auto-escalated to governance-svc |
| k6 load test | `loadtest/inventory_load.js` targets 200 VUs / 300 s; RFID scan + custody event write |
| HPA | CPU 70 %, memory 80 %; min 2, max 12 replicas |
| Django admin | Bulk import of asset manifests; offline reconciliation reports |
| IPFS channel failure | Merkle root still published to 4 of 5 channels; IPFS CID field left null; alert raised; retry on IPFS recovery |
| GPS mismatch alert | custody-api emits `eel.inventory.custody_gps_mismatch.v1` to ecc-svc when RFID scan location disagrees with registered station boundary |
| Asset state replay | `GET /v1/inventory/assets/{asset_id}/custody-chain?rebuild=true` triggers on-demand event replay to recompute `eel_asset_current_state`; used after incident recovery |

### 11.1 Pre-election reconciliation run

Seven days before election day, an automated reconciliation job runs against all deployed kits:

1. Query `eel_asset_current_state` for all kits with state `AtStation`.
2. For each kit: verify RFID scan received within the last 24 h (confirming kit is physically present and reader operational).
3. Kits with no recent scan: flagged as `scan_overdue`; alert sent to county logistics coordinator.
4. Seals: verify `eel_seals` record shows seal_status = `APPLIED` for all AtStation kits.
5. Consumable levels: verify `eel_consumables.quantity` meets minimum thresholds per station.
6. Report: `eel_pre_election_reconciliation_reports` table; report_id, election_id, run_at, kits_verified, kits_overdue, seals_intact, seals_broken, consumables_ok, consumables_low, discrepancy_count.

---

## 12. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| ISO 18000-6C (EPC Gen2 RFID) | RFID tag read and authentication | crates/custody/src/rfid.rs |
| RFC 2104 HMAC-SHA256 | RFID tag authentication | crates/custody/src/rfid.rs |
| OASIS PKCS#11 v3.0 | HSM interface for all signing operations | crates/custody/src/hsm.rs |
| RFC 7049 CBOR | Event payload serialization before HSM signing | crates/custody |
| RFC 6962 | Merkle tree and inclusion proof format | crates/custody/src/merkle.rs |
| CloudEvents 1.0 | All event envelopes | crates/custody, crates/printing |
| PostGIS / OGC | GPS location geospatial queries | crates/custody/src/rfid.rs |
| OpenAPI 3.1 | API contract at packages/openapi-contracts/inventory/ | crates/custody |

---

## 13. Validation harnesses

| Harness | Scope | Command |
|---|---|---|
| HSM signature tests | Event signing + chain verification | `cargo test custody::hsm_signature --lib` |
| Custody event replay | Full state reconstruction from event stream | `cargo test ledger::replay --lib` |
| Merkle root tests | Leaf hash, root computation, 5-channel publication | `cargo test merkle:: --lib` |
| RFID tag authentication | HMAC verification, EPC decode | `cargo test rfid:: --lib` |
| Reconciliation tests | Discrepancy detection, audit escalation | `cargo test reconciliation:: --lib` |
| Integration tests | Event sourcing + snapshot recovery | `cargo nextest run` |
| k6 load test | RFID scans + custody events, 1000 assets | `cd loadtest && k6 run inventory_load.js` |

---

## 14. Data lifecycle and security

### 14.1 Encryption

Custody event payload: no PII (asset-centric; voter data not stored here). HSM signatures provide non-repudiation. Django admin sessions: TLS required; no plaintext passwords.

### 14.2 Audit

`eel_custody_events` append-only (INSERT only trigger; no UPDATE or DELETE). Per-asset hash chain detects tampering. Merkle root every 10 min anchored to transparency-svc provides external tamper-evidence.

### 14.3 Retention

- `eel_custody_events`: permanent (election archive; physical asset audit trail).
- `eel_ballot_lots`: permanent (election integrity).
- `eel_pre_clearance_manifests`: 10 years (customs compliance).
- `eel_station_ballot_reconciliation`: permanent.
- `eel_consumables`: 3 years post-election.
- `eel_reserve_pools`: updated in-place; no historical retention required.

### 14.4 HSM dependency

Primary site: hardware HSM (PKCS#11). Mobile app: Android Keystore software HSM in offline mode; signature re-issued from hardware HSM on reconnect. Edge clusters: event relay only; no signing authority.

---

## 15. API surface

### 15.1 REST (`crates/custody`, axum 0.8, port 8080)

- `GET /v1/inventory/assets/{asset_id}/custody-chain`: full event-sourced custody chain.
- `POST /v1/inventory/custody-events`: record custody event (RFID scan or manual).
- `GET /v1/inventory/kits/{serial}`: kit metadata and current state.
- `GET /v1/inventory/ballot-lots/{lot_id}`: lot certification and receipt log.
- `GET /v1/inventory/seals/{seal_id}`: seal inventory and assignment.
- `GET /v1/inventory/reserve-pools/{county_id}`: reserve pool status.
- `POST /v1/inventory/diaspora-pouches/{pouch_id}/manifest`: update pouch manifest.
- `GET /v1/inventory/reconciliation`: batch reconciliation report.
- `GET /v1/inventory/merkle-roots`: recent Merkle roots with publication status.
- `GET /healthz` + `/readyz` + `/openapi.json`.

All authenticated routes require SPIRE SVID. Geographic RBAC via SpiceDB `check` on every write.

### 15.2 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.rts.form34_submitted.v1` (reconciliation trigger)
- `eel.biometric_edge.kit_attested.v1` (kit registration)

Emitted:
- `eel.inventory.custody_transferred.v1`
- `eel.inventory.kit_dispatched.v1`, `eel.inventory.asset_received.v1`
- `eel.inventory.ballot_lot_certified.v1`
- `eel.inventory.diaspora_pouch_dispatched.v1`
- `eel.inventory.merkle_root_published.v1`

### 15.3 Django admin (port 8000)

Bulk asset manifest import; reconciliation report export (CSV, PDF); reserve pool management.

---

## 16. Summary

Total footprint: 3 Rust crates (custody, printing, diaspora-logistics) + 1 Flutter mobile app + 1 Django admin + 3 PostgreSQL migrations (V001-V003). Covers M20 (event-sourced custody ledger with HSM-signed events, RFID ingest, geographic SpiceDB RBAC, and 10-min Merkle root across 5 publication channels), M39 (ballot printing chain with MES double and RFID tagging), and M65 (diaspora pouch logistics with KRA e-Clearance pre-clearance manifests). Key production inputs remaining: printer MES vendor API credentials; KRA e-Clearance API MoU; airline tracking API access for each diaspora route; hardware HSM key provisioning for `inventory_anchor` role.

### 16.1 Inter-service dependencies

| Upstream service | Data consumed | Failure mode |
|---|---|---|
| rts-svc | `eel.rts.form34_submitted.v1` (M39 post-election ballot reconciliation trigger) | Reconciliation deferred; batch retried on rts-svc recovery |
| biometric-edge-svc | `eel.biometric_edge.kit_attested.v1` (M20 kit registration) | Kit not registered; RFID scans for unregistered kit logged with `unregistered` status and quarantined |
| boundary-svc | PostGIS constituency boundaries for GPS location cross-check | GPS cross-check degraded; anomalies not flagged; alert to ops |
| transparency-svc | Merkle root anchor (M20 5-channel publication) | Root anchored to 4 of 5 channels; IPFS/Rekor channel retried |
| ecc-svc | Dispatch commands for reserve allocation | Reserve allocation waits for command ack; no autonomous reserve release |
| governance-svc | Reconciliation discrepancy escalation (M39) | Discrepancies queued locally; retried on governance-svc recovery |
