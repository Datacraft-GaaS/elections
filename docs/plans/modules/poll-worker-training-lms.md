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
| POST /v1/lms/certificates/{id}/revoke | 2 | JWT+SpiceDB (COMMISSION_HR) | yes | revoke a certificate |
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

Curriculum management:
1. FR-M38-001: Master Trainer creates and versions course modules (text, embedded video from M37 asset store, quizzes, case studies); each module tagged to target audience (PO/DPO/CLERK/SUPPORT)
2. FR-M38-045: course module versioning: each publish creates immutable version; Master Trainer can deprecate old version; enrolled workers on old version see upgrade prompt
3. FR-M38-017: module pass-mark configurable per module by Master Trainer (range 60-100%); default 80%; change requires Training Director approval; audit logged
4. FR-M38-030: M27 case study import: Master Trainer selects case studies from M27 catalogue; import cached locally on fetch; cache refreshed on case study update in M27

Cascaded delivery:
5. FR-M38-002: cascaded delivery: Master Trainer certifies County Trainers; County Trainers certify Constituency Trainers; Constituency Trainers certify poll workers; each level locked until trainer's own cert confirmed
6. FR-M38-023: cascade level locking: County Trainer training package unlocked only after Master Trainer certification confirmation
7. FR-M38-024: cascade level locking: Constituency Trainer training package unlocked only after County Trainer certification confirmation
8. FR-M38-025: cascade level locking: Poll Worker training package unlocked only after Constituency Trainer certification confirmation
9. FR-M38-026: trainer cohort view: Constituency Trainer sees list of assigned poll workers with status badges; filter by NOT_CERTIFIED; batch notification to selected workers
10. FR-M38-046: shared-device mode: Constituency Trainer can initiate group training session; each worker authenticates individually on shared tablet; session isolation enforced

Case studies:
11. FR-M38-003: M27 ballot-adjudication case study catalogue embedded as interactive exercises; ballot image shown; worker selects VALID/REJECTED/QUERY; immediate feedback with statutory basis

Assessment engine:
12. FR-M38-004: assessment engine; configurable pass mark (default 80%); up to 3 retake attempts; third fail triggers mandatory face-to-face with Constituency Trainer; eel.training.third_fail_triggered.v1 emitted
13. FR-M38-015: randomised question ordering within each assessment sitting to prevent answer-sharing; seed derived from worker ID + attempt number
14. FR-M38-016: assessment answers stored immutably per attempt; pass/fail computed server-side; client cannot submit a pre-computed score
15. FR-M38-018: third-fail event emits eel.training.third_fail_triggered.v1; Constituency Trainer notified via push; face-to-face session must be recorded before further retakes unlocked

Certification:
16. FR-M38-005: digital certificate issued on passing all required modules; PDF + QR-coded; IEBC-HSM-signed; includes name, ID (masked last 4 digits), station code, modules, pass marks, issue date, expiry (election day + 30 days)
17. FR-M38-019: certificate PDF generated with SHA-256 hash embedded; hash stored on WorkerCertificate.certificate_hash; hash independently verifiable
18. FR-M38-020: certificate revocation: HR roster removal triggers eel.training.worker_certification_revoked.v1; QR endpoint returns REVOKED status within 60 s
19. FR-M38-021: certificate expiry date is election day + 30 days; expired certificates cannot be reactivated; worker must re-complete training in next cycle
20. FR-M38-022: certificate QR verification endpoint rate-limited at 10,000 requests/min; public endpoint; no authentication required; returns status only (not PII)
21. FR-M38-013: certificate QR resolves to live status in <= 2 s; revocation possible; IEBC-key-signed; third-party verifiable
22. FR-M38-035: P06 HSM signing of certificates: certificate PDF generated, signed via P06; signing request queued if HSM temporarily unavailable; alert raised on HSM timeout; no certificate issued without valid signature

Refresher and briefing:
23. FR-M38-006: mandatory refresher module published at T-30; all certified workers must complete to retain deployment eligibility; M33 flags non-compliant workers; eel.training.refresher_completed.v1 emitted
24. FR-M38-007: election-morning briefing module published at T-1 evening; workers confirm receipt before 06:00 election day; eel.training.briefing_confirmed.v1 emitted; completion pushed to M33 within 60 s

