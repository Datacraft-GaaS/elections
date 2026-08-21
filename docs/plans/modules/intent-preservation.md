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
| GET /v1/intent/adjudication/{id} | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | fetch adjudication detail |
| GET /v1/intent/adjudication/{id}/photo | 2 | JWT+SpiceDB (IAB_AUDITOR) | n/a | access ballot photo (access-logged, AES-256) |
| GET /v1/intent/rejection-rates | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | rejection rates per station and constituency |
| GET /v1/intent/rejection-alerts | 2 | JWT+SpiceDB (COMMISSION_RO) | n/a | statistical outlier alerts |
| PATCH /v1/intent/rejection-alerts/{id} | 2 | JWT+SpiceDB (COMMISSION_RO) | yes | record review outcome |
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
4. FR-M27-004: ballot design published at T-30 for party-agent and public review; change after T-30 requires chair approval + IAB notification
5. FR-M27-005: ballot design artefacts (PDF + test-results summary) in T-14 evidentiary bundle
6. FR-M27-006: large-print ballot version available at every polling station on request; Presiding Officer records provision on Form 34A
7. FR-M27-007: ballot design accessibility reviewed with voters with low vision and low literacy at T-60; results included in T-14 evidentiary bundle
8. FR-M27-008: opt-in pre-mark check prompt displayed after voter marks ballot, before depositing; voter can decline with one tap; 10-second auto-dismiss defaults to Skip; no default to participation
9. FR-M27-009: opt-in prompt and result display localised in 12 Kenyan languages (EN, SW, Turkana, Somali, Dholuo, Gikuyu, Kamba, Luhya, Kalenjin, Meru, Borana, Taita); audio plays automatically without voter navigating a menu
10. FR-M27-010: on-device vision model (INT8 quantised, <= 10 MB, no internet call during inference); ballot held face-down to camera; inference <= 3 s; returns VALID/UNMARKED/MULTIPLE_MARKS/AMBIGUOUS_MARK/WRONG_BALLOT
11. FR-M27-011: result displayed in plain language in active session language; replacement offered if issue detected per Elections Act s.54
12. FR-M27-012: MANDATORY: ballot image processed on-device; deleted within 1 second of inference via OS secure-erase API; no image, feature vector, or artefact transmitted or stored
13. FR-M27-013: kit logs opted_in (bool), result_code, voter_proceeded per event; no voter_id; station-aggregate only
14. FR-M27-014: opt-in rate published as anonymised station-level aggregate in post-election report
15. FR-M27-015: Presiding Officer can disable pre-mark validation for a session; requires reason code + party-agent witness; disable event feeds M26 Layer 1 telemetry; ECC analyst reviews any station with > 5% disable rate
16. FR-M27-016: pre-mark validation disabled gracefully if camera hardware faults; kit records hardware_fault disable reason; event flows to M26 as telemetry alert
17. FR-M27-017: pre-mark vision model registered in M28 model registry with published model card before T-90; eval harness results included; model card public-accessible via /v1/intent/model-card
18. FR-M27-018: Elections Act s.54 spoilt-ballot replacement; up to 3 times per voter per contest
19. FR-M27-019: kit tracks spoilt events per voter per contest; session-only; third spoil locks further replacement and logs event
20. FR-M27-020: third spoilt-ballot lock: kit locks further replacement; Presiding Officer guidance to voter; lock event included in Form 34A
21. FR-M27-021: spoilt ballot retained by Presiding Officer in RFID-tagged sealed envelope; count recorded on Form 34A
22. FR-M27-022: spoilt ballots retained for 30-day petition window; available for IAB sample audit
23. FR-M27-023: any ballot Presiding Officer cannot clearly accept or reject enters adjudication flow
24. FR-M27-024: Presiding Officer + >= 2 party agents view ballot together; each party agent states position; Presiding Officer determines with reason code
25. FR-M27-025: reason-code catalogue: A1 UNMARKED_BALLOT, A2 OVER_MARK, A3 AMBIGUOUS_MARK, A4 TORN_DEFACED, A5 VOTER_IDENTIFIABLE, A6 NON_STANDARD_MARK, A7 WRONG_CONTEST_BALLOT, A8 UNAUTHENTICATED_BALLOT, V1 VALID_STANDARD_MARK, V2 VALID_CROSS_ACCEPTED, V3 VALID_AMBIGUOUS_ACCEPTED; new category requires chair sign-off
26. FR-M27-026: kit records adjudication event: reason code, PO decision, party-agent signatures, timestamp; ballot photographed with kit camera; photo encrypted AES-256; synced centrally; included in Form 34A attachments
27. FR-M27-027: adjudication event signed by kit HSM key before transmission; merkle_node_id assigned on central receipt; anchored in P05 transparency log
28. FR-M27-028: party-agent dissent recorded with agent name and reason; dissent log included in Form 34A
29. FR-M27-029: adjudicated-ballot photographs accessible only to Returning Officers and IAB auditors; all access logged with officer identity and reason; deletion 7 years post-election
30. FR-M27-030: ballot photo access triggers access-log event; IAB has continuous read access to access log
31. FR-M27-031: voter education: interactive tutorial in Public Verifier app (Learn tab); IVR audio guide in EN + SW + 5 additional languages; sample ballot at every polling station entrance; 15-episode radio drama series
32. FR-M27-032: practice ballot: voter completes marking exercise in Public Verifier app; anonymous; discarded; shows real ballot design
33. FR-M27-033: barrier-specific messaging: elderly voters (simplified audio), first-time voters (step-by-step illustrated), North Eastern and Coast regions (language-specific)
34. FR-M27-034: public Learn tab visit target > 2M before election day; analytics reported in post-election report
35. FR-M27-035: Presiding Officer standardised adjudication training: 100+ example ballots with official reasoning; interactive quiz; 30-day pre-election online refresher; in-person training; laminated reference card
36. FR-M27-036: training completion tracked in CRMS (M12); deployment blocked without completing online module and in-person session
37. FR-M27-037: per-station reconciliation after close: rejected + spoilt + valid + assisted = ballots issued; arithmetic mismatch triggers Form 34A hold pending PO explanation
38. FR-M27-038: reconciliation result stored as eel.support.premark_aggregate.v1 event; signed and queued on kit; replayed on reconnect without duplication (idempotent)
39. FR-M27-039: outlier flagging: any station rejection rate > 3 sigma above constituency mean auto-flagged for Returning Officer review; flag published in post-election report
40. FR-M27-040: z-score computed per constituency per contest; RejectionRateAlert records station_rejection_rate, constituency_mean_rejection_rate, z_score
41. FR-M27-041: IAB adjudication-accuracy audit: >= 5% sample of adjudicated-ballot photographs; IAB auditor vs PO comparison; disagreement rate published; target > 99% agreement
42. FR-M27-042: post-election rejection analysis report: per-contest, per-county, per-reason-code; comparison to 2013/2017/2022 baselines; published within T+30
43. FR-M27-043: rejection rate report on public portal as signed content bundle; interactive choropleth map; downloadable CSV
44. FR-M27-044: all eel.support.intent.v1 events delivered at-least-once idempotent; duplicate submission same id+body returns cached response; no re-execution
45. FR-M27-045: duplicate submission same key, different body returns 422 idempotency-key-reuse
46. FR-M27-046: API rate limiting on all /v1/intent endpoints per rate table; 429 with Retry-After header on breach; no voter-enumeration surface
47. FR-M27-047: secrets (HSM keys, JWT signing keys) excluded from all telemetry, traces, and logs; CI regex gate enforces no Tier A/B secret in log lines
48. FR-M27-048: WCAG 2.2 AA on all voter-facing screens (opt-in prompt, result display) and officer-facing screens (adjudication workflow, rejection rate dashboard); axe-core audit in CI
49. FR-M27-049: spoilt-ballot reconciliation completed within 60 minutes of station close; latency alert if exceeded
50. FR-M27-050: consumer lag on eel.support.intent.v1 topic monitored; alert fires if lag > 60 s; backpressure applied; no OOM under sustained load
51. FR-M27-051: SpiceDB enforces role boundaries for ballot photo access; write operations fail closed if SpiceDB unavailable; class-C reads served from cache
52. FR-M27-052: JWT expiry mid-adjudication handled gracefully; in-flight event completes; subsequent 401 triggers silent refresh; no data loss

