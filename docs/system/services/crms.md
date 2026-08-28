# crms-svc

## 1. Overview

crms-svc is the Candidate and Party Registration Management System: candidate nominations
with a state machine, KRA/EACC/police clearance aggregation, gazette production, ballot
proof generation, party and coalition registration, primaries oversight, nomination fee
payments (M-Pesa Daraja C2B), and the party symbol/colour registry.

- **Runtime**: Rust (axum) single API binary composed with the shared `eel-api-server-rs`
  server builder; a domain crate holds aggregates. Django (`django/crms-admin`), Elixir
  (`ex/party_portal`), and Python (`py/daraja`) directories are README-only stubs.
- **Port**: default `0.0.0.0:8080`, override with `HTTP_ADDR`
  (services/crms-svc/crates/crms-api/src/main.rs:21–23).
- **Entrypoint**: services/crms-svc/crates/crms-api/src/main.rs.
- Tier T2; availability SLO 99.9%; nomination submission p95 < 2 s; RTO 60 min,
  RPO 15 min (services/crms-svc/README.md).

## 2. Ownership & Boundaries

No TRACEABILITY.yaml exists for this service. Ground truth: docs/architecture/services/
crms.md, services/crms-svc/README.md, services/crms-svc/docs/CAPABILITIES.md.

| M-ID | Module | Implementation |
|---|---|---|
| M12 | crms | crates/crms-domain (candidate, nomination, clearance, ballot, gazette, quota) |
| M45 | party-registration | crates/crms-domain/src/party.rs, coalition.rs |
| M46 | party-primaries-oversight | crates/crms-domain/src/primaries.rs |
| M47 | nomination-fee-payment | crates/crms-domain/src/payment.rs (py/daraja stub) |
| M67 | party-symbol-registry | crates/crms-domain/src/symbol.rs |

- **Data ownership**: candidates, nominations + timeline, clearance decisions, gazettes,
  ballot proofs, political parties, coalitions, primary results/disputes, fee schedules/
  payments, symbols, domain events (all held in-process — see §5).
- **Upstream**: bvr-svc (voter register extract for eligibility), external KRA/EACC/DCI
  police APIs, Daraja payment gateway. The clearance aggregation pipeline described in
  docs/CAPABILITIES.md §3 ("py/crms/clearance_aggregator.py") does not exist in the tree;
  no HTTP clients to these authorities are present in code (not verified beyond tree scan).
- **Downstream**: inventory-svc (ballot design packages), portal-svc (public candidate
  list), transparency-svc (campaign finance linkage). Event emission via Kafka outbox is
  documented but not wired; events accumulate in an in-memory log only
  (crates/crms-api/src/state.rs:52–57).

## 3. Repository Layout

```
services/crms-svc/
  README.md                 # overview, architecture, key APIs
  Cargo.toml                # workspace
  crates/
    crms-api/               # axum handlers/router/state/main
      src/handlers.rs       # ~50 endpoint handlers (lines 114–929)
      src/router.rs         # full route table
      src/state.rs          # in-memory registries (AppState)
    crms-domain/            # pure domain: candidate.rs, nomination.rs, clearance.rs,
                            # ballot.rs, gazette.rs, party.rs, coalition.rs, primaries.rs,
                            # payment.rs, symbol.rs, quota.rs, events.rs
  data_dictionary/          # empty directory (gitkeep-equivalent)
  data_dictionary.yaml      # (referenced in README; not present at root — see gaps)
  django/crms-admin/        # README-only stub
  ex/party_portal/          # README-only stub
  py/daraja/                # README-only stub
  migrations/               # empty (.gitkeep)
  deploy/service.yaml       # kustomize descriptor (blue-green primary/mirror)
  docs/CAPABILITIES.md      # capability catalogue incl. algorithms
  tests/ci/                 # CI integration test hooks
```

## 4. Capabilities & Functions

