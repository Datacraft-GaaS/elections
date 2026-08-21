# M48 recall

| field | value |
|---|---|
| M-number | M48 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M48-recall.md |

## Purpose

Manages the constitutional recall petition process under Article 104, including signatory collection and verification, threshold reporting, and the consequent by-election handoff to M50. Enforces the 30% registered-voter signature threshold and the 30-day collection window.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Public recall petition status portal | Astro | astro/governance-public/ |
| Petition management admin | Django | django/governance-admin/ |
| Signatory OCR import | Python worker | py/recall-ocr/ |
| Internal API | Rust (axum) | crates/recall/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| RecallPetition | id, constituency_id, mp_id, election_cycle_id, grounds, status (draft/active/closed/upheld/dismissed), signatory_target, filed_at | C | T1-consensus |
| SignatoryRecord | id, petition_id, voter_id, ward, ocr_confidence, verified_at, verification_method | B | T1-consensus |
| ThresholdReport | id, petition_id, total_signatories, registered_voters, percentage, meets_threshold, generated_at | C | T1-audit |
| RecallElection | id, petition_id, by_election_id (FK to M50), created_at | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/recall/petitions | 1 signed-static | public | n/a | list active petitions |
| GET /v1/recall/petitions/{id} | 2 live-per-user | public | n/a | petition status and threshold progress |
| POST /v1/recall/petitions | 3 accredited | JWT+SpiceDB | yes | file new petition (Commissioner dual-sign) |
| POST /v1/recall/petitions/{id}/signatories/import | 3 accredited | JWT+SpiceDB | yes | bulk OCR import of signature sheets |
| GET /v1/recall/petitions/{id}/threshold-report | 3 accredited | JWT+SpiceDB | n/a | current threshold report |
| PUT /v1/recall/petitions/{id}/close | 3 accredited | JWT+SpiceDB | yes | close petition window |
| POST /v1/recall/petitions/{id}/election | 3 accredited | JWT+SpiceDB | yes | create recall election via M50 |

OpenAPI: packages/openapi-contracts/governance/recall.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.recall_petition_filed.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_threshold_met.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_dismissed.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_created.v1 | consumes | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR voter register | internal | crates/recall/ | yes | circuit breaker; threshold computation deferred |
| M05/M06 BVI (recall election BVI mode) | internal | crates/recall/ | yes | advisory; by-election provisioning continues |
| M50 by-election-operations | internal | crates/recall/ | yes | hard dependency for recall election handoff |
| M08 RTS kit (recall BVI mode) | internal | crates/rts-device-api/ | yes | mode switch via signed config bundle |

## ML and AI

- OCR confidence scoring on scanned signature sheets via M28 model registry; images deleted within 1 s of extraction.

## Dependencies

- Modules: M02 (voter register for threshold), M05/M06 (identification for recall election), M08 (kit mode switch), M50 (by-election creation)
- Platform: P01 (SpiceDB authz), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

Petition phase:
1. FR-M48-001: file recall petition: organiser submits petition document PDF, MP details (constituency, NIN), recall grounds (Art. 104(1) closed list: conviction/sentence, Chapter Six violation, abuse of office), and initial signatory list; Commissioner dual-sign required
2. FR-M48-002: recall grounds validation against Art. 104(1) closed list; petition rejected with statutory reference if grounds outside closed list; organiser notified with reason
3. FR-M48-003: validate constituency exists and MP is currently holding office before petition activation; rejected with reason if MP seat vacant or constituency unrecognised
4. FR-M48-004: set 30-day signatory collection window on petition activation; window computed in EAT; boundary-tested +/- 1 s
5. FR-M48-005: import bulk signatory sheets via OCR with per-row confidence score; rows below configurable threshold (default 0.80) flagged for manual re-entry; batch continues
6. FR-M48-006: verify signatory voter IDs against M02 register extract for that constituency; per-row result (VALID / NOT_ON_ROLL / DUPLICATE) returned
7. FR-M48-007: reject duplicate voter ID in same petition; idempotent import; verified count unchanged on re-import of same rows
8. FR-M48-008: compute running total of verified signatories toward Art. 104(2) 30% threshold using integer arithmetic (no floating point)
9. FR-M48-009: generate threshold report with percentage, registered voter count, and ward breakdown; signed by IEBC Petition Officer via P06 HSM; published within 7 days of petition submission
10. FR-M48-010: alert Commission when 30% threshold is met; alert includes constituency, petition ID, and percentage achieved
11. FR-M48-011: close petition window automatically at 30-day expiry; scheduler enforces close at EAT midnight
12. FR-M48-012: dismiss petition when window closes below threshold; written reason and public notice published
13. FR-M48-013: uphold petition and create recall election via M50 when threshold met; uphold event Merkle-anchored in P05
14. FR-M48-014: MP notification: formal email and portal notice on petition submission; MP has 21-day window to file High Court challenge per Elections Act s.45(5); notification timestamp logged
15. FR-M48-015: legal challenge hold: MP files High Court petition; Petition Officer records court-reference number; system sets LEGAL_CHALLENGE_HOLD status; recall election declaration suspended; public portal shows challenge status
16. FR-M48-016: legal challenge resolution: court determination received; Petition Officer records outcome; system transitions to ELECTION_DECLARED or PETITION_DISMISSED accordingly
17. FR-M48-017: signatory CSV format validation: UTF-8 encoding, required columns (voter_id, name, ward, signature_date); per-row error report on format failure; valid rows accepted
18. FR-M48-018: OCR import worker processes batches asynchronously and reports per-row results with confidence score; batch progress visible to Petition Officer
19. FR-M48-019: OCR image deleted from storage within 1 s of data extraction (PII minimisation); deletion timestamp logged; DPIA registered