Offline and mobile:
25. FR-M38-008: LMS mobile app caches up to 5 modules and all pending assessments for offline completion; syncs on connectivity restore; no duplicate assessment submissions; idempotency key enforced
26. FR-M38-027: offline cache uses encrypted local storage; 5-module limit enforced; oldest cached module evicted when limit reached; in-progress module never evicted
27. FR-M38-028: assessment submission uses idempotency key; duplicate submission within 24 h returns cached result; no double score recorded
28. FR-M38-029: app minimum requirements: Android 8+ / iOS 14+; cache <= 150 MB for 5 modules; no minimum RAM assumption; low-RAM mode degrades video to audio-only narration
29. FR-M38-047: progress resume: worker can leave a module mid-way and resume from same position on any device after online sync

M33 integration:
30. FR-M38-009: M33 integration: certification status (CERTIFIED/REFRESHER_PENDING/NOT_CERTIFIED) pushed to M33 at T-7 and T-1; M33 blocks deployment of non-certified workers
31. FR-M38-031: M33 push at T-7: all CERTIFIED workers' status pushed via gRPC; retry with exponential backoff; M33 falls back to last-known snapshot if push fails; alert to Training Director
32. FR-M38-032: M33 push at T-1: incremental update of workers changed since T-7 push; only deltas transmitted; confirmation logged
33. FR-M38-033: election-morning briefing confirmation pushed to M33 via gRPC within 60 s of worker submission; M33 gates station opening on briefing status

HR roster:
34. FR-M38-034: HR roster import: worker records imported from IEBC HR system on schedule; fallback to manual CSV upload if API unavailable; roster delta triggers M33 notification; deleted workers trigger certificate revocation

Progress dashboards:
35. FR-M38-010: Training Director progress dashboards: national, county, constituency roll-up of certification rates; drill-through to individual worker status; exportable CSV
36. FR-M38-038: training completion report exportable as CSV by Training Director; fields include worker ID (hashed), module, score, attempts, date; ODPC-compliant: no full IDs
37. FR-M38-039: IAB aggregate certification metrics published to M15; non-critical; circuit breaker; failure does not block training operations

Languages and accessibility:
38. FR-M38-011: EN + SW baseline; target-language audio narration via M37 (Kikuyu, Luo, Kamba, Kalenjin, Luhya)
39. FR-M38-012: WCAG 2.2 AA; video subtitles; audio narration as alternative; keyboard navigation complete on all screens
40. FR-M38-048: video subtitle toggle: all embedded video content has SRT subtitles loadable via M37 asset store; subtitle display on/off per user preference

Security and compliance:
41. FR-M38-040: trainee cannot access another worker's certificate via API; SpiceDB scopes GET /certificates/{worker_id} to self or HR role only
42. FR-M38-041: trainer cannot publish a module without MASTER_TRAINER SpiceDB role; role assignment requires COMMISSION_HR approval; audit logged
43. FR-M38-042: assessment records classified as personal data under DPA 2019 s.30(1)(b); DPIA registered with ODPC; 7-year retention; encrypted at rest
44. FR-M38-043: worker name and ID masked to last 4 digits on all public-facing certificate and QR display; full data available only to HR and audit roles
45. FR-M38-044: all LMS API endpoints: TLS 1.3; certificate QR endpoint served over HTTPS with HSTS; no 1.2 fallback

Content features:
46. FR-M38-014: LMS p99 <= 1 s for module page load under 50,000 concurrent trainees; auto-scale on pod CPU > 70%
47. FR-M38-049: module completion certificate preview screen: worker can view and download PDF before printing; QR code visible on screen for quick verification
48. FR-M38-050: Training Director can publish a training bulletin (non-assessed announcement) to all or filtered worker groups; bulletin delivered via push notification and LMS inbox

Observability:
49. FR-M38-036: OpenTelemetry traces exported to central collector; Grafana dashboard eel-iems-38; alert ruleset iems-38-alerts
50. FR-M38-037: SLO alerts: certification push to M33 within 1 hour of T-7 midnight; QR verification p99 <= 2 s; refresher completion rate >= 99% by T-1; LMS availability >= 99.9% during T-30 to T-1

## Acceptance criteria

1. AC-M38-01: poll worker completes module, passes assessment, receives certificate; QR verifies live against M38 status endpoint; UAT confirms
2. AC-M38-02: refresher completion updates M33 certification status within 2 minutes; integration test confirms
3. AC-M38-03: election-morning briefing confirmation visible in M33 before 06:00 gate; UAT with mock M33 confirms
4. AC-M38-04: certificate revoked on HR roster update; QR returns REVOKED status within 60 seconds; integration test confirms
5. AC-M38-05: 250,000 concurrent refresher completions processed without queue starvation; k6 load test confirms p99 <= 1 s maintained
6. AC-M38-06: offline module completion syncs on reconnect; no duplicate assessment entry; airplane-mode test confirms idempotency
7. AC-M38-07: WCAG 2.2 AA audit pass on all trainee and director screens; independent accessibility audit at OAT gate
8. AC-M38-08: national certification rate >= 98% of rostered workers at T-7; progress dashboard confirms; Training Director sign-off
9. AC-M38-09: certification push to M33 100% within 1 hour of T-7 midnight; M33 integration log verified; alert fires if push incomplete at T-7 + 45 min
10. AC-M38-10: QR certificate verification p99 <= 2 s at 10,000 requests/min; k6 load test confirms
11. AC-M38-11: assessment records retained >= 7 years; encrypted at rest; DPIA registered with ODPC; retention audit at OAT gate
12. AC-M38-12: LMS availability >= 99.9% during T-30 to T-1 refresher and briefing window; SLO alert at 99.5% threshold; uptime monitor confirms

