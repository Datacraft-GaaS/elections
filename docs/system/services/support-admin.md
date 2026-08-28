# support-admin-svc

## 1. Overview

support-admin-svc hosts administrative workflows outside election-day field operations:
special voter facilities and IDP reassignment with s.44A guardrails (M25), ballot intent
preservation (pre-mark validation, spoilt ballots, adjudication catalogue, M27), budget/
procurement/vouchers with CoB and OAG reporting (M54), HR and payroll for election staff
with statutory deductions (M55), and BagIt long-term archival (M69).

- **Runtime**: Rust axum — three independently runnable binaries (`archive`, `budget`,
  `payroll`) plus two library-only domain crates (`intent`, `special-voter`); Django admin
  apps (`django/support-admin`, `django/hr`, `django/finance`).
- **Ports**: archive binds hard-coded `0.0.0.0:8080`
  (crates/archive/src/main.rs:15); payroll defaults to `0.0.0.0:8080` via `LISTEN_ADDR`
  (crates/payroll/src/main.rs:14); budget reads `HTTP_ADDR`
  (crates/budget/src/main.rs:10). Default 8080 collides between archive and payroll — run
  with distinct overrides.
- **Entrypoints**: crates/archive/src/main.rs, crates/budget/src/main.rs,
  crates/payroll/src/main.rs. `intent` and `special-voter` expose no binary.
- No TRACEABILITY.yaml exists for this service; ground truth is
  docs/architecture/services/support-admin.md.

## 2. Ownership & Boundaries

From docs/architecture/services/support-admin.md:

| M-ID | Module | Implementation |
|---|---|---|
| M25 | special-voter-workflows | crates/special-voter + migrations/V0001 + django/support-admin |
| M27 | intent-preservation | crates/intent + migrations/V0002 |
| M54 | budget-expenditure | crates/budget + django/finance |
| M55 | hr-payroll | crates/payroll + django/hr |
| M69 | long-term-archive | crates/archive |

- **Data ownership**: special-voter facilities/events/Form-34/IDP reassignments/s44a
  overrides; pre-mark validations, spoilt-ballot events, adjudication events, rejection-rate
  alerts, adjudication toggle audit; budgets/procurement/vouchers; personnel/contracts/
  payroll runs/payslips; archive collections/retrieval requests/ingest jobs.
- **Upstream**: field-mobile-svc (attendance for payroll), ecc-svc (s.44A events),
  governance-svc (OAG evidence requests). External: IFMIS, PPRA, M-Pesa B2C, KRA/NSSF/NHIF,
  KNADS/DataCite. None of these integrations have client code in the tree (stubs).
- **Downstream**: governance-svc (archive access for forensics).

## 3. Repository Layout

```
services/support-admin-svc/
  crates/
    archive/     # BagIt archive API (routes.rs, handlers.rs, domain.rs) — binary
    budget/      # budget/procurement/voucher API — binary
    payroll/     # HR + payroll/disbursement API — binary
    intent/      # library: aggregates/, catalogue.rs, model.rs, events.rs
    special-voter/ # library: aggregates/, guards.rs, workflows.rs, events.rs
  django/
    support-admin/ # M25/M27 admin views (facilities, s44a queue, adjudication catalogue)
    hr/            # DRF ViewSets: applications, contracts, runs, payslips, deductions
    finance/       # DRF ViewSets: budgets, lines, procurements, vouchers, cob-reports
  migrations/V0001__special_voter_schema.sql, V0002__intent_schema.sql
  deploy/service.yaml
  target/          # build artifacts (not source)
```

Note: only V0001/V0002 migrations exist; budget/payroll/archive schemas are not migrated
in this tree (handlers return static JSON).

## 4. Capabilities & Functions

### Archive (M69) — crates/archive/src/routes.rs

Public contract routes (`/api/v1/support/archive/...`, per openapi-contracts/support-archive):

| Method | Path | Handler (handlers.rs line) |
|---|---|---|
| POST/GET | /api/v1/support/archive/collections | `create_archival_collection` :16 / `list_archival_collections` :22 |
| GET | .../collections/{collection_id} | `get_archival_collection` :28 |
| POST/GET | /api/v1/support/archive/retrieval-requests | `create_retrieval_request` :36 / `get_retrieval_request` :42 |

Module routes (`/v1/archive/collections/{id}/...`): ingest start/status (:53/:58),
seal (:65), attest (:70), access request (:75), destroy (:80),
format-migration list/start (:85/:90).

### Budget (M54) — crates/budget/src/routes.rs

v1 workflow routes: POST /v1/budgets (`create_budget`, handlers.rs:11),
PUT /v1/budgets/{id}/approve (:26); procurement (:88); voucher lifecycle create/approve/
counter-approve/pay (:109–144); CoB report get/export (:167/:175).
OpenAPI-aligned routes: /api/v1/support/budget/line-items (:41/:58), expenditures (:67),
reports/summary (:149), reports/oag-export (:159).

