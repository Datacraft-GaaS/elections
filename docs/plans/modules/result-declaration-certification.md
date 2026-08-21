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

1. FR-M51-001: Initiate declaration for contest linked to M10 tally.
2. FR-M51-002: Declare result with HSM-countersigned certificate.
3. FR-M51-003: Enforce 50%+1 rule for presidential declaration; block if not met.
4. FR-M51-004: Enforce 25% in 24 counties for presidential (Art. 138(4)); block if not met.
5. FR-M51-005: Create run-off state when 50%+1 not met for presidential.
6. FR-M51-006: Create TIE_PENDING when candidates tied; no certificate issued.
7. FR-M51-007: Issue CertificateOfElection with HSM signature and gazette package URL.
8. FR-M51-008: Gazette package submitted to M52 automatically on certificate issuance.
9. FR-M51-009: Judiciary acknowledgement hook recorded on M58 handover.
10. FR-M51-010: Petition window opened automatically on declaration; close at 28 days.
11. FR-M51-011: Petition window status publicly queryable without PII.
12. FR-M51-012: Correction via compensating event with dual-officer approval; original immutable.
13. FR-M51-013: Re-declaration after amendment re-signs certificate with new version.
14. FR-M51-014: Both original and amended declarations shown with inclusion proofs (EC-LEG-04).
15. FR-M51-015: Declaration blocked when tally arithmetic invalid (EC-DAT-11).
16. FR-M51-016: Admin console lists all declarations with contest, status, and certificate link.
17. FR-M51-017: Admin can trigger judicial evidence bundle compilation in M40.
18. FR-M51-018: All events signed with Ed25519 SPIFFE-issued key; consumers verify before applying.
19. FR-M51-019: Event stream append-only; kani append-only property passes.
20. FR-M51-020: RTO ≤30 min, RPO ≤5 min; DR drill documented.
21. FR-M51-021: Referendum threshold handoff from M49 triggers declaration path.
22. FR-M51-022: By-election declaration handoff from M50.
23. FR-M51-023: Recall election declaration handoff from M48.
24. FR-M51-024: Audit trail of all declaration state transitions.

## Acceptance criteria

1. AC-M51-01: Amend-after-declare issues compensating event; original event immutable; kani append-only proof passes (EC-LEG-04, EC-LEG-11).
2. AC-M51-02: Run-off property test: all distributions where no candidate reaches 50%+1 produce runoff_created event; tie property produces tie_pending (EC-LEG-05, EC-LEG-06).
3. AC-M51-03: Certificate HSM-signed; SoftHSM failure triggers degraded queue; page within 30 s (EC-SEC-03).
4. AC-M51-04: Petition window boundary: T+28d 23:59:59 EAT inclusive; boundary test +-1 s (EC-TIM-08).
5. AC-M51-05: result_declared.v1 emitted exactly once per contest; Kafka transactions; consumer idempotency test.
6. AC-M51-06: Declaration blocked when arithmetic invalid; RO override creates DisputeCase (EC-DAT-11).
7. AC-M51-07: p99 API latency within election-critical 99.99% SLA under 60k concurrent load.
8. AC-M51-08: RTO ≤30 min, RPO ≤5 min verified in staging DR drill.

## Applicable edge cases

**Legal:**
- EC-LEG-04: recount ordered; new version via compensating event; both shown with proofs.
- EC-LEG-05: presidential run-off; declaration blocked; run-off created.
- EC-LEG-06: tie; TIE_PENDING; no certificate.
- EC-LEG-11: correction after declaration; amendment + re-declaration dual custody.
- EC-TIM-08: petition window T+28d midnight EAT statutory; boundary test.

**Time:**
- EC-TIM-03: backend clock jump backward; HLC never regresses.
- EC-TIM-06: event time < causation time; accept; audit skew anomaly.

**Security:**
- EC-SEC-01: signature mismatch; tamper topic; page ≤30 s.
- EC-SEC-03: HSM unavailable; sign queue; degraded mode.
- EC-SEC-13: split-view on transparency log; witness divergence.

**Data:**
- EC-DAT-11: totals exceed registered voters; block with reason.
- EC-DAT-01: duplicate declare request; idempotent.

**Scale:**
- EC-SCL-07: 60k concurrent RTS submissions feeding declaration; SLAs met.
- EC-CON-04: site partition; RPO ≤5 min.

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