## Acceptance criteria

1. AC-M27-01: rejected ballot rate < 1.0% per contest nationally; measurement: Form 34A aggregate post-election; sign-off: chair
2. AC-M27-02: pre-mark validation inference time <= 3 s on reference kit hardware; measurement: kit performance test on reference hardware under sustained load
3. AC-M27-03: ballot image deleted within 1 second of inference; 100% coverage; measurement: OS erase log audit; 0 exceptions
4. AC-M27-04: 100% of adjudicated ballots have a reason code from the standardised catalogue; measurement: Form 34A audit by IAB post-election
5. AC-M27-05: IAB adjudication-accuracy audit agreement rate > 99%; measurement: IAB audit report on >= 5% sample of adjudicated-ballot photographs
6. AC-M27-06: rejection-rate outlier alert available to Returning Officers within 4 hours post Form 34C publication; measurement: alert timestamp vs Form 34C publication timestamp
7. AC-M27-07: post-election rejection report published <= T+30; measurement: publication timestamp
8. AC-M27-08: ballot image and feature vector absent from all network logs and server-side stores; measurement: packet capture + storage audit; 0 occurrences
9. AC-M27-09: WCAG 2.2 AA audit pass on all voter-facing and officer-facing screens; measurement: independent axe-core + manual audit report
10. AC-M27-10: 100% Presiding Officers completed online adjudication module and in-person session before deployment; measurement: CRMS training log
11. AC-M27-11: eel.support.intent.v1 events delivered without loss over 8-hour simulated offline session; measurement: netem offline integration test; gap audit shows 0 missing events
12. AC-M27-12: duplicate pre-mark event submission same key+body returns cached response without re-execution; measurement: auto-generated idempotency test per write endpoint

