# training-svc capabilities and algorithms

Comprehensive capability catalogue for the Training, Accreditation, and Capacity Building service (modules M37, M38, M42, M43, M60). Every capability lists the algorithm, the source reference (standard, paper, or statutory instrument), and the crate or file that implements it.

## Contents

1. Module scope
2. Voter education CMS and content workflow (M37)
3. Eleven-language translation catalogue (M37)
4. Poll-worker LMS: modules, assessments, and offline sync (M38)
5. HSM-signed training certificates (M38)
6. Media accreditation: MCK partnership and NPS checks (M42)
7. Observer accreditation: domestic and international (M43)
8. QR badge issuance and CRL (M42, M43)
9. Medical first response and no-PHI enforcement (M60)
10. Key algorithms and references
11. Integration points
12. Standards and compliance (SCORM 2004, xAPI 1.0.3)
13. Operational excellence
14. Validation harnesses
15. Data lifecycle and security
16. API surface and events

---

## 1. Module scope

| Module | Name | Crate / app | Key tables |
|---|---|---|---|
| M37 | Voter Education CMS | training-api | training_cms_content_items, training_cms_content_versions, training_cms_fact_check_reviews, training_cms_distribution_events |
| M38 | Poll-Worker Training LMS | training-api, apps/flutter-lms | training_lms_modules, training_lms_enrollments, training_lms_certificates, training_lms_progress |
| M42 | Media Accreditation Portal | training-api | training_accred_media_orgs, training_accred_media_accreditations, training_accred_badge_revocations |
| M43 | Observer Accreditation Portal | training-api | training_accred_observer_orgs, training_accred_observers, training_accred_credential_revocations |
| M60 | Medical First Response | training-api | training_medical_incidents, training_medical_dispatch_requests, training_medical_cluster_alerts, training_medical_units |

Tier: T3. Availability SLO: 99.5%. LMS completion sync SLO: < 30 seconds. Badge issuance SLO: < 5 minutes. Medical dispatch SLO: < 10 minutes.

Single unified Rust service (crate: `training-api`) spanning all five modules with separate resource routes (`/training/cms`, `/training/lms`, `/training/media-accreditation`, `/training/observer-accreditation`, `/training/medical`). Django admin for CMS and LMS management. Flutter offline app for poll-worker training and medical incident field reporting. Astro static site for public accreditation portal.

---

## 2. Voter education CMS and content workflow (M37)

### 2.1 Content item lifecycle

`training_cms_content_items` records each content asset (video, audio, text, infographic) with a `status` column following the FSM:

```
draft -> in_review -> approved -> published -> expired -> archived
```

State transitions are enforced by the API layer (`training-api/src/cms/workflow.rs`); no direct status updates are permitted outside the defined transitions. Content is versioned in `training_cms_content_versions` with a monotonically increasing `version_num` per item.

### 2.2 Fact-check review pipeline

Before a content item transitions from `in_review` to `approved`, at least one `training_cms_fact_check_reviews` row must exist with `decision: approved`. Reviewers can annotate specific claims in `annotations: JSONB`. Items with `decision: rejected` are returned to `draft` status. Items requiring revision transition to `draft` with an annotation referencing required changes.

The fact-check step implements a two-reviewer policy for items in the `disinformation_risk: high` category (set by the CMS editor at creation time). Two independent `approved` reviews are required before publication.

### 2.3 Distribution events

Content published to multiple channels is tracked in `training_cms_distribution_events` (`channel IN ('M13_feed','sms','whatsapp','county_portal')`). Each distribution event records `county_code` to support per-county content targeting.

### 2.4 Content expiry

`training_cms_content_items.expiry_date` drives automated archiving. A nightly sweep (`training-api/src/cms/expiry_job.rs`) transitions items past `expiry_date` to `expired` status and emits a `training.cms.content.expired` event. Expired items are not surfaced in the public portal.

---