Public portal and admin:
20. FR-M48-020: public portal displays petition status and threshold progress bar without voter PII; constituency-level aggregate only
21. FR-M48-021: public portal lists all active and closed petitions with status, constituency, and declared date
22. FR-M48-022: admin lists signatories with filter by ward and verification status (VALID / NOT_ON_ROLL / DUPLICATE / FLAGGED)
23. FR-M48-023: admin can manually flag disputed signatory for adjudication; flag reason recorded; flagged rows excluded from threshold count pending resolution
24. FR-M48-024: petition closed status is immutable; corrections applied via compensating events only; both versions retained in audit trail
25. FR-M48-025: export signatory verification report as signed PDF for Commission resolution; signed via P06 HSM

Recall election:
26. FR-M48-026: voter roll preparation: M02 CVR snapshot scoped to constituency at date of recall declaration; snapshot hash computed and signed; published to M15 IAB and public portal within 24 hours
27. FR-M48-027: ballot: single binary question (Recall Yes / Recall No); ballot design follows IEBC standard format; ballot paper generated for print approval
28. FR-M48-028: KIEMS kits provisioned in recall mode for single constituency; biometric voter authentication against recall voter roll; kit serial bound to constituency and election date; BVI recall mode signalled via signed config bundle to M08
29. FR-M48-029: polling-day hourly turnout reported to M16 Election Command Centre and M44 Turnout Forecasting via M08 RTS device
30. FR-M48-030: Art. 104(3) success threshold: majority of registered voters in constituency (not just votes cast) must vote Yes; computation uses voter roll snapshot count; result displayed with full workings; integer arithmetic enforced
31. FR-M48-031: Recall Form R-34A submission by RO per polling station: Yes votes, No votes, spoilt ballots, registered voters, voters turned out; signed and Merkle-anchored
32. FR-M48-032: result publication within 48 hours of poll close; Recall Successful / Recall Unsuccessful determination displayed with supporting counts on public portal
33. FR-M48-033: M50 by-election trigger: if Recall Successful, by-election record created in M50 with constituency details; trigger idempotent; failure retried up to 3 times; IEBC Operations Officer alerted if trigger fails after 3 retries
34. FR-M48-034: petition certificate signed PDF issued by Registrar via P06 HSM; available for download on portal as formal evidence for legal proceedings
35. FR-M48-035: Merkle anchoring of all petition and election events in P05 within 60 seconds of event; public verifier available

Data protection and audit:
36. FR-M48-036: audit trail of all petition status transitions with actor and timestamp; appended to P05 Merkle chain
37. FR-M48-037: concurrent recall petitions in different constituencies supported; M02 cross-reference handles parallel load within 2-hour SLA per petition
38. FR-M48-038: petition filing window and success threshold rules governed by Elections Act s.104 rule engine; formula published before election day; independently verifiable
39. FR-M48-039: signatory personal data (voter_id, name) not retained beyond threshold assessment; deletion confirmed within 1 hour; DPIA registered
40. FR-M48-040: public petition and election pages contain no voter PII; only constituency-level aggregate data published

