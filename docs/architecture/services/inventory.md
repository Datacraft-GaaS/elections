# inventory-svc

## Purpose

Maintains the full lifecycle of election assets: kits, ballot papers, seals, consumables, and reserve pools. Tracks physical custody chains via RFID and logistics integrations, manages ballot serial assignments, coordinates ballot printing chain MES integration, and handles diaspora ballot logistics including air-freight pre-clearance manifests.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M20 | inventory-management | Assets, custody events, seals, consumables, reserve pools, MDM integration |
| M39 | ballot-printing-chain | Print orders, lot data, custody events, station reconciliation, MES/KRA interface |
| M65 | diaspora-logistics | Ballot containers, custody events, pre-clearance manifests, airline/customs interface |

## Key Aggregates and State Stores

- `Asset` (PostgreSQL): every kit and ballot-paper lot with serial, status, assignment.
- `CustodyEvent` (PostgreSQL, append-only): chain-of-custody log with operator, location, timestamp.
- `Seal` (PostgreSQL): tamper-evident seal assignments and scan confirmations.
- `Consumable` (PostgreSQL): ballot paper lots, ink, ink ribbon.
- `PrintOrder` (PostgreSQL): ballot print orders with MES tracking.
- `BallotContainer` (PostgreSQL): diaspora ballot containers with customs manifests.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | boundary-svc | Geofence polygons for kit assignment |
| Downstream | biometric-edge-svc | Kit provisioning packages |
| Downstream | bvi-svc | Kit configuration references |
| Downstream | rts-svc | Ballot serial ranges |
| Downstream | ecc-svc | Custody anomaly alerts |
| External | MES (printer partner) | Print order tracking |
| External | KRA, airlines, customs | Ballot import clearance |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.inventory.kit.provisioned` | `eel.inventory` | Kit provisioned and MDM-enrolled |
| `eel.inventory.kit.assigned` | `eel.inventory` | Kit assigned to polling station |
| `eel.inventory.custody.event` | `eel.inventory` | Physical custody transfer recorded |
| `eel.inventory.serial.range.allocated` | `eel.inventory` | Ballot serial range allocated to station |
| `eel.inventory.print_order.confirmed` | `eel.inventory` | Print lot confirmed by MES |
| `eel.inventory.diaspora.manifest.cleared` | `eel.inventory` | Diaspora ballot container cleared by customs |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.boundary.boundary.changed` | boundary-svc |
| `eel.ecc.s44a.event` | ecc-svc |

## Primary APIs

- `packages/openapi-contracts/inventory/openapi.yaml` - asset management, custody chain, MDM
- `packages/openapi-contracts/printing/openapi.yaml` - ballot print orders
- `packages/openapi-contracts/diaspora-logistics/openapi.yaml` - diaspora ballot logistics

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Custody scan P99 | less than 2 s |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- Offline custody scan mode stores events locally and syncs on reconnect.
- Print order MES API failures queue; no ballot production is blocked for more than 24 h.
- Diaspora logistics delays trigger ECC alert; customs clearance follows manual pouch procedure.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