### Payroll & HR (M55) — crates/payroll/src/routes.rs

Contract routes: personnel register/list (handlers.rs :20/:25), disbursements initiate +
job status (:34/:40), county payroll report (:49).
HR: applications create/get (:57/:62), contracts create/terminate (:70/:75).
Payroll runs: create/approve/disburse/report/export (:83–104); payslip by staff+period (:108).
M-Pesa B2C disbursement is documented in docs/architecture/services/support-admin.md;
no Daraja client code present (stub).

### Special voter (M25) — library

S.44A guardrails in crates/special-voter/src/guards.rs: dual sign-off validation
(`validate_s44a_dual_signoff` :30), IAB sample-fraction check (:41), deterministic
IAB sample selection from a seed (:54).

### Django admin surfaces

- support_admin/urls.py: facility list/detail, `s44a/` override queue,
  `adjudication/catalogue/` browser (django/support-admin/support_admin/urls.py).
- hr_app/urls.py: DRF routers v1/hr/applications, v1/hr/contracts, v1/payroll/runs,
  v1/payroll/payslips, v1/payroll/deductions.
- finance_app/urls.py: DRF routers under v1/finance/ (budgets, budget-lines,
  procurements, vouchers, cob-reports).

## 5. Data Model

V0001__special_voter_schema.sql: eel_special_voter_facilities (category/county/date
indexes), eel_special_voter_events, eel_special_form34, eel_idp_reassignments,
eel_s44a_overrides.

V0002__intent_schema.sql: eel_premark_validations (station/outcome indexes),
eel_spoilt_ballot_events, eel_adjudication_events (decision/reason_code indexes),
eel_rejection_rate_alerts, eel_adjudication_toggle_audit (append-only toggle audit).

Budget/payroll/archive tables exist only as domain structs (crates/*/src/domain.rs);
persistence and their migrations are not implemented.

Events (documented): produced `eel.support_admin.facility.configured`,
`spoilt_ballot.event` (image purged within 1 s per docs), `payroll.run.completed`,
`archive.ingested`; consumed attendance, s44a, form34 topics. Producer/consumer code not
present in the tree (stub).

## 6. Operation

- Build/run: `cargo run -p <archive|budget|payroll>`; env vars `HTTP_ADDR` (budget),
  `LISTEN_ADDR` (payroll), RUST_LOG. Archive port fixed at 8080.
- Test: `cargo test` per crate; no service-level tests directory exists.
- Deploy: services/support-admin-svc/deploy/service.yaml (template filled with image
  harbor.eel.internal/eel/support-admin-svc; strategy fields follow WP-1.9 pattern used by
  sibling services).
- Degradation modes documented in docs/architecture/services/support-admin.md (IFMIS/PPRA
  outages queue submissions); no code implements the queues.

## 7. Safeguards & Security

- **Auth posture**: none of the three binaries attach auth middleware; every route —
  including destructive ones like `/v1/archive/collections/{id}/destroy` and voucher
  payment — is unauthenticated in code. The Django admin presumably relies on Django's
  session auth, but permission classes on the DRF ViewSets were not audited (not verified).
- **Guardrails**: s.44A override requires dual sign-off and bounded sample fractions
  (guards.rs:30,:41) — enforced as pure functions ready for embedding, but not wired to any
  HTTP surface.
- **Idempotency/rate limiting**: absent in all routers (no shared middleware stack).
- **PII handling**: spoilt-ballot events are designed to purge images within 1 s
  (docs/architecture/services/support-admin.md); schema keeps images out of the table.
  Payroll/personnel data is PII-heavy; no field-level protection implemented.
- **Audit logging**: eel_adjudication_toggle_audit captures adjudication-toggle changes;
  accreditation-style append-only audit patterns otherwise absent.
- **Validation**: CHECK constraints in the two SQL migrations; handler-level bodies are
  largely ignored stubs (`Json(_body)` parameters, payroll/handlers.rs:20–104).
- **Tests**: no test suites found in the service directory.
- **Known deferrals**: handlers in all three binaries return hard-coded JSON (e.g. budget
  create_budget handlers.rs:11–24); persistence layer missing for archive/budget/payroll;
  external integrations (IFMIS, PPRA, M-Pesa B2C, KRA/NSSF/NHIF, KNADS) not implemented;
  intent/special-voter crates lack any HTTP exposure; port default collisions.

## 8. Observability

- Logging: tracing_subscriber fmt in each main (archive main.rs:16 "listening" log,
  budget :14, payroll :15).
- Tracing/metrics: none wired.
- Health endpoints: none defined in any router (no /health route found) — probes must
  target TCP liveness or be added.
