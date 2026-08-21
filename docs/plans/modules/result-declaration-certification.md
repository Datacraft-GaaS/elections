# M51 result-declaration-certification

| field | value |
|---|---|
| M-number | M51 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.3 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M51-result-declaration-certification.md |

## Purpose

Event-sourced declaration lifecycle: produces HSM-countersigned CertificateOfElection for each contest, handles run-off and tie states, supports correction via compensating events, and manages the petition window tracker with judiciary acknowledgement hooks.

## Surfaces and languages

| surface | technology | path under services/rts-svc/ |
|---|---|---|
| Declaration admin console | Django | django/declaration-admin/ |
| Internal API | Rust (axum + tonic) | crates/declaration/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Declaration | id (UUIDv7), contest_id, election_id, status (pending/declared/run-off/tie/amended/annulled), declared_at, declared_by | C | T1-consensus (event-sourced) |
| CertificateOfElection | id, declaration_id, winner_id, hsm_signature, gazette_package_url, issued_at | C | T1-consensus |
| PetitionWindowTracker | id, declaration_id, window_open_at, window_close_at, petition_filed | C | T1-consensus |
| DeclarationAmendment | id, declaration_id, reason, amended_by, previous_snapshot_hash, new_snapshot_hash, amendment_at | B | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/declarations | 3 accredited | JWT+SpiceDB | yes | initiate declaration for contest |
| PUT /v1/declarations/{id}/declare | 3 accredited | JWT+SpiceDB | yes | declare result; HSM sign; emit event |
| PUT /v1/declarations/{id}/runoff | 3 accredited | JWT+SpiceDB | yes | set run-off state |
| PUT /v1/declarations/{id}/tie | 3 accredited | JWT+SpiceDB | yes | set TIE_PENDING |
| POST /v1/declarations/{id}/amend | 3 accredited | JWT+SpiceDB | yes | compensating event + re-declaration |
| GET /v1/declarations/{id} | 2 live-per-user | JWT | n/a | declaration state |
| GET /v1/declarations/{id}/certificate | 2 live-per-user | JWT | n/a | signed certificate PDF/JSON |
| GET /v1/declarations/{id}/petition-window | 2 live-per-user | public | n/a | petition window status |
| PUT /v1/declarations/{id}/judiciary-ack | 3 accredited | JWT+SpiceDB | yes | judiciary acknowledgement hook |

OpenAPI: packages/openapi-contracts/rts/result-declaration-certification.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.declaration.result_declared.v1 | emits | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.declaration.result_amended.v1 | emits | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.declaration.runoff_created.v1 | emits | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.declaration.tie_pending.v1 | emits | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.declaration.certificate_issued.v1 | emits | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.rts.tally_published.v1 | consumes | eel.rts.tally.v1 | event-source | yes | at-least-once idempotent |
| eel.governance.referendum_threshold_met.v1 | consumes | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M10 rts-backend (tally source) | internal | crates/declaration/ | yes | declaration blocked until tally published |
| M52 election-calendar-gazette | internal | crates/declaration/ | yes | advisory; gazette package submitted |
| M40 election-court-it-interface | internal | crates/declaration/ | yes | advisory; petition bundle available |
| M58 parliamentary-judiciary-reporting | internal | crates/declaration/ | yes | advisory; judiciary handover |
| HSM PKCS#11 | I07 | integrations/ (internal crate) | yes (SoftHSM) | degraded: sign queue; page |

## ML and AI

none

## Dependencies

- Modules: M10 (tally), M52 (gazette), M40 (court bundle), M58 (judiciary)
- Platform: P04 (Kafka exactly-once), P05 (transparency log for certificate), P06 (HSM)
- Packages: eel-outbox-rs (exactly-once mode), eel-crypto-rs, eel-resilience-rs

## Feature checklist

