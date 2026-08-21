# training-svc

Training, accreditation, and capacity building service. Implements 5 modules spanning voter education, poll-worker training, media and observer accreditation, and medical first response.

## Service overview

Tier: T3 (99.5% availability). SLO: LMS course completion sync under 30 seconds; accreditation badge issuance under 5 minutes.

Modules:
- M37: Voter Education CMS (multilingual, 11 languages, WhatsApp distribution, fact-check review pipeline)
- M38: Poll-Worker Training LMS (offline Flutter app, QR-code attendance tracking)
- M42: Media Accreditation Portal (MCK partnership, badge printing)
- M43: Observer Accreditation Portal (accreditation review, badge issuance)
- M60: Medical First Response (incident reporting, KRCS/MoH/NDMU dispatch integration)

## Architecture

Single unified Rust service spanning all five modules with separate resource routes (`/training/cms`, `/training/lms`, `/training/media-accreditation`, `/training/observer-accreditation`, `/training/medical`). Django admin for CMS and LMS management. Flutter offline app for poll-worker training and medical incident field reporting. Astro static site for public accreditation portal and course directory. Messaging via Kafka event bus with hybrid logical clock causality.

LMS course completion state is synced via event log: when a learner completes a quiz offline (on Flutter app), completion event queues locally, syncs on next network connection, and fires training.course.completed event for treasury/reconciliation.

## Directory layout

```
services/training-svc/
  apps/                   # Flutter LMS offline app + medical incident reporter
  crates/
    training-api/         # Unified REST API (CMS, LMS, accreditation, medical)
  django/                 # Admin UI: CMS content, LMS enrollment, accreditation review
  docs/                   # Architecture, API reference
  deploy/                 # Kustomize overlays (primary, mirror)
  loadtest/               # k6 scenarios
  migrations/             # PostgreSQL schema versions
  TRACEABILITY.yaml       # M37-M60 edge case matrix
  data_dictionary.yaml    # Column classification (PII types, retention)
```

## Key APIs

OpenAPI spec at `packages/openapi-contracts/training/openapi.yaml`.

Core endpoints:

```
GET /v1/training/courses                    # list courses (filter by language, status)
GET /v1/training/courses/{id}              # fetch course metadata + lessons
POST /v1/training/enrolments                # enroll learner in course
GET /v1/training/enrolments/{id}           # fetch enrollment status
POST /v1/training/enrolments/{id}/complete # mark course complete (offline or online)

GET /v1/training/accreditations/{id}       # fetch accreditation status
POST /v1/training/accreditations           # submit application
POST /v1/training/accreditations/{id}/badge # issue badge (staff only)

POST /v1/training/medical-incidents         # file incident report
GET /v1/training/medical-incidents/{id}    # fetch report + dispatch status
```

All endpoints require valid JWT. Accreditation badge download requires CRCS (Kenyan Red Cross Society) admin role. Medical incident creation requires field staff or KRCS/MoH authorization.

## Key events

Emitted: `training.course.completed`, `training.accreditation.issued`, `training.accreditation.rejected`, `training.medical.incident.reported`, `training.medical.incident.dispatched`.

Consumed: `field-mobile.attendance.checked-in` (for LMS completion cross-check to flag attendance fraud), `crms.party.registered` (for observer organization accreditation eligibility).

## Data model

Schema: `migrations/001_training.sql` onward. Column classification per `data_dictionary.yaml`.

Aggregates: `Course` (multilingual, versioned lessons, media assets), `Enrollment` (state machine: enrolled, in-progress, completed), `Accreditation` (state machine: submitted, under-review, approved, badge-issued), `MedicalIncident` (geolocation, responder dispatch status).

PII classification: `LearnerIdentity` (Restricted, hashed for analytics), `AccreditationApplicant` (Sensitive, stripped in public accreditation register), `MedicalIncidentReporter` (Restricted, visible only to KRCS/MoH/NDMU).

## Building and running

Requires Rust 1.70+ (nix flake.nix pins it), Python 3.12 (uv), Flutter 3.2x.

