# Inventory management (M20)

Module M20 (`inventory-management`) owned by `inventory-svc`. Tracks all election materials from central warehouse through county stores to polling stations and back, using an event-sourced custody chain backed by RFID + GPS multi-modal tracking. Every seal break is dual-officer HSM-signed. Merkle roots are published every 10 minutes to five channels and the Auditor-General receives read-only access under EOA 2016.

## Contents

1. What inventory management does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What inventory management does (vs related modules)

- **M31 Mobile poll clerk** = DPO companion app; it mirrors the kit status dashboard from M20 at the polling station level but does not write custody events.
- **M20** = full event-sourced custody chain from manufacturer to polling station and back to archive; every movement, seal break, and discrepancy is recorded and Merkle-anchored.
- **M33 Personnel dispatch** = tracks people, not materials; check-in events can reference an inventory unit for co-location verification.

The custody chain is append-only. No record is ever deleted. Soft corrections attach a signed `CORRECTION_EVENT` to the original event rather than mutating it.

## 2. Data plane

Five aggregates under `services/inventory-svc/crates/inventory-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **InventoryUnit** | Serialised item: unit_id (UUID7), type (BALLOT_BOX, BALLOT_PAPER, SEAL, BVR_KIT, etc.), batch_ref, current_holder_id, location_id, status. | B |
| **CustodyEvent** | Every transfer, seal break, or discrepancy: event_type, unit_id, from_holder, to_holder, location_id, rfid_tag, gps_lat, gps_lon, sig (dual HSM for seal events). | A |
| **MerkleRoot** | 10-minute interval anchor: root_hash over all CustodyEvent hashes since last root, ts_hlc, published_to (array of 5 channel refs), sig. | A |
| **Discrepancy** | Missing or damaged item: raised_by, status (OPEN / RESOLVED / ESCALATED), resolution_note, closed_by. | B |
| **AuditAccess** | AG and IAB read-only access log: accessor, resource, ts_hlc. | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| RFID scanner (warehouse / county / polling station) | Real-time | BLE/USB serial -> adapter | Device cert (SVID) | Buffer locally; flush on reconnect; no event lost |
| GPS beacon (vehicle in transit) | 30 s | MQTT | Device cert | Last-known position retained; gap logged |
| Manual officer entry (Django back-office) | On demand | Form | JWT + SpiceDB | Requires dual-officer confirmation for seal events |
| M31 poll clerk kit mirror | At kit arrival and EOD | REST | JWT (M31 device cert) | Idempotent; duplicate events de-duped by unit_id + event_type + ts window |
| M32 field support diagnostics | On demand | REST | JWT (M32 device cert) | Append-only discrepancy events only |

## 4. Pipeline

```
RFID/GPS event (scanner or beacon)
  -> adapter normalises to CustodyEvent schema
  -> unit_id resolved against InventoryUnit
  -> event_type classifier:
        TRANSFER      -> from_holder, to_holder recorded
        SEAL_BREAK    -> dual-officer HSM required (two distinct YubiKey tokens);
                         rejects if <2 signers or same officer twice
        DISCREPANCY   -> Discrepancy record created; ops alert
  -> CustodyEvent persisted (append-only)
  -> eel.inventory.custody_event.v1 emitted via eel-outbox-rs
  -> every 10 min: MerkleRoot over all events since last root
       -> anchored to 5 channels:
            P05 transparency log (primary)
            Kenya National Archives REST endpoint
            IAB observer node
            County commissioner node
            AG read-only mirror
       -> MerkleRoot.published_to updated with anchor receipts
  -> Discrepancy unresolved for >2 h: escalation event to ops pager
