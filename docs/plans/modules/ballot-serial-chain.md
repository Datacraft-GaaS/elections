# M63 ballot-serial-chain

| field | value |
|---|---|
| M-number | M63 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.3 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M63-ballot-serial-chain.md |

## Purpose

Event-sourced chain of custody for ballot serial numbers: assignment from printer lots, issuance to polling stations, spoilt ballot recording, and end-of-day reconciliation. Serial mismatches block result holds silently; all discrepancies raise explicit flags.

## Surfaces and languages

| surface | technology | path under services/rts-svc/ |
|---|---|---|
| Reconciliation admin | Django | django/declaration-admin/ |
| Mobile reconciliation (M31 integration) | Flutter (via M31 poll-clerk) | apps/poll-clerk (field in M31) |
| Internal API | Rust (axum + tonic) | crates/ballot-serial/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| SerialAssignment | id, lot_id (FK M39), station_id, serial_range_start, serial_range_end, assigned_at, assigned_by | C | T1-consensus (event-sourced) |
| IssuanceReceipt | id, assignment_id, clerk_id, received_at, receipt_hash | C | T1-consensus (event-sourced) |
| SpoiltBallotEvent | id, station_id, serial_number, reason, recorded_at, clerk_id | B | T1-consensus (event-sourced) |
| Reconciliation | id, station_id, election_id, issued_count, spoilt_count, cast_count, unaccounted_count, status (pending/held/cleared/flagged), computed_at | C | T1-consensus (event-sourced) |
| ReconciliationHold | id, reconciliation_id, flag_reason, raised_at, resolved_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/ballot-serial/assignments | 3 accredited | JWT+SpiceDB | yes | assign serial range from lot |
| POST /v1/ballot-serial/receipts | 3 accredited | JWT+SpiceDB | yes | confirm issuance receipt |
| POST /v1/ballot-serial/spoilt | 3 accredited | JWT+SpiceDB | yes | record spoilt ballot |
| POST /v1/ballot-serial/reconcile | 3 accredited | JWT+SpiceDB | yes | trigger end-of-day reconciliation |
| GET /v1/ballot-serial/reconcile/{station_id} | 3 accredited | JWT+SpiceDB | n/a | reconciliation status |
| PUT /v1/ballot-serial/reconcile/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | resolve reconciliation hold |
| GET /v1/ballot-serial/assignments | 4 bulk | JWT+SpiceDB | n/a | bulk export for audit |

OpenAPI: packages/openapi-contracts/rts/ballot-serial-chain.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.ballot_serial.serial_assigned.v1 | emits | eel.ballot_serial.chain.v1 | event-source | yes | exactly-once |
| eel.ballot_serial.serial_issued.v1 | emits | eel.ballot_serial.chain.v1 | event-source | yes | exactly-once |
| eel.ballot_serial.ballot_spoilt.v1 | emits | eel.ballot_serial.chain.v1 | event-source | yes | exactly-once |
| eel.ballot_serial.reconciliation_held.v1 | emits | eel.ballot_serial.chain.v1 | event-source | yes | exactly-once |
| eel.ballot_serial.reconciliation_cleared.v1 | emits | eel.ballot_serial.chain.v1 | event-source | yes | exactly-once |
| eel.custody.kit_dispatched.v1 | consumes | eel.custody.kit.v1 | event-source | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M39 ballot-printing-chain (lot source) | internal | crates/ballot-serial/ | yes | hard dependency; assignment blocked without lot |
| M20 inventory-management (kit link) | internal | crates/ballot-serial/ | yes | advisory; kit-station mapping |
| M31 mobile-poll-clerk (spoilt recording) | internal | crates/ballot-serial/ | yes | advisory; offline queue on M31 |
| M08 rts-device (RFID lot scan) | internal | crates/rts-device-api/ | yes | advisory; RFID confirms lot receipt |
| M10 rts-backend (result hold) | internal | crates/ballot-serial/ | yes | hard dependency; mismatch holds result |

## ML and AI

none

## Dependencies

- Modules: M39 (print lot), M20 (inventory), M31 (mobile recording), M08 (RFID), M10 (result hold)
- Platform: P04 (Kafka exactly-once), P05 (transparency log), P06 (HSM)
- Packages: eel-outbox-rs (exactly-once), eel-crypto-rs, eel-resilience-rs

## Feature checklist