```bash
cd services/training-svc
cargo build --release

# Run tests
cargo test -p eel-training-api --all-features

# Run load test (requires running service)
k6 run loadtest/training.js

# Build Flutter app
cd apps && flutter build apk

# Run Django admin locally
cd django && uv run python manage.py runserver
```

Check `CLAUDE.md` for test conventions: async tests are plain `async fn`, no `@pytest.mark.asyncio`.

## Testing

Test layout: `tests/ci/` for CI autodiscovery.

Fixtures:
- `test_fixtures::course()` returns a seeded Course with 5 lessons
- `test_fixtures::enrollment()` returns a pre-enrolled Enrollment ready for lesson completion
- `test_fixtures::accreditation()` returns a pre-submitted application ready for review
- pytest-httpserver for stubbing MCK, KRCS APIs

Scenarios:
- LMS: enroll learner, complete lessons offline, sync on reconnection, verify training.course.completed event
- Accreditation: submit application, review and approve, issue badge with QR code
- Medical: file incident, dispatch to nearest KRCS facility, track responder ETA

Run `cargo test --test integration_training -- --include-ignored` for full matrix. Tag long tests with `#[ignore]`.

## Deployment

Kustomize overlays in `deploy/overlays/primary` and `deploy/overlays/mirror`.

Bring-up sequence:
1. Apply PostgreSQL schema: `kubectl apply -f deploy/db/schema-training.sql`
2. Deploy services: `kubectl apply -k deploy/overlays/primary`
3. Verify readiness: `kubectl rollout status deployment/training-api -n eel-training`

RTO/RPO: 15 minutes (Kafka consumer resumption + snapshot replay).

## Observability

Metrics (Prometheus):
- `training_course_enrollments{course_id, language}` (gauge, active learners)
- `training_course_completion_latency_ms` (histogram, offline sync to training.course.completed event)
- `training_accreditation_badge_print_latency_ms` (histogram, approval to QR badge ready)
- `training_medical_incident_dispatch_latency_ms` (histogram, report to responder dispatch)

Traces (Jaeger): every enrollment, accreditation review, and medical dispatch tagged with W3C baggage `election_id`, `training_module_id`.

Logs (Loki): structured JSON, fields `event_type`, `learner_id` (hashed), `course_id`, `duration_ms`.

SLOs (Grafana):
- LMS: 99.5% of course completions synced within 30 sec
- Accreditation: 99.5% of badge issuances within 5 min
- Medical: 99.5% of incident dispatches within 10 min

Dashboard: `http://localhost:3000/d/training-svc/`.

## Runbooks

Degradation modes: `docs/runbooks/training-degradation.md`.

Chaos scenarios: `docs/runbooks/training-chaos.md`.

Common incidents:
- LMS offline app sync backlog: trigger batch replay on wifi reconnection; upgrade max batch size
- Accreditation badge printer offline: buffer requests locally; retry on printer recovery
- Medical dispatch unavailable: escalate to manual routing via SMS to field coordinators

## Degradation contract

**Normal**: all workflows operational; live accreditation review, badge printing, medical dispatch.

**Degraded**: accreditation review queued (service read-only); medical dispatch read-only (manual escalation only).

**Read-only**: serve cached course content; all enrollments and accreditations rejected with 503 Service Unavailable.

## Chaos scenarios

Scenarios in `docs/runbooks/training-chaos.md`:

1. LMS offline app storage full: simulate local SQLite DB quota breach, verify graceful queue-to-file fallback
2. Accreditation badge printer network unreachable: verify graceful queuing and retry
3. Medical dispatch API (KRCS, MoH, NDMU) timeout: circuit breaker to manual routing
4. Course CMS database connection pool exhaustion: trigger load shedding on lower-priority endpoints

## Statutes satisfied

- Elections Act §§48-52: poll-worker training and deployment (M38)
- Elections Act §54: observer accreditation (M43)
- Elections Act §55: media accreditation (M42)
- Election Offences Act: voter education content review (M37)

## Follow-ups

- M38 gamification (badge progression for poll-worker training)
- M37 AI-generated localizations (auto-generate regional language content from English + manual review)
- M60 real-time medical incident heat map (for field coordinators)
