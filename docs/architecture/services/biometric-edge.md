# biometric-edge-svc

## Purpose

Runs the on-device firmware for field kit hardware: biometric enrolment and voter registration (M01), biometric voter identification at polling stations (M05), and diaspora embassy voting stations (M24). It is the physical boundary between the IEBC network and field devices; it handles sensor SDK integration, device attestation, GNSS binding, and delayed-batch transmission for low-connectivity environments.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M01 | bvr-device | On-kit voter enrolment, biometric capture, ID document scan, kit attestation |
| M05 | bvi-device | On-kit polling session management, identification events, register extract |
| M24 | diaspora-voting | Embassy station management, diaspora roll, Form 34A-D, pouch custody |

## Key Aggregates and State Stores

- `Voter` (on-device SQLite, encrypted): local voter register extract for offline BVI.
- `BiometricTemplate` (on-device encrypted store): captured ISO templates pending sync.
- `PollingSession` (on-device SQLite): session state, event log, sync queue.
- `EmbassyStation` (PostgreSQL, cloud): diaspora station configuration and status.
- `DiasporaRoll` (PostgreSQL): frozen voter roll for diaspora polling.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvr-svc | Register extract download to device |
| Upstream | bvi-svc | Kit config bundle and session authorisation |
| Upstream | inventory-svc | Kit provisioning and MDM policies |
| Downstream | abis-svc | Template enrollment ingest |
| Downstream | bvr-svc | Enrollment events and template uploads |
| Downstream | rts-svc | Form 34A-D transmission from diaspora stations |
| Downstream | inventory-svc | Pouch custody events |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.biometric_edge.enrollment.completed` | `eel.kit` | Biometric enrollment package ready for sync |
| `eel.biometric_edge.identification.attempt` | `eel.kit` | On-device identification event (pre-ABIS confirmation) |
| `eel.biometric_edge.session.event` | `eel.kit` | Polling session lifecycle event from kit |
| `eel.biometric_edge.diaspora.form.transmitted` | `eel.kit` | Diaspora Form 34 batch dispatched |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvr.register.snapshot.published` | bvr-svc |
| `eel.bvi.kit.session.opened` | bvi-svc |
| `eel.inventory.kit.provisioned` | inventory-svc |

## Primary APIs

- `packages/openapi-contracts/biometric-edge/openapi.yaml` - kit sync, enrollment upload, session management
- `packages/openapi-contracts/diaspora-logistics/openapi.yaml` - diaspora station and pouch custody

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Enrollment sync P99 | less than 30 s (when connected) |
| Offline operation | indefinite (batch on reconnect) |
| Diaspora form transmission | within pouch custody window |
| RTO | 30 min (device re-attestation) |
| RPO | zero (events queued on device until confirmed) |

## Degradation Modes

- Kits operate fully offline; all events queue on-device and flush on reconnect.
- If GNSS is spoofed, station lock is denied and an ECC alert fires.
- Diaspora stations with embassy connectivity loss use manual pouch procedures.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/dr-failover-primary-to-mirror.md](../../runbooks/dr-failover-primary-to-mirror.md)
