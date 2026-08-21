# M27 intent-preservation

| field | value |
|---|---|
| M-number | M27 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.5 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M27-intent-preservation.md |

## Purpose

Minimise rejected and spoilt ballots through seven complementary elements: optimised ballot design, opt-in kit-side pre-mark visual validation (on-device only, image deleted within 1 second of inference), spoilt-ballot replacement per Elections Act s.54, human adjudication of ambiguous ballots using a standardised reason-code catalogue (A1-A8 for rejection, V1-V3 for acceptance), multi-channel voter education, presiding officer training, and ex-post per-station rejection-rate reconciliation with statistical outlier flagging; target is rejected ballot rate < 1.0% per contest nationally.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Voter pre-mark opt-in prompt (on kit) | Flutter | services/support-admin-svc/apps/intent-kit/ |
| Adjudication workflow (Presiding Officer, on kit) | Flutter | services/support-admin-svc/apps/intent-kit/ |
| Rejection Rate Dashboard (IEBC Returning Officer) | Phoenix LiveView | services/support-admin-svc/ex/intent_web/ |
| IAB Adjudication Audit panel | Django | services/support-admin-svc/django/iab_audit/ |
| Public rejection analysis report | Astro | services/support-admin-svc/astro/intent-public-site/ |
| API | Rust (axum) | services/support-admin-svc/crates/support-admin-api/ |

Languages: EN + SW baseline; opt-in prompt and result display in 12 Kenyan languages including Turkana, Somali, Dholuo, Gikuyu, Kamba, Luhya, Kalenjin, Meru, Borana, Taita, Mijikenda, Nandi.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PreMarkValidationEvent | id, station_code, contest_id, opted_in (bool), result_code (VALID/UNMARKED/MULTIPLE_MARKS/AMBIGUOUS_MARK/WRONG_BALLOT), voter_proceeded; no voter_id field; station-aggregate only | C | T2 |
| SpoiltBallotEvent | id, station_code, contest_id, spoilt_number (1-3), replacement_issued, presiding_officer_id, event_at; voter_id encrypted, session-only, cleared on session close | C | T2 |
| AdjudicationEvent | id, station_code, contest_id, reason_code (A1-A8 or V1-V3), presiding_officer_decision, presiding_officer_id, party_agent_positions (agent_id to VALID/REJECT), dissent_recorded, dissent_agent_id, dissent_reason, ballot_photo_blob_id (encrypted, access-logged), event_at, merkle_node_id | A | T1-audit |
| RejectionRateAlert | id, station_code, contest_id, station_rejection_rate, constituency_mean_rejection_rate, z_score, alert_generated_at, reviewed_by_officer_id, review_outcome, resolved_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/support/intent-preservation.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/intent/pre-mark-events | internal gRPC | SVID mTLS | yes | submit pre-mark validation aggregate from kit |
| POST /v1/intent/spoilt-events | internal gRPC | SVID mTLS | yes | submit spoilt ballot event from kit |
| POST /v1/intent/adjudication | internal gRPC | SVID mTLS | yes | submit adjudication event from kit |
| GET /v1/intent/adjudication/{id} | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | fetch adjudication detail |
| GET /v1/intent/adjudication/{id}/photo | 2 | JWT+SpiceDB (IAB_AUDITOR) | n/a | access ballot photo (access-logged, AES-256) |
| GET /v1/intent/rejection-rates | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | rejection rates per station and constituency |
| GET /v1/intent/rejection-alerts | 2 | JWT+SpiceDB (IEBC_RO) | n/a | statistical outlier alerts |
| PATCH /v1/intent/rejection-alerts/{id} | 2 | JWT+SpiceDB (IEBC_RO) | yes | record review outcome |
| GET /v1/intent/report | 1 | public | n/a | post-election rejection analysis report |
| GET /v1/intent/model-card | 1 | public | n/a | pre-mark vision model card |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.support.premark_aggregate.v1 | emits | eel.support.intent.v1 | operational | yes | at-least-once idempotent |
| eel.support.adjudication_event.v1 | emits | eel.support.intent.v1 | operational | yes | at-least-once idempotent |
| eel.support.rejection_rate_alert.v1 | emits | eel.support.intent.v1 | operational | yes | at-least-once idempotent |
| eel.support.rejection_report_published.v1 | emits | eel.support.intent.v1 | operational | yes | exactly-once |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M08 RTS Device | internal | integrations/rts-kit/ | yes | events queued on kit; synced on reconnect |
| M10 RTS Back-End | internal | integrations/rts/ | yes | circuit breaker; aggregates retried |
| M26 Anomaly Detection | internal | integrations/detection/ | yes | rejection-rate outliers fed to Layer 2; pre-mark disable to Layer 1 |
| M28 AI Monitoring Stack | internal | integrations/ai-monitoring/ | yes | pre-mark vision model A7 registered in model registry; eval harness |
| M11 Public Portal | internal | integrations/portal/ | yes | Learn tab voter education; public report |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | adjudication events anchored |
| MinIO | internal | integrations/minio/ | yes | adjudicated-ballot photographs (AES-256, access-logged) |

