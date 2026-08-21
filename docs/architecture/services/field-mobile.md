# field-mobile-svc

## Purpose

Backend for all mobile and field-agent surfaces: citizen voter verification app (M13/M29), MDM-managed party-agent and poll-clerk apps (M30/M31), field support and dispatch (M32/M33), and observer visit logging (M34). Handles 250K concurrent personnel attendance streams on election day, USSD/WA/IVR multi-channel verification, and grievance intake forwarding to ECC.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M13 | public-verifier-app | Voter verification, participation token query, grievance filing (mobile/web/USSD/WA/IVR) |
| M29 | mobile-citizen-verify | Native iOS/Android/PWA voter verification (low-cost devices) |
| M30 | mobile-party-agent | MDM agent countersignature, HSM-backed, form attestation |
| M31 | mobile-poll-clerk | Incident reporting, end-of-day reports, M05/M06 companion |
| M32 | mobile-field-support | Field support tickets, diagnostic log submission |
| M33 | mobile-personnel-dispatch | Personnel attendance, GPS-tracked dispatch, payroll integration |
| M34 | mobile-observer | Observer visit logs, verification records |

## Key Aggregates and State Stores

- `VerificationQuery` (PostgreSQL, time-partitioned): public verification requests with anti-enumeration controls.
- `ParticipationTokenQuery` (PostgreSQL): token verification attempts.
- `AgentCountersignature` (PostgreSQL): party agent HSM-signed form attestations.
- `PersonnelAttendanceRecord` (PostgreSQL): 250K+ daily attendance events.
- `FieldSupportTicket` (PostgreSQL): field support requests with device diagnostics.
- `ObserverVisitLog` (PostgreSQL): accredited observer station visits.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | rts-svc | Form data for public verification |
| Upstream | bvi-svc | Voter identification status |
| Upstream | transparency-svc | Participation token lookup |
| Upstream | ecc-svc | Alert context for field support |
| Downstream | ecc-svc | Grievances and incident reports |
| Downstream | support-admin-svc | Payroll attendance records |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.field_mobile.verification.query` | `eel.field_mobile` | Voter verification query (rate-limited) |
| `eel.field_mobile.grievance.filed` | `eel.field_mobile` | Grievance filed via mobile channel |
| `eel.field_mobile.attendance.recorded` | `eel.field_mobile` | Personnel attendance event |
| `eel.field_mobile.observer.visit` | `eel.field_mobile` | Observer station visit logged |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.rts.form34.published` | rts-svc |
| `eel.transparency.token.issued` | transparency-svc |
| `eel.ecc.alert.created` | ecc-svc |

## Primary APIs

- `packages/openapi-contracts/field-mobile-bff/openapi.yaml` - BFF for citizen verification app
- `packages/openapi-contracts/field-mobile-dispatch/openapi.yaml` - personnel dispatch
- `packages/openapi-contracts/field-mobile-support/openapi.yaml` - field support tickets
- `packages/openapi-contracts/ussd/openapi.yaml` - USSD channel gateway
- `packages/openapi-contracts/whatsapp/openapi.yaml` - WhatsApp webhook
- `packages/openapi-contracts/ivr/openapi.yaml` - IVR channel

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Citizen verification P99 | less than 3 s (USSD budget) |
| Personnel dispatch P99 concurrent | 250K sessions |
| Availability | 99.5% |
| RTO | 15 min |
| RPO | zero |

## Degradation Modes

- USSD and SMS channels remain available if WA is rate-limited.
- Personnel attendance degrades to batch upload; payroll is not blocked.
- Observer visit log falls back to manual form if mobile network is unavailable.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/election-day-mode.md](../../runbooks/election-day-mode.md)
