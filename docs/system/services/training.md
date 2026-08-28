# training-svc

## 1. Overview

training-svc runs voter education content management, poll-worker training LMS with
certification, media and observer accreditation with QR badge verification and CRL, and
medical first-response dispatch. Rust axum API (`training-api`) plus four Django/DRF apps
(cms, lms, accreditation, medical), two Flutter mobile apps (offline LMS, badge scanner),
and a k6 loadtest.

- **Runtime**: Rust (axum, single binary `training-api`), Python/Django REST Framework,
  Flutter (apps/lms-offline `eel_lms_offline`, apps/accred-scanner `eel_accred_scanner`).
  No Elixir app exists under services/training-svc.
- **Port**: default `0.0.0.0:3000`, override via `HTTP_ADDR`
  (services/training-svc/crates/training-api/src/main.rs:13).
- **Entrypoints**: crates/training-api/src/main.rs; Django manage.py per app project
  (django/cms/cms_project, django/lms/lms_project, django/accreditation/accreditation_project,
  django/medical/medical_project).
- Tier T2 (99.5%); certificate issuance P99 < 5 s; CRL refresh every 5 min; offline LMS sync
  within 4 h (docs/architecture/services/training.md).

## 2. Ownership & Boundaries

Ground truth: services/training-svc/TRACEABILITY.yaml (WP-5.4) and
docs/architecture/services/training.md.

| M-ID | Module | Component |
|---|---|---|
| M37 | voter-education-cms | django/cms + training-api cms handlers |
| M38 | poll-worker-training-lms | django/lms + training-api lms handlers + apps/lms-offline |
| M42 | media-accreditation-portal | django/accreditation + training-api accreditation handlers |
| M43 | observer-accreditation-portal | django/accreditation |
| M60 | medical-first-response | django/medical + training-api medical handlers |

- **Data ownership**: training_cms_* content tables, training_lms_* course/enrolment/
  certificate/progress tables, training_accred_* organisation/badge/CRL tables,
  training_medical_* incident/dispatch/alert/unit tables.
- **Upstream**: governance-svc (referendum/recall training triggers), ecc-svc (alert context).
- **Downstream**: field-mobile-svc (LMS morning briefing feed), portal-svc (education
  content). External: MCK media validation, HSM certificate signing (documented; no HSM
  client code found in the tree — stub).
- Events per TRACEABILITY.yaml: `eel.training.content_published.v1`,
  `worker_certified.v1`, `media_badge_issued.v1`, etc. Emission wiring not present in the
  Rust binary or Django apps inspected (stub).

## 3. Repository Layout

```
services/training-svc/
  TRACEABILITY.yaml         # M37/M38/M42/M43(+M60) FR/AC/EC counts, events, integrations
  crates/training-api/src/  # router.rs, handlers/{cms,lms,accreditation,medical}.rs, models.rs
  django/
    cms/                    # DRF: content items, versions, fact-checks, distribution events
    lms/                    # DRF: modules, enrollments, certificates, progress
    accreditation/          # DRF: orgs, badges, observers, crl.py, badge.py, audit.py
    medical/                # DRF: incidents, dispatches, cluster alerts, units
  apps/
    lms-offline/            # Flutter offline LMS (M38)
    accred-scanner/         # Flutter QR badge scanner (M42)
  migrations/0001..0004*.sql
  deploy/service.yaml
  loadtest/                 # k6 scenario
```

## 4. Capabilities & Functions

### Rust API (crates/training-api)

Routes in src/router.rs; handlers in src/handlers/{mod,cms,lms,accreditation,medical}.rs.

CMS (M37):

| Method | Path | Handler | Purpose |
|---|---|---|---|
| POST/GET | /v1/training/cms/content | `post_content` (handlers/cms.rs:10) / `get_content_list` :23 | Create/list versioned content |
| GET | .../content/public | `get_content_public` :27 | Public catalogue (11 languages) |
| PATCH | .../content/{id}/fact-check | `submit_fact_check` :31 | Fact-check review |
| POST | .../content/{id}/approve | `approve_content` :39 | Approval workflow |

LMS (M38): POST/GET /v1/training/lms/modules (`post_module` handlers/lms.rs:10 /
`get_modules` :15); assessments (:19); GET certificates/{worker_id} (`get_certificate` :27);
QR verification GET certificates/verify/{qr_token} (`verify_certificate_qr` :39); revocation
(:47); progress (:52); briefing confirmation (:56).

Accreditation (M42/M43): POST accreditations (`submit_media_application`
handlers/accreditation.rs:10); GET badges/{serial}/verify (`verify_badge` :20); GET /crl
(`get_crl` :30 — 5-minute CRL surface); POST accreditations/{id}/revoke (:37); observer
applications (:42); public summary (:55).

Medical (M60): POST/GET incidents (`report_incident` handlers/medical.rs:10 /
`list_incidents` :23); dispatch (:27); resolve (:35); cluster alerts list/resolve
(:40/:44). Incident schema forbids PHI (migrations/0004_training_medical.sql comments,
FR-M60-011).

### Django/DRF apps

- cms_app/urls.py: ViewSets content-items, content-versions, fact-check-reviews,
  distribution-events.
- lms_app/urls.py: modules, enrollments, certificates, progress.
- accreditation_app/urls.py: media-organisations, media-accreditations,
  observer-organisations, observers ViewSets plus function views
  `verify_badge_view` and `crl_view`. Badge serial generation and status resolution:
  badge.py `generate_badge_serial` :10, `verify_badge(serial, crl)` :15; CRL document
  builder `generate_crl()` crl.py:10; audit trail in audit.py.
