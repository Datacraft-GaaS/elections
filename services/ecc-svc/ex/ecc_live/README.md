# ecc_live (Phoenix app)

Election Command Centre multi-role real-time dashboard for kit telemetry, alerts, and dispatch.

## Module

M16 (election-command-centre) | ecc-svc | WP-4.1

## Purpose

Internal Phoenix LiveView application for the Commission's Election Command Centre.
Multi-role dashboard (ECC Director, County Coordinator, Logistics Officer, ICT Officer)
providing real-time kit telemetry snapshots, anomaly/fraud alerts, S44A events, and
dispatch actions. Network access restricted to ECC LAN. Consumes high-volume Kafka
telemetry via Broadway.

## LiveViews

| Module | Description |
|---|---|
| `CommandCentreLive` | Top-level national overview map and alert banner |
| `KitTelemetryLive` | Per-kit telemetry feed (GPS, battery, scan rates) |
| `AlertQueueLive` | Ranked anomaly alert queue with acknowledge/escalate actions |
| `DispatchLive` | Field dispatch: assign ICT officers and support teams |
| `S44AEventLive` | S44A exceptional circumstances event log and actions |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `TelemetryConsumer` | `eel.kits.telemetry` | Real-time kit telemetry (high volume) |
| `AlertConsumer` | `eel.ai.anomaly-alerts` | Anomaly detection alerts from ai-platform-svc |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:alert_escalation` | `AlertEscalationWorker` | Timed escalation of unacknowledged alerts |
| `:dispatch_notify` | `DispatchNotifyWorker` | SMS/WhatsApp notify for field dispatch |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/ecc-svc/ex/ecc_live
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `KAFKA_BROKERS` | Comma-separated broker addresses |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `ECC_API_URL` | ecc-svc/crates/ecc-api |
| `SPICEDB_ENDPOINT` | SpiceDB for RBAC enforcement |
| `TELCO_SMS_ADAPTER_URL` | SMS dispatch notifications |

## WCAG

2.2 AA. Dashboard designed for low-light environments.

## Deployment Notes

- Kubernetes: ecc-svc deployment (restricted to ECC LAN network policy)
- K8s manifest: services/ecc-svc/deploy/base/ecc-live-deployment.yaml
- Network policy blocks public internet access

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M16)
- ecc-svc/crates/ecc-api
- ai-platform-svc/py/anomaly (alert source)
- docs/runbooks/ecc-operations.md
