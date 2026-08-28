# ecc-svc

Source of truth: `services/ecc-svc/` (read directly). Cross-checked against `docs/architecture/services/ecc.md`, `services/ecc-svc/README.md`, and `services/ecc-svc/docs/CAPABILITIES.md`. Code is authoritative; discrepancies flagged. Note: there is no `TRACEABILITY.yaml` at the service root (unlike transparency-svc/bvi-svc).

## 1. Overview

ecc-svc is the Election Command Centre: operational dashboards for Commission staff, Section 44A dual-officer verification, IAB restricted views, grievance intake across channels, telemetry query tooling, and threat-intel correlation. The implemented core is `ecc-api` (Rust/Axum) serving ECC/IAB/grievance/SSE routes plus speech-integrity (M17) persistence; `telemetry-query` provides a validated DSL query engine with evidence export; `threat-intel` ingests STIX/TAXII feeds and correlates ECC flags. Phoenix LiveView apps (`ex/ecc_live`, `ex/iab_live`) render operator dashboards; a Django admin (`django/ecc-admin`) exposes telemetry models; `py/grievance-channels` implements the grievance case lifecycle library.

- Runtime languages: Rust, Elixir (Phoenix LiveView), Python (Django + grievance service library).
- Entrypoints: `crates/ecc-api/src/main.rs`; `ex/ecc_live/mix.exs` (`lib/ecc_live/application.ex`); `ex/iab_live/mix.exs`; `django/ecc-admin/manage.py`.
- Listen ports:
  - ecc-api: `0.0.0.0:8084` via `HTTP_ADDR` (`crates/ecc-api/src/main.rs:13`)
  - ecc_live: port 4010 dev/prod default, 4013 test (`ex/ecc_live/config/dev.exs:4`, `runtime.exs`)
  - iab_live / bvv-style ports: not configured beyond endpoint scaffolding (no router shipped — see Known deferrals).

## 2. Ownership & Boundaries

From `docs/architecture/services/ecc.md` and `docs/CAPABILITIES.md`: owns M16 (election-command-centre), M15 (iab-dashboard), M17 contributions (speech-integrity persistence lives here), M23 (grievance-channels). Tier T2, SLO 99.9%, alert delivery < 5 s, grievance ack < 10 s, RTO 1 h, RPO 2 min.

Data ownership rules:

- Owns `request_fingerprints`, speech-integrity tables, and `s44a_confirmations` in PostgreSQL (migrations V1-V2). The in-memory `MemStore` mirrors identical semantics for tests.
- Grievance case state in the Python library is an in-process dict ("replace with DB-backed repo in production" — `py/grievance-channels/src/grievance_channels/service.py:24`).
- Telemetry query results are audited to `eel_telemetry_queries` via `crates/telemetry-query/src/audit.rs`.

Dependencies (from architecture README; most integrations are not visible in code):

| Direction | Service | Basis |
|---|---|---|
| Upstream | rts-svc, bvi-svc, queue-svc, ai-platform-svc, inventory-svc | README event list (tally updates, turnout, disenfranchisement flags, anomaly alerts, kit telemetry) |
| Downstream | ECC/IAB operators, grievance officers, dispatch staff | Dashboards |
| External | TAXII threat feeds | `crates/threat-intel/src/feed.rs:32` `default_endpoints` |
| External | LiteLLM (NL→DSL bridge) | `EEL_LITELLM_URL`, `EEL_TELEMETRY_NL_MODEL` env vars |
| Internal | SpiceDB (permissions), eel-auth-rs | `crates/telemetry-query/src/permission.rs` (stubbed) |

Events declared in docs but with no producer/consumer code found: `ecc.alert.raised`, `ecc.s44a.triggered`, `ecc.grievance.filed`, `ecc.iab.escalated`, and all consumed topics. The Python library logs pseudo-events (`grievance_filed.v1`, `grievance_assigned.v1`) as log lines only.

## 3. Repository Layout