## 3. Eleven-language translation catalogue (M37)

Content is authored in English (`en`) or Swahili (`sw`) and translated into 9 additional languages. The BCP-47 language tags used in `training_cms_content_items.language` are:

| Code | Language |
|---|---|
| en | English |
| sw | Swahili |
| ki | Kikuyu |
| luo | Dholuo |
| kam | Kamba |
| kal | Kalenjin (Nandi dialect) |
| luy | Luhya |
| mer | Meru |
| guz | Gusii |
| som | Somali |
| om | Oromo |

Translation entries are separate `training_cms_content_versions` rows with the same `item_id` and a language-specific version record. Translations carry their own `expiry_date` to handle cases where the source content is updated but translations are not yet refreshed.

A translation expiry alert fires when a translated version's `expiry_date` is within 7 days and no updated translation has been published. The alert targets the county-level CMS coordinator for the affected language region.

---

## 4. Poll-worker LMS: modules, assessments, and offline sync (M38)

### 4.1 Module structure

`training_lms_modules` defines training modules with `module_type IN ('core','refresher','briefing')` and a `pass_mark_pct` threshold. Enrolment state in `training_lms_enrollments` follows:

```
enrolled -> in_progress -> passed | failed | expired
```

A `failed` learner may re-attempt up to the `max_attempts` limit (configurable per module; stored in `training_lms_modules`). An `expired` enrolment is triggered when the enrolled worker has not completed the module within the `expiry_days` window.

### 4.2 SCORM 2004 compliance

Course packages are authored as SCORM 2004 4th Edition (Advanced Distributed Learning Initiative, 2009) packages. The Flutter offline app implements a SCORM 2004 run-time environment that:
- Parses `imsmanifest.xml` to enumerate SCOs (Sharable Content Objects).
- Tracks `cmi.completion_status` and `cmi.success_status` per SCO.
- Persists state to local SQLite on the device.
- Syncs state to the server via `POST /v1/training/enrolments/{id}/complete` on network reconnection.

### 4.3 xAPI 1.0.3 event stream

Alongside SCORM 2004 state, the Flutter app emits xAPI (Tin Can API) 1.0.3 statements (Advanced Distributed Learning Initiative, Experience API v1.0.3, 2016) for granular learning analytics:
- `Actor` is the poll-worker UUID (hashed for analytics).
- `Verb` follows the ADL xAPI vocabulary (`completed`, `passed`, `failed`, `experienced`).
- Statements are buffered locally and posted to the xAPI LRS endpoint (`POST /v1/training/xapi/statements`) on reconnection.

### 4.4 Offline sync conflict resolution

If a learner completes a module offline on two devices before syncing, the server applies a last-write-wins policy based on `completed_at` timestamp. The device with the later `completed_at` wins. Conflicts are logged to `training_lms_conflict_log` for manual review.

---

## 5. HSM-signed training certificates (M38)

On a successful module completion (`status: passed`), a certificate PDF is generated and an HSM signature is produced:

1. Certificate PDF rendered with candidate name, module title, score, issue date, expiry date, and QR verification URL.
2. SHA-256 hash of the PDF computed; stored in `training_lms_certificates.certificate_hash`.
3. Ed25519 signature over the hash produced by HSM (key role P06, `spiffe://eel.local/training-svc/cert-sign`).
4. Signature stored in `training_lms_certificates.hsm_signature` (base64).
5. QR code encodes `{ certificate_id, verification_url, hsm_signature }`.
6. `training.accreditation.issued` event emitted.

Certificate revocation: `training_lms_certificates.revoked: bool` + `revoked_at` + `revoked_reason`. Revoked certificates are rejected by the QR badge scanner in real time.

---

## 6. Media accreditation: MCK partnership and NPS checks (M42)

### 6.1 MCK registration check

