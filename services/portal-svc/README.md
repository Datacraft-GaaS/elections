# Portal service

Public-facing election portal with live results, inclusion proofs, voter roll inspection, and referendum explainer. Serves 5-10M concurrent viewers during election night.

## Purpose and modules

The portal publishes live election results (M11 public-portal), archives election calendars and official gazette notices (M52 election-calendar-gazette), provides voter list public inspection per §6A (M64 voter-list-public-inspection), and delivers a 11-language referendum explainer with WCAG 2.2 AAA accessibility (M68 referendum-explainer).

Tier: T2.

Availability SLO: 99.9%. Response latency p95 < 200ms, p99 < 500ms at 5M/hr sustained load. Inclusion proof delivery SLO: 30s.

RTO: 1 hour. RPO: 5 min.

## Architecture overview

Core aggregates: Form (results form), Proof (Merkle inclusion proof), CalendarEvent, GazetteNotice, Objection (§6A inspection), CorrectionClaim. Backed by PostgreSQL with materialized read-model projections. Static Astro 5 frontend served from CDN (M11, M52, M68). Elixir LiveView real-time results dashboard consumes Kafka events. Python TTS pipeline generates 11-language explainer audio (M68). REST API serves form publication, inclusion proofs, gazette searches, and inspection endpoints.

Upstream dependencies: RTS (Form34 published events), transparency-svc (STH for proof verification), BVR (for §6A inspection roll freezing).

Downstream consumers: CDN (static results), citizen viewers (form queries), auditors (proof verification), inspection officers (objection/correction workflow).

## Directory layout

```
services/portal-svc/
  astro/                - Public portal frontends (Astro 5, SSR)
    explainer/          - M68 referendum explainer with TTS
  crates/
    gazette/            - M52 calendar and gazette: CalendarEvent, GazetteNotice
    inspection/         - M64 §6A voter roll inspection: RollSnapshot, Objection
    portal-api/         - REST API for form pub, proofs, gazette, inspection
  deploy/               - Kustomize base + overlays (primary, bomas, edge)
  docs/                 - Service documentation
  ex/                   - Elixir LiveView results dashboard
  loadtest/             - k6 scenarios (5M/hr)
  migrations/           - PostgreSQL migrations
  py/                   - Python TTS pipeline for M68
  src/                  - Main Rust binary (API server)
  tests/                - Integration tests (WCAG, load)
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/portal/):
- GET /v1/portal/results/{constituency_id}: Live Form34 results by constituency
- GET /v1/portal/proof/{leaf_hash}: Merkle inclusion proof + STH anchor
- GET /v1/portal/gazette/notices: Search and list official notices
- GET /v1/portal/inspection/roll: §6A voter roll snapshot (frozen)
- GET /v1/portal/inspection/objections: Public objection/correction records
- GET /v1/portal/referendum/{id}/explainer: Referendum explainer content + TTS URLs

## Key events emitted and consumed

Emitted:
- portal.form.published: Form34 indexed and proofs generated
- portal.proof.served: Inclusion proof delivered to viewer
- portal.objection.filed: §6A objection submitted
- portal.correction.claimed: §6A correction submitted

Consumed:
- rts.form34.published: Ingest results for public display
- rts.tally.updated: Update live dashboard
- transparency.sth.published: Proof anchor updates
- bvr.register.frozen: Lock §6A inspection roll

## Data model

Consult migrations/ for versioned PostgreSQL schema. data_dictionary.yaml defines:
- forms: Form34 data, leaf_hash, merkle_path, published_at, cached_ttl
- proofs: Merkle inclusion proof, sth_anchor, proof_verified_at
- gazette_notices: Notice text, effective_date, statutory_rule_id, gazette_date
- objections: Voter ID, objection_text, filed_at, status, assigned_officer
- correction_claims: Voter ID, correction_text, evidence_link, status

Read-model projections materialized via Kafka event sourcing. Results cached via CDN (30s TTL); proofs served from Postgres with proof verification.

## Building and running

Build the service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/portal-svc
cargo build --release
```

