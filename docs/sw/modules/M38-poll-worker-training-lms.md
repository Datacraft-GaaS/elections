# M38 — Poll Worker Training LMS

**Module ID:** M38
**Name:** Poll Worker Training, Assessment, and Certification LMS
**Runs on:** Central servers (Kenyan cloud) + offline-capable mobile/tablet app
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Deliver, assess, and certify the training of approximately 250,000 poll workers (Presiding Officers, Deputy Presiding Officers, Poll Clerks, Support Staff) through a cascaded train-the-trainer model. Maintain a standardised curriculum including adjudication case studies drawn from M27 (Intent Preservation Module). Enforce a 30-day pre-election refresher module and an election-morning briefing. Track certification status per worker and integrate with M33 (Personnel Dispatch) so only certified workers are deployed.

Strategy reference: §XVII.3 Element 6 (training programme for poll workers).

## 2. User Personas

- **Master Trainer (the Commission National)** — designs and publishes curriculum modules; manages trainer cohort
- **County Trainer (Cascade Level 1)** — trained by Master Trainer; trains Constituency Trainers
- **Constituency Trainer (Cascade Level 2)** — trained by County Trainer; trains poll workers
- **Poll Worker (Trainee)** — completes modules; takes assessments; receives certificate
- **the Commission Training Director** — monitors national certification progress; sets pass-mark policy
- **M33 Personnel Dispatch Officer** — checks certification status before deployment
- **HR Administrator** — manages poll worker roster; links to M33 personnel records
- **IAB Observer** — views aggregate certification statistics

## 3. Functional Requirements

- **FR-M38-001** Curriculum management: Master Trainer creates and versions course modules (text, embedded video from M37 asset store, quizzes, adjudication case studies from M27). Each module has a target audience tag (PO / DPO / CLERK / SUPPORT).
- **FR-M38-002** Cascaded delivery: Master Trainer certifies County Trainers via the LMS; County Trainers certify Constituency Trainers; Constituency Trainers certify poll workers. Each level's training package is locked until the trainer's own certification is confirmed.
- **FR-M38-003** Adjudication case study catalogue: M27-sourced ballot-adjudication case studies (image + ground-truth ruling) embedded as interactive exercises. Workers see a ballot image, select a ruling (VALID / REJECTED / QUERY), and receive immediate feedback with statutory basis.
- **FR-M38-004** Assessment engine: each module has an assessment (multiple choice + case study). Configurable pass mark (default 80%). Workers may retake up to 3 times; third fail triggers mandatory face-to-face session with Constituency Trainer.
- **FR-M38-005** Certification issuance: on passing all required modules, the system issues a digital certificate (PDF, QR-coded, signed with the Commission public key). Certificate includes: worker name, ID number (masked to last 4 digits on PDF), station code (if assigned), modules completed, pass marks, issue date, expiry (election day + 30 days).
- **FR-M38-006** 30-day pre-election refresher: a mandatory refresher module is published at T-30; all certified workers must complete it to retain certification for deployment. M33 integration flags non-compliant workers.
- **FR-M38-007** Election-morning briefing: a short (15-minute) mandatory briefing module published at T-1 evening; workers must confirm receipt on their mobile/tablet before 06:00 on election day. Completion status pushed to M33.
- **FR-M38-008** Offline capability: LMS mobile app caches up to 5 course modules and all pending assessments for offline completion; syncs on connectivity restore.
- **FR-M38-009** M33 integration handoff: at T-7 and T-1, LMS pushes certification status (CERTIFIED / REFRESHER_PENDING / NOT_CERTIFIED) per worker to M33. M33 blocks deployment of non-certified workers and alerts dispatch officer.
- **FR-M38-010** Progress dashboards: Training Director sees national, county, constituency roll-up of certification rates; drill-through to individual worker status; exportable CSV.
- **FR-M38-011** Languages: course content delivered in EN + SW baseline; target-language audio narration via M37 for available languages (Kikuyu, Luo, Kamba, Kalenjin, Luhya).
- **FR-M38-012** Accessibility: all text content and assessment UI WCAG 2.2 AA compliant; video content has subtitles; audio narration available as alternative.

## 4. Non-Functional Requirements

- **NFR-M38-001** LMS platform: p99 <= 1 second for module page load under 50,000 concurrent trainees.
- **NFR-M38-002** Availability: >= 99.5% during training windows; >= 99.9% at T-30 to T-1 (refresher and briefing period).
- **NFR-M38-003** Offline sync: assessment submissions sync within 60 seconds of connectivity restore.
- **NFR-M38-004** Certificate QR verification: third-party QR scan resolves to certificate status in <= 2 seconds.
- **NFR-M38-005** DPA 2019: poll worker assessment scores are personal data; lawful basis s.30(1)(b) (employment/election administration); retention 7 years; DPIA registered with ODPC.
- **NFR-M38-006** Scalability: 250,000 workers completing refresher within a 2-week window at T-30.
- **NFR-M38-007** Security: certificates are Commission-key-signed; QR resolves to a live status endpoint; revocation possible (e.g. worker removed from roster).
- **NFR-M38-008** WCAG 2.2 AA: all training interfaces including mobile app.

