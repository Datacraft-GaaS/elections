# M36 voter-deregistration-appeal

| field | value |
|---|---|
| M-number | M36 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M36-voter-deregistration-appeal.md |

## Purpose

Provide the statutory workflow for removing a voter from the register and adjudicating that voter's (or their representative's) appeal against removal per Elections Act s.8A; enforce ODPC-compliant data-minimisation on deregistered records; publish a quarterly deregistration report on the public portal (M11); distinct from M35 which proposes removals based on agency feeds.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Case Officer Console | Phoenix LiveView | services/bvr-svc/ex/deregistration_web/ |
| Adjudicator Workspace | Phoenix LiveView | services/bvr-svc/ex/deregistration_web/ |
| Commissioner Sign-Off Panel | Django | services/bvr-svc/django/gazette_admin/ |
| Voter Notice Page + Appeal Form | Astro | services/bvr-svc/astro/deregistration-public-site/ |
| ODPC Compliance Log | Django | services/bvr-svc/django/gazette_admin/ |
| API | Rust (axum) | services/bvr-svc/crates/bvr-api/ |

Languages: EN + SW; notices in county-detected Kenyan language; appeal intake in EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| DeregistrationCase | id, voter_id, m35_event_id, status (NOTICED/APPEAL_FILED/UNDER_REVIEW/ADJUDICATION/OVERTURNED/CONFIRMED/DEREGISTERED/EXPIRED_NO_APPEAL), notice_dispatched_at, notice_deadline (dispatched + 30 days), appeal_id, officer_id, officer_recommendation, adjudicator_id, adjudicator_decision, adjudicator_decision_at, adjudicator_notes, data_minimisation_at, data_minimisation_officer, created_at | A | T1-audit |
| AppealRecord | id, case_id, intake_channel (VERIFIER_APP/WEB/USSD/IN_PERSON), appellant_id_hash (SHA-256 of National ID), representative_auth_file_id, grounds (IDENTITY_ERROR/ALIVE_AND_PRESENT/DATA_MATCH_ERROR/FAMILY_DISPUTE/OTHER), grounds_detail, supporting_doc_ids, language (BCP-47), filed_at, hearing_date | A | T1-audit |
| QuarterlyDeregistrationReport | id, period_start, period_end, county_breakdown, total_proposed, total_noticed, total_appealed, total_overturned, total_confirmed, report_pdf_hash, merkle_inclusion_proof, commissioner_signed_at, published_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/bvr/deregistration-appeal.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/deregistration/cases | internal gRPC | SVID mTLS (M35_INTERNAL) | yes | create deregistration case from M35 approval |
| GET /v1/deregistration/cases | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | triage queue |
| GET /v1/deregistration/cases/{id} | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | case detail |
| PATCH /v1/deregistration/cases/{id} | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | yes | update triage recommendation |
| POST /v1/deregistration/cases/{id}/appeals | 1 | public + case reference | yes | file appeal (voter or representative) |
| GET /v1/deregistration/cases/{id}/appeals/{aid} | 2 | JWT+SpiceDB (COMMISSION_ADJUDICATOR) | n/a | get appeal record |
| POST /v1/deregistration/cases/{id}/decision | 2 | JWT+SpiceDB (COMMISSION_ADJUDICATOR) | yes | record adjudicator decision |
| POST /v1/deregistration/cases/{id}/minimise | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | yes | execute data-minimisation action |
| GET /v1/deregistration/reports | 1 | public | n/a | list quarterly deregistration reports |
| GET /v1/deregistration/reports/{id} | 1 | public | n/a | download report PDF + Merkle proof |
| POST /v1/deregistration/reports/{id}/sign | 2 | JWT+SpiceDB (COMMISSION_COMMISSIONER) + HSM | yes | Commissioner sign and publish; 2/quarter |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.deregistration_notice_sent.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.appeal_filed.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_overturned.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.voter_deregistered.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.data_minimisation.v1 | emits | eel.bvr.deregistration.v1 | operational | yes | exactly-once |
| eel.bvr.removal_approved.v1 | consumes | eel.bvr.cvr.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M35 CVR | internal | integrations/cvr/ | yes | proposed-removal event source |
| M02 BVR Back-End | internal | integrations/bvr/ | yes | voter record reinstatement or final-status update |
| M03 ABIS | internal | integrations/abis/ | yes | soft-delete or lift of biometric soft-hold |
| M13 Public Verifier App | internal | integrations/verifier/ | yes | appellant self-service portal |
| M11 Public Portal | internal | integrations/portal/ | yes | quarterly report publication |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | aggregate deregistration metrics |
| M40 Election Court IT Interface | internal | integrations/court/ | yes | adjudicator decisions for petitioners |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | quarterly report anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes | Commissioner countersigning |
| Safaricom / Africa's Talking | I08 | integrations/ussd-gateway/ | yes | *the Commission# USSD appeal intake |
| ODPC | external | integrations/odpc/ | yes (stub) | DPIA register lookup; compliance receipt submission |

