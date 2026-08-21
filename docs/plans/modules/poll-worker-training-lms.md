# M38 poll-worker-training-lms

| field | value |
|---|---|
| M-number | M38 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5%; 99.9% during T-30 to T-1 refresher and briefing window |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M38-poll-worker-training-lms.md |

## Purpose

Deliver, assess, and certify the training of approximately 250,000 poll workers (Presiding Officers, Deputy Presiding Officers, Poll Clerks, Support Staff) through a cascaded train-the-trainer model. Enforce a 30-day pre-election refresher and an election-morning briefing. Track certification status per worker and integrate with M33 (Personnel Dispatch) so only certified workers are deployed. Adjudication case studies are drawn from M27 (Intent Preservation) as live ballot-image exercises.

Strategy reference: §XVII.3 Element 6 (training programme for poll workers).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Trainee LMS App (Android 8+ / iOS 14+ / PWA) | Flutter | services/training-svc/apps/lms-trainee/ |
| Training Director Console | Phoenix LiveView | services/training-svc/ex/training_web/ |
| HR Administration Panel | Django | services/training-svc/django/hr_admin/ |
| API | Rust (axum) | services/training-svc/crates/training-api/ |

Distribution: trainee app via IEBC MDM + public app stores. Languages: EN + SW baseline; audio narration in Kikuyu, Luo, Kamba, Kalenjin, Luhya via M37 asset store.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| CourseModule | id, title, target_audience (PO/DPO/CLERK/SUPPORT/TRAINER), language, content_markdown, asset_ids, case_study_ids (from M27), pass_mark_pct, max_retakes, module_type (CORE/REFRESHER/BRIEFING/TRAINER_CERT), version, published_at, expiry_date | B | T2 |
| WorkerEnrollment | id, worker_id (FK to M33 personnel record), module_id, status (NOT_STARTED/IN_PROGRESS/PASSED/FAILED/RETAKE_REQUIRED), attempts, best_score_pct, completed_at, certified | B | T2 |
| WorkerCertificate | id, worker_id, station_code, modules_completed, issued_at, expires_at, revoked, revoked_at, revoked_by, certificate_hash (SHA-256 of signed PDF), qr_verification_url | A | T1-audit |
| TrainingProgress | id, scope (NATIONAL/COUNTY/CONSTITUENCY), scope_code, snapshot_at, total_workers, certified, refresher_pending, not_certified, certification_rate_pct | C | T2 |

## APIs

