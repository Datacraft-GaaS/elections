# M41 post-election-review

| field | value |
|---|---|
| M-number | M41 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M41-post-election-review.md |

## Purpose

Capture structured lessons-learned from every stakeholder category (the Commission departments, IAB, KEOG, ELOG, accredited observers, media, and citizens) after each election cycle. Track each finding through a lifecycle from submission to recommendation status, parliamentary response, and the Commission Chair sign-off. Feed confirmed recommendations back into next-cycle planning. The long-drip attack register (§XXI.C.20) ensures systemic weaknesses are formally tracked with mandatory 6-month review intervals so institutional memory does not dissipate between cycles.

Strategy reference: §XXI.C.20 (long-drip attack mitigation), §XXI (post-election review cycle).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Internal Analyst Console | Phoenix LiveView | services/governance-svc/ex/review_web/ |
| the Commission Chair Sign-Off Panel | Phoenix LiveView | services/governance-svc/ex/review_web/ |
| Observer and Stakeholder Portal | Astro | services/governance-svc/astro/review-portal/ |
| Citizen Intake Form (WCAG 2.2 AA) | Astro | services/governance-svc/astro/review-portal/ |
| Public Recommendation Tracker (WCAG 2.2 AA) | Astro | services/governance-svc/astro/review-portal/ |
| API | Rust (axum) | services/governance-svc/crates/governance-api/ |

Languages: citizen intake EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya; internal portal EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| LessonsLearningFinding | id, election_cycle, submitter_category (COMMISSION_DEPT/IAB/KEOG/ELOG/EU_EOM/DOMESTIC_OBSERVER/MEDIA/CITIZEN), submitter_id (None if anonymous), finding_type (TECHNICAL/PROCESS/LEGAL/SECURITY/ACCESSIBILITY/COMMUNICATION), severity (HIGH/MEDIUM/LOW), description, evidence_refs, recommended_action, is_long_drip, duplicate_of, status, submitted_at | C | T2 |
| Recommendation | id, finding_ids (consolidated), recommendation_text, eel_response_type (ACCEPTED/ACCEPTED_WITH_MODIFICATION/DECLINED/DEFERRED), eel_response_rationale, owner_dept_id, implementation_date, requires_legislation, parliament_referral_id, chair_signed_at, chair_signed_by, next_cycle_handoff_at, last_long_drip_review | B | T1-consensus |
| ParliamentReferral | id, recommendation_ids, referral_package_hash, chair_signed_at, clerk_received_at, parliament_response (ADOPTED/REJECTED/DEFERRED), bill_reference, parliament_response_at | A | T1-audit |
| LongDripAlert | id, finding_id, review_due_at (every 6 months post-closure), reviewer_id, reviewed_at, escalated_to_commissioner, escalated_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/governance/review.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/review/findings | 2 / public anon | Submitter JWT or public | yes | submit finding; anonymous citizens accepted |
| POST /v1/review/findings/bulk | 3 | JWT+SpiceDB (OBSERVER) | yes | bulk CSV/JSON finding import |
| GET /v1/review/findings | 2 | JWT+SpiceDB (COMMISSION_ANALYST) | n/a | list findings; filterable by type and status |
| PATCH /v1/review/findings/{id} | 2 | JWT+SpiceDB (COMMISSION_ANALYST) | yes | triage: deduplicate, categorise, escalate |
| GET /v1/review/recommendations | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | list recommendations |
| PATCH /v1/review/recommendations/{id} | 2 | JWT+SpiceDB (DEPT_LEAD) | yes | submit departmental response |
| POST /v1/review/recommendations/{id}/sign | 2 | JWT+SpiceDB (COMMISSION_CHAIR) + HSM | yes | Chair sign-off; gate: no unresolved disputes |
| GET /v1/review/public | 1 | public | n/a | public recommendation tracker; no PII |
| POST /v1/review/parliament-referrals | 2 | JWT+SpiceDB (COMMISSION_CHAIR) + HSM | yes | create and sign Parliament referral package |
| PATCH /v1/review/parliament-referrals/{id} | 2 | JWT+SpiceDB (PARLIAMENT_LIAISON) | yes | record Parliament response |
| GET /v1/review/long-drip | 2 | JWT+SpiceDB (COMMISSION_SECURITY) | n/a | long-drip register with review status |
| POST /v1/review/long-drip/{id}/review | 2 | JWT+SpiceDB (COMMISSION_SECURITY) | yes | record 6-month review |
| GET /v1/review/next-cycle-handoff | 2 | JWT+SpiceDB (PLANNING_LEAD) | n/a | pull ASSIGNED/IN_PROGRESS recs for next cycle |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.finding_submitted.v1 | emits | eel.governance.review.v1 | operational | no | at-least-once idempotent |
| eel.governance.recommendation_signed.v1 | emits | eel.governance.review.v1 | audit | yes | exactly-once |
| eel.governance.parliament_referral_dispatched.v1 | emits | eel.governance.review.v1 | audit | yes | exactly-once |
| eel.governance.long_drip_escalated.v1 | emits | eel.governance.review.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M13 Public Verifier App | internal | integrations/verifier-app/ | yes | citizen finding intake channel |
| M11 Public Portal | internal | integrations/portal/ | yes | public recommendation tracker publication |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | IAB findings submission and aggregate analytics |
| M28 AI Monitoring Stack | internal | integrations/ai-platform/ | yes | optional finding deduplication clustering; graceful degradation to keyword match if unavailable |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | Chair sign-off and Parliament referral anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | Chair signing |
| Parliament eCitizen portal | external | integrations/parliament/ | yes (stub) | referral delivery; fallback to encrypted email with manual confirmation |