Media organisations must be registered with the Media Council of Kenya (MCK). The accreditation flow requires:
- `mck_registered: true` on `training_accred_media_orgs`.
- `mck_membership_number` on `training_accred_media_accreditations`.

The MCK registration status is verified against the MCK public registry API (`py/accreditation/mck_client.py`) at submission time. The API response (registered/not registered) is not persisted; only the `mck_registered: bool` flag is stored.

### 6.2 NPS advisory check

The National Police Service (NPS) advisory check is a security vetting step for individual journalists. Advisory data is fetched from the NPS API at review time. Per the schema comment in `training_accred_media_accreditations`: "NPS advisory data is session-cleared and not persisted here." The NPS result influences the reviewer's decision but is not stored in the database, satisfying data minimisation requirements under the Kenya Data Protection Act 2019 §34.

### 6.3 Access zone assignment

`training_accred_media_accreditations.access_zones: TEXT[]` lists the zones the accredited journalist may access (e.g. `['tallying_centre','national_results_centre']`). Zone assignments are controlled by the commission operations team via Django admin.

---

## 7. Observer accreditation: domestic and international (M43)

### 7.1 Organisation types

`training_accred_observer_orgs.org_type IN ('domestic','international')`. International observer organisations require a signed MoU (`mou_signed_at`) before individual observers can be accredited. Domestic organisations require a letter of authorization from the commission.

### 7.2 Individual observer accreditation FSM

`training_accred_observers.status` follows:

```
pending -> approved -> deployed -> credential_revoked
```

Deployment county codes (`deployment_county_codes: TEXT[]`) restrict the counties where the observer may operate. Credentials are issued as Ed25519-signed QR badges (same HSM key P06 as training certificates).

### 7.3 PII vault separation

Per the schema comment in `training_accred_observers`: "full_name and id_number: data class B/PII, stored in separate encrypted vault". The main table holds only the UUID and status; plaintext identity data lives in an encrypted vault keyed by `observer_id`. This limits PII exposure from SQL queries on the primary table.

---

## 8. QR badge issuance and CRL (M42, M43)

### 8.1 Signed badge payload

QR badges encode:

```json
{
  "badge_type": "media | observer | poll_worker",
  "serial": "<badge_serial>",
  "holder_id": "<hashed_uuid>",
  "access_zones": ["..."],
  "issued_at": "<ISO8601>",
  "expires_at": "<ISO8601>",
  "sig": "<Ed25519 base64>"
}
```

The `sig` field is an Ed25519 signature over the canonical JSON (keys sorted, no whitespace) using HSM key P06. Badge scanners at venue access points verify the signature and check expiry before granting access.

### 8.2 Certificate Revocation List (CRL)

The CRL is served at `GET /v1/training/badges/crl` and refreshed every 5 minutes. The CRL payload is a JSON array of `{ serial, revoked_at, reason }` for all revoked badges across M42, M43, and M38 certificates. Badge scanners cache the CRL locally and refresh every 5 minutes. A badge whose `serial` appears in the cached CRL is rejected even if the signature is valid.

The 5-minute CRL refresh interval bounds the window between revocation and effective denial of access to 5 minutes in the normal operating case and up to 10 minutes under degraded connectivity.

---

## 9. Medical first response and no-PHI enforcement (M60)

### 9.1 No-PHI schema enforcement

FR-M60-011 prohibits storing patient name, diagnosis, or health identifiers. The schema (`migrations/0004_training_medical.sql`) contains no name, diagnosis, or ID columns on any M60 table. Comments in the DDL explicitly restate the prohibition. The `training_medical_incidents` table records only: `station_id`, `incident_type`, `severity`, `status`, `persons_affected`, `county_code`, `reported_at`, `resolved_at`.

Enforcement at the application layer (`training-api/src/medical/validator.rs`): any request body containing keys matching `patient_name`, `national_id`, `diagnosis`, `dob`, or `health_id` is rejected with HTTP 422 and an audit entry in `training_accred_audit_entries`.

