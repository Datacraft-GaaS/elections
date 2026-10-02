# Poll Worker Lifecycle — Capability Design

Full support for recruiting, evaluating, training, deploying, tasking, equipping, managing, and off-boarding election workers (Presiding Officers, Deputy POs, Clerks, support staff — ~250K personnel). Statutory basis: Elections Act ss.33–44 (appointment/qualification of polling officials), Employment Act s.27, DPA 2019 (document handling), the Commission HR manuals.

**Service map** (existing unless noted):

| Domain | Home | Status |
|---|---|---|
| Application intake & job definitions | support-admin-svc M55 (`crates/payroll`, Django `django/hr`, Astro `astro/hr-public`) | exists — extended here |
| **Document submission & verification** | support-admin-svc M55 | **new (this design)** |
| **Screening & evaluation incl. online competency exam** | training-svc M38 | **new (this design)** |
| Selection → appointment → contract | support-admin-svc M55 (`StaffContract`) | exists |
| Worker registry, postings, tasking, equipment issue, performance, contacts | **workforce-svc (new service)** | **new (this design)** |
| Training delivery & accreditation | training-svc M38 (`training-api` LMS/accreditation) + apps `lms-offline`, `accred-scanner` | exists — exam engine added |
| Equipment custody chain | inventory-svc M20 (RFID covers, seals, hash chains) | exists — workforce issues link to it |
| Payroll/disbursement | support-admin-svc M55 (M-Pesa B2C, statutory deductions) | exists |
| Field UX | field-mobile-svc apps: `personnel`, `poll-clerk`, `field-support` | exist — consume new APIs |

---

## 1. Recruitment pipeline (state machine)

```
applied ──► screened ──► documents_complete ──► evaluated ──► selected ──► appointed
   │           │                │                   │             │            │
   └─ rejected └─ rejected     └─ rejected         └─ rejected  └─ waitlist  └─ contracted (M55)
                                                                    └─ rejected
```

- **Gate owners**: screening (HR clerk), documents (compliance verifier), evaluation (auto-score + exam board override), selection (County Returning Officer), appointment (commission seal).
- Every transition is attributed (`actor_id`, timestamp, reason codes); rejection reasons are structured enums so funnel analytics work.
- Waitlist tiering: candidates who pass all gates but exceed station slots stay ranked by composite score for replacement draw-down during attrition.
- Quota rules per county/station: role mix (1 PO + 1 DPO + 2 clerks minimum per stream), gender balance targets, public-sector release constraints — enforced as configurable policies checked at selection time.

## 2. Document submission & verification (new)

**Required-document matrix** is role-driven configuration, e.g.:

| Doc type | Mandatory for | Validity | Verification |
|---|---|---|---|
| National ID (both sides) | all | until expiry date printed | manual + ID-format checksum |
| KRA PIN certificate | all (payroll) | none | manual |
| Police Clearance / DCI Certificate of Good Conduct | all | 6–12 months from issue | manual against DCI reference |
| Academic/professional certificates | PO/DPO | none | manual |
| Passport photo | all | recency 6 months | automatic quality check |
| Bank confirmation letter | payroll recipients | none | manual |
| Signed oath/declaration of secrecy | all, post-appointment | per election | countersigned |

**Model** (`DocumentRequirement`, `DocumentSubmission`):
- submission carries: applicant id, doc type, file SHA-256, size/mime, upload channel (web portal `astro/hr-public`, mobile `personnel` app offline capture, in-person scan), verifier id + verdict {verified, rejected(reason)}, verified_at; **no raw PII in event stream — only hashes and metadata** (DPA 2019).
- status logic: an application reaches `documents_complete` when every mandatory requirement has ≥1 verified non-expired submission; expiring documents re-open the gate automatically (e.g. police clearance older than policy window).
- storage: object bytes go to MinIO under the applicant's tenant prefix via presigned URLs; the API stores only hash+object key. Re-upload replaces (supersede), history retained for audit.
- integrity: upload endpoint is idempotent per `(application_id, doc_type, content_hash)`; verification decisions are dual-controlled above a severity threshold (e.g. rejected ID triggers second-reviewer requirement).

**APIs** (support-admin-svc, `/v1/hr/documents/*`): list requirements per application; create submission (metadata + presign); verifier verdict endpoints; completeness probe used by the recruitment state machine; expiry sweep job.

## 3. Evaluation & online competency exam (new)

Three evaluation layers, all recorded on the application:

### 3.1 Screening scorecard
Structured rubric (education, experience, language proficiency, regional availability) scored 0–5 per criterion with weights per role; computed at `screened`.

