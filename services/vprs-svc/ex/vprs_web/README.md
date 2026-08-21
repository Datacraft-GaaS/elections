# vprs_web (Phoenix app)

Voter Pre-Registration System Phoenix LiveView for youth pre-registration with real-time IPRS verification.

## Module

M04 (vprs - voter pre-registration) | vprs-svc | WP-3.9

## Purpose

Public-facing Phoenix LiveView allowing citizens aged 16-18 to apply for voter
pre-registration. Integrates with IPRS (via integrations/iprs adapter) for real-time
identity verification. Applications are processed and approved by BVR registrars.
PWA (Progressive Web App) mode supported for low-connectivity use. Anti-enumeration
enforced: no bulk lookup, IPRS rate limit applied per session.

## LiveViews

| Module | Description |
|---|---|
| `ApplicationLive` | Pre-registration application form with IPRS lookup |
| `StatusLive` | Application status check by reference number |
| `VerificationLive` | IPRS identity verification result display |
| `PWALive` | Progressive Web App shell for offline-capable access |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `IPRSVerificationConsumer` | `eel.vprs.iprs-results` | Async IPRS verification results |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:identity_verify` | `IPRSVerifyWorker` | Async IPRS identity check with retry |
| `:approval_notify` | `ApprovalNotifyWorker` | SMS/WhatsApp notify on approval decision |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/vprs-svc/ex/vprs_web
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
| `IPRS_ADAPTER_URL` | integrations/iprs adapter endpoint |
| `VPRS_API_URL` | vprs-svc/crates/vprs-api |
| `TELCO_SMS_ADAPTER_URL` | SMS notification for approval |
| `WA_ADAPTER_URL` | WhatsApp notification for approval |
| `RECAPTCHA_SECRET_KEY` | reCAPTCHA v3 secret |

## WCAG

2.2 AA. Youth-friendly design with large text and simple language.

## Anti-Enumeration Controls

- IPRS rate limit: 60 req/min per cert (shared with all IPRS consumers)
- CAPTCHA on application submission
- No bulk or repeat lookups permitted per session

## Deployment Notes

- Kubernetes: vprs-svc ingress (public-facing, TLS)
- K8s manifest: services/vprs-svc/deploy/base/vprs-web-deployment.yaml
- Pre-registration windows: defined in election calendar (M52)

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M04)
- vprs-svc/crates/vprs-api
- vprs-svc/crates/vprs-domain
- integrations/iprs (I01)
- bvr-svc (approved applicants converted to voter records at age 18)