### 9.2 Dispatch workflow

Dispatch to external responders follows a priority routing:

1. Nearest KRCS (Kenya Red Cross Society) unit in `training_medical_units` with `status: standby` and matching `county_code`.
2. If no KRCS unit available within 2 km: route to MoH facility.
3. If MoH unreachable: route to NDMU (National Disaster Management Unit).

Dispatch is recorded in `training_medical_dispatch_requests` (`dispatched_to IN ('KRCS','MoH','NDMU')`). Circuit breaker in `training-api/src/medical/dispatch.rs` opens after 3 consecutive API failures; on open, the system falls back to manual SMS routing via the field coordinator.

### 9.3 Cluster alerts

A cluster alert fires when more than `alert_threshold` incidents are reported in a single `county_code` within a 60-minute window. The threshold is configurable per county via ConfigMap. Cluster alerts are stored in `training_medical_cluster_alerts` and emitted as `training.medical.cluster.alert` events for the operations dashboard.

---

## 10. Key algorithms and references

| Algorithm | Implementation path | Reference |
|---|---|---|
| Content version + fact-check workflow FSM | training-api/src/cms/workflow.rs | Domain-driven design aggregate pattern |
| SCORM 2004 run-time environment | apps/flutter-lms/lib/scorm/ | ADL Initiative SCORM 2004 4th Edition, 2009 |
| xAPI 1.0.3 statement emission | apps/flutter-lms/lib/xapi/ | ADL Initiative Experience API v1.0.3, 2016 |
| Ed25519 HSM certificate signing | training-api/src/cert/hsm_signer.rs | Bernstein et al., "High-speed high-security signatures", J. Cryptographic Engineering, 2012 |
| QR badge CRL (5-minute refresh) | training-api/src/badge/crl.rs | X.509 CRL concept adapted to JSON; RFC 5280 (IETF, 2008) for revocation semantics |
| pHash badge serial uniqueness | training-api/src/badge/serial.rs | UUID7 (uuid6 crate); monotonic generation per platform CLAUDE.md |
| Offline sync conflict resolution (last-write-wins) | training-api/src/lms/sync.rs | Lamport clock ordering |
| No-PHI field validation | training-api/src/medical/validator.rs | Kenya Data Protection Act 2019 §34 (data minimisation) |
| Cluster alert threshold | training-api/src/medical/cluster.rs | Configurable threshold; Poisson process baseline |
| MCK registry API check | py/accreditation/mck_client.py | MCK public API (mediacouncil.or.ke) |

---

## 11. Integration points

| Direction | Service | Mechanism | Data |
|---|---|---|---|
| Upstream | field-mobile-svc | Kafka `field-mobile.attendance.checked-in` | LMS completion cross-check for attendance fraud |
| Upstream | crms-svc | Kafka `crms.party.registered` | Observer org accreditation eligibility |
| Upstream | MCK registry API | HTTP | Media org registration status |
| Upstream | NPS advisory API | HTTP (session-only, not persisted) | Journalist security vetting |
| Upstream | KRCS dispatch API | HTTP | Medical unit availability and dispatch |
| Upstream | MoH dispatch API | HTTP | Medical unit availability fallback |
| Upstream | NDMU dispatch API | HTTP | Medical unit availability fallback |
| Downstream | Payroll (support-admin-svc M55) | Kafka `training.course.completed` | Training gate check before payroll disbursement |
| Downstream | field-mobile-svc | Kafka `training.accreditation.issued` | Observer/media deployment credential |

---

## 12. Standards and compliance