```
services/ecc-svc/
  Cargo.toml               workspace manifest
  README.md                Service overview, APIs, events (partly aspirational)
  crates/
    ecc-api/               REST API + SSE stubs + speech-integrity/44A store
      src/router.rs        Full route table
      src/handlers.rs      Handlers (many are stubs returning fixture data)
      src/store.rs         Store enum {Pg, Mem} for speech-integrity + 44A + fingerprints
      src/problem.rs       RFC 9457 problem+json builder
      src/sse.rs           Party/observer/journalist SSE endpoints
      tests/smoke.rs       Smoke tests
    telemetry-query/       Telemetry query DSL: validator, planner, executor, presets,
                           NL bridge (mock + LiteLLM), permission checks, audit, evidence bundles, metrics
    threat-intel/          STIX/TAXII feed manager, correlator, Grafana export, REST router
  ex/
    ecc_live/              Phoenix LiveView operator dashboard (8 live views)
    iab_live/              IAB restricted dashboard skeleton (endpoint only; no router/LiveViews)
  django/ecc-admin/        Django project with telemetry app (admin/views/tests)
  py/grievance-channels/   Grievance lifecycle library: service, sla, evidence, referrals,
                           whistleblower (Fernet-encrypted contacts), ussd, whatsapp
  migrations/
    V1__speech_integrity.sql     request_fingerprints + speech tables
    V2__s44a_confirmations.sql   Section 44A dual-officer confirmations
  deploy/service.yaml      WP-1.9 deployment template (7 sites)
  docs/CAPABILITIES.md     Capability catalogue (M15/M16/M23 algorithms and flows)
  loadtest/, tests/        Empty directories
  src/, .omc/              Placeholder dirs
```

## 4. Capabilities & Functions

### 4.1 REST API (ecc-api, port 8084)

Router: `crates/ecc-api/src/router.rs:8` `build_router`.

**Health**
- GET `/health` — `handlers.rs:8` `health`, plain 200.

**ECC situational awareness** (handlers currently return fixtures/stubs):
- GET `/ecc/situational-awareness` — `handlers.rs:12` `get_situational_awareness`: hardcoded zeroed summary `{total_stations, reporting_stations, pending_escalations, active_alerts, turnout_pct, as_of}`.
- GET `/ecc/stations` — `handlers.rs:24`, returns empty list.
- GET `/ecc/stations/{station_code}` — `handlers.rs:28` `get_station`: echoes code with "stub" fields.
- GET+POST `/ecc/escalations` — `handlers.rs:40,44` (`list_escalations` empty; `create_escalation` accepts `EscalationRequest{station_code,...}`, returns 201 `{escalation_id, status:"open"}` without persisting).
- GET+PATCH `/ecc/escalations/{escalation_id}` — `handlers.rs:52,65`: GET returns stub record; PATCH echoes status.
- GET `/ecc/turnout-heatmap`, `/ecc/audit-log` — `handlers.rs:70,74`: empty entries.

**Section 44A dual-officer verification** (fully persisted):
- POST `/ecc/s44a/officer1` — `handlers.rs:210` `create_s44a_officer1`: inserts `S44aRow` (county/station/officer1/timestamp, status `officer1_confirmed`); 409 if station already confirmed (`store.rs:537` `insert_s44a_officer1`).
- POST `/ecc/s44a/officer2` — `handlers.rs:243` `complete_s44a_officer2`: transitions to `complete`; 404 when no officer1 row, 409 when not awaiting officer2 (`store.rs:571` `complete_s44a_officer2`).
- Store also offers `list_s44a_by_county` (`store.rs:609`) and `s44a_county_status` (`store.rs:637`) — implemented but no routes expose them yet.

**IAB** (stubs): GET+POST `/iab/audit-reports` (`handlers.rs:80,84`), GET `/iab/audit-reports/{report_id}` (:92), GET `/iab/dashboards/summary` (:102 zeroed summary), GET `/iab/audit-trails/{module_code}` (:111), GET+POST `/iab/flags` (:116,120). None persist.

