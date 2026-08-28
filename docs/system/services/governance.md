# governance-svc

## 1. Overview

governance-svc implements post-election legal, audit, and governance workflows: risk-limiting
audits (RLA/SHANGRLA), election-court evidence bundles, transmission legal-basis auditing,
parliamentary/judiciary reporting, OAG engagements, recall petitions, referendum mechanics,
by-election operations, county/national coordination, international observer coordination,
post-election review, financial forensics, and a parallel manual count library.

- **Runtime**: Rust (axum/tokio) — one binary per governance module; Python (SHANGRLA
  statistics, forensics ML); Django admin (`django/governance-admin`); Elixir LiveView
  (`ex/observer_workspace`); Tauri desktop app for offline RLA (`apps/rla-desktop`).
- **Ports** (HTTP_ADDR env override; defaults from each `crates/*/src/main.rs:13`):
  rla 8091, court-interface 8092, legal-audit 8093, recall 8094, reporting 8094,
  oag 8095, referendum 8095, by-election 8096, review 8096, county-coord 8097,
  intl-observer 8098, forensics 8099.
- **Entrypoints**: `crates/<module>/src/main.rs` for each of the 12 binaries listed above;
  `apps/rla-desktop/src-tauri/src/main.rs`; `ex/observer_workspace/lib/observer_workspace/application.ex`.

The service is decomposed into independently runnable axum servers rather than one monolith;
each binary builds its own router and serves it directly (no shared `eel-api-server-rs`
middleware stack — unlike crms-svc).

## 2. Ownership & Boundaries

Ground truth: `docs/architecture/services/governance.md`,
`services/governance-svc/TRACEABILITY.yaml`, `services/governance-svc/README.md`.

| M-ID | Module | Crate |
|---|---|---|
| M21 | rla-audit-tool | crates/rla, py/shangrla, apps/rla-desktop |
| M40 | election-court-it-interface | crates/court-interface |
| M41 | post-election-review | crates/review |
| M48 | recall | crates/recall |
| M49 | referendum-mechanics | crates/referendum |
| M50 | by-election-operations | crates/by-election |
| M56 | county-national-coord | crates/county-coord |
| M57 | intl-observer-coord | crates/intl-observer (+ ex/observer_workspace UI) |
| M58 | parliamentary-judiciary-reporting | crates/reporting |
| M59 | oag-interface | crates/oag |
| M62 | transmission-legal-audit | crates/legal-audit |
| M66 | financial-forensics | crates/forensics, py/forensics-ml |

- **Data ownership**: RLA runs/samples/entropy/witnesses, court petitions and evidence
  bundles, legal transmission records and compliance verdicts, statutory reports/testimony/
  judiciary handovers, OAG engagements/extracts/responses, MoUs/datasets/incidents,
  observer missions/briefings/reports, findings/recommendations/referrals, forensic cycles/
  estimates/referrals, recall petitions/signatories, referendum configs/results.
- **Upstream**: rts-svc (transmission events), transparency-svc (reproducibility bundles),
  ecc-svc (grievance referrals), ai-platform-svc (forensics referrals). External: eCourt,
  NIST randomness beacon, OAG.
- **Downstream**: portal-svc (referendum/recall calendar events), training-svc (recall/
  referendum training content).
- Event contracts are documented in `docs/architecture/services/governance.md`
  (`eel.governance.*` emitted on topic `eel.governance`); no Kafka producer is wired in any
  crate's `main.rs` (not verified beyond router/binary inspection).

## 3. Repository Layout

```
services/governance-svc/
  README.md                 # service overview, tier T2, architecture notes
  TRACEABILITY.yaml         # WP-5.1 FR/EC matrix (all entries status: stub)
  data_dictionary.yaml      # PII tiers per column (ADR-0016)
  apps/rla-desktop/src-tauri/src/   # Tauri offline audit laptop (commands.rs, db.rs)
  crates/
    rla/                    # M21 RLA API (router.rs, handlers.rs) + BRAVO engine (engine.rs: PairwiseAssertion, compute_t_statistic, draw_sample_positions)
    court-interface/        # M40 court petitions + evidence bundles
    legal-audit/            # M62 transmission compliance verdicts
    reporting/              # M58 statutory reports, testimony, handovers
    oag/                    # M59 Auditor-General interface
    recall/                 # M48 recall petitions
    referendum/             # M49 referendum configuration/results
    by-election/            # M50 by-election operations
    county-coord/           # M56 MoUs, datasets, SAML ACS
    intl-observer/          # M57 observer missions
    review/                 # M41 findings, recommendations, long-drip
    forensics/              # M66 analysis cycles, referrals
    parallel-manual-count/  # service binary (src/main.rs, :8097) + library: plan/ingest/divergence/tallies API
  django/governance-admin/  # Django project; app urls.py empty
  ex/observer_workspace/    # Phoenix LiveView observer workspace
  py/shangrla/src/shangrla/ # engine.py, nist_beacon.py, iab_seed.py, witness.py
  py/forensics-ml/          # forensics ML models
  migrations/V1..V4*.sql    # rla/court/legal-audit/reporting+oag tables
  deploy/service.yaml       # kustomize deployment descriptor
  loadtest/governance.js    # k6 scenario
```

