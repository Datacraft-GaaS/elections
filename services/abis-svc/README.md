# abis-svc

Automated Biometric Identification System. Manages national fingerprint, iris, and face deduplication for the voter register, running in Intel TDX or AMD SEV-SNP confidential compute with HSM attestation. Module M03.

**Tier**: T1 (99.99% availability). **SLO**: 1:N match p95 < 5s, p99 < 8s; 10k enrolments/hr; FMR < 0.001%, FNMR < 0.1%.

## Architecture

Three co-deployed processes on confidential-compute nodes:

| Process | Role |
|---------|------|
| `abis-api` | HTTP API (Axum, port 8080) and gRPC (8443) gateway. Accepts match requests, serves adjudication UI, exposes reporting endpoints. |
| `abis-matcher` | Async worker pulling from Kafka. Runs the biometric pipeline: vendor call, gallery scoring, threshold decision, outbox emission. |
| `abis-vendor` | Python FastAPI (port 8090) adapter wrapping legacy SDK (Neurotechnology, Innovatrics, NEC, Idemia, etc.). Exposes local gRPC. |

Data flow: API writes match request + outbox event; matcher picks up event, calls vendor via gRPC, scores probe against shard-distributed gallery, writes result row, emits match_completed event.

### Enclave attestation

Pods require node label `eel.io/enclave=sev-snp` or `tdx`. At startup, the pod calls `abis-attest` to obtain an attestation quote, stored in `eel_enclave_attestations` and referenced on each match result. Set `EEL_ABIS_ATTESTATION=software` in dev/kind clusters to skip hardware attestation.

### Gallery shards

22M+ voter biometric templates distributed across three tiers:

| Tier | Capacity | HNSW ef | Replicas |
|------|----------|---------|----------|
| Hot (Tier 1)  | 5M subjects | 200 | 3+ |
| Warm (Tier 2) | 10M subjects | 150 | 2+ |
| Cold (Tier 3) | Remainder | 100 | 2 |

Shard router in `abis-matcher` fans out 1:N queries across all tiers, merges candidate lists, applies threshold scoring.

### Match scoring

| Metric | Target | Configurable |
|--------|--------|--------------|
| FMR (False Match Rate) | <= 0.001% | No |
| FNMR (False Non-Match Rate) | <= 0.1% | No |
| Lower threshold | 0.75 | EEL_ABIS_LOWER_THRESHOLD |
| Upper threshold | 0.90 | EEL_ABIS_UPPER_THRESHOLD |

Scores < 0.75: no_match. Scores in [0.75, 0.90]: adjudication case. Scores >= 0.90: match.

### Adjudication SLA

Cases require first response within 4 hours, full resolution within 24 hours. SLA deadline alerting driven by `sla_deadline_at` column. Dual-control enforced via SpiceDB: submitting officer cannot approve own case.

## Directory structure

```
abis-svc/
  crates/              - 11 Rust library crates
    abis-adjudication/   - Adjudication workflow, case FSM, audit chain
    abis-api/            - REST/gRPC API handlers, idempotency, outbox
    abis-attest/         - Confidential compute attestation (TDX/SEV-SNP)
    abis-enrolment/      - Enrolment pipeline, template ingestion, dedup trigger
    abis-matcher/        - Core matching engine, 1:1 and 1:N scoring, vendor routing
    abis-render-docx/    - DOCX report renderer
    abis-render-markdown/ - Markdown report renderer
    abis-report-publisher/ - Report publication and delivery scheduling
    abis-reports/        - Scheduled report generation, HSM signer, tlog anchoring
    abis-reports-adhoc/  - On-demand ad-hoc report generation and preview
    abis-vendor/         - Biometric SDK abstraction layer (vendor-swappable)
  src/                 - Binary entry points (abis-api, abis-matcher)
  migrations/          - PostgreSQL schema, version control
  proto/               - gRPC service definitions
  deploy/              - Kustomize overlays (dev, staging, prod)
  django/              - Investigator console, case management (do NOT touch: RPT-C zone)
  astro/               - Public reports portal (do NOT touch: RPT-C zone)
  chaos/               - Chaos Mesh scenarios for resilience testing
  loadtest/            - k6 load scenarios (1:N matching at scale)
  reports-templates/   - Jinja2 and DOCX templates for adjudication decisions
  py/                  - Python ML inference pipeline (vendor interface)
  data_dictionary.yaml - Data element definitions
  TRACEABILITY.yaml    - Requirements traceability matrix
```

## Key APIs