Accessibility and i18n:
41. FR-M48-041: i18n in 11 languages on public portal; no hardcoded strings; ICU plurals; Swahili mandatory
42. FR-M48-042: WCAG 2.2 AA on all public surfaces and petition submission portal; EN + SW
43. FR-M48-043: threshold computation uses integer arithmetic (no floating point); Semgrep rule enforced in CI

Observability:
44. FR-M48-044: Grafana dashboard `eel-iems-48` exists; SLO alerts for signatory CVR cross-reference <= 2 hours, result publication <= 48 hours, portal p99 <= 2 s; alert ruleset `iems-48-alerts`
45. FR-M48-045: OpenTelemetry traces exported to central collector; trace IDs correlated with petition and result events

Additional:
46. FR-M48-046: petition submission rate-limited by IP/64 and identity; abuse-pattern detection alert raised to Oversight Officer
47. FR-M48-047: results page: Yes/No bar chart; registered-voter-count vs threshold visualisation; recall-determination badge; Merkle path verifier; WCAG 2.2 AA
48. FR-M48-048: all petition and election events signed and appended to P05; P06 HSM signs petition certificate and Recall Form R-34A results
49. FR-M48-049: system availability >= 99.9% during petition upload and election-day windows; SLO burn-rate alert configured
50. FR-M48-050: signatory CSV cross-reference for 50,000 rows completes within 2 hours; streaming with checkpoints; kill-resume test passes
51. FR-M48-051: voter roll snapshot published to M15 IAB within 24 hours of recall declaration; snapshot hash on IAB matches signed hash on portal

## Acceptance criteria

1. AC-M48-01: Art. 104(2) threshold report computed correctly for boundary values (exactly 30.00%, 29.99%, 30.01%); integer arithmetic boundary test passes; no floating-point used
2. AC-M48-02: duplicate voter ID in same petition rejected with reason; idempotent import confirmed; verified count unchanged on re-import; test passes
3. AC-M48-03: OCR image deleted within 1 s of data extraction; integration test measures deletion timestamp; DPIA compliance confirmed
4. AC-M48-04: petition window closes automatically at 30-day EAT midnight expiry with no manual intervention; scheduler test with mocked clock passes
5. AC-M48-05: recall election creation in M50 is idempotent; double-fire test confirms single by-election record; retry mechanism fires within 5 minutes of failure
6. AC-M48-06: petition status FSM transitions are valid; invalid transitions rejected with 422; property test over all state pairs passes
7. AC-M48-07: API p99 <= 2 s under nominal load; k6 test at 99.9% availability class SLA passes
8. AC-M48-08: all public petition and result pages contain no voter PII; automated OpenAPI response scanner confirms; ZAP scan passes
9. AC-M48-09: 11-locale snapshots pass; no hardcoded strings; 150% pseudo-locale renders without truncation
10. AC-M48-10: system availability >= 99.9% during petition upload and election-day windows; SLO burn-rate alert configured and tested
11. AC-M48-11: signatory personal data deleted within 1 hour of threshold assessment completion; integration test measures deletion timestamp against assessment_completed_at
12. AC-M48-12: M50 by-election trigger idempotent; retry fires within 5 minutes of first failure; IEBC Operations Officer alerted after 3 retries; integration test
13. AC-M48-13: WCAG 2.2 AA axe-core audit passes on public petition portal and results page; EN + SW strings verified; 11-locale snapshot tests pass
14. AC-M48-14: signatory CSV cross-reference for 50,000 rows completes within 2 hours; load test with M02 stub passes; streaming checkpoint confirmed by kill-resume test
15. AC-M48-15: Art. 104(3) majority-of-registered-voters success threshold computed correctly; result_yes_votes / voter_roll_count displayed with full workings; verification formula published before election day

## Applicable edge cases

data:
- EC-DAT-01: duplicate signatory import; idempotent; no double-count; verified count unchanged on re-import of same voter IDs
- EC-DAT-02: same signatory import batch with different voter_id body on same idempotency key; 422 idempotency-key-reuse; petition count unchanged
- EC-DAT-07: partial batch failure in OCR import; per-row report returned; valid rows committed; failed rows flagged for manual re-entry; batch continues
- EC-DAT-08: large signatory set (constituency up to 300,000 voters); streaming with checkpoints; kill-resume test passes; bounded memory consumption
- EC-DAT-09: Unicode voter names on signatory sheets; NFC normalisation applied; correct ICU collation in verification report
- EC-DAT-11: Yes votes + No votes + spoilt ballots exceeds voter roll count in R-34A submission; system blocks with reason; RO must correct before result accepted

