# governance-svc

Comprehensive governance and legal compliance service for the eel IEMS. Implements 12 modules spanning post-election audit, election-court interface, recall procedures, referendum mechanics, international observer coordination, financial forensics, and statutory reporting.

## Service overview

Tier: T2 (99.9% availability). SLO: evidence bundle delivery under 60 seconds; RLA sampling under 10 minutes per constituency.

Modules:
- M21: Risk-Limiting Audit (SHANGRLA statistical method, NIST Randomness Beacon)
- M40: Election Court IT Interface (eCourt evidence bundle delivery)
- M41: Post-Election Review (findings, recommendations, parliament referrals)
- M48: Recall Petitions (Article 104, signature verification)
- M49: Referendum Mechanics (county-threshold TLA+-verified logic)
- M50: By-Election Operations (under 21-day notice constraint)
- M56: County-National Coordination (SAML-federated county IdP)
- M57: International Observer Coordination (workspace isolation, joint statements)
- M58: Parliamentary and Judiciary Reporting (statutory drafts, testimony versioning)
- M59: Auditor-General Interface (IFMIS/PPRA evidence export)
- M62: Transmission Legal Basis Audit (real-time Elections Act §39(1C) compliance)
- M66: Election Financial Forensics (CBK/FRC API integration, ML anomaly detection)

## Architecture

Five microservices run independently on separate ports, each with its own axum server, database connection pool, and Kafka consumer. A shared event bus (Kafka with Hybrid Logical Clock causality) coordinates state transitions. All services store aggregate events in PostgreSQL outbox; async CDC via Debezium publishes to Kafka topics with message signing. Off-kit audit laptop (Tauri-based rla-desktop) runs the RLA sampling engine offline, enqueues verdicts locally, and syncs on reconnection.

Evidence bundles are immutable: once created, they are signed and pinned to IPFS via the transparency-svc. County coordination queries use federated SAML attributes; OAG/CBK interfaces use OAuth2 mTLS with PKCS#11 client certificates.

## Directory layout

```
services/governance-svc/
  apps/                   # Tauri desktop app (RLA offline audit tool)
  crates/
    by-election/          # M50 by-election operations
    county-coord/         # M56 county/national coordination
    court-interface/      # M40 election court interface
    forensics/            # M66 financial forensics
    intl-observer/        # M57 international observer coordination
    legal-audit/          # M62 transmission legal basis audit
    oag/                  # M59 Auditor-General interface
    recall/               # M48 recall petitions
    referendum/           # M49 referendum mechanics
    reporting/            # M58 parliamentary/judiciary reporting
    review/               # M41 post-election review
    rla/                  # M21 risk-limiting audit
  django/                 # Admin UI: RLA runs, court petitions, accreditation review
  ex/                     # LiveView dashboards: RLA progress, forensics alerts
  py/                     # Python RLA statistics, anomaly detection models
  deploy/                 # Kustomize overlays (primary, mirror)
  docs/                   # Architecture, API reference, TLA+ models
  loadtest/               # k6 scenarios
  migrations/             # PostgreSQL schema versions
  TRACEABILITY.yaml       # M21-M66 edge case matrix
  data_dictionary.yaml    # Column classification (PII types, retention)
```

## Key APIs

OpenAPI spec at `packages/openapi-contracts/governance/openapi.yaml`.

Core endpoints:

```
POST /v1/governance/rla/runs                  # initiate audit run
GET /v1/governance/rla/runs/{run_id}         # fetch run status
POST /v1/governance/rla/runs/{run_id}/samples # draw sample
GET /v1/governance/rla/runs/{run_id}/report  # verdict report

POST /v1/governance/court/petitions           # file court petition
GET /v1/governance/court/petitions/{id}      # fetch petition
POST /v1/governance/court/petitions/{id}/evidence-bundles
GET /v1/governance/court/petitions/{id}/evidence-bundles/{bundle_id}

GET /v1/governance/recall/petitions           # list active recalls
POST /v1/governance/recall/petitions          # initiate recall
GET /v1/governance/recall/petitions/{id}     # fetch recall status

GET /v1/governance/referendum/{id}/result    # fetch certified result
POST /v1/governance/referendum/{id}/county-results # submit county tallies

GET /v1/governance/special-elections/{id}    # fetch by-election metadata
POST /v1/governance/special-elections/{id}/cost-report

GET /v1/governance/county-coord/mou/{county_id}
GET /v1/governance/international-observers/{mission_id}/briefing
GET /v1/governance/parliament-reporting/{report_id}/draft
POST /v1/governance/oag/audit-engagements
GET /v1/governance/legal-audit/transmissions/{id}/verdict
GET /v1/governance/forensics/cycles/{cycle_id}/anomalies
```