## ML and AI

- A7 pre-mark ballot validation: on-device vision model (INT8 quantised, <= 10 MB); detects VALID/UNMARKED/MULTIPLE_MARKS/AMBIGUOUS_MARK/WRONG_BALLOT; inference <= 3 s; ballot image deleted within 1 second of inference via OS secure-erase API; no image, feature vector, or inference artefact transmitted or stored; governed by M28 model registry; model card published at T-90

## Dependencies

- Modules: M08, M10, M26, M28, M11
- Platform: P05
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, kit-sdk-flutter

## Feature checklist

1. FR-M27-001: ballot design for all 6 simultaneous contests; tested with >= 200 real Kenyan voters across literacy strata and regions before finalisation
2. FR-M27-002: candidate photograph + party symbol; marking box >= 15 mm per side; colour per contest (presidential yellow, gubernatorial blue, senate green, MP red, women-rep purple, MCA orange)
3. FR-M27-003: per-ballot instructions at Grade 5 reading level in EN + SW; pictogram "make one mark inside the box"
4. FR-M27-004: ballot design published at T-30 for party-agent and public review; change after T-30 requires IEBC Chair approval + IAB notification
5. FR-M27-005: ballot design artefacts (PDF + test-results summary) in T-14 evidentiary bundle
6. FR-M27-006: opt-in pre-mark check prompt displayed after voter marks ballot, before depositing; voter can decline with one tap; 10-second auto-dismiss defaults to Skip; no default to participation
7. FR-M27-007: on-device vision model; ballot held face-down to camera; inference in <= 3 s; returns VALID/UNMARKED/MULTIPLE_MARKS/AMBIGUOUS_MARK/WRONG_BALLOT
8. FR-M27-008: result displayed in plain language; replacement offered if issue detected per Elections Act s.54
9. FR-M27-009: MANDATORY: ballot image processed on-device; deleted within 1 second of inference; OS secure-erase call; no image, feature vector, or artefact transmitted or stored
10. FR-M27-010: kit logs opted_in (bool), result_code, voter_proceeded per event; no voter_id; station-aggregate only
11. FR-M27-011: opt-in rate published as anonymised station-level aggregate in post-election report
12. FR-M27-012: Presiding Officer can disable pre-mark validation for a session; requires reason code + party-agent witness; disable event feeds M26 Layer 1 telemetry; ECC analyst reviews any station with > 5% disable rate
13. FR-M27-013: Elections Act s.54 spoilt-ballot replacement; up to 3 times per voter per contest
14. FR-M27-014: kit tracks spoilt events per voter per contest; session-only; third spoil locks further replacement and logs event
15. FR-M27-015: spoilt ballot retained by Presiding Officer in RFID-tagged sealed envelope; count recorded on Form 34A
16. FR-M27-016: spoilt ballots retained for 30-day petition window; available for IAB sample audit
17. FR-M27-017: any ballot Presiding Officer cannot clearly accept or reject enters adjudication flow
18. FR-M27-018: Presiding Officer + >= 2 party agents view ballot together; each party agent states position; Presiding Officer determines with reason code
19. FR-M27-019: reason-code catalogue: A1 UNMARKED_BALLOT, A2 OVER_MARK, A3 AMBIGUOUS_MARK, A4 TORN_DEFACED, A5 VOTER_IDENTIFIABLE, A6 NON_STANDARD_MARK, A7 WRONG_CONTEST_BALLOT, A8 UNAUTHENTICATED_BALLOT, V1 VALID_STANDARD_MARK, V2 VALID_CROSS_ACCEPTED, V3 VALID_AMBIGUOUS_ACCEPTED; new category requires IEBC Chair sign-off
20. FR-M27-020: kit records adjudication event: reason code, PO decision, party-agent signatures, timestamp; ballot photographed with kit camera; photo encrypted AES-256; synced centrally; included in Form 34A attachments
21. FR-M27-021: party-agent dissent recorded with agent name and reason; dissent log included in Form 34A
22. FR-M27-022: voter education: interactive tutorial in Public Verifier app (Learn tab); IVR audio guide in EN + SW + 5 additional languages; sample ballot at every polling station entrance; 15-episode radio drama series
23. FR-M27-023: practice ballot: voter completes marking exercise in Public Verifier app; anonymous; discarded; shows real ballot design
24. FR-M27-024: barrier-specific messaging: elderly voters (simplified audio), first-time voters (step-by-step illustrated), North Eastern and Coast regions (language-specific)
25. FR-M27-025: Presiding Officer standardised adjudication training: 100+ example ballots with official reasoning; interactive quiz; 30-day pre-election online refresher; in-person training; laminated reference card
26. FR-M27-026: training completion tracked in CRMS (M12); deployment blocked without completing online module and in-person session
27. FR-M27-027: per-station reconciliation after close: rejected + spoilt + valid + assisted = ballots issued; arithmetic mismatch triggers Form 34A hold pending PO explanation
28. FR-M27-028: outlier flagging: any station rejection rate > 3 sigma above constituency mean auto-flagged for IEBC Returning Officer review; flag published in post-election report
29. FR-M27-029: IAB adjudication-accuracy audit: >= 5% sample of adjudicated-ballot photographs; IAB auditor vs Presiding Officer comparison; disagreement rate published; target > 99% agreement
30. FR-M27-030: post-election rejection analysis report: per-contest, per-county, per-reason-code; comparison to 2013/2017/2022 baselines; published within T+30
31. Adjudicated-ballot photos: accessible only to IEBC Returning Officer and IAB auditor; all access logged with officer identity and reason; deletion 7 years post-election
32. Large-print ballot version available at station on request
33. Opt-in prompt and result display in 12 Kenyan languages; audio plays automatically; presiding officer training includes anti-coercion instruction
34. Third spoilt-ballot lock: kit locks further replacement; Presiding Officer guidance to voter; log included in Form 34A