All routes from crates/crms-api/src/router.rs (single `build_router` function). Handlers in
crates/crms-api/src/handlers.rs. `GET /health` returns `{"status":"ok"}`
(crates/crms-api/src/router.rs:11).

### Candidates (M12)

| Method | Path | Handler (handlers.rs line) | Purpose |
|---|---|---|---|
| GET/POST | /api/v1/crms/candidates | `list_candidates` :137 / `create_candidate` :114 | Registry + creation |
| GET | /api/v1/crms/candidates/{id} | `get_candidate` :150 | Detail |
| POST | .../{id}/submit | `submit_candidate` :164 | Draft → Submitted |
| POST | .../{id}/docs-uploaded | `mark_docs_uploaded` :178 | Document checkpoint |
| POST | .../{id}/ro-review | `begin_ro_review` :185 | Returning-officer review |
| POST | .../{id}/clear / reject | `clear_candidate` :192 / `reject_candidate` :206 | Decision |
| POST | .../{id}/gazette | `gazette_candidate` :220 | Gazette publication link |
| POST | .../{id}/withdraw | `withdraw_candidate` :234 | Withdrawal |

State machine lives in crates/crms-domain/src/nomination.rs (`NominationStatus` :10) and
candidate lifecycle in crates/crms-domain/src/candidate.rs (`CandidateStatus` :33).

### Nominations & clearance

POST /api/v1/crms/nominations (`create_nomination` :245), GET
/api/v1/crms/nominations/{id} (:266), POST .../documents (`add_document` :280);
GET/POST /api/v1/crms/clearances (`list_clearances` :323 / `create_clearance` :302),
POST /api/v1/crms/clearances/advisory (`advisory_clearance` :335).
Aggregate rule: `ClearanceDecision::overall_status` requires all three authorities clear
(crates/crms-domain/src/clearance.rs:27; docs/CAPABILITIES.md §3.4).

### Quota engine

POST /api/v1/crms/quota/evaluate (`evaluate_quota_rule` :352); rules in
crates/crms-domain/src/quota.rs (`QuotaStatus` :24).

### Gazette

POST /api/v1/crms/gazettes (`build_gazette` :377), GET {id} (:396), publish (:410),
CSV export (`gazette_csv` :430), signed JSON (`gazette_signed_json` :444).

### Ballot proofs

POST /api/v1/crms/ballots (`build_ballot` :464), GET {id} (:482), party sign-off (:496),
approve (`approve_ballot` :508).

### Parties, coalitions, primaries (M45/M46)

Parties: list/create/get/submit/approve/suspend/deregister (handlers.rs :533–646);
domain logic in crates/crms-domain/src/party.rs (incl. `AnnualReturn` :110).
Coalitions: create (:648), confirm-member (:661), dissolve (:686).
Primaries: submit result (:710), get (:737), publish (:751), disputes (:763/:782);
domain in primaries.rs (`PrimaryResult` :26, `PrimaryDispute` :93).

### Payments (M47)

GET/POST /api/v1/crms/fee-schedules (:806/:793), POST /payments (`record_payment` :813),
POST /payments/{id}/confirm (`confirm_payment` :826). Domain: payment.rs
(`FeeSchedule` :13, `FeePayment` :59, `PaymentStatus` :44).

### Symbols (M67)

GET/POST /api/v1/crms/symbols (:882/:860), approve/reject (:893/:913);
domain in symbol.rs (`PartySymbol` :27).

### Events feed

GET /api/v1/crms/events returns the accumulated `DomainEvent` log
(`list_events` :929; event types in crates/crms-domain/src/events.rs `EventType` :13).

## 5. Data Model

**There are no migrations** — services/crms-svc/migrations/ contains only .gitkeep. The
API is backed entirely by process-local `Mutex<HashMap>` registries
(crates/crms-api/src/state.rs:31–45): candidates, nominations, clearances (Vec), gazettes,
ballots, parties, coalitions, primary_results, disputes (Vec), fee_schedules (Vec),
payments, symbols, events (Vec). state.rs comments state that "a real deployment replaces
these registries with the CRMS persistence layer; the handler surface is unchanged".