security:
- EC-SEC-05: injection on grounds or petition fields; parameterised queries only; Semgrep and ZAP pass
- EC-SEC-06: SSRF via petition_doc_url or evidence URL; allowlist enforced; private-range redirect blocked; Nuclei test passes
- EC-SEC-10: voter enumeration via signatory list; signatory data redacted on public portal; only constituency-level totals shown; timing oracle delta < 5 ms test passes
- EC-SEC-16: voter IDs must not appear in logs, traces, or OpenTelemetry spans; log-scanner CI fixture passes on every build

identity:
- EC-IDN-02: Petition Officer JWT expires mid-signoff; in-flight operation completes; next request returns 401 and silent refresh
- EC-IDN-09: Commissioner dual-sign required for petition filing; one Commissioner revoked mid-flow; re-validation at commit; filing blocked until valid dual sign obtained
- EC-IDN-10: SpiceDB down when Commissioner attempts dual-sign; fail closed; 503 returned; no partial petition created

scale:
- EC-SCL-04: Kafka consumer lag > 60 s during large signatory batch import; backpressure applied; no row lost; Merkle anchor drains without OOM
- EC-SCL-07: concurrent recall petitions across multiple constituencies; M02 cross-reference handles parallel load; 2-hour SLA per petition maintained

legal:
- EC-LEG-02: court-ordered postponement of recall election; signed calendar amendment propagated to KIEMS kits; kits refuse to open on original date; portal notice updated
- EC-LEG-04: MP files High Court challenge within 21-day window; system transitions to LEGAL_CHALLENGE_HOLD; election declaration suspended; public portal updated; audit event logged

time:
- EC-TIM-08: 30-day petition collection window T+30 midnight EAT statutory; boundary test +/- 1 s passes; inclusive-end semantics per Elections Act
- EC-TIM-09: threshold computation race at petition-window close T+30; OPA hard-close at transaction time; signatories submitted in same second as close handled atomically

human:
- EC-HUM-08: 11 languages on public petition portal; long EN and SW strings; no truncation; 150% pseudo-locale test passes
- EC-HUM-12: OCR on degraded signature sheets (torn, wet, faded); confidence gate < 0.80 forces manual re-entry; valid rows continue without block

ML:
- EC-ML-03: torn, wet, or screen-photographed signature sheets; quality gate routes to manual; 2k degraded corpus evaluation passes confidence threshold

operations:
- EC-OPS-10: PITR restore during active recall petition; drill executed; RPO <= 15 min; RTO <= 4 hours; restore logged to tlog; petition state recovered to last checkpoint

connectivity:
- EC-CON-04: site partition during recall election day; Patroni fencing prevents split-brain; KIEMS kit results queue until primary recovered; no duplicate R-34A submission

security:
- EC-SEC-03: P06 HSM unavailable during petition certificate signing; signing deferred to HSM recovery; certificate marked pending; IEBC Petition Officer notified; Merkle anchoring continues

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | FSM transitions, threshold arithmetic, signatory dedup | nextest | M48-UNIT- |
| property | Threshold computation at boundary; signatory import idempotency | proptest | M48-PROP- |
| contract | M02 register extract; M50 by-election creation | Pact | M48-PACT- |
| integration | Full petition lifecycle with testcontainers | nextest + testcontainers | M48-INT- |
| e2e | Public portal petition status; admin import and close flows | Playwright | M48-E2E- |
| fuzz | OCR import payload; petition grounds field | cargo-fuzz | M48-FUZZ- |
| security | ZAP on public endpoints; PII log scanner | ZAP + Semgrep | M48-SEC- |
| load | Signatory import batch at peak | k6 | M48-LOAD- |
| i18n | 11 locales + pseudo | custom + Playwright | M48-I18N- |
| a11y | axe on public and admin surfaces | axe-core + Playwright | M48-A11Y- |

Tags: `#[trace("FR-M48-001","AC-M48-01","EC-TIM-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with recall section)
- docs/runbooks/recall-petition-ocr-failure.md (new)
- platform/observability/governance-svc/recall-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Article 104(3) interpretation: whether 30% threshold is of registered voters at last election or current register; legal opinion required before threshold rule engine is finalised.
- Whether recall petitions can overlap in time for the same MP; rule engine must handle edge case.
