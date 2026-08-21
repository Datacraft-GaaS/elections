# bvi-svc

## Purpose

Manages the biometric voter identification layer during polling: distributes kit configuration bundles to field devices, processes identification events from kits (42M+ events on election day), maintains real-time turnout rollups, and provides 60-second kit certificate revocation. Operates the public voter verification lookup (BVV) with anti-enumeration protections.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M06 | bvi-backend | Kit config bundles, identification events, turnout rollup, kit certificate management |
| M07 | bvv | Voter lookup audit, objection handling, public kiosk interface |

## Key Aggregates and State Stores

- `KitConfigBundle` (PostgreSQL + CDN): signed configuration package per kit serial.
- `IdentificationEvent` (Kafka + PostgreSQL, time-partitioned): biometric ID outcomes from kits.
- `TurnoutRollup` (Redis + PostgreSQL): per-station voter counts, refreshed every 60 s.
- `KitCert` (PostgreSQL + CRL store): kit TLS certificates with 60-second revocation propagation.
- `VoterLookupAudit` (PostgreSQL): public lookup requests with rate-limiting records.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvr-svc | Register extract and voter records for kit bundles |
| Upstream | abis-svc | Biometric match decisions |
| Upstream | inventory-svc | Kit serial registry |
| Downstream | rts-svc | Polling session open events gating form acceptance |
| Downstream | ecc-svc | Turnout telemetry and low-turnout alerts |
| Downstream | portal-svc | Public turnout feed |
| Downstream | queue-svc | Queue snapshot feed |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.bvi.kit.session.opened` | `eel.bvi` | Polling session opened at station |
| `eel.bvi.kit.session.closed` | `eel.bvi` | Polling session closed |
| `eel.bvi.identification.event` | `eel.bvi` | Biometric ID outcome (pass/fail/exception) |
| `eel.bvi.turnout.rollup` | `eel.bvi` | Station turnout snapshot |
| `eel.bvi.kit.cert.revoked` | `eel.bvi` | Kit certificate revoked |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvr.register.snapshot.published` | bvr-svc |
| `eel.inventory.kit.assigned` | inventory-svc |
| `eel.abis.match.completed` | abis-svc |

## Primary APIs

- `packages/openapi-contracts/bvi/openapi.yaml` - kit config distribution, session management, turnout API
- `packages/openapi-contracts/verifier/openapi.yaml` - public voter lookup

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Identification event ingest P99 | less than 25 ms |
| Kit cert revocation propagation | less than 60 s |
| Turnout rollup freshness | less than 60 s |
| Availability | 99.99% |
| RTO | 15 min |
| RPO | zero |

## Degradation Modes

- Kit devices operate in degraded offline mode if bvi-svc is unreachable; events are queued on-device.
- BVV public lookup rate-limiting increases under load to prevent enumeration.
- Turnout rollups stale-serve from Redis if database write is delayed.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/election-day-mode.md](../../runbooks/election-day-mode.md)