## ML and AI

- Optional deduplication clustering from M28 (keyword matching + AI semantic clustering): flags semantically similar findings across submitter categories; analyst confirms or keeps separate; M41 does not run its own model.
- Automated PII scan on free-text fields in citizen submissions (M28-provided NLP): redact before storage; reduce ODPC exposure.

## Dependencies

- Modules: M11, M13, M15, M28
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs

## Feature checklist

Intake:
1. FR-M41-001: structured intake form with submitter category, election cycle reference, finding type (TECHNICAL/PROCESS/LEGAL/SECURITY/ACCESSIBILITY/COMMUNICATION), severity (HIGH/MEDIUM/LOW), description, evidence refs, recommended action, and long-drip flag
2. FR-M41-002: multi-channel intake: internal portal for the Commission departments and IAB, credentialed observer portal for KEOG/ELOG/EU EOM, public form via M13 and web, bulk CSV/JSON upload for observer reports
3. FR-M41-003: anonymous citizen intake path: no PII collected, DPA 2019 s.30(1)(a) consent screen skipped on anonymous path, reference token returned on submission
4. FR-M41-004: multi-language citizen intake: EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya; language selector on intake form; all 7 languages render without truncation
5. FR-M41-005: finding deduplication: keyword matching always on; optional M28 AI semantic clustering invoked per submission and returns candidate duplicates within 10 seconds; analyst confirms or keeps separate; duplicate_of FK set on merged findings
6. FR-M41-006: intake acknowledgement: identified submitters receive email/SMS within 5 minutes; anonymous submissions receive on-screen reference token only; no PII stored for anonymous path
7. FR-M41-007: evidence attachment upload (optional): PDF, image, video accepted up to 50 MB; stored in MinIO; URL reference linked to finding record; oversized files rejected at intake with clear error