Declaration initiation and pre-checks:
1. FR-M51-001: Initiate Declaration for a contest only after eel.rts.tally_published.v1 for that contest is consumed and signature-verified; otherwise 409 `tally-not-published`.
2. FR-M51-002: Block initiation while unconfirmed Form 34A/37A count for the scope exceeds the configurable threshold (default 0); response lists outstanding station codes.
3. FR-M51-003: Pre-declaration checklist computed server-side: forms confirmed, totals reconciled to turnout, agent quorum recorded, legal sign-off present; each item green/red with reason.
4. FR-M51-004: Legal-review step (POST legal-review) required before sign; sign endpoint returns 412 `legal-review-missing` until recorded.
5. FR-M51-005: Legal review lead time enforced: RO sign allowed no earlier than 30 min after legal review opened unless Legal Officer waives with reason.
6. FR-M51-006: Declaration blocked when tally arithmetic invalid (spoilt > valid, totals > registered); RO override creates a DisputeCase in M16 and is logged (EC-DAT-11).
7. FR-M51-007: Declaration blocked for national presidential contest until M21 RLA pass event recorded; 412 `rla-pending`.
8. FR-M51-008: Each declare action requires FIDO2 step-up plus biometric sign-off of the acting officer; assertion hash stored on the event.

Winner determination and statutory rules:
9. FR-M51-009: Compute winner by simple plurality for MP, Senator, Governor, CWR, MCA from the signed tally snapshot; never from mutable projections.
10. FR-M51-010: Enforce presidential 50%+1 of valid votes cast; block declaration and set RUN_OFF when not met (Art. 138(4)).
11. FR-M51-011: Enforce presidential 25% in at least 24 of 47 counties with exact >= semantics; boundary test at 24 and 23 counties and at exactly 25.00%.
12. FR-M51-012: Emit eel.declaration.runoff_created.v1 with the top two candidates and activate M49 second-event planning.
13. FR-M51-013: Tie produces TIE_PENDING and eel.declaration.tie_pending.v1; no certificate issuable while in that state.
14. FR-M51-014: DeclarationTieBreak records draw method (LOT/COIN), winner, at least two witness signatures including a Legal Officer; transitions TIE_PENDING to DECLARED.
15. FR-M51-015: Referendum path: consume eel.governance.referendum_threshold_met.v1 from M49, record PASSED/FAILED, winner id null; 24-of-47 county check uses >= (EC-LEG-07).
16. FR-M51-016: By-election (M50) and recall (M48) declarations reuse the same FSM with election_event_id scoping; no code path forks.
17. FR-M51-017: Candidate withdrawn or deceased after ballot print remains in the tally with annotation; declaration applies vacancy rules and flags Legal (EC-LEG-01).

Declaration statement and certificate:
18. FR-M51-018: Generate bilingual (EN + SW) statutory declaration statement pre-filled from tally and M12 candidate identity; RO verifies rather than types.
19. FR-M51-019: Statement edit-locked once legal review is recorded; any change after lock requires re-review.
20. FR-M51-020: RO signs via HSM-backed e-signature; wet-ink path accepts scan upload with SHA-256 recorded as declaration_statement_hash.
21. FR-M51-021: Generate Form 39 Certificate of Election PDF with winner, seat, constituency, election date, RO signature, Commission seal, within 60 s of signature.
22. FR-M51-022: Certificate PDF HSM-signed (P06) and its hash anchored in P05 transparency log; inclusion proof stored on CertificateOfElection.
23. FR-M51-023: Certificate viewer exposes PDF, JSON, Merkle inclusion proof, and QR verification link; PublicRead tier, no PII beyond candidate name and totals.
24. FR-M51-024: Certificate verification endpoint recomputes hash and checks inclusion against the current STH; returns tamper verdict.
25. FR-M51-025: Unicode candidate names rendered NFC with grapheme-safe truncation on PDF and in notifications (EC-DAT-09).
26. FR-M51-026: Zero-vote candidates and zero-turnout stations render "0"; no division by zero in margin computation (EC-DAT-10).