1. FR-M63-001: Assign serial range from print lot to polling station.
2. FR-M63-002: Record issuance receipt signed by clerk.
3. FR-M63-003: Record each spoilt ballot with serial number and reason.
4. FR-M63-004: Reject serial number outside assigned range for station.
5. FR-M63-005: Trigger end-of-day reconciliation: issued = cast + spoilt + unaccounted.
6. FR-M63-006: Reconciliation mismatch raises explicit hold flag; never silently ignored.
7. FR-M63-007: Hold flag blocks result publication in M10 until resolved.
8. FR-M63-008: Resolve hold with dual-officer approval and reason.
9. FR-M63-009: All events signed with Ed25519; consumers verify before applying.
10. FR-M63-010: Event stream append-only; compensating events for corrections.
11. FR-M63-011: Kani append-only property proof.
12. FR-M63-012: Bulk export for audit with inclusion proofs from transparency log.
13. FR-M63-013: RFID lot scan from M08 confirms physical lot receipt; mismatch flagged.
14. FR-M63-014: Offline spoilt recording on M31; synced on reconnect; idempotent.
15. FR-M63-015: RTO ≤30 min, RPO ≤5 min; DR drill documented.
16. FR-M63-016: Admin reconciliation dashboard per station and constituency.
17. FR-M63-017: Alert on unresolved hold at T+2 h after poll close.

## Acceptance criteria

1. AC-M63-01: Serial mismatch raises explicit hold; never blocks silently; result hold integration test with M10.
2. AC-M63-02: Kani append-only proof passes; event stream immutable; compensating event issued for corrections.
3. AC-M63-03: All events exactly-once via Kafka transactions; consumer idempotency test.
4. AC-M63-04: Reconciliation arithmetic correct at boundary (zero spoilt, all cast, all spoilt); property test.
5. AC-M63-05: Offline M31 spoilt recording synced and deduplicated on reconnect; test.
6. AC-M63-06: Hold resolution requires dual-officer approval; SpiceDB policy test.
7. AC-M63-07: API p99 within election-critical 99.99% SLA under load.
8. AC-M63-08: RTO ≤30 min, RPO ≤5 min in staging DR drill.

## Applicable edge cases

**Data:**
- EC-DAT-01: duplicate spoilt ballot submission; idempotent on serial number + station.
- EC-DAT-11: cast count exceeds assigned range; block with reason; TLA+ invariant.

**Security:**
- EC-SEC-01: signature mismatch on event; tamper topic; page ≤30 s.
- EC-SEC-03: HSM unavailable; sign queue in degraded mode.
- EC-SEC-13: split-view on transparency log; witness divergence alert.

**Time:**
- EC-TIM-03: backend clock jump backward; HLC never regresses.
- EC-TIM-06: event time < causation time; accept; audit skew anomaly.

**Scale:**
- EC-SCL-07: 60k concurrent station reconciliations at election close; SLAs met.

**Connectivity:**
- EC-CON-04: site partition; Patroni fencing; RPO ≤5 min.
- EC-MOB-01: OS kills M31 mid-spoilt recording; resumable; no duplicate.

**Kit:**
- EC-KIT-01: RFID GNSS jam; multi-constellation fallback; lot scan falls back to manual entry.

**Operations:**
- EC-OPS-01: kit battery death mid-reconciliation; journal-before-act; exact resume.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Reconciliation arithmetic, assignment range guard, hold FSM | nextest | M63-UNIT- |
| property | Append-only; idempotency; reconciliation boundary | proptest | M63-PROP- |
| formal | Kani append-only proof | kani | M63-FORMAL- |
| contract | M39 lot; M10 result hold; M31 mobile | Pact | M63-PACT- |
| integration | Full serial chain lifecycle; hold and resolve; DR failover | nextest + testcontainers | M63-INT- |
| e2e | Admin reconciliation; hold resolution | Playwright | M63-E2E- |
| fuzz | Serial number payload; reconciliation totals | cargo-fuzz | M63-FUZZ- |
| load | 60k station reconciliations at election-critical load | k6 | M63-LOAD- |
| chaos | HSM kill; Kafka ISR shrink; site partition | Chaos Mesh | M63-CHAOS- |
| security | ZAP; tamper event test; log scanner | ZAP + Semgrep | M63-SEC- |

Tags: `#[trace("FR-M63-006","AC-M63-01","EC-DAT-11")]`

## Runbooks and dashboards

- docs/runbooks/rts-svc-degradation.md (extend with ballot-serial section)
- docs/runbooks/ballot-serial-hold-resolution.md (new)
- platform/observability/rts-svc/ballot-serial-dashboard.json
- platform/observability/rts-svc/slo-alerts.yaml (election-critical burn rate, hold alert)

## Open questions

- Whether serial number assignment is done at national level or constituency level; affects partition strategy for the event stream.
- False result-hold risk: unresolved holds at declaration time; escalation protocol with ECC and legal team to be defined before election-day rehearsal.