Logical tables documented (not implemented): nominations + nomination_timeline,
clearance_decisions, ballot_proofs, political_parties, primaries sub-schema,
nomination_fees, party_symbols (docs/CAPABILITIES.md §1).

Events produced (documented in docs/architecture/services/crms.md):
`eel.crms.candidate.registered/cleared`, `eel.crms.party.registered`,
`eel.crms.primary.result.certified`, `eel.crms.ballot_design.approved`; consumed:
`eel.bvr.voter.registered`. In code, events are appended to the in-memory log only.

## 6. Operation

- Build/run: `cargo run -p crms-api` from services/crms-svc (workspace Cargo.toml);
  env vars: `HTTP_ADDR` (default 0.0.0.0:8080), `RUST_LOG`.
- Shared server stack: `eel_api_server_rs::ApiServer::builder()
  .otel_service_name("crms-api").bind_addr(http_addr)` (crates/crms-api/src/main.rs:25–29)
  layers JWKS auth, load-shedding, idempotency, content negotiation, request-id, OTel
  tracing (packages/eel-api-server-rs/src/builder.rs).
- Test: `cargo test -p crms-api -p crms-domain`; integration hooks under tests/ci.
- Deploy: services/crms-svc/deploy/service.yaml — blue-green on primary/mirror, rolling on
  5 edge sites, replicas 2–3.
- Loadtest: services/crms-svc/loadtest/ (k6 scenario referenced by README).
- Degradation modes (documented, docs/architecture/services/crms.md): KRA/EACC failures
  queue with manual override; Daraja callback deduplication; ballot approval blocked when
  HSM unavailable. Circuit breaker for EACC 5xx documented in docs/CAPABILITIES.md §3.2 —
  not implemented in current code.

## 7. Safeguards & Security

- **Auth posture**: every request passes the shared `eel-api-server-rs` stack — JWKS JWT
  verification built unconditionally into `ApiServer` (packages/eel-api-server-rs/src/
  builder.rs:93 uses `eel_auth_rs::JwksVerifier`). Per-route role/permission scoping
  (e.g. who may approve a party) is not visible in the router; enforcement level beyond
  "authenticated principal" is (not verified).
- **Idempotency**: `IdempotencyStore` with 86,400 s default TTL is layered by the shared
  builder (packages/eel-api-server-rs/src/builder.rs:30,:94).
- **Rate limiting / load shedding**: load-shedding middleware included in ApiServer
  builder (main.rs doc comment, packages/eel-api-server-rs).
- **PII handling**: candidate records include personal data (names, gender enum
  candidate.rs:24); no field-level encryption or hashing applied in the domain crate;
  data_dictionary.yaml referenced by README was not found in the service tree.
- **Audit logging**: `record_event` appends `DomainEvent`s for ECC/telemetry consumers
  (state.rs:52); the `/events` endpoint exposes them. No tamper-evident signing of the log.
- **Validation**: strong typing in the domain layer (CHECK-style enums, typed UUIDs);
  handler-level validation relies on serde deserialization errors returning problem
  responses via eel-api-server-rs problem.rs.
- **Tests**: cargo unit/integration tests in both crates; no security-specific suites found.
- **Known deferrals**: persistence layer absent (in-memory state lost on restart); Django
  admin, LiveView portal, and Python Daraja integration are empty stubs; clearance HTTP
  integrations (KRA/EACC/DCI) not implemented; HSM signing of ballot proofs not implemented;
  migrations directory empty despite README claiming "PostgreSQL schema versioning".

## 8. Observability

- Logging: JSON `tracing_subscriber` with EnvFilter (main.rs:15–17).
- Tracing: OpenTelemetry via ApiServer (`otel_service_name("crms-api")`, main.rs:28).
- Metrics: none exposed directly by crms-svc (relies on platform-level scraping; not found).
- Health: `GET /health` (router.rs:10–12).
