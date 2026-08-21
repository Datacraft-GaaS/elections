# ecc-svc

## Purpose

Election Command Centre: the operational nerve centre for election-day management. Integrates telemetry from all kit devices, manages grievance intake across all channels (WhatsApp, USSD, SMS, kiosk), runs the IAB compliance dashboard, and coordinates S.44A override events. Operators and commissioners use ECC to monitor, escalate, and respond in real time.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M15 | iab-dashboard | IAB escalations, complaints, reports; hardware-key logistics |
| M16 | election-command-centre | Kit telemetry, ECC alerts, S.44A events, multi-role dispatch |
| M23 | grievance-channels | Grievance intake (WA/USSD/SMS/kiosk), evidence packages, whistleblower profiles |

## Key Aggregates and State Stores

- `KitTelemetrySnapshot` (Kafka + TimescaleDB): continuous device health and activity feed.
- `ECCAlert` (PostgreSQL): classified alerts with priority and assignment.
- `S44AEvent` (PostgreSQL, append-only): Section 44A override events with full audit trail.
- `GrievanceCase` (PostgreSQL): multi-channel grievance records.
- `EvidencePackage` (PostgreSQL + object store): grievance supporting media.
- `IABEscalation` (PostgreSQL): Independent Advisory Board escalations.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvi-svc | Turnout telemetry and low-identification alerts |
| Upstream | rts-svc | Late/missing form alerts |
| Upstream | queue-svc | Queue and disenfranchisement signals |
| Upstream | ai-platform-svc | Anomaly detection alerts |
| Upstream | inventory-svc | Asset custody alerts |
| Upstream | field-mobile-svc | Dispatch and incident reports |
| Downstream | governance-svc | Grievance referrals to legal review |
| Downstream | transparency-svc | Bug-bounty and evidence escalation |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.ecc.alert.created` | `eel.ecc` | New operational alert raised |
| `eel.ecc.s44a.event` | `eel.ecc` | Section 44A action recorded |
| `eel.ecc.grievance.received` | `eel.ecc` | Grievance case opened |
| `eel.ecc.grievance.resolved` | `eel.ecc` | Grievance case closed with determination |
| `eel.ecc.iab.escalation` | `eel.ecc` | IAB escalation triggered |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvi.turnout.rollup` | bvi-svc |
| `eel.rts.form34.received` | rts-svc |
| `eel.queue.snapshot` | queue-svc |
| `eel.ai_platform.alert.anomaly` | ai-platform-svc |

## Primary APIs

- `packages/openapi-contracts/ecc/openapi.yaml` - ECC dashboard, alert management, S.44A
- `packages/openapi-contracts/grievance/openapi.yaml` - grievance intake and tracking
- `packages/openapi-contracts/iab/openapi.yaml` - IAB escalations and reports

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Telemetry ingest P99 | less than 5 s end-to-end |
| Grievance receipt acknowledgement | less than 10 s |
| Availability | 99.9% |
| RTO | 15 min |
| RPO | zero |

## Degradation Modes

- ECC dashboard falls back to read replica; alert creation continues via Kafka.
- Grievance channels queue in outbox if database is unreachable.
- WhatsApp channel subject to Meta quota limits; USSD/SMS remain available as fallback.

## Runbook Links

- [docs/runbooks/incident-response-p0.md](../../runbooks/incident-response-p0.md)
- [docs/runbooks/election-day-mode.md](../../runbooks/election-day-mode.md)
- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