```

All writes are transactional via `packages/eel-outbox-rs`. Debezium publishes to `eel.inventory.v1` topic.

## 5. API surface

Contract: `packages/openapi-contracts/inventory/custody.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /inventory/v1/units` | Warehouse officer (JWT + SpiceDB) | Register new inventory unit |
| `POST /inventory/v1/events` | Device / officer (JWT / device cert) | Record custody event |
| `GET /inventory/v1/units/{id}/history` | Officer / AG (JWT) | Full custody chain for a unit |
| `GET /inventory/v1/roots` | IAB / AG (JWT) | Merkle root feed |
| `GET /inventory/v1/roots/{id}/proof/{event_id}` | Court / AG | Merkle inclusion proof for an event |
| `GET /inventory/v1/discrepancies` | Ops (JWT + SpiceDB) | Open discrepancy feed |
| `PATCH /inventory/v1/discrepancies/{id}` | Ops officer | Resolve or escalate discrepancy |
| `GET /inventory/v1/units` | AG read-only (JWT, geographic RBAC) | Unit list filtered to AG's county scope |

Geographic RBAC: AG access is scoped to the county boundary polygon stored in `boundary-svc` (M18); cross-county access rejected with HTTP 403.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Warehouse back-office (Django) | `services/inventory-svc/django/inventory-admin/` | Batch registration, transfer, seal, discrepancy management |
| County ops dashboard (Phoenix LiveView) | `services/inventory-svc/ex/inventory_web/` | Real-time unit locations, Merkle root status, discrepancy queue |
| AG read-only portal (Astro) | `services/inventory-svc/astro/ag-portal/` | County-scoped read of unit history and Merkle roots; no write access |
| Read API (Rust axum) | `services/inventory-svc/crates/inventory-api/` | REST surface for M31, M32, and external consumers |

WCAG 2.2 AA on all surfaces. Languages: EN + SW.

## 7. Integrity + anti-tamper

- **Event-sourced append-only store** -- no UPDATE or DELETE on `custody_events`; corrections are new events with `CORRECTION_EVENT` type referencing the original event UUID7. PostgreSQL row-level triggers enforce this.
- **Dual HSM for seal breaks** -- a `SEAL_BREAK` event requires two `Ed25519` signatures from two distinct officer HSM keys before the event is accepted; single-signer attempts return HTTP 403.
- **10-minute Merkle roots to 5 channels** -- an independent Merkle root job (`crates/merkle-anchor/src/lib.rs`) fans out to five external channels; if fewer than 3 of 5 channels acknowledge within 5 minutes, an ops alert fires and the root status is marked `PARTIAL`.
- **Geographic RBAC** -- AG access scoped by PostGIS boundary polygon at auth middleware; evaluated per-request against the `boundary-svc` boundary cache (refreshed every 5 min).
- **Canary units** -- one synthetic inventory unit per batch round-trips through the full pipeline in every CI and DR drill; if the canary is not found at the expected location after a transfer, the drill fails.
- **Tamper-evident transport** -- GPS beacons on transit vehicles post signed location proofs every 30 s; gaps >5 min are flagged as `TRANSIT_GAP` discrepancy events.

## 8. Kenya statute alignment

- **Elections Act s.44A (four-eyes for sensitive materials)** -- dual HSM requirement on seal-break events satisfies the four-eyes obligation for ballot materials.
- **EOA 2016 s.11 (AG access)** -- AG read-only portal and geographic RBAC implement the statutory access right; access logs under `AuditAccess` satisfy the access-control audit obligation.
- **DPA 2019 s.26 (minimisation)** -- GPS coordinates stored only for transit events; no persistent location tracking of individual officers.
- **KEBS KS 4300:2023** -- physical seal specification referenced in `InventoryUnit.type = SEAL`; seal batch numbers reconciled against the KEBS-certified supplier manifest at ingest.
- **Budget and Appropriations** -- inventory unit counts feed M36 budget reporting via the `eel.inventory.unit_count.v1` aggregate event.

## 9. Honest caveats + external prerequisites

- RFID scanner firmware is vendor-supplied and the BLE/USB adapter is written to the vendor's documented serial protocol; any firmware update must be regression-tested against the adapter before deployment.
- GPS beacon coverage in remote areas is not guaranteed; the `TRANSIT_GAP` flag is a detection mechanism, not a prevention mechanism.
- The Kenya National Archives REST endpoint (one of the 5 Merkle anchor channels) requires a separate API agreement; the adapter and stub are ready under `integrations/kna/`; the live endpoint URL is not finalised.
- IAB observer node and county commissioner node are infrastructure prerequisites; their TLS certificates and endpoint URLs must be provisioned before the Merkle fan-out is enabled.
- The 10-minute Merkle root SLA can be violated during a database failover; the anchor job resumes automatically from the last committed root but the gap is logged as a `ROOT_GAP` event and reported in the Grafana dashboard.

## 10. Test coverage + operational runbooks

Coverage under `services/inventory-svc/crates/inventory-api/tests/` and `crates/merkle-anchor/tests/`: unit tests for the event classifier, dual-HSM seal-break validator, Merkle tree builder, geographic RBAC middleware, and discrepancy escalation timer; property-based tests (proptest) asserting Merkle root consistency over arbitrary event sequences; contract tests (Pact) for M31 kit mirror, M32 diagnostics, and M18 boundary RBAC; integration tests with SoftHSM dual-signer, GPS stub, RFID replay, and five-channel Merkle anchor stubs; e2e scenario covering warehouse ingest through seal-break dual-sign, transit gap, polling station arrival, and AG read-only proof query; load tests (k6) for 100k custody events in 30 minutes.

Runbooks under `docs/runbooks/`:

- `inventory-svc-degradation.md` (create): RFID adapter offline; GPS beacon loss; Merkle fan-out partial; HSM unavailable; discrepancy escalation.
- `pgbackrest-pitr.md` (extend): PITR drill for custody events (T1-audit store, RPO 15 min, RTO 60 min).

Dashboards: `platform/observability/inventory-svc/grafana-inventory.json` (custody event rate, Merkle root lag, discrepancy queue depth, AG access count, RFID error rate). SLO alerts: 99.9% event write success; Merkle root lag >12 min; discrepancy unresolved >2 h.

Chaos scenarios under `chaos/scenarios/eel-inventory-*.yaml`: RFID gate failure during batch receive; dual-officer HSM card loss during seal event; Merkle root fan-out failure (2 of 5 channels unavailable); GPS beacon gap injection during transit.

Performance targets: 5 000 concurrent custody scans on receive day at p99 < 500 ms; Merkle root fan-out < 5 min per cycle; discrepancy alert < 60 s of detection.

Custody event throughput tests: 5 000 barcode scans/s during port-of-entry receive day; RFID gate 120 units/min sustained; p99 scan acknowledgement < 500 ms. Dual-HSM seal latency < 3 s per seal event. Merkle root fan-out to 5 channels < 5 min per cycle.

AG access geographic RBAC evaluated per-request against the `boundary-svc` boundary cache (refreshed every 5 min); cross-county access rejected HTTP 403 with audit entry.

Dual-HSM seal event: two distinct officers hold separate HSM token slots; the seal-break API validates that both signers have distinct officer IDs and distinct key material; a single officer presenting two tokens is rejected as a single-signer attempt.

Related: `docs/plans/modules/inventory-management.md`; `services/inventory-svc/docs/CAPABILITIES.md`; edge-case matrix EC-INV-* in `docs/plans/08-edge-case-matrix.md`.