## 4. Capabilities & Functions

All routes are defined in `crates/*/src/router.rs`; handlers in `crates/*/src/handlers.rs`.
Every module exposes `GET /health`.

### M21 RLA (port 8091)

| Method | Path | Handler | What it does |
|---|---|---|---|
| GET/POST | /rla/audits | `list_audits` / `create_audit` — crates/rla/src/handlers.rs:10/:21 | List/create audit runs |
| GET | /rla/audits/{run_id} | `get_audit` — handlers.rs:16 | Run detail |
| GET | /rla/audits/{run_id}/samples | `list_samples` — handlers.rs:26 | Sampled ballots |
| GET | /rla/audits/{run_id}/samples/{ballot_id} | `get_sample` — handlers.rs:32 | Single sample |
| PATCH | .../samples/{ballot_id}/count | `record_count` — handlers.rs:37 | Record auditor interpretation |
| GET | /rla/audits/{run_id}/report | `get_report` — handlers.rs:42 | Audit report (T-statistic, conclusion) |
| POST | /rla/audits/{run_id}/finalise | `finalise_audit` — handlers.rs:47 | Finalise run / trigger recount |

Python statistics (BRAVO, twin of the Rust engine): `ShangrlaEngine.compute_t_statistic`,
`evaluate_run`, `draw_sample` — without-replacement seeded sampling and pairwise
assertions against the exact-tie null (py/shangrla/src/shangrla/engine.py); witness workflow `WitnessWorkflow.register/
record_signature/list_witnesses` (py/shangrla/src/shangrla/witness.py:10,:20,:33); NIST beacon
with dice fallback (py/shangrla/src/shangrla/nist_beacon.py, EC-GOV-03).

Offline desktop (Tauri): `list_ballots`, `record_verdict`, `pending_sync_count`,
`mark_synced` (apps/rla-desktop/src-tauri/src/commands.rs:27,:45,:71,:79) over local SQLite.

### M40 Court interface (port 8092)

| Method | Path | Handler | What it does |
|---|---|---|---|
| GET/POST | /court/petitions | `list_petitions` / `file_petition` — crates/court-interface/src/handlers.rs:10/:16 | Petition registry |
| GET | /court/petitions/{petition_id} | `get_petition` — handlers.rs:21 | Petition detail |
| POST | .../evidence/request | `request_evidence` — handlers.rs:26 | Request evidence assembly |
| GET | .../evidence/bundle | `get_evidence_bundle` — handlers.rs:31 | 15-item evidence bundle |
| GET/POST | .../witnesses | `list_witnesses` / `register_witness` — handlers.rs:41/:36 | Technical-witness pre-registration |
| POST | .../supplementary | `add_supplementary_request` — handlers.rs:47 | Supplementary request workflow |

### M62 Legal audit (port 8093)

| Method | Path | Handler | What it does |
|---|---|---|---|
| GET/POST | /legal-audit/transmissions | `list_transmissions` / `record_transmission` — crates/legal-audit/src/handlers.rs:10/:16 | Append transmission events |
| GET | .../transmissions/{record_id}/verdict | `get_verdict` — handlers.rs:21 | Per-event §39 verdict |
| GET | /legal-audit/verdicts | `list_verdicts` — handlers.rs:26 | All verdicts |
| GET | /legal-audit/parties/{party_id}/summary | `party_summary` — handlers.rs:32 | Party-agent summary |
| POST | /legal-audit/reports | `generate_report` — handlers.rs:37 | Generate compliance report |

### M48 Recall (port 8094) and M58 Reporting (port 8094 — port collision)

Recall routes in crates/recall/src/router.rs: list/create petitions, signatory import
(`import_signatories`, handlers.rs:34), threshold report (`get_threshold_report`,
handlers.rs:47 — computes against constituency thresholds), close (`close_petition`, :68),
recall-election creation (:73), signatory listing (:87).

Reporting routes in crates/reporting/src/router.rs: statutory reports
(`generate_report`/`get_report`, crates/reporting/src/handlers.rs:16/:21), testimony
workspaces (:26/:31), judiciary handovers (:37), SC IT pre-registration (:42).

### M49 Referendum (8095) and M59 OAG (8095 — port collision)

Referendum routes in crates/referendum/src/router.rs: create/get referendums, activate
(handlers.rs:44), submit county result (:58, validates against TLA+-verified county
threshold logic), certify county result (:82), national result (:89), declare (:105).