Recommendation lifecycle:
8. FR-M41-008: finding lifecycle: SUBMITTED -> TRIAGED -> DRAFT_RECOMMENDATION -> COMMISSION_RESPONSE -> STAKEHOLDER_REVIEW -> FINALISED -> ASSIGNED -> IN_PROGRESS -> PARLIAMENT_REFERRAL -> CLOSED; state transitions role-gated and immutable once made
9. FR-M41-009: the Commission departmental response per recommendation: response type (ACCEPTED/ACCEPTED_WITH_MODIFICATION/DECLINED/DEFERRED), rationale, proposed implementation date, required resources; HIGH severity responses mandatory within T+30
10. FR-M41-010: non-response alert: the Commission Chair receives alert at T+25 for any HIGH finding without departmental response; alert repeats daily until resolved
11. FR-M41-011: stakeholder 30-day review window after the Commission response; IAB/KEOG/ELOG endorses or disputes; disputes escalated to the Commission Chair; Chair resolves within 14 days
12. FR-M41-012: the Commission Chair sign-off (HSM): consolidated set submitted; gate blocks if any finding in STAKEHOLDER_REVIEW has unresolved dispute; Chair can approve, return to department, or escalate to Commissioner panel; sign-off triggers owner assignment and implementation date
13. FR-M41-013: sign-off anchored to P05 Merkle chain; Chair certificate reference stored on Recommendation record; anchor confirmed before public tracker update
14. FR-M41-014: Parliament referral package: legislative-action recommendations packaged as signed PDF with Chair HSM signature; transmitted to Parliament Clerk via eCitizen portal; fallback to encrypted email with manual confirmation
15. FR-M41-015: Parliament Liaison records parliamentary response (ADOPTED/REJECTED/DEFERRED) with bill reference number; status and bill reference visible on public tracker
16. FR-M41-016: non-response tracking: if Parliament does not respond within 90 days of referral dispatch, the Commission Chair publishes non-response status on public tracker

Long-drip register:
17. FR-M41-017: long-drip attack register (§XXI.C.20): dedicated register for slow-burn systemic threat findings; is_long_drip flag set at triage; the Commission Security Lead approval required before closure of any long-drip finding
18. FR-M41-018: long-drip scheduler: durable scheduled job triggers review-due alert every 6 months post-closure; dead-letter queue on scheduler failures; daily health-check alert; tested by clock advance in integration suite
19. FR-M41-019: second missed long-drip review auto-escalates to Commissioner panel within 24 hours; escalation event logged with timestamp and commissioner IDs
20. FR-M41-020: long-drip register view: list of §XXI.C.20 findings, next-review-due countdown, owner, escalation status, review history per finding

Next-cycle handoff:
21. FR-M41-021: next-cycle handoff: Planning Lead pulls all ASSIGNED/IN_PROGRESS recommendations into planning register; system generates one-page brief per recommendation with owner, implementation date, and current status
22. FR-M41-022: handoff completeness check: system confirms 100% of ASSIGNED/IN_PROGRESS recommendations have been pulled; alert if any are missed; handoff report signed off by Planning Lead

Public tracker:
23. FR-M41-023: public recommendation tracker on M11: recommendation number, finding type, severity, the Commission response summary, current status, responsible department; updated at least weekly; no PII in any response field
24. FR-M41-024: public tracker filterable by finding type, severity, status, and department; last-updated timestamp shown; public API rate-limited at 500 requests/minute
25. FR-M41-025: intake window: platform live from T+2; citizen and web intake live from T+3; credentialed observer portal from T+5; public tracker first published at T+7

Analytics:
26. FR-M41-026: aggregate dashboards: findings by category, severity, submitter type; response rates; open vs closed; parliament referrals; by-election-cycle comparison charts
27. FR-M41-027: audit trail: every state transition, the Commission response, Chair sign-off, and Parliament response immutably logged with actor, timestamp, role, and IP; appended to P05 Merkle chain

Data and compliance:
28. FR-M41-028: citizen submissions stored encrypted (AES-256-GCM at rest); 7-year retention; data class C; DPIA registered with ODPC; DPA 2019 s.30(1)(a) consent and s.30(1)(b) legal obligation bases documented
29. FR-M41-029: automated PII scan on free-text fields using M28-provided NLP before storage; redacts detected PII; fallback to regex-based scanner if M28 unavailable; scan result logged per submission
30. FR-M41-030: RBAC: COMMISSION_ANALYST required to triage; DEPT_LEAD required to submit departmental response; COMMISSION_CHAIR + HSM required for sign-off; PARLIAMENT_LIAISON required to record Parliament response; SpiceDB enforces all role gates
31. FR-M41-031: rate limiting: public intake 100 requests/hr per IP; bulk upload 10/day per OBSERVER JWT; analyst endpoints 200/hr; Chair sign-off 5/hr
32. FR-M41-032: intake form submission p99 <= 800 ms; M28 clustering response p99 <= 10 seconds per finding
33. FR-M41-033: platform availability >= 99% from T+1 through T+180 (six months post-election); SLO dashboard `eel-iems-41` monitors threshold