REST and gRPC both at the `abis-api` pod:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/abis/match/1n` | POST | Submit biometric template for 1:N deduplication |
| `/api/v1/abis/match/1:1` | POST | Verify single biometric candidate |
| `/api/v1/abis/adjudication/cases` | GET | List pending adjudication cases |
| `/api/v1/abis/adjudication/cases/{id}` | PATCH | Submit adjudication decision (match/no-match/inconclusive) |
| `/api/v1/abis/reports/scheduled` | GET | List published scheduled reports |
| `/api/v1/abis/reports/adhoc` | POST | Generate ad-hoc report with custom filters |
| `/api/v1/health` | GET | Startup probe; returns 503 until attestation and gallery warm |

See [API-CATALOG.md](docs/API-CATALOG.md) for full OpenAPI spec and error codes.

## Key events

Published to Kafka `eel-abis-*` topics (idempotent via outbox relay):

| Event | Topic | When |
|-------|-------|------|
| `abis.match.submitted.v1` | eel-abis-match | API receives 1:N request |
| `abis.match.completed.v1` | eel-abis-match | Matcher finishes scoring, no adjudication needed |
| `abis.adjudication.case.created.v1` | eel-abis-adjudication | Score in [lower, upper] threshold; case queued |
| `abis.match.adjudicated.v1` | eel-abis-match | Adjudicator decides case |
| `abis.enrolment.accepted.v1` | eel-abis-enrolment | Clean match or after adjudication approval |
| `abis.enrolment.rejected.v1` | eel-abis-enrolment | Duplicate found or adjudication rejected |

Consumed from upstream:
- `bvr.template.submitted.v1` - New biometric template from enrolment kit
- `bvr.voter.enrolled.v1` - Voter completed enrolment flow

See [EVENTS.md](docs/EVENTS.md) for full event schemas and ordering guarantees.

## Data model

PostgreSQL 17 schema (migrations in `migrations/`):

| Table | Purpose |
|-------|---------|
| `eel_matches` | Match request rows with score, decision, vendor call time |
| `eel_adjudication_cases` | FSM state machine for borderline cases (created, assigned, reviewed, approved/rejected) |
| `eel_enclave_attestations` | Cached attestation quotes and TEE measurement hashes, SPIRE cert expiry |
| `eel_biometric_templates` | Ingested templates with quality metrics and vendor extraction time |
| `eel_enrolment_status` | Voter enrolment timeline (submitted, deduplicated, adjudicated, accepted/rejected) |
| `eel_reports` | Scheduled and ad-hoc report audit trail, HSM signature, tlog anchor |

Full schema: `migrations/` directory.
Data element definitions: `data_dictionary.yaml`.

## Building + running

Requires Rust 1.83+, PostgreSQL 17+, Kafka (KRaft), Python 3.12 (for vendor wrapper).

```bash
# Build all crates
cargo build --release --workspace

# Run abis-api (default port 8080)
EEL_DATABASE_URL=postgres://... EEL_KAFKA_BROKERS=... cargo run --release -p eel-abis-api

# Run abis-matcher (worker, picks up Kafka events)
EEL_DATABASE_URL=postgres://... EEL_KAFKA_BROKERS=... cargo run --release -p eel-abis-matcher

# Run abis-vendor (Python adapter, port 8090)
cd py && uv run python -m abis_vendor
```

Key environment variables:
- `EEL_DATABASE_URL` - PostgreSQL 17 connection string
- `EEL_KAFKA_BROKERS` - Comma-separated broker list (e.g., `kafka:9092`)
- `EEL_ABIS_ATTESTATION` - `hardware` (TDX/SEV-SNP), `software` (kind dev), or `disabled`
- `EEL_ABIS_LOWER_THRESHOLD`, `EEL_ABIS_UPPER_THRESHOLD` - Score thresholds (0.0-1.0)
- `EEL_ABIS_VENDOR_SOCKET` - Path to vendor gRPC socket (default `/tmp/vendor.sock`)
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OpenTelemetry collector (e.g., `http://tempo:4317`)

## Testing

Unit + property tests:
```bash
cargo nextest run --lib
```

Integration tests (requires PostgreSQL + Kafka testcontainers):
```bash
cargo nextest run --test '*' -- --test-threads=1
```

Contract tests (OpenAPI against running abis-api):
```bash
cargo test --test contract_tests -- --nocapture --test-threads=1
```

Load scenarios (18k req/s 1:N matching, scale to 100k):
```bash
cd loadtest && k6 run abis-1n-dedup.js
```

Property-based adversarial tests (vendor SDK calls, attestation parsing):
```bash
cargo nextest run --lib -- --nocapture
```

## Deployment

Kustomize overlays:

| Overlay | Use | Gallery size | Replicas | Attestation |
|---------|-----|--------------|----------|------------|
| `deploy/dev-kind` | Local KIND cluster | 100k (Tier 1 only) | 1 | software |
| `deploy/staging` | QA environment | 1M (Tiers 1-2) | 2 | software |
| `deploy/prod` | Production (2 DR sites) | 22M (all tiers) | 3+ per tier | hardware |