## ML and AI

- None.

## Dependencies

- Modules: M02, M03, M11, M13, M15, M35, M40
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

Notice generation and dispatch:

1. FR-M36-001: deregistration notice PDF generated for every M35-approved removal; content includes grounds, evidence summary (non-technical), 30-day appeal deadline, and case reference in voter's county-detected language
2. FR-M36-002: notice dispatched via SMS link within 1 hour of M35 approval event; SMS includes case reference and short appeal URL
3. FR-M36-003: notice dispatched via postal address on file; postal dispatch logged with date and tracking reference
4. FR-M36-004: notice dispatched via M13 Public Verifier App in-app push notification sent in parallel with SMS
5. FR-M36-005: notice page available in EN, SW, Kikuyu, Luo, Kamba, Kalenjin, Luhya; language auto-detected from voter's registered county; manual language selector available
6. FR-M36-006: 30-day notice period countdown displayed on voter notice page; page remains accessible to voter until deadline

Appeal intake:

7. FR-M36-007: appeal intake via M13 Public Verifier App; authenticated by OTP to voter's registered phone; case reference pre-filled from notification link
8. FR-M36-008: appeal intake via web form at public appeal URL; accessible without the Commission account; case reference required to proceed
9. FR-M36-009: appeal intake via *the Commission# USSD; voter enters case reference and grounds in <= 5 numeric menu steps; case reference SMS returned within 5 minutes
10. FR-M36-010: appeal intake in-person at IAB office; officer creates record on behalf of voter; identity verified by document; intake channel recorded as IN_PERSON
11. FR-M36-011: legal representative appeal: power of attorney PDF uploaded and stored against appeal record; representative identity and relationship logged; system accepts authorisation before creating case
12. FR-M36-012: appeal record created with uuid7 appeal_id; fields include intake_channel, appellant_id_hash (SHA-256 of National ID), grounds (IDENTITY_ERROR/ALIVE_AND_PRESENT/DATA_MATCH_ERROR/FAMILY_DISPUTE/OTHER), grounds_detail, supporting_doc_ids, language (BCP-47), filed_at
13. FR-M36-013: appeal acknowledgment delivered to appellant within 5 minutes of submission; includes case reference and hearing-scheduled advisory; delivery via SMS and M13 push
14. FR-M36-014: raw CRS/NRB/KRA source documents never stored in appeal case file; adjudicator sees only Commission-generated summary of source data

Case officer triage:

15. FR-M36-015: case officer console shows all pending appeals with status badge and SLA countdown; sortable by deadline proximity and grounds category
16. FR-M36-016: officer categorises each appeal: IDENTITY_ERROR, ALIVE_AND_PRESENT, DATA_MATCH_ERROR, FAMILY_DISPUTE, or OTHER; category logged immutably
17. FR-M36-017: officer can request additional evidence from appellant; appellant notified by SMS with secure upload link; uploaded document stored against case file
18. FR-M36-018: officer records recommendation (UPHOLD_REMOVAL or OVERTURN) with mandatory written basis before submission; recommendation immutable once submitted
19. FR-M36-019: officer recommendation and evidence package forwarded to adjudicator queue automatically after officer submission

Adjudicator hearing:

20. FR-M36-020: adjudicator workspace shows full case file: officer recommendation, evidence list, M35 source summary; no raw agency (CRS/NRB/KRA) documents accessible at any path
21. FR-M36-021: adjudicator issues written decision within 14 days of appeal receipt; decision field (UPHOLD/OVERTURN) with mandatory written rationale; rationale stored immutably
22. FR-M36-022: adjudicator credential (JWT + SpiceDB COMMISSION_ADJUDICATOR role) required at decision submission; decision immutable once submitted; dual-control: officer who recommended may not also decide
23. FR-M36-023: voter notified of court petition right in county-detected language in every uphold notification; advisory text legally approved per EN and SW versions

