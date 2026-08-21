# M62 transmission-legal-basis-audit

| field | value |
|---|---|
| M-number | M62 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M62-transmission-legal-basis-audit.md |

## Purpose

Verifies 100% of RTS transmission events against the statutory legal rules engine, producing per-event compliance verdicts and party-level summaries, with full audit SDK coverage as the primary acceptance criterion.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Compliance admin console | Django | django/governance-admin/ |
| Party compliance portal | Django (party role) | django/governance-admin/ (party subdomain) |
| Internal API | Rust (axum) | crates/legal-audit/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| TransmissionEvent | id, form34a_event_id (FK to M10 event), station_id, transmitted_at, legal_rule_set_version, verdict (compliant/non-compliant/disputed), verdict_reason | C | T1-audit |
| ComplianceVerdict | id, transmission_event_id, rule_code, passed, detail | C | T1-audit |
| LegalRule | id, rule_code, description, statutory_basis, version, effective_from | C | T1-consensus |
| PartySummary | id, election_id, party_id, total_transmissions, compliant_count, non_compliant_count, computed_at | D | T2 |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/legal-audit/rules | 1 signed-static | public | n/a | current rule set |
| GET /v1/legal-audit/verdicts | 3 accredited | JWT+SpiceDB | n/a | list verdicts with filter |
| GET /v1/legal-audit/verdicts/{id} | 3 accredited | JWT+SpiceDB | n/a | verdict detail |
| GET /v1/legal-audit/party-summary | 3 accredited | JWT+SpiceDB | n/a | party compliance summary |
| POST /v1/legal-audit/verdicts/{id}/dispute | 3 accredited | JWT+SpiceDB | yes | party disputes verdict |
| GET /v1/legal-audit/verdicts/export | 4 bulk | JWT+SpiceDB | n/a | full audit export |

OpenAPI: packages/openapi-contracts/governance/transmission-legal-basis-audit.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.transmission_verdict_issued.v1 | emits | eel.governance.legal_audit.v1 | operational | no | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |
| eel.rts.form34a_amended.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M10 rts-backend (transmission events) | internal | crates/legal-audit/ | yes | hard dependency; audit runs on event stream |
| M40 election-court-it-interface | internal | crates/legal-audit/ | yes | advisory; non-compliant bundle available |
| P05 transparency log | internal | crates/legal-audit/ | yes (SoftHSM) | degraded; verdict still issued locally |

## ML and AI

none

## Dependencies

- Modules: M10 (transmission events), M40 (court bundle)
- Platform: P04 (Kafka), P05 (transparency log for verdict hashing)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Event ingestion and coverage:

1. FR-M62-001: Consume every eel.rts.form34a_captured.v1 and eel.rts.form34a_amended.v1 event from M10 within 30 s of publication (FR-M62-01).
2. FR-M62-002: Track ingestion lag; alert SRE if ingestion lag exceeds 60 s.
3. FR-M62-003: Achieve 100% coverage of all transmission events in the election cycle; alert if any gap detected.
4. FR-M62-004: Duplicate transmission event produces idempotent verdict; no double-count.
5. FR-M62-005: Handle form types: 32A, 32B, 34A, 34B, 34C.

Rule engine:

6. FR-M62-006: Apply versioned legal rule set to each transmission event; produce per-rule result: pass, fail, or not-applicable (FR-M62-02).
7. FR-M62-007: Rules cover: form compliance (reg. 6), transmission window (reg. 7), authorised signatory (s.39(3)), digital signature (reg. 10), party-agent notification (s.44A), chain-of-custody completeness (M20 link), and sequence integrity including no pre-close transmission (FR-M62-02).
8. FR-M62-008: Verdict includes: overall pass/fail/partial, per-rule result with failure reason codes, and evidence references as event hashes (FR-M62-03).
9. FR-M62-009: Verdict immutable once generated; Merkle-anchored in P05 immediately; retroactive modification blocked at API level and logged as security event.
10. FR-M62-010: Version-control every rule set change: author, change description, effective date, Commission Legal Counsel approval (FR-M62-08).
11. FR-M62-011: Verdicts reference rule-engine version active at time of audit; retrospective re-audit with alternate version possible for analysis only.
12. FR-M62-012: Alert within 5 minutes when rule set changes after election activation (rule drift alert).
13. FR-M62-013: Degrade gracefully when M12 CRMS unavailable; cache last-known accreditation state; flag degraded-mode verdicts.

Dashboard and escalation:

14. FR-M62-014: Real-time aggregate compliance dashboard: total transmissions, compliance rate by rule, non-compliant queue, county compliance heat map (FR-M62-04).
15. FR-M62-015: Dashboard refresh p99 < 1 s.
16. FR-M62-016: Non-compliance escalation for any verdict with overall fail or critical-rule fail (signatory, signature, pre-close); alert to Commission RTS Operations Lead and Legal Counsel within 30 s of verdict generation (FR-M62-05).
17. FR-M62-017: Escalation includes station code, failing rule codes, and evidence hashes.
18. FR-M62-018: Operations Lead can trigger re-transmission request or initiate manual verification workflow from escalation queue.
19. FR-M62-019: Escalation resolution status tracked: retransmission, manual_verification, accepted, unresolved.

Compliance records and evidence:

20. FR-M62-020: Generate per-station s.39 compliance record after transmission window closes; all verdicts for the station; final compliance status; HSM-signed (FR-M62-06).
21. FR-M62-021: Generate aggregate s.39 compliance certificate for the full election on Commission Legal Counsel request.
22. FR-M62-022: Generate signed evidence bundle for specified stations or full election within 4 hours of request; bundle includes verdicts, rule-engine version at audit time, source event hashes, and P05 Merkle inclusion proofs (FR-M62-07).
23. FR-M62-023: Pre-generate and store full-election evidence bundle at T+7 for petition-window readiness.
24. FR-M62-024: s.39 records for all stations generated by T+2 (NFR-M62-08).

Party agent and IAB:

25. FR-M62-025: Party portal shows aggregate compliance summary per party: pass rate, most common failure reason, escalation count; no per-transmission detail (FR-M62-09).
26. FR-M62-026: Party can dispute a non-compliant verdict with a reason code; dispute idempotent.
27. FR-M62-027: Dispute triggers manual review workflow; reviewer assigns final verdict; finding moves to disputed or resolved.
28. FR-M62-028: s.44A invocation audit: track officer, station, timestamp, reason code, and protocol compliance for every s.44A event (FR-M62-10).
29. FR-M62-029: Real-time compliance event SSE stream to M15 IAB Dashboard.
30. FR-M62-030: Export full audit as signed XLSX and JSON.

Admin:

31. FR-M62-031: Rule set published publicly; signed; CDN-served; immutable reference.
32. FR-M62-032: Admin dashboard with compliance rate by party and station.
33. FR-M62-033: Audit trail of all verdicts, escalations, and dispute events in P05.
34. FR-M62-034: Per-station s.39 record available within 1 hour individually if full evidence bundle unavailable.

## Acceptance criteria

1. AC-M62-01: 100% of form34a_captured events receive a verdict within 60 s of event arrival; coverage gap metric alert fires if any event remains unprocessed; integration test.
2. AC-M62-02: Verdict p99 < 5 s from event ingestion to verdict stored; measured at 2,000 events/minute sustained for 2 hours; k6 load test.
3. AC-M62-03: Non-compliance alert delivered to Legal Counsel p99 < 30 s from verdict generation; integration test with alert double.
4. AC-M62-04: Every verdict hash included in P05 Merkle chain; inclusion proof independently verifiable; integration test.
5. AC-M62-05: Rule drift alert fires within 5 minutes of LegalRule change after election activation; timed integration test.
6. AC-M62-06: Verdict immutability: attempt to modify generated verdict via API blocked with 403 and logged as security event; test.
7. AC-M62-07: Evidence bundle for full election generated within 4 hours of request in staging simulation; all stations represented; HSM signature valid.
8. AC-M62-08: Duplicate transmission event produces idempotent verdict; no double-count; property test.
9. AC-M62-09: Dispute workflow routes to manual review; idempotent double-fire produces single dispute record; integration test.
10. AC-M62-10: Party agent cannot see stations outside their registered party's contests; SpiceDB access-control integration test with opposing-party credentials.
11. AC-M62-11: M12 CRMS unavailable: rule engine degrades gracefully using cached accreditation state; degraded-mode verdicts flagged; chaos test.
12. AC-M62-12: M62 availability during transmission window T-1 to T+2: 99.9%; SLO burn-rate alert fires before threshold breach.
13. AC-M62-13: s.44A audit coverage: 100% of s.44A events audited by T+1; integration test with M10 double emitting s.44A events.
14. AC-M62-14: Consumer lag <= 60 s under 2,000 events/minute peak; lag alert fires at 60 s; k6 load test.