### 3.2 Online competency exam (training-svc)
- **Blueprint per role**: PO/DPO/clerk exams differ — question counts, sections (law & procedure, ethics & secrecy, biometric device operation, incident handling), pass marks (e.g. PO 80%, clerk 70%), duration, attempts.
- **Question bank**: versioned questions tagged {role-level, section, difficulty, language}; each question is MCQ or multi-select with deterministic scoring; items carry `active_from/to` and are randomly sampled per candidate seed (beacon-style deterministic shuffle so paper and digital cohorts can be compared).
- **Exam session lifecycle**: `issued → in_progress → submitted → scored | voided`. Sessions are idempotent per (application, attempt#); answers accepted as idempotent upserts keyed by (session, question) so flaky networks never double-count — mirrors the lms-offline queue contract.
- **Anti-abuse**: one active session per application; server-side timing (start + deadline enforced at submit); question order personalised by seed; answer payloads hashed into the submission receipt which is signed and anchored to the transparency log meta-domain (results are audit-relevant).
- **Scoring**: immediate auto-score per section + overall; pass/fail vs role blueprint; board-override endpoint (exam board chair) records human decision distinct from machine score.
- **Offline cohort fallback**: paper exams marked in bulk via `accred-scanner`; OMR-style batch import creates sessions with `channel=paper`.
- **Results**: `ExamResult {application_id, blueprint_id, attempt, section_scores, total_pct, passed, signed_receipt_hash}` consumed by the recruitment state machine at the `evaluated` gate.

### 3.3 Interview record (optional gate)
Structured interview form for shortlisted borderline candidates; scores stored like the screen scorecard.

## 4. Worker registry & contacts (workforce-svc)

- **Worker profile** created at appointment: worker_id (UUIDv7), legal name, national ID hash, role eligibility (derived from passed exam blueprints + contracts), home county, bank/payroll ref link (M55 `StaffContract`), photo hash.
- **Contacts**: multi-channel contact points (SMS voice-capable phone, WhatsApp msisdn, email) each with {verified?, consented? (DPA), preferred}; next-of-kin emergency pair; change history append-only. Bulk messaging hooks emit recipient batches to field-mobile-svc channels (WhatsApp/SMS/USSD) — never raw PII beyond what the channel adapter needs.
- Registry queries power everything downstream (postings, tasking, performance).

## 5. Posting & equipment assignment (workforce-svc + inventory-svc)

- **PostingOrder**: {worker_id, station_code, role, effective window, posted_by, superseded_by}. One active posting per worker per election; supersession chain preserved (redeployments happen).
- **Equipment kit templates** per role: PO kit (BVI kit serial, phone, first-aid, seals, register set), clerk kit… Kits are *claimed* against inventory-svc custody events: issue creates a custody-chain transition (`DISPATCHED→STATION_RECEIVED`) referencing the posting; return closes it (`SEALED_RETURN`). Loss/damage flags raise inventory quarantines and payroll deduction referrals (M55).
- Pre-poll checklist: a worker cannot be marked `ready` until (posting active) ∧ (kit fully issued+acknowledged) ∧ (accreditation card printed/scanned) ∧ (oath recorded).

## 6. Tasking (workforce-svc)

- **Task templates** per role × phase: pre-poll setup (verify seals, zero BVI, post signage), polling-day shifts (open, hourly turnout report, close), post-poll (reconcile ballot serials, handover).
- **TaskAssignment**: {posting, template, due window, status: pending→acked→done{evidence_hash} | blocked(reason)} — ack/done from the `personnel` app offline-first (queued via kit-sdk sync, idempotent keys), completion evidence hashes feed the transparency log for critical tasks (seal verification, serial reconciliation already anchored there).
- ECC consumes aggregated completion states for operational dashboards.

## 7. Performance management (workforce-svc)

- Inputs: task punctuality/completion, attendance (check-in/out via app or accred-scanner), incident records (linked to ECC escalations/grievances), RO end-of-election evaluation form (structured rubric per role), training compliance.
- Outputs: per-worker performance record (append-only), station-level rollups, pool tiering for future cycles (re-hire eligibility), and structured referral events to M55 for breach handling.
- All records attributable and retained per employment-records schedule; workers may request their own records (DPA subject access).

## 8. Cross-service events (outbox, `eel.workforce.*`)

| Event | Producer → consumers |
|---|---|
| `application.stage_changed.v1` | support-admin → workforce, analytics |
| `document.submitted / verified / expired.v1` | support-admin → workforce, compliance |
| `exam.scored.v1` | training → workforce |
| `worker.appointed.v1` / `posting.activated.v1` | workforce → payroll(M55), training, field-mobile |
| `equipment.kit_issued / kit_returned.v1` | workforce ↔ inventory custody events |
| `task.completed.v1` (critical subset anchored) | workforce → ecc, transparency |
| `performance.evaluation_recorded.v1` | workforce → payroll, analytics |

All outbox-mediated (ADR-0015); payloads carry hashes/ids, not PII blobs (DPA).

## 9. Offline-first commitments

- `personnel` app caches postings/tasks/checklists at login (low connectivity standard); ack/done flow through the existing queued-sync pattern with Idempotency-Keys.
- Exam sessions tolerate network loss (idempotent answer upserts, resumable sessions, receipt on submit).
- Document capture works offline in `personnel` (photo + hash queued; upload on flush).

## 10. Implementation slices

| Slice | Content | Where |
|---|---|---|
| W1 | Document requirements/submissions/verification APIs + expiry sweep + completeness probe | support-admin-svc `crates/payroll` (hr domain) |
| W2 | Exam blueprints, question bank, sessions, idempotent answers, auto-scoring, signed receipts, paper-batch import | training-svc `crates/training-api` |
| W3 | New `workforce-svc`: registry, contacts, recruitment state machine consuming W1/W2 ids, postings, kit claims, tasking, performance | new `services/workforce-svc/crates/workforce-api` |
| W4 | Mobile wiring (`personnel`, `field-support`) + ECC dashboard consumption | field-mobile-svc, ecc-svc |

W1–W3 implemented and tested in this wave; W4 follows once the APIs stabilise.