OVERTURN path:

24. FR-M36-024: M35 event reversed on OVERTURN; eel.bvr.voter_overturned.v1 emitted with exactly-once delivery to eel.bvr.deregistration.v1 topic
25. FR-M36-025: ABIS soft-hold lifted within 60 seconds of OVERTURN decision; voter biometric reactivated in M03
26. FR-M36-026: voter record restored to ACTIVE status in M02 BVR Back-End; voter notified of reinstatement within 24 hours via SMS and M13 push

UPHOLD and no-appeal expiry:

27. FR-M36-027: record moved to DEREGISTERED on UPHOLD decision; ABIS template soft-deleted (recoverable for 7 years); eel.bvr.voter_deregistered.v1 emitted with exactly-once delivery
28. FR-M36-028: contact data (address, photo, phone) purged per DPA 2019 s.26 within 30 days of DEREGISTERED status; eel.bvr.data_minimisation.v1 emitted; data-minimisation action logged
29. FR-M36-029: voter notified of UPHOLD outcome with right-to-petition advisory in county-detected language; notification includes adjudicator written decision reference
30. FR-M36-030: no-appeal expiry: after 30-day notice period with no appeal filed, case officer confirms expiry; Commissioner countersigns; record moved to DEREGISTERED

ODPC compliance:

31. FR-M36-031: DPIA register checked before final deregistration; processing basis code required and logged for every data-minimisation action
32. FR-M36-032: data-minimisation action logged with officer_id, timestamp, DPA basis code; compliance receipt retained 7 years; receipt accessible to ODPC Compliance Lead
33. FR-M36-033: 7-year audit-hold flag prevents irreversible purge while case is in any non-terminal state; officer confirmation required; flag clears only on reaching DEREGISTERED terminal state

Quarterly report:

34. FR-M36-034: quarterly deregistration report generated per county with breakdown: proposed, noticed, appealed, overturned, confirmed deregistered
35. FR-M36-035: Commissioner HSM-signs report using P06 HSM credential; published on M11 public portal within T+7 of quarter-end
36. FR-M36-036: report PDF hash anchored in P05 Merkle chain at publication; Merkle inclusion proof stored with QuarterlyDeregistrationReport record
37. FR-M36-037: automated PII scan run on report PDF before publication; publication blocked if any voter name, National ID, or address detected in output

Deadline and SLA tracking:

38. FR-M36-038: 30-day notice deadline tracked per case; supervisor alert fired when < 3 days remain without appeal filed or expiry confirmed
39. FR-M36-039: 14-day adjudication SLA tracked per appeal; supervisor alert fired when < 48 hours remain without decision issued
40. FR-M36-040: SLA breach dashboard available to the Commission Supervisor showing red-flagged cases; sortable by breach severity

Accessibility and language:

41. FR-M36-041: WCAG 2.2 AA on voter notice page, appeal intake form, and status tracker; axe-core zero critical errors; keyboard navigation complete for all flows
42. FR-M36-042: screen reader completes full appeal intake flow (web and USSD paths) with zero accessibility blockers; tested with NVDA and VoiceOver at each release

## Acceptance criteria

