# speech_analyst (Phoenix app)

Content safety analyst LiveView dashboard for reviewing AI-flagged speech integrity incidents.

## Module

M17 (speech-integrity) | ai-platform-svc | WP-4.4

## Purpose

Internal-only Phoenix LiveView application for Commission content safety analysts.
Displays ContentFlag queue from the AI pipeline, allows analysts to review, approve, or
escalate flags to safety incidents, and packages evidence for referral to NCIC or
other authorities. Consumes Kafka events via Broadway consumers. Read access restricted
to authorised analyst roles via SpiceDB.

## LiveViews

| Module | Description |
|---|---|
| `ContentFlagLive` | Real-time flag queue with review actions |
| `IncidentLive` | Safety incident detail and evidence packaging |
| `EvidencePackageLive` | Evidence bundle management and export |
| `SafetyQueueLive` | Aggregate queue metrics and triage dashboard |

## Broadway Consumers

| Consumer | Kafka Topic | Description |
|---|---|---|
| `ContentFlagConsumer` | `eel.ai.content-flags` | Ingest flags from speech-integrity ML service |
| `IncidentConsumer` | `eel.ai.incidents` | Update incident status |

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:evidence_export` | `EvidenceExportWorker` | Async evidence bundle export |
| `:report_generation` | `AnalystReportWorker` | Scheduled analyst report generation |

## Rustler NIF Usage

None. AI inference runs in ai-platform-svc/py/speech-integrity; this app is display only.

## Mix Commands

```bash
cd services/ai-platform-svc/ex/speech_analyst
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server                    # development
MIX_ENV=prod mix phx.server       # production (with releases)
MIX_ENV=test mix test             # tests
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `KAFKA_BROKERS` | Comma-separated broker list |
| `SECRET_KEY_BASE` | Phoenix secret key (64+ bytes) |
| `AI_GATEWAY_URL` | ai-platform-svc/crates/ai-gateway |
| `NCIC_ADAPTER_URL` | NCIC/DCI/ODPP adapter for referrals |
| `SPICEDB_ENDPOINT` | SpiceDB authz endpoint |

## WCAG

2.2 AA. All data tables have ARIA labels and keyboard navigation.

## Deployment Notes

- Kubernetes: ai-platform-svc ingress (internal only, not public)
- K8s manifest: services/ai-platform-svc/deploy/base/speech-analyst-deployment.yaml

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M17)
- ai-platform-svc/py/speech-integrity
- integrations/social-platforms
- integrations/ncic-dci-odpp