Notifications and handoffs:
27. FR-M51-027: Gazette notice package pushed to M52 within 5 min of RO signature; async with retry; declaration status GAZETTE_PENDING until M52 ack.
28. FR-M51-028: Judiciary notification letter (HSM-signed PDF) sent to High Court or Supreme Court per seat via secure API; 3 retries, then secure-email fallback with manual ack.
29. FR-M51-029: Attorney General notice for presidential result delivered within 24 h with read-receipt (Art. 138(5)); missing receipt at 20 h pages Legal.
30. FR-M51-030: Losing candidates notified by email and SMS with final totals and petition-window close date; payload minimised to name and totals.
31. FR-M51-031: Notification log shows per-candidate delivery status with manual retry; retries idempotent per candidate and declaration version.
32. FR-M51-032: Judiciary acknowledgement hook (PUT judiciary-ack) records registrar identity and timestamp; M58 handover event emitted.
33. FR-M51-033: Admin can trigger M40 judicial evidence bundle compilation for a declaration; 24 h clock honoured even inside freeze (EC-LEG-08).
34. FR-M51-034: Publication trigger to M11 emitted only after certificate issued; portal shows declaration and certificate link.

Petition window:
35. FR-M51-035: Petition window opens at declaration; closes T+28 d 23:59:59 EAT inclusive, computed in UTC and displayed in gazetted TZ (EC-TIM-04, EC-TIM-08).
36. FR-M51-036: Alerts to Legal at T-7, T-3, T-1 with 100% delivery; calendar entry pushed to M52.
37. FR-M51-037: POST petition (JudiciaryAPIKey) marks petition_lodged and status PETITIONED; idempotent on court case number.
38. FR-M51-038: Petition window status publicly queryable without PII; Legal dashboard shows countdown per constituency.

Amendment and correction:
39. FR-M51-039: Amend after declare only via compensating eel.declaration.result_amended.v1; original event never mutated or deleted.
40. FR-M51-040: Amendment requires dual-officer approval (RO plus Legal Officer) with distinct FIDO2 assertions and a signed reason.
41. FR-M51-041: Re-declaration after amendment issues a new certificate version; prior certificate marked superseded, both retrievable with inclusion proofs (EC-LEG-04, EC-LEG-11).
42. FR-M51-042: Recount-ordered amendments link to the court order reference and to the new M10 tally event id.
43. FR-M51-043: Annulment state reachable only from a court-order event; certificate revocation published to P05 and M11.

