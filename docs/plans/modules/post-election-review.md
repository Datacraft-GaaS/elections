# M41 post-election-review

| field | value |
|---|---|
| M-number | M41 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M41-post-election-review.md |

## Purpose

Capture structured lessons-learned from every stakeholder category (IEBC departments, IAB, KEOG, ELOG, accredited observers, media, and citizens) after each election cycle. Track each finding through a lifecycle from submission to recommendation status, parliamentary response, and IEBC Chair sign-off. Feed confirmed recommendations back into next-cycle planning. The long-drip attack register (§XXI.C.20) ensures systemic weaknesses are formally tracked with mandatory 6-month review intervals so institutional memory does not dissipate between cycles.

Strategy reference: §XXI.C.20 (long-drip attack mitigation), §XXI (post-election review cycle).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Internal Analyst Console | Phoenix LiveView | services/governance-svc/ex/review_web/ |
| IEBC Chair Sign-Off Panel | Phoenix LiveView | services/governance-svc/ex/review_web/ |
| Observer and Stakeholder Portal | Astro | services/governance-svc/astro/review-portal/ |
| Citizen Intake Form (WCAG 2.2 AA) | Astro | services/governance-svc/astro/review-portal/ |
| Public Recommendation Tracker (WCAG 2.2 AA) | Astro | services/governance-svc/astro/review-portal/ |
| API | Rust (axum) | services/governance-svc/crates/governance-api/ |

Languages: citizen intake EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya; internal portal EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| LessonsLearningFinding | id, election_cycle, submitter_category (IEBC_DEPT/IAB/KEOG/ELOG/EU_EOM/DOMESTIC_OBSERVER/MEDIA/CITIZEN), submitter_id (None if anonymous), finding_type (TECHNICAL/PROCESS/LEGAL/SECURITY/ACCESSIBILITY/COMMUNICATION), severity (HIGH/MEDIUM/LOW), description, evidence_refs, recommended_action, is_long_drip, duplicate_of, status, submitted_at | C | T2 |
| Recommendation | id, finding_ids (consolidated), recommendation_text, eel_response_type (ACCEPTED/ACCEPTED_WITH_MODIFICATION/DECLINED/DEFERRED), eel_response_rationale, owner_dept_id, implementation_date, requires_legislation, parliament_referral_id, chair_signed_at, chair_signed_by, next_cycle_handoff_at, last_long_drip_review | B | T1-consensus |
| ParliamentReferral | id, recommendation_ids, referral_package_hash, chair_signed_at, clerk_received_at, parliament_response (ADOPTED/REJECTED/DEFERRED), bill_reference, parliament_response_at | A | T1-audit |
| LongDripAlert | id, finding_id, review_due_at (every 6 months post-closure), reviewer_id, reviewed_at, escalated_to_commissioner, escalated_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/governance/review.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/review/findings | 2 / public anon | Submitter JWT or public | yes | submit finding; anonymous citizens accepted |
| POST /v1/review/findings/bulk | 3 | JWT+SpiceDB (OBSERVER) | yes | bulk CSV/JSON finding import |
| GET /v1/review/findings | 2 | JWT+SpiceDB (IEBC_ANALYST) | n/a | list findings; filterable by type and status |
| PATCH /v1/review/findings/{id} | 2 | JWT+SpiceDB (IEBC_ANALYST) | yes | triage: deduplicate, categorise, escalate |
| GET /v1/review/recommendations | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | list recommendations |
| PATCH /v1/review/recommendations/{id} | 2 | JWT+SpiceDB (DEPT_LEAD) | yes | submit departmental response |
| POST /v1/review/recommendations/{id}/sign | 2 | JWT+SpiceDB (IEBC_CHAIR) + HSM | yes | Chair sign-off; gate: no unresolved disputes |
| GET /v1/review/public | 1 | public | n/a | public recommendation tracker; no PII |
| POST /v1/review/parliament-referrals | 2 | JWT+SpiceDB (IEBC_CHAIR) + HSM | yes | create and sign Parliament referral package |
| PATCH /v1/review/parliament-referrals/{id} | 2 | JWT+SpiceDB (PARLIAMENT_LIAISON) | yes | record Parliament response |
| GET /v1/review/long-drip | 2 | JWT+SpiceDB (IEBC_SECURITY) | n/a | long-drip register with review status |
| POST /v1/review/long-drip/{id}/review | 2 | JWT+SpiceDB (IEBC_SECURITY) | yes | record 6-month review |
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