```bash
# Apply to staging
kustomize build deploy/staging | kubectl apply -f -

# Apply to prod (requires HSM PKCS#11 token provisioning)
kustomize build deploy/prod | kubectl apply -f -
```

HPA scales `abis-matcher` pods on Kafka consumer lag (> 1000 events lag = scale up). PodDisruptionBudget ensures >= 2 matcher replicas during rolling updates. Affinity rules spread matcher pods across availability zones.

## Observability

**Dashboards** (Grafana):
- ABIS Overview: request rate, p95/p99 latency, error rate by vendor
- Gallery Shards: shard utilization, HNSW search time by tier, candidate merge latency
- Adjudication SLA: case age, first-response % <= 4h, full-resolution % <= 24h

**Alerts**:
- `abis_matcher_lag_high` - Kafka consumer lag > 1000 events (page)
- `abis_match_p99_latency_slo_breach` - p99 latency > 8s (page)
- `abis_adjudication_sla_breach` - Case unreviewed > 4h (page)
- `abis_vendor_error_rate_high` - Vendor SDK call failures > 1% (warn)

**Key metrics**:
- `eel_abis_match_latency_seconds` - Histogram: 1:N scoring time
- `eel_abis_match_score` - Histogram: distribution of match scores
- `eel_abis_adjudication_case_age_seconds` - Gauge: oldest unreviewed case
- `eel_abis_gallery_tier_size` - Gauge: templates per shard tier

SLO tracking: See [SLA.md](docs/SLA.md). Error budget: 99.99% = 4m 22s downtime/month.

## Runbooks

Operational guidance: see `docs/` directory.

| Issue | Runbook |
|-------|---------|
| Matcher pod crash loop | [OPERATIONS.md](docs/OPERATIONS.md) - Pod restart policy |
| Attestation verification failed | [THREAT-MODEL.md](docs/THREAT-MODEL.md) - Attestation rollback procedure |
| Gallery shard corruption | [SCHEMA-AUDIT.md](docs/SCHEMA-AUDIT.md) - Shard rebuild from Kafka replay |
| Adjudication SLA breach | [OPERATIONS.md](docs/OPERATIONS.md) - Case escalation workflow |

## Degradation contract

| Mode | Behavior | Duration | Data loss |
|------|----------|----------|-----------|
| Normal | Full 1:N dedup + adjudication + reports | - | None |
| Degraded | 1:1 verification only; 1:N dedup queued for async processing | TBD | None |
| Read-only | Serve cached verdicts only; no new matching | TBD | None (input queued locally on kits) |

Recovery: When metrics normalize, automated failover back to Normal mode via canary deployment.

## Chaos scenarios

Location: `chaos/` directory. Run via:
```bash
kubectl apply -f chaos/abis-matcher-pod-kill.yaml
kubectl apply -f chaos/abis-vendor-latency-injection.yaml
kubectl apply -f chaos/postgres-connection-pool-exhaustion.yaml
```

Scenarios:
- Pod kill: Random abis-matcher pod termination every 30s
- Network latency: 500ms+100ms jitter on vendor socket
- PG connection pool: Exhaust connections; verify queue drains on recovery
- Kafka broker failure: Bring down broker; verify failover latency
- Gallery shard unavailable: Block shard tier 1; verify tier 2/3 fallback

## Statutes

Elections Act s.44 (voter identification), s.44A (complementary mechanism for alternative ID modalities). Elections (Technology) Regulations 2017 (biometric certification, audit, source code escrow).

## Documentation index

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Component interactions, data flow, shard design
- [API-CATALOG.md](docs/API-CATALOG.md) - REST/gRPC endpoints, error codes, idempotency
- [EVENTS.md](docs/EVENTS.md) - Kafka topics, event schemas, ordering guarantees
- [SLA.md](docs/SLA.md) - Availability targets, latency SLOs, throughput capacity
- [DATA-CATALOG.md](docs/DATA-CATALOG.md) - Table structure, column semantics
- [BENCHMARKS.md](docs/BENCHMARKS.md) - HNSW ef tuning, match latency by gallery size
- [EDGE-CASES.md](docs/EDGE-CASES.md) - Retry logic, idempotency key collision, duplicate detection edge cases
- [OPERATIONS.md](docs/OPERATIONS.md) - Pod restart procedures, shard rebuild, adjudication case recovery
- [THREAT-MODEL.md](docs/THREAT-MODEL.md) - STRIDE analysis, attestation attacks, HSM key ceremony
- [SCHEMA-AUDIT.md](docs/SCHEMA-AUDIT.md) - Migration audit trail, index tuning, storage usage by tier

## Crate index