All endpoints require valid JWT + SpiceDB authz check. Evidence bundle downloads stream multipart/form-data; chunk size 1 MB with resumable offset header support.

## Key events

Emitted: `governance.rla.completed`, `governance.rla.recount.triggered`, `governance.petition.bundle.delivered`, `governance.recall.petition.verified`, `governance.referendum.result.certified`, `governance.special.election.declared`, `governance.county.coordination.approved`, `governance.observer.mission.briefed`, `governance.parliament.report.drafted`, `governance.oag.audit.engaged`, `governance.transmission.verdict.issued`, `governance.forensics.anomaly.detected`.

Consumed: `rts.form34.published` (results for referendum tally, RLA sampling), `rts.tally.updated` (for forensic anomaly baseline), `rts.declaration.signed` (for legal audit, court evidence), `transparency.sth.published` (for court evidence archival), `field-mobile.attendance.checked-in` (to cross-verify special-election staff), `crms.party.registered` (for recall party eligibility).

## Data model

Schema: `migrations/001_governance.sql` onward. Column classification per `data_dictionary.yaml`.

Aggregates: `AuditRun` (immutable, append-only samples and verdicts), `Petition` (state machine: filed, bundled, served, concluded), `RecallPetition` (signature records with de-duplication logic), `Referendum` (county results aggregation), `SpecialElection` (cost tracking, staff register), `CountyCoordination` (MoU, approvals, incidents), `ObserverMission` (briefing packs, reports, joint statements), `StatutoryReport` (draft versioning, sign-off trails), `AuditEngagement` (IFMIS evidence scope), `TransmissionEvent` (legal verdicts, evidence citations), `AnalysisCycle` (anomaly detector output, CBK referrals).

PII classification: `VoterIdentity` (Restricted, hashed ID + biometric template ref), `StaffContact` (Sensitive, redacted in external reports), `PetitionerDetails` (Public if published verdict, Restricted otherwise).

## Building and running

Requires Rust 1.70+ (nix flake.nix pins it), Python 3.12 (uv), Elixir 1.17.

```bash
# Build all crates and binaries
cd services/governance-svc
cargo build --release

# Run unit tests
cargo test -p eel-governance-rla -p eel-governance-court-interface --all-features

# Run integration tests (requires PostgreSQL + Kafka)
cargo test --test '*' -- --test-threads=1

# Run load test (requires running service)
k6 run loadtest/governance.js

# Run RLA desktop app
cd apps && npm run tauri dev

# Run Python models locally
cd py && python -m pytest tests/
```

Check `CLAUDE.md` for test conventions: async tests are plain `async fn`, no `@pytest.mark.asyncio`; use `pytest.get_event_loop()` inside test body; crate tests tag with `#[test]` or feature gates (`#[cfg(test)]`).

## Testing

Test layout: `tests/ci/` for CI autodiscovery, `examples/` for integration demos, `**/tests/` for unit tests within each crate.

Fixtures:
- `test_fixtures::audit_run()` returns a seeded AuditRun with mock NIST beacon seed
- `test_fixtures::petition()` returns a pre-filed petition ready for bundle attachment
- `test_fixtures::county_coord()` returns a pre-approved county coordination record
- pytest-httpserver for stubbing IFMIS, CBK, eCourt APIs

Scenarios:
- RLA: draw sample, record verdicts, verify recount trigger at thresholds per SHANGRLA spec
- Court: attach 15-item bundle, verify signatures, simulate eCourt API failure (circuit breaker to read-only)
- Recall: verify signature dedup, Art. 104 threshold checks, trigger special election
- Referendum: aggregate county results, apply tie-breaker logic, verify TLA+ model correctness
- Forensics: inject synthetic anomalies, verify detector finds them, trace CBK referral