OpenAPI file: `packages/openapi-contracts/training/lms.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/lms/modules | 2 | JWT+SpiceDB (MASTER_TRAINER) | yes | create or update course module |
| GET /v1/lms/modules | 2 | JWT+SpiceDB (TRAINEE) | n/a | list available modules for audience |
| GET /v1/lms/modules/{id}/content | 2 | JWT+SpiceDB (TRAINEE) | n/a | fetch module content and asset URLs |
| POST /v1/lms/modules/{id}/assessments | 2 | JWT+SpiceDB (TRAINEE) | yes | submit assessment answers |
| GET /v1/lms/enrollments | 2 | JWT+SpiceDB (TRAINEE) | n/a | list my enrollments and progress |
| GET /v1/lms/certificates/{worker_id} | internal gRPC | M33_INTERNAL mTLS | n/a | certification status for M33 |
| GET /v1/lms/certificates/verify/{qr_token} | 1 | public | n/a | QR code certificate verification; <= 2 s |
| POST /v1/lms/certificates/{id}/revoke | 2 | JWT+SpiceDB (IEBC_HR) | yes | revoke a certificate |
| GET /v1/lms/progress | 2 | JWT+SpiceDB (TRAINING_DIRECTOR) | n/a | national/county/constituency progress |
| POST /v1/lms/briefing/{worker_id}/confirm | 2 | JWT+SpiceDB (TRAINEE) | yes | confirm election-morning briefing receipt |
| GET /v1/lms/briefing/status | internal gRPC | M33_INTERNAL mTLS | n/a | bulk briefing confirmation (election day) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.worker_certified.v1 | emits | eel.training.worker.v1 | operational | yes | at-least-once idempotent |
| eel.training.worker_certification_revoked.v1 | emits | eel.training.worker.v1 | operational | yes | exactly-once |
| eel.training.refresher_completed.v1 | emits | eel.training.worker.v1 | operational | yes | at-least-once idempotent |
| eel.training.briefing_confirmed.v1 | emits | eel.training.worker.v1 | operational | yes | at-least-once idempotent |
| eel.training.third_fail_triggered.v1 | emits | eel.training.worker.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M27 Intent Preservation | internal | integrations/intent-preservation/ | yes | adjudication case study import; cache locally on fetch |
| M33 Personnel Dispatch | internal | integrations/personnel-dispatch/ | yes | certification push at T-7 and T-1; queue on failure; M33 falls back to last-known snapshot |
| M37 Voter Education CMS | internal | integrations/education-cms/ | yes | shared asset store for video and audio content |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | aggregate certification metrics; non-critical |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | certificate signing; queue signing requests; alert on HSM timeout |
| IEBC HR System | external | integrations/hr-system/ | yes (stub) | poll worker roster import; sync on failure retried with exponential back-off |

## ML and AI

- None (assessment scoring is rule-based; pass-mark threshold set by IEBC Training Director).

## Dependencies

- Modules: M27, M33, M37, M15
- Platform: P06
- Packages: kit-sdk-flutter, eel-resilience-rs, eel-crypto-rs, eel-outbox-rs

## Feature checklist

1. FR-M38-001: Master Trainer creates and versions course modules (text, embedded video, quizzes, case studies); each module tagged to target audience (PO/DPO/CLERK/SUPPORT)
2. FR-M38-002: cascaded delivery: Master Trainer certifies County Trainers; County Trainers certify Constituency Trainers; Constituency Trainers certify poll workers; each level locked until trainer's own cert is confirmed
3. FR-M38-003: M27 ballot-adjudication case study catalogue embedded as interactive exercises; ballot image shown; worker selects VALID/REJECTED/QUERY; immediate feedback with statutory basis
4. FR-M38-004: assessment engine; configurable pass mark (default 80%); up to 3 retake attempts; third fail triggers mandatory face-to-face with Constituency Trainer
5. FR-M38-005: digital certificate issued on passing all required modules; PDF + QR-coded; IEBC-HSM-signed; includes name, ID (masked last 4 digits), station code, modules, pass marks, issue date, expiry (election day + 30 days)
6. FR-M38-006: mandatory refresher module published at T-30; all certified workers must complete to retain deployment eligibility; M33 flags non-compliant workers
7. FR-M38-007: election-morning briefing module published at T-1 evening; workers confirm receipt before 06:00 election day; completion pushed to M33
8. FR-M38-008: LMS mobile app caches up to 5 modules and all pending assessments for offline completion; syncs on connectivity restore; no duplicate assessment submissions
9. FR-M38-009: M33 integration: certification status (CERTIFIED/REFRESHER_PENDING/NOT_CERTIFIED) pushed to M33 at T-7 and T-1; M33 blocks deployment of non-certified workers
10. FR-M38-010: Training Director progress dashboards: national, county, constituency roll-up; drill-through to individual; exportable CSV
11. FR-M38-011: EN + SW baseline; target-language audio narration via M37 (Kikuyu, Luo, Kamba, Kalenjin, Luhya)
12. FR-M38-012: WCAG 2.2 AA; video subtitles; audio narration as alternative
13. Certificate QR resolves to live status in <= 2 s; revocation possible (worker removed from roster); IEBC-key-signed; third-party verifiable
14. LMS p99 <= 1 s for module page load under 50,000 concurrent trainees

## Acceptance criteria

1. AC-M38-01: poll worker completes module, passes assessment, receives certificate; QR verifies live against M38 status endpoint; UAT
2. AC-M38-02: refresher completion updates M33 certification status within 2 minutes; integration test
3. AC-M38-03: election-morning briefing confirmation visible in M33 before 06:00 gate; UAT
4. AC-M38-04: certificate revoked on HR roster update; QR returns revoked status within 60 seconds; integration test
5. AC-M38-05: 250,000 concurrent refresher completions processed without queue starvation; load test
6. AC-M38-06: offline module completion syncs on reconnect; no duplicate assessment entry; airplane-mode test
7. AC-M38-07: WCAG 2.2 AA audit pass on all trainee and director screens; independent audit

## Applicable edge cases

**connectivity**
- EC-CON-01: trainee offline; module and pending assessment cached; submitted on reconnect with idempotency key; no duplicate score recorded
- EC-CON-03: 2G connectivity; video content falls back to text + audio narration only

**operations**
- EC-OPS-05: M33 push fails at T-7; training-svc queues with eel-outbox-rs; M33 falls back to last-known certification snapshot; alert to Training Director

**identity**
- EC-IDN-09: worker removed from HR roster mid-cycle; certificate revoked; M33 notified within 60 s; station assignment flagged

**legal**
- EC-LEG-02: DPA 2019 s.30(1)(b) lawful basis for assessment records; DPIA registered with ODPC; 7-year retention

**human**
- EC-HUM-06: WCAG 2.2 AA on all surfaces including mobile app
- EC-HUM-08: audio narration in 5 Kenyan languages; subtitles on all video

**scale**
- EC-SCL-01: 250,000 workers completing refresher in 2-week window at T-30; LMS auto-scales; p99 <= 1 s target maintained

**mobile**
- EC-MOB-01: Android 8 low-RAM device; 5-module cache fits in <= 150 MB; no minimum RAM assumption

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | assessment scoring, cascade-level lock, refresher-deadline calculator, QR generator, certificate revocation logic | nextest + flutter_test | M38-UNIT- |
| property | idempotent assessment submission; certificate always verifiable if not revoked | proptest | M38-PROP- |
| contract | M27 case-study import, M33 push, M37 asset URL, P06 HSM signing | Pact | M38-PACT- |
| integration | full enrolment, pass, certificate, refresher, briefing; M33 status at each stage | integration_test | M38-INT- |
| e2e | trainee enrols, passes, receives cert, completes refresher, confirms briefing; verify M33 | Flutter driver | M38-E2E- |
| security | trainee accesses another worker's cert (must fail); trainer publishes without Master Trainer perm (must fail); QR forgery attempt (must fail via live status check) | ZAP + red team | M38-SEC- |
| a11y | WCAG 2.2 AA; subtitle test on embedded video | axe-core + manual | M38-A11Y- |
| offline | mid-module disconnect; local state preserved; sync on reconnect; no duplicate score | netem | M38-OFFLN- |
| load | 50,000 concurrent refresher completions; 10,000 QR verifications/min on election morning | k6 | M38-LOAD- |

Tags: `#[trace("FR-M38-006","AC-M38-02","EC-SCL-01")]`

## Runbooks and dashboards

- `docs/runbooks/training-svc-degradation.md` - LMS degradation; offline cache validation; M33 push failure paths
- `docs/runbooks/certificate-revocation.md` - new runbook; HR roster update triggers revocation; M33 notification SLA
- Grafana dashboard: `eel-iems-38` (alert ruleset: `iems-38-alerts`)
- SLO alerts: certification push to M33 <= 1 hr of T-7 midnight; QR verification p99 <= 2 s; refresher completion rate

## Open questions

- IEBC HR system API schema: authentication and data model must be confirmed before integration sprint; fallback is manual CSV roster upload if API delayed.
- P06 HSM key custody for certificate signing: who holds the IEBC training certificate signing key? Distinct from the Legal Director key used in M40? Clarify before implementation.
- Cascade level locking: if a Constituency Trainer's own certification lapses (e.g., fails refresher), do their trainees lose certification? IEBC HR policy decision required.
- Audio narration translator contracts: Kikuyu, Luo, Kamba, Kalenjin, Luhya narration requires translator procurement at T-180 if confirmed.
