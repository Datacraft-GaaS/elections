# Poll-worker training LMS (M38)

Module M38 (`poll-worker-training-lms`) owned by `training-svc`. Delivers structured pre-election training to poll workers via a learning management system (LMS): course modules authored in M37 CMS, worker enrolment, offline-capable mobile delivery, HSM-signed certificates, and a dispatch gate that blocks M33 (personnel dispatch) until certification thresholds are met.

## Contents

1. What M38 does (vs M37 CMS and M33 dispatch)
2. Data plane (schema)
3. Course content and delivery modes
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M38 does (vs M37 CMS and M33 dispatch)

- **M37** = voter education CMS; authors content items used as raw material for LMS course modules.
- **M38** = structures that material into curriculum, enrols poll workers, tracks completion, issues HSM-signed certificates, and signals M33 dispatch that a worker is certified.
- **M33** = mobile personnel dispatch; holds dispatch orders for polling station officers until M38 confirms the officer holds a valid certificate for the current election cycle.

A poll worker who fails the final assessment cannot be dispatched by M33 until they retake and pass. The gate is a SpiceDB permission check: `eel.training.dispatch_eligible:{worker_id}` is only granted when `WorkerCertificate.status = VALID` for the current election cycle.

## 2. Data plane

Four aggregates (per `services/training-svc/crates/lms/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **CourseModule** | A single training unit: `module_id` (UUID7), `election_cycle`, `topic` (`PROCEDURE / BIOMETRIC_KIT / SECURITY / ACCESSIBILITY / RESULTS_FORM`), `content_version_id` (FK to M37 ContentVersion), `assessment_questions[]`, `pass_score_pct` (default 80), `max_attempts` (default 3). | B |
| **WorkerEnrollment** | Links a worker to a course: `enrollment_id` (UUID7), `worker_id`, `module_id`, `enrolled_at`, `attempts_used`, `latest_score`, `status` (`ENROLLED / IN_PROGRESS / PASSED / FAILED`), `offline_package_downloaded_at`. | B |
| **WorkerCertificate** | Issued on passing all required modules: `certificate_id` (UUID7), `worker_id`, `election_cycle`, `issued_at`, `valid_until`, `modules_completed[]`, `hsm_signature`, `status` (`VALID / REVOKED / EXPIRED`). | A |
| **AssessmentAttempt** | Each quiz attempt: `attempt_id` (UUID7), `enrollment_id`, `started_at`, `submitted_at`, `answers_sha256`, `score`, `passed`. | B |

`WorkerCertificate.hsm_signature` is produced by the Commissioner HSM key (per-cycle KEK from `eel_kek_generations` in V022). A revoked certificate immediately removes the SpiceDB `dispatch_eligible` grant.

## 3. Course content and delivery modes

| Delivery mode | Description | Offline support |
|---|---|---|
| Online web | Astro SPA fetching module content from API; assessments submitted in real time | No |
| Offline mobile (Android APK) | Packaged `offline_package` bundle downloaded pre-election; assessment answers queued locally and synced on reconnect | Yes; CRDT-based sync via `packages/eel-sync-rs` |
| Facilitator-led (printed) | PDF printout of module content; facilitator marks completion in officer console | No sync needed; officer records manually |

Offline packages are signed with the election cycle KEK before download. The mobile app verifies the signature on open; a tampered package is rejected with an error code logged to the device telemetry endpoint.

## 4. Pipeline

```
CourseModule authored by training coordinator (pulls ContentVersion from M37)
      -> modules grouped into election-cycle curriculum
      -> workers enrolled via HR/payroll feed (M28) or manual officer action
            -> WorkerEnrollment created (status = ENROLLED)
            -> offline_package generated + signed + pushed to S3
            -> push notification to worker mobile app
      -> worker completes modules (online or offline)
            -> AssessmentAttempt recorded per quiz
            -> if score >= pass_score_pct: enrollment -> PASSED
            -> if score < pass_score_pct and attempts_used < max_attempts: retry
            -> if attempts_used == max_attempts and not PASSED: enrollment -> FAILED
               -> alert to training coordinator for manual remediation
      -> when all required modules PASSED:
            -> WorkerCertificate issued + HSM-signed
            -> status = VALID
            -> SpiceDB grant: eel.training.dispatch_eligible:{worker_id}
            -> emit eel.training.worker_certified.v1
            -> M33 dispatch gate lifted for this worker
      -> on cycle expiry:
            -> WorkerCertificate status -> EXPIRED
            -> SpiceDB grant revoked
      -> offline sync: attempt queue flushed on reconnect via CRDT merge
         (Shapiro et al., "Conflict-free Replicated Data Types", SSS 2011)
```

## 5. API surface

Contract: `packages/openapi-contracts/training/lms.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/lms/modules` | Training coordinator | List CourseModules for current cycle |
| `POST /v1/lms/modules` | Training coordinator | Create CourseModule |
| `POST /v1/lms/enrollments` | Officer / HR system (SVID) | Enrol worker in module |
| `GET /v1/lms/enrollments/{enrollment_id}` | Worker (JWT) or officer | Retrieve enrollment + attempt history |
| `POST /v1/lms/enrollments/{enrollment_id}/attempts` | Worker (JWT) | Submit AssessmentAttempt |
| `GET /v1/lms/enrollments/{enrollment_id}/offline-package` | Worker (JWT) | Download signed offline package |
| `POST /v1/lms/enrollments/{enrollment_id}/offline-sync` | Worker mobile app (JWT) | Flush queued offline attempts (CRDT merge) |
| `GET /v1/lms/certificates/{worker_id}` | Worker (JWT) or M33 (SVID) | Retrieve WorkerCertificate for cycle |
| `POST /v1/lms/certificates/{certificate_id}/revoke` | Commissioner + HSM | Revoke certificate (e.g., conduct issue) |
| `GET /v1/lms/dispatch-gate/{worker_id}` | M33 (SVID) | Boolean dispatch-eligible check (SpiceDB) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Training coordinator console (LiveView) | `services/training-svc/ex/lms_web` | Manage modules, curriculum, view completion dashboards |
| Worker learning portal (Astro SPA) | `services/training-svc/astro/worker-lms` | Worker takes modules, views certificate status |
| Offline mobile app (Android) | `services/training-svc/mobile/android` | Offline module delivery + assessment queue |
| Commissioner certificate panel (Django) | `services/training-svc/django/cert_admin` | HSM-sign certificates in bulk; revoke |

Languages: EN + SW on all staff and worker surfaces. WCAG 2.2 AA. Offline app has high-contrast mode and font-size controls for accessibility in field conditions.

## 7. Integrity + anti-tamper

- **HSM-signed certificates** - `WorkerCertificate.hsm_signature` is verifiable offline using the public key published in the IEBC key transparency log. A field supervisor can verify a certificate QR code without network connectivity.
- **Offline package integrity** - packages are signed (Ed25519, key from `eel_kek_generations`) and the app verifies the signature on first open and on each module load. Tampering causes immediate rejection and telemetry alert.
- **Assessment immutability** - `AssessmentAttempt` rows are insert-only; no UPDATE on answers or score. Disputes trigger a forensic re-score from the raw answers (SHA-256 locked).
- **SpiceDB gate** - `dispatch_eligible` is a computed relation backed by `WorkerCertificate.status = VALID` for the current election cycle. M33 queries SpiceDB at dispatch time; a stale cache is detected by a TTL of 60 seconds.
- **CRDT sync correctness** - offline answer queues use a state-based CRDT (last-write-wins per attempt, keyed by `attempt_id` UUID7) to prevent duplicate submission on reconnect. Correctness follows Shapiro et al. (SSS 2011).

## 8. Kenya statute alignment

- **Elections Act §27** - Commission shall train election officials; M38 operationalises the training record and certification mandate.
- **Elections (Technology) Regulations 2017** - training on technology-assisted tallying must be documented and certified; M38's `WorkerCertificate` satisfies the documentation requirement.
- **Kenya Data Protection Act §26 (minimisation)** - only assessment scores and completion status retained post-cycle; raw answer payloads purged 3 months post-election.
- **Public Service Commission Act** - training records for Commission officers must be retained for 5 years; `WorkerCertificate` rows are retained in T1-audit with RPO 15 min, RTO 60 min.

## 9. Honest caveats + external prerequisites

- Offline mobile app requires Android 8.0+ (Oreo). Devices older than 2019 may not be field-deployable. This is a procurement constraint, not a software constraint.
- HR / payroll integration (M28) provides the worker roster. If M28 is unavailable, officers must enrol workers manually via the API. Bulk CSV import is supported as a fallback.
- Assessment question banks must be authored per election cycle by a qualified training expert. The LMS does not auto-generate questions; it only structures and delivers them.
- M33 dispatch gate requires SpiceDB to be reachable. If SpiceDB is unavailable, M33 falls back to a cached `dispatch_eligible` snapshot (TTL 5 min) and alerts ops. A prolonged SpiceDB outage could block dispatch; the runbook at `docs/runbooks/spicedb-degraded.md` covers the manual override procedure.
- HSM-signing of certificates requires the Commissioner's HSM to be online. Batch signing is supported; a queue of pending certificates can be signed in a single HSM session.

## 10. Test coverage + operational runbooks

Tests under `services/training-svc/crates/lms/tests/`: property-based via `proptest` on score thresholds and CRDT merge, integration via `testcontainers` for PG + Kafka + S3, mobile offline sync round-trip under simulated network loss.

Runbooks under `docs/runbooks/`:

- `training-svc.md` - LMS module deploy, rollback, offline package re-sign on key rotation.
- `spicedb-degraded.md` - dispatch gate manual override during SpiceDB outage.
- `certificate-revocation.md` - emergency revocation procedure + M33 cache invalidation.

Related documents: spec at `docs/plans/modules/poll-worker-training-lms.md`; M37 docs at `docs/features/voter-education-cms.md`; M33 docs at `docs/features/`; capability catalogue at `services/training-svc/docs/CAPABILITIES.md`.
