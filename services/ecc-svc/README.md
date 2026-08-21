# Election Command Centre

Operational dashboard for election staff, Independent Assurance Board, and public grievance management. Fuses real-time RTS progress, kit telemetry, anomalies, and personnel dispatch into unified control plane.

## Purpose and modules

The Election Command Centre (M16 election-command-centre) provides the primary operational dashboard for the Commission: monitors RTS results progress, kit telemetry, S.44A trigger events, and AI-flagged anomalies. Independent Assurance Board dashboard (M15 iab-dashboard) offers escalation workflow and override authority. Public grievance channel (M23 grievance-channels) ingests complaints via WhatsApp, USSD, SMS, web kiosk, and web portal with case management and whistleblower protection.

Tier: T2.

Availability SLO: 99.9%. Alert delivery latency < 5s. Grievance acknowledgement < 10s.

RTO: 1 hour. RPO: 2 min.

## Architecture overview

Core aggregates: Alert (system event), EccDashboard (read-model), IABEscalation (override request), Grievance (complaint record), GrievanceWorkflow (case management). Backed by PostgreSQL with materialized read-model projections. Elixir LiveView real-time dashboard consumes Kafka alerts. Python NLP pipeline triages grievances (sentiment, anomaly category, urgency). REST and gRPC APIs serve alerts, grievance submission, dispatch commands, and escalations.

Upstream dependencies: RTS (tally updates), BVI (turnout signals), queue-svc (disenfranchisement flags), ai-platform-svc (anomaly alerts), inventory-svc (kit telemetry).

Downstream consumers: ECC operators (dashboard), IAB members (escalation), grievance officers (workflow), dispatch staff (personnel commands).

## Directory layout

```
services/ecc-svc/
  crates/
    ecc-api/            - REST + gRPC API: alerts, grievances, dispatch, IAB
  deploy/               - Kustomize base + overlays (primary, bomas, edge)
  docs/                 - Service documentation
  ex/                   - Elixir LiveView ECC and IAB dashboards
  loadtest/             - k6 scenarios
  migrations/           - PostgreSQL migrations
  py/                   - Python NLP grievance triage
  src/                  - Main Rust binary (API server)
  tests/                - Integration tests
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/ecc/):
- GET /v1/ecc/alerts: Real-time system alerts (RTS progress, kit errors, anomalies)
- POST /v1/ecc/grievances: Submit new grievance (WhatsApp/USSD/web)
- GET /v1/ecc/grievances/{id}: Grievance status and workflow
- GET /v1/ecc/iab/escalations: IAB escalation queue
- POST /v1/ecc/iab/escalations/{id}/override: IAB override decision
- POST /v1/ecc/dispatch/{personnel_id}/command: Send dispatch instruction

gRPC:
- StreamAlerts(stream_opts) -> Stream(Alert)
- SubmitGrievance(GrievanceRequest) -> GrievanceId
- GetGrievanceWorkflow(id) -> WorkflowState
- OverrideDecision(decision) -> Confirmation

## Key events emitted and consumed

Emitted:
- ecc.alert.raised: New system alert (warning, critical)
- ecc.s44a.triggered: S.44A election extension candidate flagged
- ecc.grievance.filed: Grievance submitted and triaged
- ecc.iab.escalated: Grievance escalated to IAB for decision

Consumed:
- rts.tally.updated: Update ECC results progress
- bvi.turnout.updated: Update voter turnout dashboard
- queue-svc.disenfranchisement.flagged: Queue disenfranchisement alerts
- queue-svc.s14a.recommended: S.14A extension recommendations
- ai-platform-svc.alert.raised: AI anomaly alerts
- inventory-svc.kit.telemetry.received: Kit status and errors

## Data model

Consult migrations/ for versioned PostgreSQL schema. data_dictionary.yaml defines:
- alerts: alert_type, severity, source_service, timestamp, resolved_at
- grievances: grievance_id, submission_channel (whatsapp/ussd/sms/web), text, triage_category, urgency_score, filing_status
- grievance_workflows: grievance_id, assigned_officer, status (filed, acknowledged, investigating, resolved), timeline
- iab_escalations: escalation_id, grievance_id, iab_member, override_decision, decision_timestamp
- dispatch_commands: personnel_id, command_type, destination, status, ack_timestamp

Read-model projections materialized via Kafka event sourcing. Grievance NLP triage output stored in separate triage_results table.

## Building and running

Build the service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/ecc-svc
cargo build --release
```