## Acceptance criteria

1. AC-M41-01: findings intake platform live at T+2; citizen intake live at T+3; deployment timestamps verified by smoke test; UAT sign-off from the Commission Chair
2. AC-M41-02: citizen submits anonymous finding; no PII stored in any persisted field; automated PII scan returns clean on free text; anonymous path skips consent screen; verified by automated security test and DPA audit
3. AC-M41-03: the Commission Chair signs consolidated recommendation set with HSM; P05 Merkle anchor recorded; sign-off gate blocks when any finding in STAKEHOLDER_REVIEW has unresolved dispute; verified by integration test
4. AC-M41-04: Parliament referral package dispatched within T+90; Parliament Liaison records response; bill reference and status visible on public tracker within 24 hours; verified by UAT
5. AC-M41-05: long-drip 6-month scheduler alert fires within 1 hour of review-due timestamp; second missed review auto-escalates to Commissioner panel within 24 hours; verified by clock-advance integration test
6. AC-M41-06: 20,000 citizen submissions processed within T+7 without queue starvation; deduplication queue drains within 1 hour of peak; p99 submission response <= 800 ms; verified by k6 load test
7. AC-M41-07: M28 AI clustering unavailable; system falls back to keyword-match deduplication; no submission blocked; the Commission Analyst notification sent within 5 minutes; verified by fault-injection test
8. AC-M41-08: WCAG 2.2 AA on citizen intake form, public recommendation tracker, and observer portal; verified by axe-core automated scan and manual audit; zero critical violations
9. AC-M41-09: citizen intake available in 7 languages (EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya); all 7 language packs render without truncation; verified by 7-language smoke test
10. AC-M41-10: HIGH-severity departmental responses 100% complete within T+30; non-response alert fires at T+25; mandatory response deadline enforced by system; verified by UAT
11. AC-M41-11: public tracker updated at least weekly; zero PII fields exposed via any public endpoint; verified by scheduled update integration test and security scan
12. AC-M41-12: platform availability >= 99% from T+1 through T+180; SLO breach triggers alert within 5 minutes; measured by SLO dashboard; verified by SLO report at T+180

## Applicable edge cases