1. AC-M36-01: notice dispatched to voter within 1 hour of M35 approval event; measured by dispatch_timestamp minus m35_approval_timestamp; 100% of cases within SLA; SLO alert fires when any case exceeds 1 hour
2. AC-M36-02: appeal acknowledgment (case reference by SMS) delivered within 5 minutes of submission; measured by case_created_at minus filed_at; verified in integration test across USSD, web, and M13 channels
3. AC-M36-03: adjudication SLA compliance >= 95% of appeals decided within 14 days of filing; measured from filed_at to adjudicator_decision_at; reported weekly to the Commission Commissioner; breach triggers P1 page
4. AC-M36-04: data-minimisation action completed for 100% of DEREGISTERED cases within 30 days of terminal status; measured from deregistered_at to data_minimisation_at; ODPC audit log confirms with zero gaps
5. AC-M36-05: quarterly deregistration report published within T+7 of quarter-end; measured by published_at minus quarter_end; failure triggers P2 alert
6. AC-M36-06: 100% of quarterly reports anchored in P05 Merkle chain before M11 publication; Merkle reconciliation report confirms zero gaps after each publish cycle
7. AC-M36-07: zero PII (voter name, National ID, address) present in public quarterly report; verified by automated PII scan on report PDF before each publication; any detected PII blocks publication and raises P1 alert
8. AC-M36-08: appeal intake p99 <= 500 ms under 5,000 concurrent submissions; >= 99.5% availability on 30-day rolling window; verified by k6 load test at OAT gate
9. AC-M36-09: dual-control enforced: officer who filed triage recommendation cannot also be adjudicator on the same case; OPA policy enforced; verified by security test attempting same-principal decision; zero exceptions
10. AC-M36-10: raw CRS/NRB/KRA source data never returned by any appeal API endpoint; verified by security test suite calling every appeal API as COMMISSION_ADJUDICATOR; zero raw-document fields in any response body
11. AC-M36-11: WCAG 2.2 AA confirmed on voter notice page, appeal intake form, and status tracker by axe-core automated scan (zero critical errors) and manual screen-reader audit; verified at each release
12. AC-M36-12: 7-year audit-hold flag prevents data-minimisation action during any active appeal; verified by attempting minimise call while case is in APPEAL_FILED state; 409 conflict returned; 100% enforcement in unit test suite
13. AC-M36-13: 10,000 concurrent deregistration cases processed without queue starvation; case creation p99 <= 1 s; verified by load test simulating post-census catch-up batch
14. AC-M36-14: module availability >= 99.5% on 30-day rolling window (routine availability class); SLO alert fires within 5 minutes of threshold breach

## Applicable edge cases

