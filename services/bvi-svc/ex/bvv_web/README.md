# bvv_web (Phoenix app)

Phoenix LiveView backend for real-time voter verification and anti-enumeration enforcement.

## Module

M07 (bvv) | bvi-svc | WP-3.7

## Purpose

Server-side Phoenix LiveView that powers the voter verification widget in the
bvi-svc/astro/bvv Astro site. Handles voter lookup queries against bvi-svc, enforces
anti-enumeration controls (Redis rate limit: 1 lookup per IP per 10 minutes), manages
CAPTCHA validation, and logs objection submissions. Query-only; no mutation of voter
records.

## LiveViews

| Module | Description |
|---|---|
| `VoterLookupLive` | Real-time voter status lookup with CAPTCHA |
| `PollingStationLive` | Polling station finder |
| `ObjectionLive` | Voter roll objection submission form |

## Broadway Consumers

None (query-only).

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:objection_processing` | `ObjectionWorker` | Async objection routing to bvr-svc |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/bvi-svc/ex/bvv_web
mix deps.get
mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `REDIS_URL` | Redis for rate limiting |
| `BVI_API_URL` | bvi-svc voter lookup API |
| `RATE_LIMIT_WINDOW_SECS` | Anti-enumeration window (default 600) |
| `RECAPTCHA_SECRET_KEY` | reCAPTCHA v3 secret for server-side verification |

## WCAG

2.2 AA.

## Deployment Notes

- Kubernetes: bvi-svc ingress (public-facing, behind CDN)
- K8s manifest: services/bvi-svc/deploy/base/bvv-web-deployment.yaml

## SLA

p99 <= 100ms at peak (M06 SLA covers BVV query path).

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M07)
- bvi-svc/astro/bvv (frontend site)
- bvi-svc/crates/bvi-api