## Applicable edge cases

**data**
- EC-DAT-01: duplicate pre-mark event same key+body; cached response; no re-execution
- EC-DAT-02: same pre-mark event key, different body; 422 idempotency-key-reuse
- EC-DAT-11: spoilt count > ballots issued in reconciliation; Form 34A hold; PO explanation required

**security**
- EC-SEC-01: adjudication event signature mismatch on ingest; rejected; tamper topic emitted; page <= 30 s
- EC-SEC-03: HSM unavailable during ballot photo sync; photo queue held; ingest continues; degraded-mode alert
- EC-SEC-05: injection on all /v1/intent inputs; parameterised queries only; Semgrep + ZAP + fuzz enforced
- EC-SEC-10: rejection rate endpoint probed for voter enumeration; constant-time response shape; rate cap
- EC-SEC-11: timing side-channel on ballot photo access check; constant-time compare
- EC-SEC-16: secrets absent from all logs and traces; CI regex gate + log-scanner fixture

**identity**
- EC-IDN-02: JWT expires mid-adjudication; in-flight completes; next 401 + silent refresh
- EC-IDN-05: Frida hooking on kit during pre-mark inference; RASP terminates; tamper event emitted
- EC-IDN-08: voter under duress; covert BVI alert; intent-preservation flow continues; officer observes
- EC-IDN-10: SpiceDB down during ballot photo access request; fail closed; 503 problem+json; no photo served

**scale**
- EC-SCL-04: consumer lag on eel.support.intent.v1 > 60 s; page; backpressure; no OOM

**ML**
- EC-ML-01: pre-mark model misclassifies valid ballot as MULTIPLE_MARKS; voter overrides; model result is advisory only
- EC-ML-02: adversarial ballot image fed to on-device model; confidence gate; no crash; AMBIGUOUS_MARK returned

**human**
- EC-HUM-01: worn fingerprints on PO for adjudication sign-off; face biometric fallback; PIN with party-agent witness
- EC-HUM-02: amputee voter; face path; reason code V3 or A6 on adjudicated ballot
- EC-HUM-05: assisted voting; assistant identity logged; no ballot content exposed; reason code recorded
- EC-HUM-06: screen readers on opt-in prompt and adjudication workflow; WCAG 2.2 AA compliance
- EC-HUM-08: 12-language opt-in prompt; audio auto-plays without voter navigating a menu

**legal**
- EC-LEG-01: candidate withdraws after ballot printed; ballot design frozen; tallied + annotated; vacancy rules applied
- EC-LEG-03: s.44A invoked for physically assisted voter; assistant identity and consent logged; no secrecy leak

**operations**
- EC-OPS-01: kit battery death mid-adjudication; journal-before-act; exact resume on restart; event not duplicated

**time**
- EC-TIM-07: s.14A station close extension; HSM-signed fence reopens; pre-mark validation continues in extended window

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

Tags: `#[trace("FR-M27-012","AC-M27-03","EC-ML-01")]`

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