OAG routes in crates/oag/src/router.rs: engagements CRUD-lite
(crates/oag/src/handlers.rs:10–33), evidence extract (:26), response recording (:31),
published-report registry (:36/:42).

### M50 By-election (8096) and M41 Review (8096 — port collision)

By-election routes in crates/by-election/src/router.rs: create/get by-elections, register
subset generation (handlers.rs:54), kit plan (:65), activate (:80), cost reports (:86/:109).

Review routes in crates/review/src/router.rs: findings submit/triage
(crates/review/src/handlers.rs:10/:41), recommendations list/sign (:49/:55), public tracker
(:63), parliament referrals (:69/:85), long-drip alerts (:97/:103), next-cycle handoff (:116).

### M56 County coordination (8097)

Routes in crates/county-coord/src/router.rs: MoUs (create/sign, crates/county-coord/src/
handlers.rs:17/:34), county datasets (:45), coordination incidents (:70/:89),
`POST /v1/auth/saml/acs` SAML assertion consumer (:98).

### M57 International observers (8098)

Routes in crates/intl-observer/src/router.rs: missions (crates/intl-observer/src/handlers.rs:7–25),
briefings (:27/:33), observation reports (:49/:65), joint statements (:71/:83).
UI: LiveViews at `/`, `/missions/:id`, `/missions/:id/briefings`, `/missions/:id/reports`,
`/joint-statements` (ex/observer_workspace/lib/observer_workspace_web/router.ex:15–24);
workspace component `MissionWorkspaceLive.mount` (.../live/mission_workspace_live.ex:7).

### M66 Forensics (8099)

Routes in crates/forensics/src/router.rs: analysis cycles (crates/forensics/src/handlers.rs:7–29),
run trigger (:26), estimates (:31), referrals (:37), replication packages (:54),
public reports (:71/:77).

### Parallel manual count (library, not deployed)

`pub fn router(state)` mounts `/v1/parallel-count/plan`, `/stations/:id/submit`,
`/divergence` (crates/parallel-manual-count/src/api.rs:21–67). No binary exists; consumed
via tests or embedding consumers only.

### Django & OpenAPI

Django admin exposes only default `admin/` URLs; `governance_app/urls.py` is an empty list
(django/governance-admin/governance_app/urls.py). OpenAPI contracts live under
packages/openapi-contracts/governance*/openapi.yaml (12 specs).

## 5. Data Model

Central migrations (migrations/, Flyway-style):

| Table | Purpose | Key columns |
|---|---|---|
| rla_audit_runs (V1) | RLA run lifecycle | run_id PK, audit_method CHECK(ballot_polling/comparison/batch_comparison), risk_limit 0..1, status(initialised→escalated), sig |
| rla_entropy_sources (V1) | Audit seed provenance | source_type(dice/beacon/hardware/iab_card), value_hash, witness_sigs JSONB |
| rla_sampled_ballots (V1) | Drawn ballots | station_code, batch_id, ballot_position, discrepancy, verdict, synced flag |
| rla_audit_reports (V1) | Conclusions | conclusion(pass/escalate/recount_triggered), risk_achieved, sig |
| rla_audit_manifests (V1) | Ballot manifests | manifest_hash, total_ballots |
| rla_witness_records (V1) | Witnesses | name, organisation, signature_b64 |
| court_petitions (V2) | Election petitions | petitioner_hash (PII class A — identity stored as hash), status lifecycle, ecourt_ref |
| court_evidence_bundles (V2) | Evidence packages | bundle_version, items JSONB, sig |
| court_technical_witnesses (V2) | Witness preregistration | expertise, ecourt_ref |
| court_supplementary_requests (V2) | Supplementary workflow | requested_by_hash, status |
| legal_transmission_records (V3) | Append-only transmission log | event_id, officer_id_hash, legal_basis_refs JSONB |
| legal_compliance_verdicts (V3) | Per-rule verdicts | rule_id, passed, failure_reason, tlog_hash (Rekor anchoring) |
| legal_rules (V3) | Rule catalogue | section_ref, applies_to, active |
| reporting_statutory_reports (V4) | Assembly/Senate/Judiciary/SC/Committee reports | period range, output_url, sig |
| reporting_testimony_workspaces (V4) | Testimony prep | committee_name, status(draft/finalised/submitted) |
| reporting_judiciary_handovers (V4) | Handover records | handed_to, handover_ref, signed_at |
| reporting_sc_preregistrations (V4) | SC IT registrar preregistration | contact_email_hash |
| oag_engagements (V4) | OAG audits | scope, status(scoping→closed), oag_ref |
| oag_evidence_extracts (V4) | Extract queries | query_hash, row_count, output_url |
| oag_responses (V4) | OAG responses | responder_hash |
| oag_published_reports (V4) | Published report registry | title, published_at, url |