## Acceptance criteria

1. AC-M27-01: rejected ballot rate < 1.0% per contest nationally; Form 34A aggregate
2. AC-M27-02: pre-mark validation inference time <= 3 s on reference kit hardware; kit performance test
3. AC-M27-03: ballot image deleted within 1 second of inference; 100% coverage; OS erase log audit
4. AC-M27-04: 100% of adjudicated ballots have a reason code from the standardised catalogue; Form 34A audit by IAB
5. AC-M27-05: IAB adjudication-accuracy audit agreement rate > 99%; IAB audit report
6. AC-M27-06: rejection-rate outlier alert available to Returning Officers within 4 hours post Form 34C; report timestamp
7. AC-M27-07: post-election rejection report published <= T+30; publication timestamp
8. AC-M27-08: ballot image and feature vector absent from all network logs and server-side stores; privacy audit
9. AC-M27-09: WCAG 2.2 AA audit pass on all voter-facing and officer-facing screens
10. AC-M27-10: 100% Presiding Officers completed online adjudication module and in-person session before deployment; CRMS training log

## Applicable edge cases

**identity**
- EC-IDN-06: voter cannot present biometrics in normal way (amputee, burns); s.44A complementary mechanism; Presiding Officer records reason code on spoilt or assisted ballot
- EC-IDN-08: voter under duress; covert alert from BVI; intent-preservation flow continues; officer observation

**data**
- EC-DAT-11: spoilt count > ballots issued in reconciliation; Form 34A hold triggered; PO explanation required
- EC-DAT-16: adjudicated-ballot photo access outside authorised roles; DPA breach notification; access-log alert