**Grievances** (stubs over HTTP; real logic in Python library): GET+POST `/grievance/complaints` (`handlers.rs:130,134` — creates `GRV-XXXXXXXX` reference codes without persisting), GET `/grievance/complaints/{reference_code}` (:144 stub), GET+POST `/grievance/complaints/{complaint_id}/updates` (:156,160), POST `/grievance/ussd/session` (:171 — fixed welcome menu response), POST `/grievance/whatsapp/webhook` (:180 — accepts payload, returns 200).

**SSE portals**: GET `/ecc/events/party|observer|journalist` — `sse.rs:15-25`, each returns a single `data: {"portal":"...","status":"connected"}` frame then closes (stub streams, not persistent connections).

### 4.2 Speech integrity store (M17, behind the same Store enum)

Not exposed by the current router, but fully implemented in `crates/ecc-api/src/store.rs`: `insert_media_item` (:170, dedup on url_sha256 → 409-class error), `fetch_media_item` (:207), `list_media_items` (:237, dynamic bound filters + total count), `fetch_findings` (:331), `record_adjudication` (:354, row-locked transaction, one adjudication per item, status transition), `fetch_adjudication` (:403), `summary_counts` (:428, by status/category/verdict/referral agency within date window). These back the speech-integrity workflow described in migration V1 (queued → flagged/cleared; verdicts harmful/borderline/cleared; referrals NPS/DPP/KNCHR).

### 4.3 Telemetry query API (crates/telemetry-query)

Router `src/api.rs:51` `router` (mounted by nothing in-repo — library):

| Route | Handler group | Purpose |
|---|---|---|
| GET `/v1/telemetry/devices/{device_id}/timeline` | `api.rs` timeline_handler | Device event timeline over a time range |
| POST `/v1/telemetry/query` | structured_query_handler | Executes JSON DSL queries (validate → plan → execute) |
| POST `/v1/telemetry/query/nl` | nl_query_handler | NL→DSL via MockBridge or LiteLLM (`nl_bridge.rs:65,116`; env `EEL_LITELLM_URL`, `EEL_TELEMETRY_NL_MODEL`) with user confirmation step |
| GET `/v1/telemetry/geo/near` | geo_near_handler | Geo-proximity device search (GeoPolygon support in DSL) |
| GET `/v1/telemetry/officers/{officer_id}/sessions` | officer_sessions_handler | Officer session history |
| GET `/v1/telemetry/drops` | drops_handler | Signal-drop aggregations |
| GET `/v1/telemetry/anomalies` | anomalies_handler | Anomaly listing |
| GET `/v1/telemetry/presets` | list_presets_handler | Preset library (`preset.rs:34` `PresetLibrary::list`) |
| POST `/v1/telemetry/presets/{name}/run` | run_preset_handler | Run preset with params |
| POST `/v1/telemetry/queries/{query_id}/export` | export_handler | Evidence bundle generation (`evidence_bundle.rs:88`) |

Pipeline modules: `dsl.rs` (SignalClass/Field/aggregation grammar), `validator.rs:42` `validate`, `planner.rs:50` `plan` (cost-model scan ordering), `executor.rs` (sqlx execution), `metrics.rs` (`queries_total`, `evidence_bundles_total`, `nl_translations_total`, `query_duration_seconds` Prometheus collectors).

### 4.4 Threat intel (crates/threat-intel)

Router `src/api.rs:132`:

| Route | Handler | Purpose |
|---|---|---|
| GET+POST `/v1/threat-intel/feeds` | list_feeds / add_feed | Manage TAXII endpoints (`feed.rs:24` FeedManager) |
| DELETE `/v1/threat-intel/feeds/:label` | remove_feed | Remove an endpoint |
| GET `/v1/threat-intel/matches` | list_matches | Paginated correlation matches |
| POST `/v1/threat-intel/correlate` | correlate_flag (`api.rs:128`) | Correlate an `EccFlag` against STIX indicators (`correlator.rs:37` `correlate`, batch variant :76) |

Supporting: STIX object model + parser (`stix.rs`), Grafana panel data + notify payloads (`dashboard_export.rs:16,59`), background refresh loop (`feed.rs:100` `refresh_loop`). Like telemetry-query, this router is not mounted by the ecc-api binary.