| Standard / statute | Coverage |
|---|---|
| SCORM 2004 4th Edition (ADL Initiative, 2009) | LMS course packaging and run-time tracking |
| Experience API (xAPI) 1.0.3 (ADL Initiative, 2016) | Granular learning analytics event stream |
| Kenya Elections Act Cap 7 §§48-55 | Poll-worker training, observer and media accreditation |
| Election Offences Act | Voter education content review obligation |
| WCAG 2.2 Level AA (W3C, October 2023) | Astro public portal and Django admin accessibility |
| Kenya Data Protection Act 2019 §34 | No-PHI enforcement (M60); NPS data session-clear (M42) |
| RFC 5280 (IETF, 2008) | CRL revocation semantics (adapted for JSON badges) |
| RFC 6238 (IETF TOTP, 2011) | OTP for step-up authentication on accreditation admin |
| CloudEvents 1.0 (CNCF, 2022) | Event envelope for all outbox messages |

Note: ISO 20387:2018 (Biobanking) is not applicable to this service; it is cited in the task prompt for domain-awareness only.

---

## 13. Operational excellence

| Capability | Path | Mechanism |
|---|---|---|
| HPA | deploy/overlays/primary | CPU 70%, memory 80% |
| Blue-green deploy | deploy/overlays/primary | Primary and mirror sites |
| Offline sync backlog | apps/flutter-lms | Batch replay on WiFi reconnection; queue-to-file on SQLite quota |
| Badge printer fallback | training-api/src/badge/printer.rs | Buffer requests locally; retry on printer recovery |
| Medical dispatch circuit breaker | training-api/src/medical/dispatch.rs | Open after 3 failures; fallback to SMS escalation |
| CRL refresh | training-api/src/badge/crl.rs | Every 5 minutes; badge scanners cache locally |
| LMS completion SLO | Grafana | 99.5 % of completions synced within 30 s |
| Badge issuance SLO | Grafana | 99.5 % of issuances within 5 min |
| Medical dispatch SLO | Grafana | 99.5 % of dispatches within 10 min |

Key metrics: `training_course_enrollments{course_id,language}`, `training_course_completion_latency_ms`, `training_accreditation_badge_print_latency_ms`, `training_medical_incident_dispatch_latency_ms`.

---

## 14. Validation harnesses

| Harness | Location | What it tests |
|---|---|---|
| CMS workflow FSM tests | crates/training-api/tests/cms_workflow.rs | All valid state transitions; invalid transitions rejected |
| Quota / multi-reviewer policy tests | crates/training-api/tests/cms_two_reviewer.rs | Two-reviewer rule for high-risk items |
| SCORM 2004 run-time unit tests | apps/flutter-lms/test/ | SCO state persistence and sync |
| xAPI statement schema tests | apps/flutter-lms/test/ | Statements conform to xAPI 1.0.3 JSON schema |
| HSM signing round-trip tests | crates/training-api/tests/cert_sign.rs | Sign, serialise, verify cycle |
| CRL freshness tests | crates/training-api/tests/crl.rs | Revoked serial rejected within 5 min |
| No-PHI field rejection tests | crates/training-api/tests/medical_validator.rs | All prohibited field names rejected with 422 |
| MCK API stub tests | py/tests/mck_stub.py | pytest-httpserver stub for MCK registry |
| Offline conflict resolution tests | crates/training-api/tests/lms_sync.rs | Last-write-wins on conflicting completion timestamps |
| Load test: 500 VUs | loadtest/training.js | p95 latency within SLO under sustained load |

---

## 15. Data lifecycle and security

### 15.1 Encryption at rest

Individual learner PII (in `training_lms_enrollments.worker_id`) is a UUID7 reference to the HR record in support-admin-svc (M55). The UUID is hashed for analytics. Observer and media applicant PII is stored in an encrypted vault keyed by the primary-table UUID.

### 15.2 Retention

| Table | Retention |
|---|---|
| training_cms_content_items | Archived items: election cycle + 5 years |
| training_lms_enrollments | Election cycle + 5 years |
| training_lms_certificates | Election cycle + 15 years (KNADS Act) |
| training_accred_media_accreditations | Election cycle + 7 years |
| training_accred_observers | Election cycle + 7 years |
| training_medical_incidents | 5 years |