## Applicable edge cases

EC-DAT-01: duplicate form34a event with same idempotency key returns cached verdict; no second verdict created; no double-count.
EC-DAT-02: same idempotency key with different form34a body returns 422 idempotency-key-reuse.
EC-DAT-03: out-of-order form34a events on the stream; buffer until gap filled or DLQ after timeout; shuffled replay produces same verdicts.
EC-DAT-05: poison message with structurally invalid transmission payload; DLQ with validation reason; partition continues processing.
EC-DAT-06: schema v1/v2 form34a events coexist during rolling M10 deployment; both accepted; Karapace compat check.
EC-DAT-12: Debezium slot lag causes WAL bloat on verdict table; alert at 60 s; outbox retained; disk guard active.
EC-SEC-01: signature mismatch on consumed form34a event; verdict issued with overall fail and tamper reason code.
EC-SEC-05: injection on verdict reason, dispute description, and rule fields; parameterised queries only; ZAP + Semgrep.
EC-SEC-13: split-view on P05 transparency log during verdict anchoring; witness divergence alert; consistency probe fails; verdict flagged pending.
EC-SEC-16: officer IDs and station codes must not appear unredacted in debug logs or traces; log-scanner fixture; CI regex.
EC-IDN-02: JWT expires mid-verdict query; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-03: SVID rotates during high-volume transmission batch; old SVID valid to expiry; eel_producer_key_id per event.
EC-IDN-10: SpiceDB down; writes fail closed; read-only class-C reads from replica; 503 problem+json returned.
EC-SCL-03: hot partition on form34a topic when all stations transmit simultaneously; stream-id spread configured; lag alert; HPA triggers additional workers.
EC-SCL-04: Kafka consumer lag > 60 s on form34a verdict topic; SRE paged; backpressure applied; no OOM.
EC-SCL-08: Valkey eviction loses idempotency cache entries; PG-durable verdict store used as authority; Valkey is cache only.
EC-TIM-06: event time < causation time (late transmission arrives after window close); verdict issued with skew annotation; projection independent of wall-clock order.
EC-TIM-07: s.14A close time moves from 17:00 to 19:00; transmission window re-evaluated against updated HSM-signed statutory time; forged extension rejected.
EC-TIM-08: transmission window boundary at statutory close; boundary test at +- 1 s; inclusive semantics confirmed.
EC-CON-11: Kafka ISR shrinks to 1 during peak transmission burst; producer blocks; outbox accumulates; SRE paged; no loss on ISR recovery.
EC-LEG-09: T-60 auto-referral race generates duplicate referral attempts; single referral produced; idempotent scheduler.
EC-OPS-10: PITR restore during petition filing window; restore logged to tlog; RPO/RTO within SLA; evidence bundle pre-generated before restore point preserved.
EC-HUM-06: compliance dashboard accessible to screen readers and keyboard-only users; full task completion at WCAG 2.2 AA; axe + manual test.
EC-SEC-04: KEK rotation during active transmission window; online re-wrap of verdict encryption keys; old verdicts readable until re-wrap completes; audit trail.
EC-IDN-09: dual-control required for manual rule engine update (Legal Counsel approval); approver revoked mid-workflow; re-validate at commit; update blocked.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Rule engine per-code, verdict FSM, dispute routing | nextest | M62-UNIT- |
| property | 100% coverage invariant; idempotency; timing boundary | proptest | M62-PROP- |
| contract | M10 form34a event; M40 court bundle | Pact | M62-PACT- |
| integration | Full verdict pipeline; transparency log inclusion | nextest + testcontainers | M62-INT- |
| e2e | Party portal compliance summary; dispute | Playwright | M62-E2E- |
| load | Consumer at 60k event/s RTS peak | k6 | M62-LOAD- |
| security | ZAP; log scanner; signature mismatch verdict | ZAP + Semgrep | M62-SEC- |

Tags: `#[trace("FR-M62-005","AC-M62-01","EC-TIM-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with legal-audit section)
- docs/runbooks/transmission-coverage-gap.md (new; trigger when 100% coverage dips)
- platform/observability/governance-svc/legal-audit-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml (coverage gap alert, lag alert)

## Open questions

- Rule drift: the upstream sw/ text may reference rule sets that conflict with ADR constraints; a rule engine config review is needed before election cycle activation.
- Whether disputed verdicts that remain unresolved by declaration day require automatic escalation to court or remain in administrative limbo.