Run `cargo test --test integration_governance -- --include-ignored` to run full matrix. Tag long tests with `#[ignore]` and `// runs ~30s` comment.

## Deployment

Kustomize overlays in `deploy/overlays/primary` and `deploy/overlays/mirror`. Base manifests in `deploy/base/`.

Bring-up sequence:
1. Apply PostgreSQL schema: `kubectl apply -f deploy/db/schema-governance.sql`
2. Seed OPA policies: `kubectl apply -f deploy/authz/governance-policies.rego`
3. Deploy services: `kubectl apply -k deploy/overlays/primary`
4. Verify readiness: `kubectl rollout status deployment/governance-rla -n eel-governance`

Each crate runs as a separate Kubernetes Deployment with its own Kustomize overlay. RTO/RPO: 15 minutes (asynchronous Kafka consumer resumption from latest committed offset + snapshot replay).

Full deployment docs: `docs/deployment.md`.

## Observability

Metrics (Prometheus):
- `governance_rla_sample_size{jurisdiction}` (gauge, samples drawn per constituency)
- `governance_rla_verdict_latency_ms{outcome}` (histogram, offline verdict entry to sync)
- `governance_court_bundle_delivery_ms` (histogram, upload to court confirmation)
- `governance_forensics_anomaly_score{type}` (gauge, ML anomaly rank per cycle)

Traces (Jaeger): every RLA run, court petition, recall vote, referendum aggregation, and forensics cycle tagged with W3C baggage `election_id`, `jurisdiction_id`, `cycle_id`.

Logs (Loki): structured JSON, fields `event_type`, `aggregate_id`, `outcome`, `duration_ms`. All audit-trail writes include `actor_id`, `timestamp`, `signature`.

SLOs (Grafana):
- RLA: 99.9% of audit run samples drawn within 10 min
- Court: 99.9% of evidence bundles delivered within 60 sec
- Recall: 99.5% of petition verifications within 5 min

Dashboard: `http://localhost:3000/d/governance-svc/`.

## Runbooks

Degradation modes: `docs/runbooks/governance-degradation.md`.

Chaos scenarios: `docs/runbooks/governance-chaos.md`.

Common incidents:
- RLA beacon fetch timeout: fallback to cached seed from prior election + log incident event
- Court API unreachable: buffer evidence bundles locally; replay on reconnection
- Forensics detector overfitting: adjust threshold in real-time via ConfigMap; backtest on historical data
- Referendum tie in more than one county: escalate to electoral commission for public tiebreaker announcement

## Degradation contract

**Normal**: all workflows operational; live NIST Randomness Beacon, real-time court interface, anomaly detection running.

**Degraded**: RLA audit runs from cached beacon seed (prior election); court evidence bundles queued locally; forensics detector read-only (batch runs only, no live alerts).

**Read-only**: serve pre-computed RLA reports from archive; court interface returns cached bundles only; all writes (petitions, recalls, referendums, special elections) rejected with 503 Service Unavailable.

## Chaos scenarios

Scenarios in `docs/runbooks/governance-chaos.md`:

1. RLA Beacon timeout: simulate NIST API 5xx errors, verify fallback to cached seed, check audit trail
2. Evidence bundle storage full: MinIO quota breach, verify graceful queue-to-file fallback
3. County federation IdP down: SAML metadata unavailable, verify circuit breaker to read-only; cached roles via JWT
4. Forensics ML model crash: inference engine OOM, restart with smaller batch size, alert ops
5. Kafka consumer lag spike: Debezium connector overwhelmed, trigger load shedding on lower-priority events

## Statutes satisfied

- Constitution Art. 87: Election management (all modules)
- Constitution Art. 104: Recall procedures (M48)
- Elections Act §§75-84: Election conduct and evidence (M21, M40, M62)
- Elections Act §39(1C): transmission legality (M62)
- Elections Act §§53-55: spoilt ballots (M27, separate service)
- Political Parties Act: party recall eligibility (M48)
- Election Offences Act: evidence handling chain of custody (M40)

## Follow-ups

- M21 Bayesian VotaCheck extension (lower sample size for high-confidence races)
- M49 automated tie-breaker logic (governors race 2031)
- M62 real-time transmission verification dashboard (for county election observers)
- M66 federated learning for anomaly detection (encrypted model sharing with CBK, FRC)
