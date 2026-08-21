# queue_map (Phoenix app)

Real-time public queue length map and disenfranchisement monitoring Phoenix LiveView.

## Module

M19 (queue-transparency) | queue-svc | WP-4.2

## Purpose

Public-facing Phoenix LiveView displaying live queue length estimates at polling stations,
derived from CV headcount (queue-svc/py/cv-headcount) and disenfranchisement flags.
Feeds aggregated data to the ECC dashboard (M16). Citizens can see wait times at their
polling station in real time. S14A recommendations (extension of voting hours) are
surfaced to ECC operators when thresholds are breached.

## LiveViews

| Module | Description |
|---|---|
| `QueueMapLive` | National map with colour-coded queue length per station |
| `StationQueueLive` | Per-station queue detail with historical chart |
| `DisenfranchisementLive` | Disenfranchisement flag alerts and resolution |
| `S14ARecommendationLive` | Auto-generated S14A (extension) recommendations for ECC |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `QueueSnapshotConsumer` | `eel.queues.snapshots` | CV headcount snapshots |
| `CVHeadcountConsumer` | `eel.queues.cv-raw` | Raw CV frame counts (aggregated here) |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:s14a_recommendation` | `S14AWorker` | Generate S14A recommendation when queue > threshold |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/queue-svc/ex/queue_map
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `KAFKA_BROKERS` | Comma-separated broker list |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `QUEUE_API_URL` | queue-svc/crates/queue-pipeline API |
| `ECC_API_URL` | ecc-svc for S14A recommendation push |
| `S14A_THRESHOLD_MINUTES` | Wait time threshold to trigger S14A recommendation |

## WCAG

2.2 AA. Map has text equivalent (sortable station table).

## Deployment Notes

- Kubernetes: queue-svc ingress (public-facing for citizen-facing map)
- K8s manifest: services/queue-svc/deploy/base/queue-map-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M19)
- queue-svc/py/cv-headcount (headcount source)
- queue-svc/crates/queue-pipeline
- ecc-svc/ex/ecc_live (S14A consumer)