### 4.5 Phoenix LiveView dashboards (ex/ecc_live)

Router `ex/ecc_live/lib/ecc_live/router.ex:17` (browser pipeline; roles noted in comment: chair/commissioner/director/analyst/observer — no enforcement in code):

| Route | Module | Behaviour |
|---|---|---|
| `/dashboard` | `live/dashboard_live.ex` | National summary from ecc-api `/ecc/situational-awareness`; 30 s auto-refresh; push via `{:dashboard_update, payload}` |
| `/alerts` | `live/alerts_live.ex` | Alert feed mapped to ecc-api escalations; "dispatch" acknowledges escalation via PATCH `/ecc/escalations/{id}` |
| `/s44a` | `live/s44a_live.ex` | Dual-officer flow calling ecc-api `/ecc/s44a/officer1|officer2` |
| `/kits` | `live/kits_live.ex` | Kit telemetry heatmap from `/ecc/stations`; 30 s refresh; `{:kit_update, kit}` push |
| `/cover-tamper` | `lib/ecc_live_web/live/cover_tamper_live.ex` | RFID cover-tamper alerts on PubSub topic `eel:inventory:cover_tamper`; 15 s refresh; severity triple-encoded (WCAG AA/AAA) |
| `/cover-tamper/history` | `cover_history_live.ex` | Per-tag state-transition timeline; verifies prev_hash chain in memory |
| `/telemetry` | `telemetry_live.ex` | Real-time mobile telemetry panels (signals/min, alerts, top devices/officers, geo heatmap, battery-low fleet, session-hijack alerts); PubSub topic `eel:telemetry:alert`; 15 s refresh |
| `/telemetry/query` | `telemetry_query_live.ex` | Interactive DSL/NL query builder hitting the telemetry-query API; preset selection; per-column filters; exports; logs actor SVID |
| GET `/health` (JSON) | `controllers/health_controller.ex` | Health probe |

### 4.6 IAB LiveView app (ex/iab_live)

Skeleton only: `application.ex` + `endpoint.ex` (session options, LiveView socket at `/live`). The README documents `IABDashboardLive`, `ComplaintLive`, `ReportLive`, `HardwareKeyAuthLive`, a Broadway consumer on `eel.ecc.iab-escalations`, and an Oban report queue — none of these files exist. `endpoint.ex` plugs `IabLive.Router` which is absent, so the app cannot serve requests as-is.

### 4.7 Grievance channels library (py/grievance-channels)

- `service.py:23` `GrievanceService` — `file_case` (channel ∈ ussd/sms/whatsapp/web/kiosk/api; severity low..critical; SHA-256 description hash; SLA deadline via `sla.py` SlaRouter), `assign_case`, `escalate_case`; logs `grievance_*_v1` markers.
- `whistleblower.py:19` `WhistleblowerService` — Fernet-encrypted contact storage, protection levels standard/enhanced/high.
- `evidence.py` EvidencePackager, `referrals.py` ReferralEmitter, `ussd.py`/`whatsapp.py` channel adapters (menu flows).

### 4.8 Django admin (django/ecc-admin)

Project `ecc_admin` with app `apps.telemetry`: models/admin/views/urls plus `test_admin.py`. Serves the telemetry admin console described in the module docs; run via `manage.py runserver` / gunicorn.

## 5. Data Model

Migrations:

| Table | Key columns | Purpose |
|---|---|---|
| `request_fingerprints` (V1) | fingerprint_id PK, UNIQUE (operation, idempotency_key), request_sha256 CHAR(64), response_status/body | ADR-0011 idempotency ledger, 24 h replay window |
| `speech_media_items` (V1) | item_id PK, url ≤2048, url_sha256 UNIQUE, media_type CHECK(audio/video/text), source_platform ≤64, status CHECK(queued/processing/flagged/cleared) | Speech-integrity intake (M17) |
| `speech_analysis_findings` (V1) | finding_id PK, FK item_id, category CHECK(hate_speech/incitement/misinformation), confidence [0,1], excerpt ≤1000 | AI analysis findings |
| `speech_adjudications` (V1) | adjudication_id PK, item_id UNIQUE FK, verdict CHECK(harmful/borderline/cleared), grounds 10..4000, referral_agency CHECK(NPS/DPP/KNCHR/none), analyst_notes ≤4000 | Human adjudication, one per item |
| `s44a_confirmations` (V2) | confirmation_id PK, county_code, station_code UNIQUE, officer1/officer2 ids + timestamps, status CHECK(officer1_confirmed/complete) | Dual-officer verification |

Store abstraction: `Store` enum (`crates/ecc-api/src/store.rs:106`) over `Pg(PgPool)` and `Mem(MemStore)` — the Mem backend replicates uniqueness semantics (duplicate media items, single adjudication, 44A station conflicts, awaiting-officer2 errors).

Telemetry-query persists audit rows to `eel_telemetry_queries` (schema defined outside these migrations — `audit.rs` insert path; table DDL not present in this service, not verified).

Event schemas/topics: none produced or consumed in code; see section 2 for doc-declared lists.

## 6. Operation

Build/test:

```bash
cargo build -p ecc-api -p telemetry-query -p threat-intel     # from services/ecc-svc
cargo test -p ecc-api -p telemetry-query -p threat-intel
cd ex/ecc_live && mix deps.get && mix test
cd django/ecc-admin && uv sync && pytest                       # per Django conventions; venv present
```

Run:

```bash
DATABASE_URL=postgres://... HTTP_ADDR=0.0.0.0:8084 cargo run -p ecc-api
# DATABASE_URL unset -> automatic in-memory Mem store (main.rs:25-29), useful for demos/tests
cd ex/ecc_live && mix phx.server    # :4010
```

Environment variables (grep of env usage):

| Variable | Consumer | Default |
|---|---|---|
| `HTTP_ADDR` | ecc-api main (`main.rs:13`) | 0.0.0.0:8084 |
| `DATABASE_URL` | ecc-api main | unset → in-memory store |
| `RUST_LOG` | EnvFilter with `ecc_api=info` directive | info |
| `PORT`, `SECRET_KEY_BASE`, `DB_URL`, `POOL_SIZE` | ecc_live runtime.exs (prod) | PORT 4010, POOL_SIZE 10 |
| `EEL_LITELLM_URL`, `EEL_TELEMETRY_NL_MODEL` | telemetry-query NL bridge (`nl_bridge.rs:123` from_env) | mock bridge fallback |

Deployment: `deploy/service.yaml` — image `harbor.eel.internal/eel/ecc-svc`, blue-green primary/mirror, rolling edge sites, replicas 3/3/2×5. No health paths declared.

Known deferrals:

- Most ECC/IAB/grievance HTTP handlers return fixture "stub" payloads and do not touch the store (`crates/ecc-api/src/handlers.rs:10-182`); only health and the two S44A routes are backed by persistence.
- SSE portals are one-shot stub responses, not event streams (`sse.rs:4` `sse_stub_response`).
- Speech-integrity store methods exist but no routes expose them.
- `ex/iab_live` cannot boot its HTTP surface (missing router/LiveViews despite README).
- `loadtest/` and root `tests/` directories are empty; `py` NLP triage described in the architecture README (sentiment/urgency scoring) is not present beyond the case-lifecycle library.
- No Kafka consumers/producers implemented despite extensive topic lists in docs.
- telemetry-query SpiceDB permission checks are stubs that allow any non-empty actor (`permission.rs:check_role` comment "Stub ... allow all").

## 7. Safeguards & Security

Verified in code:

- Problem+json error surface: `crates/ecc-api/src/problem.rs:20` `problem()` builds RFC 9457 documents with base URI `https://eel.local/problems/ecc-api` and a `eel_trace_id` (UUIDv7) per error; helpers `bad_request/not_found/conflict/unprocessable/internal` (:39-69). Note: current handlers mostly bypass it (plain Json responses); it is wired for future use.
- Idempotency plumbing: `request_fingerprints` table + `store.rs:115` `lookup_fingerprint` / `store.rs:132` `save_fingerprint` (Pg and Mem parity). Not yet enforced inside any handler.
- Uniqueness/conflict guards (DB + Mem): duplicate media ingestion rejected on url_sha256 (`store.rs:190-192`); double adjudication prevented via `SELECT ... FOR UPDATE` then existence check (`store.rs:359-366`); 44A station-level conflict and state-machine guards (`store.rs:554-556`, `store.rs:598-600`).
- SQL injection safety: dynamic filter composition always binds parameters (`store.rs:247` comment "every dynamic value is bound, never inlined"); same pattern throughout PgStore and telemetry-query executor.
- Validation rules: DB CHECK constraints enumerated in section 5 (URL length, media types, categories, confidence bounds, grounds length, agency allow-list, 44A status machine).
- Role-based access control design: telemetry-query defines `TELEMETRY_ANALYST` / `TELEMETRY_ADMIN` roles with guards `require_analyst`/`require_admin` (`permission.rs`), actor extracted from `X-Actor-Svid` header (`api.rs` `actor_svid`) — implementation is a documented stub.
- Audit logging of queries: every executed query recorded with actor SVID, DSL, row count, result hash, and export flag (`audit.rs:21` QueryAuditRecord; used by the telemetry-query API).
- Whistleblower confidentiality: Fernet encryption of contact data with key intended from eel-secrets (ephemeral key stub) (`py/grievance-channels/src/grievance_channels/whistleblower.py:22-27`).
- Evidence handling: description stored only as SHA-256 hash in grievance cases (`service.py:47`).
- Metrics counters include security-relevant signals (query totals by ok/err, NL translations) in `telemetry-query/src/metrics.rs`.
- Browser hardening on ecc_live: CSRF (`protect_from_forgery`) and secure headers in the browser pipeline (`router.ex`).

Not implemented in code: authentication on ecc-api HTTP routes (no bearer/SVID/JWT checks), rate limiting, signature verification, TLS termination assumptions unverified, timing defences. The README claims "rate limiting (per-channel)" in ecc-api — no such code exists.

Tests verifying safeguards:

- `crates/ecc-api/tests/smoke.rs` — route smoke coverage (~88 #[test]/#[tokio::test] functions across the three crate test suites).
- Store semantics (dedup, single adjudication, 44A conflicts, fingerprint replay) exercised via the Mem store in crate unit tests under `crates/ecc-api/src` and `crates/telemetry-query/tests/query_test.rs`, `crates/threat-intel/tests/threat_intel_test.rs`.
- LiveView behaviour incl. cover-chain verification: `ex/ecc_live/test/ecc_live_web/live/*` (dashboard, kits, alerts, s44a, cover_tamper, cover_history, telemetry, telemetry_query tests).
- Django admin: `django/ecc-admin/ecc_admin/apps/telemetry/tests/test_admin.py`.

## 8. Observability

- Logging: JSON tracing with EnvFilter (`ecc_api=info`) in ecc-api (`main.rs:6-12`); handler entry logging via `info!` on mutations; Elixir `Logger.info/warning` across LiveViews; Python `logging` with structured `key=value` messages in grievance services.
- Metrics: Prometheus collectors registered in telemetry-query (`metrics.rs`: `queries_total`, `evidence_bundles_total`, `nl_translations_total`, `query_duration_seconds` histograms) — no scrape endpoint mounted anywhere in the service. No metrics elsewhere.
- Health: GET `/health` (ecc-api, 200), GET `/health` JSON (ecc_live HealthController).
- Tracing/OpenTelemetry: Plug.Telemetry endpoint instrumentation in iab_live (`endpoint.ex`); no OTel exporters found.
- Notable log events: `get_situational_awareness`, `create_escalation`, `patch_escalation`, `s44a officer1/officer2 confirm`, `dispatching alert`, `dashboard update`, `kit telemetry update`, `grievance.filed/assigned`, `whistleblower.register`.