1. FR-M41-001: structured intake (submitter category, election cycle, finding type, severity, description, evidence refs, recommended action)
2. FR-M41-002: multi-channel intake: internal portal (IEBC + IAB), observer portal (KEOG/ELOG/EU EOM), public form via M13 and web, bulk CSV/JSON upload for observer reports
3. FR-M41-003: finding deduplication via keyword matching; optional M28 AI clustering; analyst confirms or keeps separate
4. FR-M41-004: recommendation lifecycle: SUBMITTED -> TRIAGED -> DRAFT_RECOMMENDATION -> IEBC_RESPONSE -> STAKEHOLDER_REVIEW -> FINALISED -> ASSIGNED -> IN_PROGRESS -> PARLIAMENT_REFERRAL -> CLOSED; state transitions immutable once made
5. FR-M41-005: departmental response per recommendation: response type (ACCEPTED/ACCEPTED_WITH_MODIFICATION/DECLINED/DEFERRED), rationale, proposed implementation date, required resources
6. FR-M41-006: stakeholder 30-day review after IEBC response; disputes escalated to IEBC Chair
7. FR-M41-007: IEBC Chair sign-off (HSM) on consolidated set; Chair can approve, return to department, or escalate to Commissioner panel; sign-off triggers owner assignment and escalation path
8. FR-M41-008: Parliament referral package for legislative-action recommendations; signed by Chair; Parliament Liaison records response (ADOPTED/REJECTED/DEFERRED) with bill reference
9. FR-M41-009: long-drip attack register (§XXI.C.20); mandatory 6-month review intervals; second missed review auto-escalates to Commissioner panel
10. FR-M41-010: next-cycle handoff: Planning Lead pulls ASSIGNED/IN_PROGRESS recommendations; one-page brief per recommendation generated
11. FR-M41-011: public recommendation tracker on M11: recommendation number, finding type, severity, IEBC response, status, responsible department; updated weekly; no PII
12. FR-M41-012: dashboards: findings by category, severity, submitter type; response rates; parliament referrals; by-cycle comparison
13. FR-M41-013: citizen intake in EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luhya; anonymous submission option
14. Citizen submissions stored encrypted; anonymous option; DPA 2019 s.30(1)(a) and s.30(1)(b); 7-year retention; DPIA registered

## Acceptance criteria

1. AC-M41-01: citizen submits finding anonymously; no PII stored; public tracker updated; UAT
2. AC-M41-02: IEBC Chair signs consolidated set with HSM; Merkle-anchored; sign gate blocks if unresolved dispute; integration test
3. AC-M41-03: Parliament referral package dispatched within T+90; liaison records response; tracking visible on portal; UAT
4. AC-M41-04: long-drip 6-month alert fires; escalation triggers on second missed review; clock-advance test
5. AC-M41-05: 20,000 citizen submissions within T+7 processed without queue starvation; load test
6. AC-M41-06: WCAG 2.2 AA on citizen intake form, public tracker, and observer portal; seven-language smoke test

## Applicable edge cases

**operations**
- EC-OPS-05: M28 clustering unavailable; M41 falls back to keyword-match deduplication; analyst notified; no submission blocked

**legal**
- EC-LEG-02: anonymous citizen submission; no PII collected; DPA consent screen skipped (anonymous path); automated PII scan on free text before storage
- EC-LEG-04: long-drip scheduler silently fails; dead-letter queue; daily health-check alert; tested by clock advance in integration suite

**human**
- EC-HUM-06: WCAG 2.2 AA; citizen intake form seven-language test
- EC-HUM-08: citizen intake available in 7 Kenyan languages including EN and SW

**scale**
- EC-SCL-01: 20,000 citizen submissions within T+7; governance-svc horizontally scales; deduplication queue drains without starvation

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
- Long-drip classification criteria: Strategy §XXI.C.20 defines the category but not a classification rubric; IEBC Security Lead must sign off on the decision criteria before the intake form is built.
