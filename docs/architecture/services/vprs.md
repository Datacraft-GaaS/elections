# vprs-svc

## Purpose

Voter Pre-Registration System: the public-facing web and PWA application where citizens submit online voter pre-registration applications before in-person biometric enrolment. It validates identity data against IPRS, queues applications for biometric-edge processing, and prevents enumeration abuse via rate-limiting and CAPTCHA.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M04 | vprs | VPRS application intake, IPRS validation, application queue |

## Key Aggregates and State Stores

- `VPRSApplication` (PostgreSQL): application records with IPRS validation status and biometric appointment.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| External | IPRS | Identity pre-validation |
| Upstream | bvr-svc | Duplicate check against existing voter records |
| Downstream | biometric-edge-svc | Appointment and enrolment queue |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.vprs.application.submitted` | `eel.vprs` | New pre-registration application received |
| `eel.vprs.application.validated` | `eel.vprs` | IPRS validation passed |
| `eel.vprs.application.rejected` | `eel.vprs` | Application rejected (duplicate, IPRS mismatch) |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.biometric_edge.enrollment.completed` | biometric-edge-svc |

## Primary APIs

- `packages/openapi-contracts/vprs/openapi.yaml` - application submission, status check

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Application submission P99 | less than 2 s |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- If IPRS is unavailable, applications queue with pending-validation status; no application is lost.
- Rate-limiting tightens automatically under enumeration-pattern traffic.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