### 15.3 Audit immutability

`training_accred_audit_entries` is append-only (PG trigger). Every badge issuance, revocation, and medical dispatch records `actor_id`, `entity_type`, `entity_id`, `action`, `recorded_at`.

### 15.4 Transport

All REST endpoints require TLS 1.3. MCK/NPS/KRCS API calls use mTLS with SPIRE SVIDs. Flutter offline app uses end-to-end encryption for the sync payload (AEAD with a session key negotiated via ECDH P-256).

---

## 16. API surface and events

### 16.1 REST (training-api, axum 0.8, port 8080)

- `GET /v1/training/courses` + `GET /v1/training/courses/{id}`: course catalogue.
- `POST /v1/training/enrolments` + `POST /v1/training/enrolments/{id}/complete`: LMS enrolment and completion.
- `POST /v1/training/accreditations` + `POST /v1/training/accreditations/{id}/badge`: accreditation submission and badge issuance.
- `POST /v1/training/medical-incidents` + `GET /v1/training/medical-incidents/{id}`: incident reporting and status.
- `GET /v1/training/badges/crl`: CRL (public, no auth required).
- `POST /v1/training/xapi/statements`: xAPI 1.0.3 LRS ingest (authenticated by learner JWT).
- `GET /healthz` + `/readyz` + `/openapi.json`.

### 16.2 Events

**Emitted:**

| Event type | Trigger |
|---|---|
| `training.course.completed` | Enrolment transitions to `passed` |
| `training.accreditation.issued` | Badge issued for media, observer, or LMS certificate |
| `training.accreditation.rejected` | Accreditation rejected after review |
| `training.medical.incident.reported` | Medical incident filed |
| `training.medical.incident.dispatched` | Dispatch request sent to KRCS/MoH/NDMU |

**Consumed:**

| Event type | Source | Purpose |
|---|---|---|
| `field-mobile.attendance.checked-in` | field-mobile-svc | Flag potential attendance fraud in LMS completion |
| `crms.party.registered` | crms-svc | Observer organisation eligibility |

---

## Summary

Total footprint: 1 Rust crate (training-api), 1 Flutter offline app (apps/), 1 Django admin (django/), 1 Astro static portal, 4 SQL migrations (0001-0004, 18 tables), Kafka outbox, SCORM 2004 + xAPI 1.0.3 LRS. Core correctness guarantees: two-reviewer fact-check gate for high-risk voter education content, HSM-signed LMS certificates with 5-minute CRL, strict no-PHI enforcement at schema and API layer for M60, SCORM 2004 offline-first with last-write-wins sync conflict resolution.

---

## Appendix A: Degradation contract

| Mode | Behaviour |
|---|---|
| Normal | All workflows live: CMS publication, LMS sync, accreditation review, badge printing, medical dispatch |
| Degraded | Accreditation review queued; medical dispatch manual escalation only |
| Read-only | Cached course content served; all enrolments and accreditations rejected with 503 |

---

## Appendix B: Accreditation badge lifecycle

```
Submit accreditation application
  -> MCK/NPS/observer eligibility check
  -> Reviewer decision (approved / rejected)
  -> On approved: generate badge serial (UUID7)
  -> Render badge QR payload (JSON: badge_type, serial, access_zones, issued_at, expires_at)
  -> Sign payload with HSM key P06 (Ed25519)
  -> Store badge_serial + hsm_signature in accreditations table
  -> Emit training.accreditation.issued
  -> Badge scanner reads QR; verifies Ed25519 sig; checks CRL; grants or denies access
```

CRL is served at `GET /v1/training/badges/crl`. Revocation propagates to all scanners within 5 minutes. For immediate access denial (e.g. security incident), the revoked serial is pushed directly to scanner devices via a Kafka `training.badge.revoked` event consumed by the field-mobile scanner app.
