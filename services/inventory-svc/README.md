# Inventory service

Chain of custody tracking for 65,000 KIEMS kits, ballot lots, seals, consumables, and diaspora ballot pouches. Event-sourced asset ledger with HSM-signed custody events for auditability and non-repudiation.

## Purpose and modules

Inventory Management (M20 inventory-management) tracks physical asset lifecycle: procurement, warehouse storage, field dispatch, station receipt, post-election return. Ballot Printing Chain (M39 ballot-printing-chain) manages printer MES integration, lot certification, RFID tagging, and ballot custody. Diaspora Ballot Logistics (M65 diaspora-ballot-physical-logistics) coordinates diaspora ballot pouch shipping: airlines, customs clearance, embassy delivery, final station receipt.

Tier: T2.

Availability SLO: 99.9%. RFID scan-to-record latency < 500ms. Batch kit reconciliation latency < 5 min per 1000 kits.

RTO: 2 hours. RPO: 30 min.

## Architecture overview

Core aggregates: Asset (kit/ballot/seal), CustodyEvent (HSM-signed handoff), Seal, Consumable, ReservePool, PrintOrder, LotData, BallotContainer, PreClearanceManifest. Event-sourced ledger backed by PostgreSQL. RFID scan handler verifies kit identity, records custody event, updates asset state. HSM integration (PKCS#11) signs custody events for cryptographic non-repudiation. Fleet mobile app (Flutter) for custody staff: scan barcodes, sign receipts, manage offline mode. Django admin for bulk import and reconciliation reports. REST API serves asset history, custody chains, and dispatch records.

Upstream dependencies: BVR (kit registration), RTS (ballot count reconciliation), biometric-edge (kit attestation).

Downstream consumers: Field teams (mobile app), warehouse staff (admin), auditors (custody chain verification), finance (asset reconciliation).

## Directory layout

```
services/inventory-svc/
  apps/                 - Flutter mobile app for custody staff
  crates/
    custody/            - Asset custody ledger: Asset, CustodyEvent (HSM-signed)
    diaspora-logistics/ - M65 diaspora ballot logistics
    printing/           - M39 ballot printing chain
  deploy/               - Kustomize base + overlays (primary, bomas, edge)
  django/               - Admin interface (bulk ops, reconciliation)
  docs/                 - Service documentation
  migrations/           - PostgreSQL migrations (event sourcing)
  src/                  - Main Rust binary (RFID handler, API server)
  tests/                - Integration tests
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/inventory/):
- GET /v1/inventory/assets/{asset_id}/custody-chain: Full custody chain (all handoffs)
- POST /v1/inventory/custody-events: Record new custody event (RFID scan, handoff)
- GET /v1/inventory/kits/{serial}: Kit metadata and current location
- GET /v1/inventory/ballot-lots/{lot_id}: Ballot lot certification and receipt log
- GET /v1/inventory/seals/{seal_id}: Seal inventory and assignment status
- POST /v1/inventory/diaspora-pouches/{pouch_id}/manifest: Update diaspora pouch manifest
- GET /v1/inventory/reconciliation: Batch reconciliation report

## Key events emitted and consumed

Emitted:
- inventory.custody.transferred: Asset handed off from one custodian to another
- inventory.kit.dispatched: Kit sent from warehouse to station
- inventory.ballot.lot.certified: Ballot lot certified by printer
- inventory.diaspora.pouch.dispatched: Diaspora pouch shipped
- inventory.asset.received: Asset received and verified at destination

Consumed:
- rts.form34.submitted: Ballot count reconciliation trigger
- biometric-edge.kit.attested: Kit registration and identity verification

## Data model

Consult migrations/ for versioned PostgreSQL event-sourced schema. data_dictionary.yaml defines:
- assets: asset_id, asset_type (kit/ballot/seal/consumable), serial/batch, lifecycle_status
- custody_events: event_id, asset_id, from_custodian, to_custodian, timestamp, hsm_signature, event_data
- print_orders: order_id, quantity, ballot_style, certification_date, printer_id
- ballot_lots: lot_id, print_order_id, quantity, sealed_at, receipt_log
- diaspora_pouches: pouch_id, shipment_id, airline, tracking_number, customs_status, delivery_status
- seals: seal_id, asset_id, applied_at, verified_at, removed_at

Event sourcing enables full audit trail reconstruction. HSM signatures on custody events prevent tampering. Materialized read-model projections for current asset state.

## Building and running

Build the service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/inventory-svc
cargo build --release
```

Run locally:
```bash
cargo run --bin inventory-svc -- --config dev.toml --db-url postgres://localhost/inventory_dev --hsm-config pkcs11_dev.toml
```

Build Flutter mobile app:
```bash
cd apps && flutter build apk --release
```

Start Django admin:
```bash
cd django && python manage.py runserver 0.0.0.0:8000
```

## Testing

Unit tests (custody logic, event sourcing):
```bash
cargo test --lib --all
```

HSM signature tests:
```bash
cargo test custody::hsm_signature --lib
```

Reconciliation report generation:
```bash
cargo test reconciliation::
```

RFID scan-to-record latency:
```bash
cargo test rfid:: --lib
```

Integration tests (event sourcing, snapshot recovery):
```bash
cargo nextest run
```

Load testing (RFID scans, 1000 assets):
```bash
cd loadtest && k6 run inventory_load.js --vus 200 --duration 300s
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service, PDB, HPA
- deploy/overlays/primary: Anniversary Towers (primary, HSM attached)
- deploy/overlays/bomas: Bomas mirror (async replication)
- deploy/overlays/edge: Regional clusters (event relay, read-only)

HPA: CPU 70%, memory 80%; min replicas 2, max 12.

PDB: minAvailable 1 (always 2+ running).

PostgreSQL: Primary async replication to bomas; event journal streamed to edge clusters.

HSM: PKCS#11 driver required on primary; offline signature fallback on edge.

## Observability

Dashboards (Grafana):
- Asset Inventory: kit dispatch timeline, in-warehouse stock, field location distribution
- Custody Events: event rate, handoff completion time, HSM signature latency
- Ballot Printing: lot certification timeline, seal application rate, receipt verification
- Diaspora Logistics: pouch shipment timeline, customs clearance duration, delivery status

Alerts (Prometheus):
- RFID scan-to-record > 500ms: warning
- HSM signature latency > 1s: warning
- Unverified custody event > 1 hour: critical (audit trail gap)
- Kit dispatch backlog > 1000: warning
- Diaspora pouch customs hold > 48 hours: critical

Key metrics:
- inventory_rfid_scan_latency_seconds
- inventory_custody_event_hsm_signature_latency_seconds
- inventory_asset_in_warehouse_count
- inventory_asset_dispatched_total
- inventory_custody_event_recorded_total
- inventory_ballot_lot_certified_total
- inventory_diaspora_pouch_dispatch_latency_seconds
- inventory_reconciliation_discrepancy_rate

## Runbooks

See docs/runbooks/inventory-degradation.md for:
- RFID reader failure (mobile fallback, manual scanning)
- HSM unavailability (offline signature queue)
- Custody event backlog (consumer scaling)
- Kit dispatch backlog (warehouse workflow review)
- Diaspora customs hold (coordination with authorities)
- Reconciliation discrepancy investigation

## Degradation contract

Normal: Full custody tracking, RFID scanning, HSM signing, diaspora logistics, reconciliation.

Degraded: Mobile offline mode (sync on reconnect); HSM unavailable (offline queue); diaspora logistics tracked manually; reconciliation delayed.

Read-only: Custody chain read-only; no new events; no RFID scanning; no dispatch commands.

## Chaos scenarios

- hsm-unavailable: Disconnect HSM; verify offline queue and recovery
- rfid-reader-failure: Disable RFID reader; verify mobile fallback
- custody-event-backlog: Delay event processing 300s+; verify asset state consistency
- kit-dispatch-spike: Generate 10x dispatch requests; verify latency SLO
- diaspora-customs-hold: Delay pouch clearance indefinitely; verify escalation

## Statutes

Elections Act §§53-55 spoilt ballot management and custody.
Elections Act §39(1C) transmission chain and audit trail.
Elections Act §42 KIEMS kit custody and security.
Elections Act §68 ballot box sealing and custody.

## Follow-ups

1. Real-time asset GPS tracking (phase 2).
2. Automated kit reconciliation via blockchain-style verification.
3. Diaspora ballot chain-of-custody video recording (phase 2).
4. Predictive maintenance for kit battery/RFID health.