Run locally:
```bash
cargo run --bin ecc-svc -- --config dev.toml --db-url postgres://localhost/ecc_dev
```

Start Elixir LiveView dashboard:
```bash
cd ex && mix ecto.create && mix phx.server
```

## Testing

Unit and domain tests:
```bash
cargo test --lib --all
```

NLP grievance triage tests:
```bash
cargo test grievance::triage --lib
```

Alert streaming and latency:
```bash
cargo test alert_stream::
```

Integration tests:
```bash
cargo nextest run
```

Load testing (alert streaming + grievance intake):
```bash
cd loadtest && k6 run ecc_load.js --vus 100 --duration 600s
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service, PDB, HPA
- deploy/overlays/primary: Anniversary Towers (primary)
- deploy/overlays/bomas: Bomas mirror (read-only dashboard)
- deploy/overlays/edge: Regional clusters (alert relay)

HPA: CPU 65%, memory 75%; min replicas 2, max 20.

PDB: minAvailable 1 (always 2+ running).

PostgreSQL: Primary async replication to bomas; event journal streamed to edge clusters.

## Observability

Dashboards (Grafana):
- ECC Operations: alert volume, RTS progress, kit error rate, dispatch completion rate
- IAB Escalations: escalation volume, override decisions, timeline
- Grievance Processing: submission volume by channel, triage category distribution, resolution time

Alerts (Prometheus):
- Alert delivery latency > 5s: warning
- Grievance acknowledgement > 10s: warning
- Grievance backlog > 100 outstanding: critical
- NLP triage error rate > 0.1%: warning
- ECC dashboard unavailability: critical

Key metrics:
- ecc_alert_raised_total
- ecc_alert_delivery_latency_seconds
- ecc_grievance_submitted_total
- ecc_grievance_acknowledgement_latency_seconds
- ecc_grievance_triage_latency_seconds
- ecc_grievance_category_distribution
- ecc_iab_escalation_total
- ecc_dispatch_command_ack_latency_seconds

## Runbooks

See docs/runbooks/ecc-degradation.md for:
- Alert processing lag (queue monitoring, consumer restart)
- Grievance backlog buildup (workflow scaling)
- NLP triage timeout (model reload)
- IAB escalation workflow stall (database checkpoint)
- ECC dashboard unavailability (read-only fallback)

## Degradation contract

Normal: Live ECC dashboard, alert ingestion, grievance submission, IAB escalations, dispatch commands.

Degraded: Grievance intake continues (async queue); dashboard read-only from cache; alert delivery delayed; dispatch commands queued; IAB escalations processed offline.

Read-only: ECC dashboard served from cache; no new alert processing; no new grievance intake; no dispatch commands.

## Chaos scenarios

- kafka-lag: Delay alert consumer 60s+; verify ECC dashboard freshness
- npl-triage-timeout: Slow NLP model; verify grievance queue handling
- iab-database-lock: Lock escalation table; verify write queue
- alert-volume-spike: Generate 10x alert rate; verify latency SLO
- network-partition-edge: Isolate edge clusters; verify failover

## Statutes

Elections Act §39(1C) transparency and audit trail.
Elections Act §44A potential electoral malpractice reporting (S.44A triggers).
Elections Act §14A election extension criteria (S.14A queue disenfranchisement).
Whistleblower Protection Act 1998 (M23 public grievance confidentiality).

## Follow-ups

1. Multi-language grievance submission UI.
2. Automated grievance routing based on category.
3. Public grievance status tracking (anonymized).
4. IAB decision rationale publication.