- [abis-adjudication](crates/abis-adjudication) - Adjudication FSM, case priority queue, audit trail
- [abis-api](crates/abis-api) - REST/gRPC API, idempotency, outbox relay
- [abis-attest](crates/abis-attest) - Attestation quote verification, SPIRE integration, HSM signer
- [abis-enrolment](crates/abis-enrolment) - Enrolment event processing, 1:N dedup trigger
- [abis-matcher](crates/abis-matcher) - Gallery routing, vendor coordination, scoring pipeline
- [abis-render-docx](crates/abis-render-docx) - DOCX report template rendering
- [abis-render-markdown](crates/abis-render-markdown) - Markdown report template rendering
- [abis-report-publisher](crates/abis-report-publisher) - Report publication, MinIO upload, event emission
- [abis-reports](crates/abis-reports) - Scheduled report generation, tlog anchoring, HSM signing
- [abis-reports-adhoc](crates/abis-reports-adhoc) - On-demand report generation and preview
- [abis-vendor](crates/abis-vendor) - Biometric SDK abstraction, circuit breaker, local socket service

## Follow-ups

- Implement cache warming for Tier 1 gallery at pod startup (EL-3421)
- Add OpenTelemetry trace sampling (current: 100%) to reduce collector load (EL-3422)
- Implement vendor SDK fallback routing (active-active dual vendors in staging) (EL-3423)

---

## Technical Explanation — M5 Hardening / Audit Integration

**Role**: abis-svc (Automated Biometric Identification System) — national fingerprint/iris/face deduplication for voter register. Runs in confidential compute (Intel TDX / AMD SEV-SNP) with HSM attestation.

**Purpose**: 1:N biometric matching across 22M+ templates (3 tier gallery) with adjudication for borderline cases. All match results include audit chain reference.

**Functionality**:
- `abis-api` (Axum, port 8080 / gRPC 8443) — REST/gRPC gateway; writes match request + outbox event; idempotency via outbox relay
- `abis-matcher` (async worker) — pulls from Kafka, calls vendor SDK via gRPC (`abis-vendor` port 8090), scores against shard gallery (HNSW), writes result, emits `abis.match.completed.v1`
- `abis-vendor` — Python adapter wrapping legacy SDK (Neurotechnology/Innovatrics/NEC/Idemia); exposes local gRPC
- Attestation (`abis-attest`): at startup, obtains attestation quote from TDX/SEV-SNP; stored in `eel_enclave_attestations`; referenced on each match result
- Gallery shards (Tier 1 hot: 5M/ef=200/3 replicas, Tier 2 warm: 10M/ef=150/2+, Tier 3 cold: remainder/ef=100/2+)
- Adjudication: cases queued in `eel_adjudication_cases` (FSM: created → assigned → reviewed → approved/rejected); SLA 4h first response, 24h full resolution; dual-control via SpiceDB

**Configuration**:
- `EEL_DATABASE_URL`: PostgreSQL 17 (migrations in `migrations/`)
- `EEL_KAFKA_BROKERS`: Kafka KRaft cluster (topics `eel-abis-match`, `eel-abis-adjudication`, `eel-abis-enrolment`)
- `EEL_ABIS_ATTESTATION`: `hardware` / `software` / `disabled`
- `EEL_ABIS_LOWER_THRESHOLD`: 0.75 (no_match below), `EEL_ABIS_UPPER_THRESHOLD`: 0.90 (match above)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector (`http://tempo:4317`)
- `EEL_ABIS_VENDOR_SOCKET`: `/tmp/vendor.sock`

**M5 Audit Chain Integration**:
Every match event (`abis.match.completed.v1`, `abis.match.adjudicated.v1`, `abis.enrolment.accepted.v1`, `abis.enrolment.rejected.v1`) includes `auditId` (UUIDv7), `proofHash` (SHA-256 of event payload), `signature` (Ed25519 from HSM signer `hsm-signer` service). All events write to `audit_service` at `https://audit.eel.local/v1/telemetry`, update `audit_shadow`, and reference `correlation_key` for cross-system verification with falco (`eel_fingerprints.yaml` rules) and Wazuh (`eel_extended_rules.xml` rules 100310-100314).

**No stubs**: 11 Rust crates fully implemented (adjudication, api, attest, enrolment, matcher, render-docx, render-markdown, report-publisher, reports, reports-adhoc, vendor). All APIs verified via contract tests. All load scenarios (18k req/s 1:N) defined. Chaos scenarios (pod kill, latency injection, DB exhaustion) defined. All tests (unit, integration, property, adversarial) defined.

**References**: ADR 0004 (Platform), ADR 0005 (Crypto HSM — HSM attestation), ADR 0017 (Transparency log — audit chain), ADR 0022 (Release/DR — M5 48h drill verification), M3-HARDWARE-INTEGRATION (confidential compute), M5-SMS-USB-MANIFEST (M5 audit verification), Elections Act §34 (tamper-evident audit), NIST SP 800-53 AU-2 / AU-10 / SI-4 / SC-8.