- **NFR-38-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-38` (replace 38 with the module number); alert ruleset ID `iems-38-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Master Trainer, I publish a new ballot-adjudication case study module sourced from M27 and assign it to all Presiding Officer trainees nationwide.
2. As a Constituency Trainer, I see which of my 50 poll workers have not yet completed the refresher module 10 days before the deadline.
3. As a Poll Worker (Presiding Officer trainee), I complete a ballot-adjudication exercise on my phone offline on the matatu home; my results sync when I get WiFi.
4. As a Poll Worker, I fail the assessment a third time; the system notifies my Constituency Trainer to arrange a face-to-face session before I can retake.
5. As a Poll Worker, I pass all modules and download my digital certificate with a QR code I can show at the polling station check-in.
6. As the Commission Training Director, I see a national map showing 92% certification rate with Turkana and West Pokot counties highlighted as below 80% — I alert the regional coordinator.
7. As an M33 Personnel Dispatch Officer, I see a red flag next to a Presiding Officer whose refresher is incomplete; I contact the County Trainer for resolution before dispatch.
8. As a Poll Worker, I confirm receipt of the election-morning briefing at 05:45 on election day; the completion status updates in M33 in real time.
9. As an HR Administrator, I upload a roster update removing a deceased worker; the system revokes their certificate and flags any station they were assigned to in M33.
10. As an IAB Observer, I see the aggregate certification rate (by worker grade and county) in the IAB dashboard at T-7.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime, date