- medical_app/urls.py: incidents, dispatches, cluster-alerts, units ViewSets.

### Flutter apps

apps/lms-offline (offline course sync + certificate wallet) and apps/accred-scanner
(QR scan → verify endpoint) — capability detail from READMEs; Dart internals not
enumerated line-by-line here.

## 5. Data Model

| Table (migration) | Purpose | Notable columns/constraints |
|---|---|---|
| training_cms_content_items (0001) | Education content | language, status lifecycle |
| training_cms_content_versions (0001) | Immutable versions | version, body ref |
| training_cms_fact_check_reviews (0001) | Fact-check workflow | reviewer, verdict |
| training_cms_distribution_events (0001) | Distribution log | channel |
| training_lms_modules (0002) | Course modules | curriculum refs |
| training_lms_enrollments (0002) | Worker enrolments | worker_id, module_id |
| training_lms_certificates (0002) | HSM-signed certificates | signature column, serial, revoked flag |
| training_lms_progress (0002) | Progress tracking | assessment scores |
| training_accred_media_orgs (0003) | Media organisations | MCK registration |
| training_accred_media_accreditations (0003) | Accreditations | badge_serial, expiry |
| training_accred_badge_revocations (0003) | Revocation list feeding CRL | revoked_at, reason |
| training_accred_audit_entries (0003) | Accreditation audit log | actor, action |
| training_accred_observer_orgs (0003) | Observer organisations | mission linkage |
| training_accred_observers (0003) | Individual observers | org FK |
| training_accred_credential_revocations (0003) | Observer credential CRL | — |
| training_medical_incidents (0004) | Incidents (no PHI by schema comment) | station, category |
| training_medical_dispatch_requests (0004) | Dispatches | unit FK, state |
| training_medical_cluster_alerts (0004) | Cluster alerts | severity, resolved_at |
| training_medical_units (0004) | Medical units | location, readiness |

Events (TRACEABILITY.yaml): emitted eel.training.content_published/expired/archived.v1,
worker_certified.v1, worker_certification_revoked.v1, refresher_completed.v1,
briefing_confirmed.v1, third_fail_triggered.v1, media_badge_issued/revoked.v1,
media_breach_referred.v1 (+ observer equivalents); consumed from governance-svc and ecc-svc.
No producer code found in this service tree (stub).

## 6. Operation

- Build/run: `cargo run -p training-api` (HTTP_ADDR default 3000); each Django project run
  with its own settings (`python manage.py runserver` per cms/lms/accreditation/medical
  project); Flutter apps built with standard flutter tooling.
- Env vars: HTTP_ADDR, RUST_LOG for the Rust API. Database env vars not present in code
  (Django settings inspection not performed — (not verified)).
- Test: `cargo test`; pytest per Django app; flutter test in both apps; k6 in loadtest/.
- Deploy: services/training-svc/deploy/service.yaml (image harbor.eel.internal/eel/training-svc).
- Degradation (documented, docs/architecture/services/training.md): offline LMS queues
  verdicts/certificates locally and syncs within 4 h; CRL served stale from CDN if refresh
  fails.

## 7. Safeguards & Security

- **Auth posture**: the axum router attaches no authentication middleware; all routes
  including certificate revocation (`POST /v1/training/lms/certificates/{id}/revoke`,
  router.rs:74–76) are unauthenticated in code. Django apps use DRF defaults; permission
  classes per ViewSet were not audited (not verified). TRACEABILITY.yaml declares SEC test
  prefixes (M37-SEC etc.) but corresponding suites were not found in tests/.
- **Badge/CRL security**: badge verification is a two-party check — serial against CRL
  (`verify_badge(serial, crl)` badge.py:15 returning active/revoked/unknown status); CRL
  refreshed every 5 minutes per SLA and exposed publicly at `/v1/training/crl`.
- **PII handling**: accreditation data is class A/B (TRACEABILITY.yaml data_classes);
  medical schema explicitly bans PHI fields (0004 migration header, FR-M60-011). Email/name
  hashing patterns used elsewhere in the platform are not visible here (not verified).
- **Audit logging**: dedicated table training_accred_audit_entries (0003) plus audit.py in
  accreditation app.
- **Validation**: DB CHECK constraints on statuses; DRF serializers (serializers.py);
  serde request types in Rust handlers.
- **Idempotency/rate limiting**: none implemented in the Rust router (no shared
  eel-api-server-rs usage).
- **Tests**: cargo smoke-level tests only in crates (tests/ dir empty at service root);
  Django app tests directories exist per app; no security suites found (not verified beyond
  directory listing).
- **Known deferrals**: no HSM client despite "HSM-signed" certificates (signature columns
  exist but signing code absent); event emission not wired; QR token issuance flow for
  `verify_certificate_qr` has no issuing counterpart in code; TRACEABILITY.yaml claims 30–50
  FRs per module while implemented handler surface is one route each for many flows.

## 8. Observability

- Logging: JSON tracing subscriber (main.rs:6–12).
- Tracing/metrics: none wired in the Rust binary; Django telemetry not configured in
  inspected files (not verified).
- Health: `GET /healthz` returns 200 (`health`, crates/training-api/src/handlers/mod.rs:8);
  DRF apps have no dedicated health route found.
