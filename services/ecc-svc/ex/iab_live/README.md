# iab_live (Phoenix app)

IAB (Internal Advisory Body) restricted dashboard for escalated complaints and AI safety reports.

## Module

M15 (iab-dashboard) | ecc-svc | WP-4.1

## Purpose

Restricted Phoenix LiveView accessible only to IAB members via mandatory FIDO2/WebAuthn
hardware key authentication. Displays escalated public complaints, AI safety incidents
from speech-integrity, model deployment approvals, and generated summary reports.
Read-mostly with narrow write surface (approve/reject decisions). Hardware key
logistics is the primary operational risk.

## LiveViews

| Module | Description |
|---|---|
| `IABDashboardLive` | Overview: open cases, pending decisions, alert counts |
| `ComplaintLive` | Escalated complaint detail with action (approve/reject/refer) |
| `ReportLive` | Periodic IAB report generation and sign-off |
| `HardwareKeyAuthLive` | FIDO2/WebAuthn challenge flow |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `IABEscalationConsumer` | `eel.ecc.iab-escalations` | Receive escalated cases |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:report_generation` | `IABReportWorker` | Scheduled IAB report compilation |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/ecc-svc/ex/iab_live
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
| `IAB_HW_KEY_POLICY` | JSON policy for allowed FIDO2 authenticator AAGUID list |
| `ECC_API_URL` | ecc-svc/crates/ecc-api |
| `AI_PLATFORM_API_URL` | ai-platform-svc for model deployment approvals |

## Auth

FIDO2/WebAuthn hardware key mandatory for all IAB member sessions. No password fallback.
Authenticator AAGUID whitelist in IAB_HW_KEY_POLICY.

## Deployment Notes

- Kubernetes: ecc-svc ingress (restricted - allowList IAB member IPs only)
- K8s manifest: services/ecc-svc/deploy/base/iab-live-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M15)
- ai-platform-svc/ex/speech_analyst (AI safety feed)
- ai-platform-svc/py/model-registry (deployment approvals)