Crate-local migrations (001_initial.sql): county-coord — mous, county_datasets,
ministry_approvals, coordination_incidents; forensics — analysis_cycles, estimate_records,
referral_packets, replication_packages, public_reports; intl-observer — observer_missions,
briefings, observation_reports, joint_statements; review — findings, recommendations,
parliament_referrals, long_drip_alerts. Column-level PII classification:
services/governance-svc/data_dictionary.yaml (tiers A–D).

Events (documented contract, docs/architecture/services/governance.md): produced —
`eel.governance.rla.run.completed`, `eel.governance.evidence_bundle.submitted`,
`eel.governance.referendum.configured`, `eel.governance.recall.threshold.met`,
`eel.governance.transmission.verdict`, `eel.governance.forensics.referral`;
consumed — `eel.rts.form34.published`, `eel.rts.declaration.issued`,
`eel.transparency.bundle.published`, `eel.ecc.grievance.resolved`. Producer/consumer code is
not present in the binaries (stubs).

## 6. Operation

- Build: `cargo build -p eel-governance-rla` etc., or workspace-level `cargo build` from
  repo root (each crate has its own Cargo.toml under services/governance-svc/crates/*).
- Run: `cargo run -p <crate>`; bind address via `HTTP_ADDR` (default per binary, see §1);
  log filter via `RUST_LOG` plus a hard-coded crate directive (e.g. `eel_governance_rla=info`,
  crates/rla/src/main.rs:9).
- Test: `cargo test` per crate (e.g. crates/rla/tests/smoke.rs, crates/recall/tests/smoke.rs);
  pytest suites under py/shangrla/tests and py/forensics-ml/tests.
- Deploy artifact: services/governance-svc/deploy/service.yaml (kustomize; image
  harbor.eel.internal/eel/governance-svc).
- Load test: services/governance-svc/loadtest/governance.js (k6).
- Degradation (documented, docs/architecture/services/governance.md): RLA degrades to
  offline mode with pre-seeded NIST beacon; eCourt failures queue evidence bundles;
  forensics jobs are batch.

## 7. Safeguards & Security

- **Auth posture**: none of the axum routers attach authentication middleware — every route
  including admin-ish endpoints (`/boundary/admin/publish`-style publish/approve/declare)
  is unauthenticated in code. The README describes SAML-federated county access and OAuth2
  mTLS for OAG/CBK, but only a stub `saml_acs` handler exists
  (crates/county-coord/src/handlers.rs:98) that parses a JSON body without signature
  verification. This is the largest gap in the service.
- **Idempotency/rate limiting**: none implemented in routers (no shared eel-api-server-rs
  layer used). Not found in code.
- **PII handling**: petitioner identity stored as hash (`petitioner_hash`, V2 migration;
  FR-M40-002 "class A PII"); SC preregistration stores `contact_email_hash`; officer IDs
  hashed in V3. Column tiers tracked in data_dictionary.yaml.
- **Integrity**: signatures (`sig`) on runs, bundles, reports; entropy value hashes with
  witnessed signatures; verdict hashes anchored to transparency log (`tlog_hash`). HSM signing
  referenced in README/data dictionary but no HSM client code found in crates (stub).
- **Validation**: DB CHECK constraints enforce state machines (status enums, risk_limit
  bounds, entropy source types). Handler-level validation is minimal (serde deserialization).
- **Deadline enforcement**: EC-GOV-02 (reject post-deadline petition with 422) is claimed in
  TRACEABILITY.yaml against crates/court-interface/src/handlers.rs; the handler is a stub
  returning static JSON — claim not verifiable in code.
- **Tests**: smoke tests exist for rla and recall (crates/*/tests/smoke.rs); pytest suites
  for shangrla/forensics-ml; no auth/security tests found (not verified beyond directory scan).
- **Known deferrals**: all requirements and edge cases in services/governance-svc/
  TRACEABILITY.yaml carry `status: stub`; several handlers return hard-coded JSON (e.g.
  crates/oag/src/handlers.rs:16, crates/reporting/src/handlers.rs:16); port collisions
  (recall/reporting on 8094, oag/referendum on 8095, by-election/review on 8096) make the
  defaults unusable concurrently.

## 8. Observability

- Logging: `tracing_subscriber` with JSON output in every binary
  (e.g. crates/rla/src/main.rs:6–11).
- Tracing/metrics: none wired (no OTel exporter or metrics layer found in crates).
- Health: `GET /health` on every module router (returns plain 200 via `health()` handler,
  e.g. crates/rla/src/handlers.rs:6). The Elixir workspace adds its own health controller
  (ex/observer_workspace/.../controllers/health_controller.ex). Readiness probes and Grafana
  dashboards: documented in docs/architecture/services/governance.md but not implemented
  here.
