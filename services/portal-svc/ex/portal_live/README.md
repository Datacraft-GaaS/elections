# portal_live (Phoenix app)

Phoenix LiveView for real-time result publication events, live turnout updates, and Merkle proof verification.

## Module

M11 (public-portal), M52 (election-calendar-gazette) | portal-svc | WP-3.9

## Purpose

Real-time backend complementing the static portal-svc/astro/public site. When new
Form 34 result publications arrive via Kafka, portal_live pushes live updates to
connected browsers via Phoenix PubSub (no page reload needed). Also handles on-demand
Merkle inclusion proof verification requests (SSR side of the /verify/{hash} route)
and Gazette publication events for the election calendar.

## LiveViews

| Module | Description |
|---|---|
| `ResultsLive` | Real-time result update stream (national + constituency) |
| `TurnoutLive` | Live turnout counter by constituency |
| `ProofVerifyLive` | On-demand Merkle inclusion proof verification |
| `GazetteLive` | Gazette notice publication feed |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `ResultPublicationConsumer` | `eel.rts.publications` | Trigger live result pushes |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:proof_generation` | `ProofWorker` | Async Merkle proof generation |
| `:gazette_publish` | `GazetteWorker` | Gazette notice archive push |

## Rustler NIF Usage

None. Merkle proof computation in transparency-svc/crates/tlog-api.

## Mix Commands

```bash
cd services/portal-svc/ex/portal_live
mix deps.get
mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `KAFKA_BROKERS` | Comma-separated broker list |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `TRANSPARENCY_API_URL` | transparency-svc for Merkle proofs |
| `PORTAL_API_URL` | portal-svc/crates/portal-api |

## WCAG

2.2 AA.

## Deployment Notes

- Kubernetes: portal-svc ingress (public-facing, behind CDN for static assets)
- K8s manifest: services/portal-svc/deploy/base/portal-live-deployment.yaml

## SLA

p95 200ms, p99 500ms at 5M req/hr (CDN absorbs most load; LiveView handles dynamic subset).

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M11, M52)
- portal-svc/astro/public (static site frontend)
- portal-svc/crates/portal-api
- transparency-svc/crates/tlog-api