Event sourcing and integrity:
44. FR-M51-044: All emitted events CloudEvents 1.0, Ed25519-signed with SPIFFE-issued key (spiffe://eel.local/ns/rts/sa/declaration); consumers verify before apply.
45. FR-M51-045: Event stream append-only; kani proof of append-only and FSM invariants passes in CI.
46. FR-M51-046: Exactly-once emission via eel-outbox-rs transactional mode; consumer-side idempotency keyed on event id.
47. FR-M51-047: Out-of-order tally events buffered until causal gap filled or routed to DLQ after timeout (EC-DAT-03).
48. FR-M51-048: Replayed signed events rejected on monotonic counter and routed to .tamper topic (EC-DAT-04).
49. FR-M51-049: Schema v1 and v2 coexist; dual-emit during migration; Karapace compat check gates deploy (EC-DAT-06).
50. FR-M51-050: Events over 1 MB rejected by outbox; PDF payloads stored in MinIO with hash reference only (EC-DAT-14).
51. FR-M51-051: Legacy pre-CloudEvents tally payload upgraded by shim with original signature preserved (EC-DAT-16).
52. FR-M51-052: HLC timestamps on every event; backend clock jump never regresses HLC; event-time-before-causation accepted and flagged (EC-TIM-03, EC-TIM-06).
53. FR-M51-053: Leap-second and 23:59:60 timestamps parsed without error (EC-TIM-05).
54. FR-M51-054: Projection rebuild from event log is checkpoint-resumable with bounded memory; full rebuild of all contests completes under 10 min.

Access, security, and operations:
55. FR-M51-055: All write endpoints idempotent via Idempotency-Key; same key and body returns cached response; same key different body returns 422 (EC-DAT-01, EC-DAT-02).
56. FR-M51-056: SpiceDB unavailable fails writes closed with 503 problem+json; class-C reads continue (EC-IDN-10).
57. FR-M51-057: HSM unavailable queues countersigns, enters degraded mode, pages within 30 s; no declaration finalises without signature (EC-SEC-03).
58. FR-M51-058: Telemetry redacts officer identity and candidate contact data; log scanner passes (EC-SEC-16).
59. FR-M51-059: RTO <= 30 min, RPO <= 5 min; site-partition chaos drill documented.
60. FR-M51-060: Admin console lists declarations by contest, status, version, certificate link; filters by county and seat type; WCAG 2.2 AA.
61. FR-M51-061: Every state transition recorded with actor, HLC timestamp, output document hash; audit export to M40 and M58.
62. FR-M51-062: Grafana dashboard and PrometheusRule burn-rate alerts for declaration latency, HSM queue depth, consumer lag, notification failures.

## Acceptance criteria

1. AC-M51-01: Amend-after-declare emits a compensating event; original event bytes unchanged on replay; kani append-only proof passes (EC-LEG-04, EC-LEG-11).
2. AC-M51-02: Property test over 10,000 random presidential vote distributions: every case with no candidate at 50%+1 or under 24 counties at 25% yields runoff_created; every exact tie yields tie_pending; zero false declarations (EC-LEG-05, EC-LEG-06, EC-LEG-07).
3. AC-M51-03: Certificate issued and HSM-signed within 60 s of RO signature at p99; P05 inclusion proof available within 60 s; verified in integration.
4. AC-M51-04: SoftHSM kill during declaration: countersign queued, degraded alert within 30 s, no unsigned certificate served, queue drains with zero gaps on recovery (EC-SEC-03).
5. AC-M51-05: Petition window boundary table at T+28 d 23:59:59 EAT passes at -1 s (open), 0 s (open), +1 s (closed); DST and leap-second rows included (EC-TIM-04, EC-TIM-05, EC-TIM-08).
6. AC-M51-06: result_declared.v1 emitted exactly once per contest version under Kafka broker kill and outbox retry; consumer replays 1,000 duplicates with a single applied state change.
7. AC-M51-07: Invalid arithmetic declaration blocked with reason; RO override creates DisputeCase with linked declaration id; TLA+ invariant holds (EC-DAT-11).
8. AC-M51-08: Certification API p99 <= 10 s, GET endpoints p95 <= 500 ms and p99 <= 1 s, with 60k concurrent RTS submissions feeding declaration; <= 10% p99 regression at 5x (EC-SCL-07).
9. AC-M51-09: Availability 99.99% over 30-day rolling window during the T-0 to T+3 window; error budget 4.38 min/month tracked by burn-rate alerts.
10. AC-M51-10: Staging DR drill with primary-site partition: RTO <= 30 min, RPO <= 5 min, no split-brain on the declaration stream (EC-CON-04).
11. AC-M51-11: Gazette package received by M52 within 5 min and Judiciary ack within 30 min at p99 in integration; 3 retries then secure-email fallback exercised in chaos test.
12. AC-M51-12: AG notice for presidential result delivered with read-receipt within 24 h in e2e; T-7, T-3, T-1 petition alerts 100% delivered across 290 constituencies.
13. AC-M51-13: Auto-generated idempotency tests pass on all 9 write endpoints: duplicate returns cached response, key reuse with different body returns 422 (EC-DAT-01, EC-DAT-02).
14. AC-M51-14: Generated documents and RO screens available in EN and SW; UI strings externalised for all 11 locales plus pseudo-locale with no hardcoded strings.
15. AC-M51-15: axe-core zero violations on RO checklist, statement preview, certificate viewer, petition dashboard; manual screen-reader script signed (WCAG 2.2 AA).
16. AC-M51-16: Rust unit coverage >= 85% and Django >= 85% with lcov uploaded; mutation score >= 70%; fuzz targets for declaration payload and certificate fields run 1 h clean.
17. AC-M51-17: Declaration and CertificateOfElection retained permanently; DeclarationAmendment audit rows retained 7 years; no soft-delete path exists (Semgrep rule).
18. AC-M51-18: ZAP and Semgrep report zero critical or high findings; ASVS L3 checklist signed; log scanner finds no officer or candidate PII in traces (EC-SEC-05, EC-SEC-16).
19. AC-M51-19: Insider tamper simulation: parallel-tally drift injected before certification is detected and blocks national certification; dual-custody enforced for national declare.
20. AC-M51-20: Full projection rebuild from event log for all contests completes under 10 min with bounded memory after kill-resume (EC-DAT-08).

## Applicable edge cases

Legal:
- EC-LEG-01: candidate withdraws or dies after print; tally annotated; declaration applies vacancy rules and routes to Legal.
- EC-LEG-02: court-ordered postponement; declaration initiation refused for the affected contest until amended calendar event is consumed.
- EC-LEG-03: s.44A invoked at a station; declaration checklist shows manual-ID stations; Legal must acknowledge before sign.
- EC-LEG-04: recount ordered; new version via compensating event; both versions shown with inclusion proofs.
- EC-LEG-05: presidential run-off; declaration blocked; runoff_created emitted; M49 activated.
- EC-LEG-06: tie; TIE_PENDING; no certificate until witnessed draw recorded.
- EC-LEG-07: referendum 24-of-47 counties; exact >= semantics; boundary test at 23, 24, 25.
- EC-LEG-08: evidence bundle requested at T+6 d 23 h; 24 h clock honoured; not freeze-blocked.
- EC-LEG-11: correction after declaration; amendment plus re-declaration under dual custody; tlog inclusion of both.
- EC-TIM-08: petition window T+28 d 23:59:59 EAT inclusive per Art. 87 and s.75; boundary table +-1 s.

Time:
- EC-TIM-01: tallying-centre kit clock skew over 5 min; tally event carries low time confidence; declaration warns and records skew.
- EC-TIM-02: NTS unreachable; events stamped with last authenticated time plus monotonic; `time_confidence=low` surfaced on checklist.
- EC-TIM-03: backend clock jumps backward; HLC never regresses; declared_at ordering preserved; skew alert.
- EC-TIM-04: DST and TZ on diaspora tallies; storage in UTC; petition close and display computed in gazetted EAT.
- EC-TIM-05: leap second; 23:59:60 timestamp in tally or court event parsed without rejection.
- EC-TIM-06: event time earlier than causation time; accepted, skew anomaly audited, never reordered in projection.
- EC-TIM-07: s.14A close moved 17:00 to 19:00; declaration initiation waits for HSM-signed extension fence; forged extension rejected.
- EC-TIM-09: register freeze race; declaration reads the frozen register snapshot id from the tally event, never live registers.

Identity:
- EC-IDN-02: RO JWT expires mid-sign; in-flight declare completes; next request 401 with silent refresh.
- EC-IDN-10: SpiceDB down; declare and amend fail closed with 503 problem+json; public certificate reads continue.

Data:
- EC-DAT-01: duplicate declare with same key and body; cached response; no second event.
- EC-DAT-02: same idempotency key, different body; 422 `idempotency-key-reuse`.
- EC-DAT-03: tally events out of order; buffer until gap filled or DLQ; declaration waits.
- EC-DAT-04: replayed signed tally or declaration event; rejected on counter; .tamper topic.
- EC-DAT-05: poison tally message with votes over registered; DLQ with reason; other contests continue.
- EC-DAT-06: tally schema v1 and v2 coexist; both accepted; dual-emit declaration events during migration.
- EC-DAT-07: partial batch failure in bulk notifications; per-candidate report; successful deliveries committed.
- EC-DAT-08: large event-log rebuild; streaming, checkpoint-resumable, bounded memory; kill-resume test.
- EC-DAT-09: Unicode candidate names; NFC, ICU collation, grapheme-safe on PDF, SMS, and portal.
- EC-DAT-10: zero votes or zero turnout; valid; margin computation has no div-by-zero; renders "0".
- EC-DAT-11: spoilt over valid or totals over registered; block with reason; RO override creates DisputeCase.
- EC-DAT-12: Debezium slot lag on outbox; alert at 60 s; outbox retained; declaration events not lost.
- EC-DAT-13: superseded certificate soft-deleted in search index; tombstone honoured; verification still finds the original by hash.
- EC-DAT-14: oversized event with embedded PDF; outbox rejects over 1 MB; blob to MinIO with hash reference.
- EC-DAT-15: M12 returns two candidate identities for one id; AdjudicationCase; certificate generation paused; no auto-merge.
- EC-DAT-16: legacy pre-CloudEvents tally payload; shim upgrades envelope; signature preserved and verified.

Security:
- EC-SEC-01: event signature mismatch; never applied; tamper topic; page within 30 s.
- EC-SEC-03: HSM unavailable; countersigns queue; degraded mode; no certificate served unsigned.
- EC-SEC-05: injection attempts on reason, witness, and case-number fields; parameterised queries only; fuzz and ZAP clean.
- EC-SEC-13: split-view on transparency log; witness divergence fails certificate verification.
- EC-SEC-16: officer biometrics, FIDO2 assertions, candidate contacts never in logs or traces; scanner fixture passes.

Scale and connectivity:
- EC-SCL-04: declaration consumer lag over 60 s behind tally stream; page; backpressure; no OOM.
- EC-SCL-07: 60k concurrent RTS submissions feeding declaration; SLAs met at 1x; <= 10% p99 regression at 5x.
- EC-CON-04: site partition primary/mirror; Patroni fencing; no split-brain on declaration stream; RPO <= 5 min.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Declaration FSM, run-off/tie rules, amendment compensating event | nextest | M51-UNIT- |
| property | Run-off/tie distributions; append-only invariant; idempotency | proptest | M51-PROP- |
| formal | Kani append-only; TLA+ Tally extension | kani + TLC | M51-FORMAL- |
| contract | M10 tally; M52 gazette; M40 court bundle | Pact | M51-PACT- |
| integration | Full declaration lifecycle; amendment; run-off; DR failover | nextest + testcontainers | M51-INT- |
| e2e | Admin declare; certificate download; petition window | Playwright | M51-E2E- |
| fuzz | Declaration payload; certificate fields | cargo-fuzz | M51-FUZZ- |
| load | 60k concurrent at election-critical 99.99% | k6 | M51-LOAD- |
| chaos | HSM kill; site partition; Kafka ISR shrink | Chaos Mesh | M51-CHAOS- |
| security | ZAP; HSM degraded; log scanner | ZAP + Semgrep | M51-SEC- |

Tags: `#[trace("FR-M51-012","AC-M51-01","EC-LEG-04")]`

## Runbooks and dashboards

- docs/runbooks/rts-svc-degradation.md (extend with declaration section)
- docs/runbooks/declaration-hsm-failure.md (new)
- docs/runbooks/declaration-amendment.md (new)
- platform/observability/rts-svc/declaration-dashboard.json
- platform/observability/rts-svc/slo-alerts.yaml (election-critical burn rate, HSM lag)

## Open questions

- Run-off and tie handling for non-presidential contests (gubernatorial tied result): whether TIE_PENDING applies or a separate lottery rule; legal opinion needed.
- Whether gazette package submission to M52 is synchronous in the critical path or async with retry; async preferred but requires testing gazette SLA.