EC-DAT-01: duplicate citizen submission with same idempotency key and body; cached response returned; finding not double-stored; verified by auto-generated per-endpoint idempotency test
EC-DAT-02: duplicate submission with same idempotency key but different body; 422 `idempotency-key-reuse` returned; no partial write
EC-DAT-03: out-of-order bulk observer findings arriving late; buffered until gap filled or routed to DLQ; analyst notified of gap
EC-DAT-07: partial bulk CSV/JSON upload failure; per-row error report generated; successfully parsed rows committed; invalid rows listed with field-level codes
EC-DAT-09: Unicode names and finding descriptions; NFC normalisation applied; grapheme-safe rendering on all surfaces including public tracker
EC-DAT-13: soft-deleted finding (duplicate merged); tombstone honoured in public tracker; purge removes from store without affecting consolidated recommendation
EC-DAT-14: oversized evidence attachment (> 50 MB); rejected at intake with clear error; no partial upload stored; blob store not written
EC-SEC-03: HSM unavailable during Chair sign-off; sign-off pauses; ingest and triage continue; countersigns queue in degraded mode; no data loss
EC-SEC-05: injection on all input fields (description, rationale, evidence URL); parameterised queries only; verified by Semgrep, sqlx compile-time checks, and ZAP
EC-SEC-09: rate-limit evasion on public intake endpoint; keyed by subject/device/IP-slash-64; priority shedding applied; 429 returned with Retry-After header
EC-SEC-10: finding enumeration by sequential public ID; public tracker returns only finalised items with opaque recommendation numbers; no internal IDs exposed
EC-SEC-16: secrets in logs/traces; telemetry refuses Tier A/B data; submitter_id and evidence refs excluded from telemetry; log-scanner fixture in CI
EC-IDN-02: JWT expires mid-request during triage or departmental response; in-flight request completes; next request returns 401 with silent refresh prompt
EC-IDN-09: dual-control Chair sign-off; co-approver certificate revoked between initiating and committing; both certificates re-validated at commit; operation rejected if either invalid
EC-IDN-10: SpiceDB down; fail-closed on all writes; class-C reads only; 503 problem+json returned; no partial state transition committed
EC-HUM-06: screen readers on citizen intake form and public recommendation tracker; full task completion at WCAG 2.2 AA; verified by axe-core and manual scripts
EC-HUM-07: deaf users; KSL video explainers on intake form key flows; captions on all video assets; verified by asset presence check and UAT
EC-HUM-08: 7 Kenyan languages including EN and SW; no string truncation; ICU plurals; font fallback for all scripts; verified by 11-locale and 150% pseudo-locale test
EC-SCL-01: surge of 20,000 citizen submissions within T+7 post-election; governance-svc horizontally scales; deduplication queue drains without starvation; SLA met
EC-SCL-04: consumer lag > 60 seconds on deduplication or recommendation processing queue; page fires; backpressure applied; no OOM; verified by slow-consumer test
EC-LEG-02: anonymous citizen submission; no PII collected or stored; DPA 2019 s.30(1)(a) consent screen skipped on anonymous path; automated PII scan confirms clean before store
EC-LEG-04: finding amendment ordered (e.g., recount of consolidated findings after Chair return); compensating event creates new version; original record immutable; both versions visible with audit proofs
EC-LEG-08: Parliament response to referral arrives near day 90 of 90-day window; accepted; no artificial freeze applied; PARLIAMENT_LIAISON can record response up to and including deadline date
EC-OPS-05: M28 AI clustering service unavailable; keyword-match deduplication fallback activates automatically; the Commission Analyst notified; no submission blocked; M28 status restored on recover
EC-OPS-08: disk full on evidence attachment blob store; 507 returned; queue bounded; no corruption; alert fires; existing records unaffected

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | recommendation lifecycle state machine, long-drip 6-month interval scheduler, deduplication scorer, Chair sign-off gate (no unresolved disputes), PII scan | nextest | M41-UNIT- |
| contract | citizen intake via M13, IAB bulk upload, M28 clustering, P05 anchor, Parliament referral dispatch | Pact | M41-PACT- |
| integration | 30-finding E2E: intake, triage, recommendations, departmental response, Chair sign-off, public tracker, Merkle anchor | integration_test | M41-INT- |
| e2e | simulate post-election: 5 submitter types, triage, draft recs, Chair sign-off, public tracker | nextest e2e | M41-E2E- |
| security | anonymous citizen: verify no PII stored; modify closed recommendation (immutable); non-public finding access without role (deny) | ZAP + red team | M41-SEC- |
| a11y | WCAG 2.2 AA on citizen intake, public tracker, observer portal; 3-language test | axe-core + manual | M41-A11Y- |
| load | 20,000 citizen submissions within T+7 | k6 | M41-LOAD- |
| long-drip | advance system clock 6 months; alert fires; escalation on second missed review | integration_test (clock-advance) | M41-LONGDRIP- |

Tags: `#[trace("FR-M41-009","AC-M41-04","EC-OPS-05")]`

## Runbooks and dashboards

- `docs/runbooks/governance-svc-degradation.md` - extend with review-platform intake failures and M28 clustering unavailability
- `docs/runbooks/long-drip-review.md` - new runbook; 6-month review SLA; Commissioner escalation procedure; dead-letter queue drain
- Grafana dashboard: `eel-iems-41` (alert ruleset: `iems-41-alerts`)
- SLO alerts: long-drip scheduler health; intake queue depth; public tracker weekly update lag

## Open questions

- Parliament eCitizen portal MoU: external; confirm referral delivery channel (API vs structured email) and authentication before T-180; fallback is encrypted email with manual confirmation.
- PII scan tooling: M28 NLP-based PII scan must be confirmed as available for M41 use; if not, a simpler regex-based scanner is needed as fallback; confirm with M28 team.
- Long-drip classification criteria: Strategy §XXI.C.20 defines the category but not a classification rubric; the Commission Security Lead must sign off on the decision criteria before the intake form is built.