- EC-DAT-01: duplicate appeal submission from same appellant with same case reference returns cached acknowledgment without creating a duplicate AppealRecord
- EC-DAT-02: same case idempotency key submitted with different appeal body returns 422 idempotency-key-reuse; original appeal record preserved
- EC-DAT-07: partial batch of DeregistrationCase creation events from M35 (some malformed); valid cases created and notices dispatched; failed cases logged to per-row error report; notice dispatch not blocked
- EC-DAT-09: appellant name or grounds_detail containing multi-codepoint Unicode characters stored as NFC; ICU plurals correct; rendered correctly in all 7 supported languages without truncation
- EC-DAT-12: data-minimisation action requested while case is in APPEAL_FILED or UNDER_REVIEW state; 7-year audit-hold flag blocks irreversible purge; 409 conflict returned; officer confirmation required
- EC-DAT-13: soft-deleted (DEREGISTERED) voter record does not reappear in active triage queue; tombstone honoured in case projection layer
- EC-SEC-05: SQL injection on appeal intake form or API endpoint; parameterized queries prevent execution; ZAP and fuzz corpus confirm at each release
- EC-SEC-10: voter enumeration attempted via appeal status lookup API; constant-time response shape returned regardless of whether case exists; rate-capped at 10 requests/hour per IP
- EC-SEC-16: officer actions, adjudicator decisions, and data-minimisation events redact Tier A/B PII before export to telemetry; CI log-scanner blocks any violations in each build
- EC-IDN-02: JWT expires mid-adjudicator decision submission; in-flight save fails gracefully; next request returns 401 with silent-refresh header; decision not partially committed to store
- EC-IDN-04: voter unreachable at registered SMS and postal address; multi-channel notice sent (SMS plus postal plus M13 push); public gazette used as constructive notice; case proceeds after gazette publication confirmation
- EC-IDN-09: adjudicator's account revoked between officer recommendation submission and adjudicator decision; re-validation at commit prevents decision by revoked principal; case returned to adjudicator queue
- EC-IDN-10: SpiceDB unavailable during adjudicator sign-off write; write fails closed; 503 problem+json returned; no partial decision state persisted
- EC-SCL-01: 5,000 concurrent appeal submissions during post-census batch; appeal intake p99 <= 500 ms; autoscale triggered within 30 seconds; no backlog over 500 items sustained
- EC-SCL-04: Kafka consumer lag on eel.bvr.deregistration.v1 topic exceeds 60 seconds; page fires; backpressure applied; no OOM condition
- EC-SCL-08: Valkey eviction loses idempotency key for appeal intake; PG-durable idempotency store is source of truth; duplicate detection falls back to PG lookup without error
- EC-HUM-06: screen reader completes full appeal intake flow (web and USSD paths) with zero accessibility blockers; tested with NVDA and VoiceOver at each release
- EC-HUM-08: Kalenjin and Luhya notice strings at 150% pseudo-length cause no truncation on voter notice page; ICU plurals correct in all 7 languages; font fallback declared for all scripts
- EC-SEC-15: LiveView session hijack attempt on case officer console; CSRF token plus session binding plus re-auth on step-up action prevent hijack; ExUnit and Playwright tests confirm
- EC-LEG-01: 30-day appeal deadline enforced strictly; late appeal rejected with advisory message; officer can escalate to Commissioner for extraordinary extension via admin action
- EC-LEG-06: adjudicator decision challenged in election court; written decision required and present; Merkle-anchored evidence trail admissible; proof retrievable via M40 court interface
- EC-LEG-08: adjudicator evidence request arrives at T+6d 23h relative to appeal deadline; 24-hour evidence response clock active; not blocked by register freeze; adjudicator retrieves without error
- EC-TIM-08: 30-day notice period deadline falls at midnight EAT (UTC+3); boundary table enforces inclusive semantics; one-second boundary test confirms no off-by-one rejection of valid same-day appeals
- EC-OPS-04: high appeal volume overwhelming adjudicator pool; case officer pre-filters weak appeals; the Commission Commissioner assigns additional adjudicators via admin console; caseload redistributed without data loss
- EC-CON-05: flapping link during notice SMS dispatch; backoff plus jitter retry; retry budget prevents duplicate SMS; final failure triggers postal fallback and alert

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | notice-deadline calculator, appeal-grounds classifier, 7-year hold guard, data-minimisation trigger | nextest | M36-UNIT- |
| property | deadline arithmetic invariants, appeal idempotency | proptest | M36-PROP- |
| contract | M35 case creation event, M02 reinstatement, M03 ABIS hold lift, M11 report publication | Pact | M36-PACT- |
| integration | M35 event to case creation; appeal via USSD webhook; OVERTURN reinstatement; quarterly report publication | testcontainers | M36-INT- |
| e2e | CRS-based removal; voter appeals via web; officer recommends OVERTURN; adjudicator confirms; voter reinstated | Playwright | M36-E2E- |
| security | access another voter's case; access raw CRS data via appeal API; verify hold prevents premature purge | ZAP + cargo-fuzz | M36-SEC- |
| a11y | WCAG 2.2 AA on notice page, appeal form, status tracker | axe-core + screen reader | M36-A11Y- |
| load | 5,000 concurrent appeals; p99 <= 500 ms | k6 | M36-LOAD- |
| chaos | M35 event queue backlog; notice dispatch unblocked | Chaos Mesh | M36-CHAOS- |

Tags: `#[trace("FR-M36-004","AC-M36-03","EC-LEG-06")]`

## Runbooks and dashboards

- `docs/runbooks/bvr-svc-degradation.md` - extend with deregistration appeal paths
- `docs/runbooks/adjudicator-sla-breach.md` - new runbook; alert escalation to the Commission Commissioner; emergency adjudicator assignment
- `docs/runbooks/data-minimisation-audit.md` - new runbook; ODPC audit trail retrieval; breach notification steps
- Grafana dashboard: `platform/observability/bvr-svc/deregistration-dashboard.json` (named `eel-iems-36`)
- SLO alerts: `platform/observability/bvr-svc/deregistration-slo-alerts.yaml` (alert ruleset `iems-36-alerts`; notice 1 h, adjudication 14 d)

## Open questions

- ODPC DPIA registration: Datacraft prepares draft DPIA at contract start; the Commission submits within 30 days of award; delays risk blocking final deregistrations.
- *the Commission# USSD short-code: same gateway as *400# or separate? Confirm with Safaricom; short-code reservation has 6-8 week lead time.
- Adjudicator pool size: the Commission must confirm number of commissioners available as adjudicators; scalability under post-census catch-up batches depends on this.
- Court petition right advisory text: legal team must approve EN + SW + 7-language versions of the right-to-petition advisory before go-live.
