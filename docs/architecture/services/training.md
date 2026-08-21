# training-svc

## Purpose

Manages all training, accreditation, and certification workflows: the voter education content management system, offline-capable poll-worker learning management system with HSM-signed certificates, media and observer accreditation with QR badge generation and 5-minute CRL refresh, and medical first-response incident dispatch with no-PHI enforcement.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M37 | voter-education-cms | Content items, versions, fact-check reviews, 11-language catalogue |
| M38 | poll-worker-training-lms | Courses, enrolments, HSM-signed certificates, offline mobile sync |
| M42 | media-accreditation | Media organisations, accreditations, QR badges, offline CRL |
| M43 | observer-accreditation | Observer organisations, individual observers, mission accreditation |
| M60 | medical-first-response | Medical incidents, dispatch requests, cluster alerts (no PHI) |

## Key Aggregates and State Stores

- `ContentItem` (PostgreSQL): versioned voter education content in 11 languages.
- `WorkerCertificate` (PostgreSQL): HSM-signed poll-worker training certificates.
- `MediaAccreditation` (PostgreSQL): media organisation and individual accreditation records.
- `CRL` (PostgreSQL + CDN): certificate revocation list, refreshed every 5 min.
- `ObserverOrganisation` (PostgreSQL): observer mission accreditation.
- `MedicalIncident` (PostgreSQL): field medical incidents with schema-enforced no-PHI constraint.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | governance-svc | Referendum and recall training content triggers |
| Upstream | ecc-svc | Alert context for medical dispatch |
| Downstream | field-mobile-svc | LMS morning briefing feed |
| Downstream | portal-svc | Voter education explainer content |
| External | MCK (Media Council of Kenya) | Media accreditation validation |
| External | HSM | Certificate signing |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.training.certificate.issued` | `eel.training` | Poll-worker HSM-signed certificate issued |
| `eel.training.accreditation.granted` | `eel.training` | Media or observer accreditation approved |
| `eel.training.crl.refreshed` | `eel.training` | CRL updated with new revocations |
| `eel.training.incident.dispatched` | `eel.training` | Medical first-response dispatch triggered |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.governance.referendum.configured` | governance-svc |
| `eel.ecc.alert.created` | ecc-svc |

## Primary APIs

- `packages/openapi-contracts/training-cms/openapi.yaml` - content management
- `packages/openapi-contracts/training-lms/openapi.yaml` - LMS enrolment and certification
- `packages/openapi-contracts/training-accreditation/openapi.yaml` - media and observer accreditation
- `packages/openapi-contracts/training-medical/openapi.yaml` - medical incident dispatch

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Certificate issuance P99 | less than 5 s |
| CRL refresh interval | every 5 min |
| Offline LMS sync | within 4 h of last connection |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 1 h |

## Degradation Modes

- LMS offline mode continues from last synced content; certification queues until reconnect.
- CRL distribution via CDN survives origin outage up to 30 min.
- Medical dispatch falls back to direct phone escalation if API is unavailable.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
