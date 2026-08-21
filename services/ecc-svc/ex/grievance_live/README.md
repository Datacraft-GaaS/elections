# grievance_live (Phoenix app)

Public grievance submission and tracking Phoenix LiveView with whistleblower mode.

## Module

M23 (grievance-channels) | ecc-svc | WP-4.1

## Purpose

Public-facing Phoenix LiveView for citizen grievance submission via web. Integrates
with WhatsApp and USSD channels (handled by field-mobile-svc/py/wa-bot and
field-mobile-svc/py/ussd-menu, whose events flow through Kafka). Supports
whistleblower mode: encrypted ephemeral profile, anonymous submission with session
token for follow-up. Evidence file upload (max 10 MB, type-validated). Kiosk mode
for supervised terminals at constituency offices.

## LiveViews

| Module | Description |
|---|---|
| `GrievanceSubmitLive` | Public grievance submission form |
| `GrievanceStatusLive` | Status check by reference code |
| `WhistleblowerLive` | Encrypted anonymous submission with session token |
| `KioskLive` | Supervised kiosk mode for constituency offices |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `WhatsAppGrievanceConsumer` | `eel.grievances.wa` | WhatsApp-sourced grievances |
| `USSDGrievanceConsumer` | `eel.grievances.ussd` | USSD-sourced grievances |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:grievance_escalation` | `EscalationWorker` | Auto-escalate stale grievances |
| `:evidence_collect` | `EvidenceWorker` | Async evidence file processing |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/ecc-svc/ex/grievance_live
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `WA_WEBHOOK_SECRET` | WhatsApp webhook HMAC secret |
| `ECC_API_URL` | ecc-svc/crates/ecc-api (case management) |
| `UPLOAD_S3_BUCKET` | Evidence file upload bucket |
| `ENCRYPTION_KEY` | Key for whistleblower profile encryption |

## WCAG

2.2 AA. Kiosk mode: large touch targets, high contrast.

## Deployment Notes

- Kubernetes: ecc-svc ingress (public-facing, TLS)
- K8s manifest: services/ecc-svc/deploy/base/grievance-live-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M23)
- field-mobile-svc/py/wa-bot
- field-mobile-svc/py/ussd-menu
- ecc-svc/py/grievance-channels (channel intake worker)