**security**
- EC-SEC-03: adjudicated-ballot photo attempted access without IAB or RO role; SpiceDB denies; access attempt logged
- EC-SEC-10: pre-mark validation disable event without party-agent witness; kit blocks disable; requires dual confirmation

**legal**
- EC-LEG-01: Elections Act s.54 replacement limit reached (third spoilt); kit locks further replacement; voter guidance
- EC-LEG-03: s.44A invoked for physically assisted voter; assistant identity and patient consent logged; no secrecy leak
- EC-LEG-05: Layer 3 rejection-rate outlier used as petition evidence; methodology disclosure in report

**human**
- EC-HUM-01: worn fingerprints on presiding officer; kit uses face biometric for PO sign-off on adjudication; fallback PIN with party-agent witness
- EC-HUM-02: amputee voter; face path; reason code V3 (valid ambiguous accepted) or A6 if mark type non-standard
- EC-HUM-05: assisted voting; assistant identity logged; no ballot content exposed to log
- EC-HUM-06: screen readers; AA compliance on all surfaces
- EC-HUM-08: 12 languages for opt-in prompt; audio plays automatically without voter needing to navigate menu

**operations**
- EC-OPS-01: kit battery death mid-adjudication; journal-before-act; exact resume on restart; adjudication event not duplicated

**ML**
- EC-ML-01: pre-mark vision model misclassifies valid ballot as MULTIPLE_MARKS; voter retains right to override and deposit as-is; model result is advisory; no autonomous action

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | reason-code catalogue validator, pre-mark inference pipeline (mock model), spoilt-ballot counter, rejection-rate z-score, form reconciliation arithmetic | nextest + pytest | M27-UNIT- |
| property | adjudication event Merkle anchor, spoilt count reconciliation invariants | proptest | M27-PROP- |
| contract | kit pre-mark event, adjudication event, Form 34A consumer | Pact | M27-PACT- |
| integration | full pre-mark flow (opt-in, inference, result display, replacement offer); adjudication event recording; Form 34A reconciliation | testcontainers | M27-INT- |
| e2e | simulated polling station with 200 synthetic voters; 20 opted-in pre-mark checks; 5 spoilts; 3 adjudications | Playwright + kit-sim | M27-E2E- |
| privacy | confirm ballot image deleted within 1 second (OS erase API hook); confirm no image or feature vector in any log or network packet | nextest + packet capture | M27-PRIV- |
| a11y | audio-guided opt-in flow; large-text mode; WCAG 2.2 AA | axe-core + manual | M27-A11Y- |
| model | accuracy of on-device vision model on 1,000 annotated synthetic ballots; target >= 97% recall on VALID class | pytest | M27-MODEL- |
| load | 55,393 stations submitting rejection-rate data simultaneously; outlier computation within 4 hours | k6 | M27-LOAD- |
| offline | 8-hour offline session; adjudication events sync on reconnect with integrity preserved | netem | M27-OFFLN- |

Tags: `#[trace("FR-M27-009","AC-M27-03","EC-ML-01")]`

## Runbooks and dashboards

- `docs/runbooks/support-admin-svc-degradation.md` - extend with intent-preservation offline paths and reconciliation failures
- `docs/runbooks/adjudicated-ballot-photo-access.md` - new runbook; DPA breach notification steps; unauthorised access escalation
- Grafana dashboard: `platform/observability/support-admin-svc/intent-preservation-dashboard.json`
- SLO alerts: `platform/observability/support-admin-svc/intent-slo-alerts.yaml` (99.99% pre-mark validation, rejection-rate alert within 4 h)

## Open questions

- Ballot design study (200-voter test) must be completed by 2026-10-31; IEBC Ballot Design Team responsible; Datacraft provides testing infrastructure.
- Vision model training requires annotated synthetic ballot images from IEBC Ballot Design Team; timeline risk if team sign-off delayed.
- National Languages Board / translators: 12-language pre-mark prompt translation must be contracted by 2026-12-01.
- CRMS (M12) integration for training completion tracking: data schema for training records must be agreed with crms-svc team before M25 integration sprint.