Run locally:
```bash
cargo run --bin portal-svc -- --config dev.toml --db-url postgres://localhost/portal_dev
```

Build and preview Astro portal:
```bash
cd astro && pnpm install && pnpm build
pnpm preview
```

Start Elixir LiveView dashboard:
```bash
cd ex/portal_live && mix ecto.create && mix phx.server
```

## Testing

Unit and domain tests:
```bash
cargo test --lib --all
```

WCAG 2.2 AAA compliance (M68):
```bash
cargo test wcag:: --features wcag-audit
cd tests && npm test axe_wcag.js
```

Load testing (5M/hr):
```bash
cd loadtest && k6 run k6_portal.js --vus 5000 --duration 300s
```

Proof verification tests:
```bash
cargo test proof:: --lib
```

Integration tests:
```bash
cargo nextest run
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service, CDN cache rules, PDB, HPA
- deploy/overlays/primary: Anniversary Towers (primary)
- deploy/overlays/bomas: Bomas mirror (read-only replicas)
- deploy/overlays/edge: Regional edge clusters (eventually consistent results cache)

HPA: CPU 70%, memory 80%; min replicas 3, max 50 (election night).

PDB: minAvailable 2 (always 3+ running).

PostgreSQL: Primary async replication to bomas; streaming to edge clusters.

## Observability

Dashboards (Grafana):
- Portal Queries: results query latency, proof serving rate, cache hit ratio
- Gazette and Inspection: objection/correction submission volume, workflow completion time
- Referendum Explainer: TTS generation latency, language distribution, WCAG audit results

Alerts (Prometheus):
- Results query p99 > 500ms: warning
- Proof serving latency > 30s: critical
- Gazette notice ingest failure: warning
- WCAG violations detected: warning
- TTS pipeline error rate > 0.1%: critical

Key metrics:
- portal_results_query_duration_seconds
- portal_proof_serving_duration_seconds
- portal_proof_verification_success_ratio
- portal_gazette_notices_ingested_total
- portal_inspection_objection_submitted_total
- portal_referendum_tts_generation_duration_seconds
- portal_wcag_audit_violations

## Runbooks

See docs/runbooks/portal-degradation.md for:
- CDN cache miss storms (cache strategy adjustment)
- Proof generation latency spike (queue monitoring)
- Gazette notice ingest failure (manual import)
- §6A roll lock failure (manual intervention)
- TTS pipeline timeout (batch retry logic)
- WCAG compliance drift (audit trigger)

## Degradation contract

Normal: Live results, inclusion proofs, gazette, inspection, TTS explainer.

Degraded: Results served from CDN/Valkey cache; proof delivery delayed; gazette read-only; inspection objections queued (async processing); TTS audio fallback to text.

Read-only: Static snapshot only; no live updates; no new submissions; cached results from CDN; no proof verification.

## Chaos scenarios

- cdn-cache-miss: Invalidate CDN; verify fallback to Valkey
- proof-generation-timeout: Delay merkle proof computation 30s+; verify SLO
- gazette-notice-lag: Delay notice ingestion 10min+; verify graceful degradation
- inspection-roll-lock-failure: Freeze roll ingest; verify rejection of late submissions
- tts-pipeline-timeout: Slow audio generation; verify text fallback + async retry
- network-partition-edge: Isolate edge cluster; verify eventual consistency

## Statutes

Elections Act §6A voter list public inspection (M64).
Elections Act §§83-84 results publication and form transmission (M11).
Elections Act §39(1C) transparency and audit trail (all modules).
WCAG 2.2 AAA accessibility (M68 referendum explainer).
Government Printer standard for official gazette notices (M52).

## Follow-ups

1. Real-time tally boards per polling station (phase 2).
2. Voter education video library (phase 2).
3. §6A objection appeal workflow automation.
4. Proof caching strategy optimization for 10M+ viewers.