class CourseModule(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    title: str
    target_audience: list[Literal["PO","DPO","CLERK","SUPPORT","TRAINER"]]
    language: str                   # BCP-47
    content_markdown: str
    asset_ids: list[str]            # M37 asset references
    case_study_ids: list[str]       # M27 adjudication case study IDs
    pass_mark_pct: int              # default 80
    max_retakes: int                # default 3
    module_type: Literal["CORE","REFRESHER","BRIEFING","TRAINER_CERT"]
    version: int
    published_at: datetime | None
    expiry_date: date | None

class WorkerEnrollment(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    worker_id: str                  # FK to M33 personnel record
    module_id: str
    status: Literal["NOT_STARTED","IN_PROGRESS","PASSED","FAILED","RETAKE_REQUIRED"]
    attempts: int
    best_score_pct: float | None
    completed_at: datetime | None
    certified: bool

class WorkerCertificate(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    worker_id: str
    station_code: str | None
    modules_completed: list[str]
    issued_at: datetime
    expires_at: datetime
    revoked: bool
    revoked_at: datetime | None
    revoked_by: str | None
    certificate_hash: str           # SHA-256 of signed PDF
    qr_verification_url: str

class TrainingProgress(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    scope: Literal["NATIONAL","COUNTY","CONSTITUENCY"]
    scope_code: str
    snapshot_at: datetime
    total_workers: int
    certified: int
    refresher_pending: int
    not_certified: int
    certification_rate_pct: float
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/lms/modules | MASTER_TRAINER JWT | 20/hr | Create / update course module |
| GET | /v1/lms/modules | TRAINEE JWT | 200/hr | List available modules for my audience |
| GET | /v1/lms/modules/{id}/content | TRAINEE JWT | 500/hr | Fetch module content (text + asset URLs) |
| POST | /v1/lms/modules/{id}/assessments | TRAINEE JWT | 10/hr | Submit assessment answers |
| GET | /v1/lms/enrollments | TRAINEE JWT | 100/hr | List my enrollments and progress |
| GET | /v1/lms/certificates/{worker_id} | M33_INTERNAL service | 500/hr | Get certification status for M33 |
| GET | /v1/lms/certificates/verify/{qr_token} | Public | 1000/min | QR code certificate verification |
| POST | /v1/lms/certificates/{id}/revoke | COMMISSION_HR JWT | 20/hr | Revoke a certificate |
| GET | /v1/lms/progress | TRAINING_DIRECTOR JWT | 50/hr | National/county/constituency progress report |
| POST | /v1/lms/briefing/{worker_id}/confirm | TRAINEE JWT | 1/day | Confirm election-morning briefing receipt |
| GET | /v1/lms/briefing/status | M33_INTERNAL service | 200/hr | Bulk briefing-confirmation status (election day) |

## 8. Integration Points

- **M27 Intent Preservation** — ballot-adjudication case study source
- **M33 Personnel Dispatch** — certification status push at T-7 and T-1; briefing confirmation on election morning
- **M37 Voter Education CMS** — shared asset store for video/audio training content
- **M15 IAB Dashboard** — aggregate certification metrics
- **P06 HSM** — Commission-key signing of digital certificates
- **HR System (external, the Commission)** — poll worker roster import and synchronisation

## 9. UI/UX Key Screens

1. **Trainee Home** — progress rings per module; next-action card; offline download button; certificate download when earned.
2. **Module Reader** — paginated text + embedded video; progress tracker; next-section button; accessible keyboard navigation; subtitles toggle.
3. **Assessment Screen** — one question per screen; case-study: ballot image + ruling options; immediate feedback post-answer; final score on completion.
4. **Certificate View** — full-page certificate with name, modules, scores, QR code; download PDF button.
5. **Training Director Dashboard** — national map with county fill by certification rate; drill-through to constituency list; trend chart; T-30 refresher countdown.
6. **Trainer Cohort View** — list of my trainee workers with status badges; filter by NOT_CERTIFIED; batch-notify selected workers.
7. **Election Morning Briefing Screen** — single-page briefing summary; CONFIRM RECEIPT button; countdown to 06:00 deadline.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Poll worker training obligation | Elections Act 2011 §44A; the Commission Act 2011 §7(1)(i) |
| Standardised curriculum | Elections (General) Regulations 2012 r.7 |
| Accessibility of training materials | Persons with Disabilities Act 2003 §21 |
| Personal data (assessment records) | DPA 2019 §30(1)(b); DPIA required |
| Certificate retention | the Commission records-management policy; 7-year retention |
| Anti-discrimination in training delivery | National Cohesion Act 2008 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| National certification rate at T-7 | >= 98% of rostered workers | Progress dashboard | the Commission Training Director |
| Refresher completion rate at T-1 | >= 99% of certified workers | Enrollment records | the Commission Training Director |
| Election-morning briefing confirmation by 06:00 | >= 99% | Briefing status API | the Commission Ops Director |
| Certificate QR verification latency | <= 2 seconds | Verification endpoint monitor | Datacraft SRE |
| Assessment pass rate (first attempt) | >= 75% of workers | Enrollment analytics | the Commission Training Director |
| LMS availability during refresher window | >= 99.9% | Uptime monitor | Datacraft SRE |
| M33 certification push at T-7 | 100% within 1 hour of T-7 midnight | M33 integration log | Datacraft Technical |

## 12. Test Strategy

- **Unit:** assessment scoring engine; cascade-level lock logic; refresher-deadline calculator; certificate QR generator.
- **Integration:** M27 case-study import; M33 certification-status push; M37 asset URL resolution; HR roster import.
- **E2E:** simulate a poll worker: enroll, complete module, pass assessment, receive certificate, complete refresher, confirm briefing; verify M33 status updates at each stage.
- **Security:** trainee attempts to access another worker's certificate; trainer attempts to publish without Master Trainer permission; verify certificate cannot be forged (QR resolves to live status, not static).
- **Accessibility:** WCAG 2.2 AA on all screens via axe-core + manual; subtitle test on embedded video.
- **Load:** 50,000 concurrent trainees completing refresher module; certificate QR verification at 10,000 requests/minute on election morning.
- **Offline:** disconnect device mid-module; verify local state preserved; sync on reconnect; verify no duplicate assessment submissions.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Core LMS live | 2026-11-01 | Module editor + assessment engine |
| Master Trainer certification round | 2027-01-15 | ~50 the Commission national trainers certified via LMS |
| County Trainer cascade | 2027-02-15 | 47 county trainers certified |
| Constituency Trainer cascade | 2027-04-01 | ~290 constituency trainers certified |
| Poll worker training opens | 2027-05-01 | T-101: all ~250,000 workers begin training |
| Core certification target | 2027-07-01 | T-40: 95% certified |
| Refresher module published | 2027-07-11 | T-30 trigger |
| Refresher deadline | 2027-08-03 | T-7 |
| Election-morning briefing | 2027-08-09 | T-1 evening |

## 14. Dependencies

- M27 Intent Preservation — adjudication case study catalogue
- M33 Personnel Dispatch — certification-status consumer
- M37 Voter Education CMS — shared video/audio assets
- P06 HSM — certificate signing
- HR System — poll worker roster
- M15 IAB Dashboard — certification metrics

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Network unavailability in remote constituencies | High | High | Offline-first mobile app; 5-module cache; sync window validated at T-14 |
| Poll worker smartphone access gap | Medium | High | Constituency Trainer can complete group training sessions; shared-device mode supported |
| Mass retake failures overwhelming Constituency Trainers | Medium | Medium | Face-to-face retake sessions built into rollout schedule; the Commission regional support team on standby |
| M33 certification push failure blocks deployment | Low | High | M33 falls back to last-known certification status with T-7 snapshot; manual override with Director authorisation |
| Case study images from M27 unsuitable for training context | Low | Medium | Master Trainer review gate on each imported case study before publication |

## 16. Cross-References

- `M27-intent-preservation.md` — adjudication case study source
- `M33-mobile-personnel-dispatch.md` — certification status consumer
- `M37-voter-education-cms.md` — shared asset store
- `M15-iab-dashboard.md` — certification metrics

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XVII.3 Element 6 — poll worker training | FR-M38-001 through FR-M38-012 |
| §XVII.3 — standardised curriculum | FR-M38-001, FR-M38-003 |
| §XVII.3 — cascaded train-the-trainer | FR-M38-002, Rollout plan |
| §XVII.3 — pre-election refresher | FR-M38-006, FR-M38-007 |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if the Commission terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and the Commission DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to the Commission ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** the Commission ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