## Applicable edge cases

data:
- EC-DAT-01: duplicate assessment submission with same idempotency key; cached result returned; no double score recorded; server confirms idempotent response
- EC-DAT-02: same assessment submission key with different body; 422 idempotency-key-reuse returned; no action taken; logged to audit trail
- EC-DAT-09: worker names with Kalenjin or Luo Unicode characters; NFC normalization applied; grapheme-safe rendering on certificate PDF; no truncation

security:
- EC-SEC-03: P06 HSM unavailable during certificate signing; signing requests queued with eel-outbox-rs; no certificate issued without valid HSM signature; SLA breach alert raised
- EC-SEC-05: injection attempt in module body or assessment answer field; parameterised queries only; Semgrep CI; ZAP scan at OAT gate
- EC-SEC-16: assessment scores and worker IDs not present in unscoped telemetry traces; log-scanner fixture in CI confirms Tier A/B fields absent

identity:
- EC-IDN-02: trainee JWT expires mid-module; in-flight progress save completes; next request 401 + silent refresh; no progress lost; confirmed by Keycloak 5 s TTL test
- EC-IDN-05: rooted or emulator device attempts to submit assessment; RASP terminates app; tamper event emitted; assessment result not recorded
- EC-IDN-09: HR officer initiates certificate revocation; Training Director must confirm; partial revocation state not possible; both signatures required at commit
- EC-IDN-10: SpiceDB unavailable; certificate issuance and module publish operations fail closed 503; QR verification continues from last-known cache; alert raised

scale:
- EC-SCL-01: 50,000 concurrent trainees completing refresher module; LMS auto-scales; p99 <= 1 s maintained; k6 load test at T-30 simulation
- EC-SCL-04: Kafka consumer lag > 60 s on eel.training.worker.v1 topic; alert raised; backpressure applied; eel-outbox-rs retains events; no M33 push loss

connectivity:
- EC-CON-01: trainee offline for up to 48 h; module cache serves content; pending assessments stored locally; sync with gap audit on reconnect; no duplicate submission
- EC-CON-03: 2G only connectivity; video falls back to audio narration; text content always served; assessment submission via compressed payload; no module blocked
- EC-CON-05: flapping link during assessment sync; backoff+jitter applied; retry budget; no duplicate score on server

mobile:
- EC-MOB-01: Android 8 low-RAM device; 5-module cache fits in <= 150 MB; no OOM on cache load; low-RAM mode falls back to audio-only
- EC-MOB-03: 1 GB RAM KES-10k device; app stays within <= 15 MB working set per session; p99 2 s on 3G confirmed by device farm

operations:
- EC-OPS-05: M33 push fails at T-7; training-svc queues with eel-outbox-rs; M33 falls back to last-known certification snapshot; alert to Training Director within 5 minutes
- EC-OPS-07: Constituency Trainer removed from roster mid-training cycle; credential revoked; affected poll workers notified to contact County Trainer; training records preserved
human:
- EC-HUM-06: WCAG 2.2 AA on trainee app and Training Director dashboard; screen reader + axe-core; keyboard navigation complete; independent audit at OAT gate
- EC-HUM-07: KSL captions required on all embedded video modules before module transitions to PUBLISHED status; caption asset presence enforced
- EC-HUM-08: Luhya and Kalenjin audio narration with long character strings in module titles; no UI truncation; ICU plurals; font fallback tested at 150 percent pseudo-length

legal:
- EC-LEG-02: court-ordered postponement; refresher deadline auto-extended by signed calendar amendment; M33 notified; worker certification window adjusted to new election date
- EC-LEG-03: s.44A invoked on election day; poll worker roster changes mid-day; certificate revocation batch processed; M33 notified within 60 s; affected stations alerted
- EC-TIM-08: refresher deadline falls at T-7 midnight EAT; boundary enforced inclusive; late completion submissions rejected with 422 and deadline-passed reason

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
