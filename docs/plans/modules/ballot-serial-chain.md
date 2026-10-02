# M63 ballot-serial-chain

| field | value |
|---|---|
| M-number | M63 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.3 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M63-ballot-serial-chain.md |

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

Serial range assignment:

1. FR-M63-001: Assign contiguous serial range per station from a verified print lot; fields: station_code, serial_low, serial_high, book_count, bundle_count, assigned_at, assigning_officer UUID, seal UIDs, M39 manifest reference (FR-63.01).
2. FR-M63-002: Validate range contiguity across all stations before election day; any gap or overlap in the national serial space triggers a P0 alert to procurement.
3. FR-M63-003: Ingest serial range assignment from M39 ballot manifest via signed API call; verify Ed25519 signature before accepting.
4. FR-M63-004: Emit eel.ballot_serial.serial_assigned.v1 exactly-once to eel.ballot_serial.chain.v1 on confirmed assignment.
5. FR-M63-005: Reject serial range assignment if the print lot is not confirmed in the M39 manifest.
6. FR-M63-006: RFID lot scan from M08 confirms physical lot receipt; any mismatch between RFID-read range and manifest range triggers an immediate P1 alert to CRO and ECC (FR-M63-13).

Station receipt:

7. FR-M63-007: Presiding Officer scans bundle seals on station arrival; record received serial range, condition (intact, damaged, short), receiving officer UUID, and timestamp (FR-63.02).
8. FR-M63-008: Any short or damaged bundle generates immediate P1 alert to CRO and ECC.
9. FR-M63-009: Emit eel.ballot_serial.serial_issued.v1 exactly-once on confirmed receipt.
10. FR-M63-010: Receipt event captures Presiding Officer TPM signature from M31 device; signature verified before event accepted.
11. FR-M63-011: Offline receipt recording on M31 tablet; events queued in offline WAL and synced on reconnect; zero events lost (FR-M63-14).
12. FR-M63-012: Offline sync idempotent; duplicate receipt event de-duplicated on serial + station; no double event created.

Spoilt ballot workflow:

13. FR-M63-013: Record spoilt ballot event: spoilt_serial, reason code (ink_bleed, mis_mark, double_mark, voter_request), replacement_serial (FR-63.03, Elections Act s.54).
14. FR-M63-014: Validate spoilt_serial falls within station assigned range; reject with 422 if out of range.
15. FR-M63-015: Validate replacement_serial falls within station assigned range and is not already deposited or spoilt; reject if invalid.
16. FR-M63-016: Emit eel.ballot_serial.ballot_spoilt.v1 exactly-once per spoilt ballot event.
17. FR-M63-017: Spoilt serials individually listed in per-station reconciliation record; spoilt count included in arithmetic totals.
18. FR-M63-018: Offline spoilt recording on M31; synced on reconnect; idempotent on spoilt_serial + station_code (FR-M63-14).
19. FR-M63-019: Replacement serial recorded in the same spoilt ballot event; replacement must not itself be already spoilt or deposited before issuance.

Deposit correlation:

20. FR-M63-020: Receive per-ballot deposit events from M20 ballot-box smart-lid RFID reads; correlate deposited serial against station assigned range (FR-63.05).
21. FR-M63-021: Any deposited serial outside station assigned range rejected at ingest with P1 alert; discrepancy threshold is 0.
22. FR-M63-022: Manual serial entry fallback when RFID chip fails; manually entered serials flagged for CRO review; barcode scan as secondary backup.

Reconciliation:

23. FR-M63-023: Trigger end-of-day reconciliation: unused_count = (serial_high - serial_low + 1) minus deposited_count minus spoilt_count; issued_count = authenticated voters (FR-63.04, FR-M63-05).
24. FR-M63-024: Any arithmetic mismatch > 0 triggers P1 result-hold flag against that station; flag is explicit; never silently ignored (FR-M63-06).
25. FR-M63-025: Result-hold flag blocks result publication in M10 until hold resolved (FR-M63-07).
26. FR-M63-026: Resolve hold with dual-officer approval (Presiding Officer and CRO); reason recorded; SpiceDB dual-approval policy enforced (FR-M63-08).
27. FR-M63-027: Emit eel.ballot_serial.reconciliation_held.v1 on hold raise and eel.ballot_serial.reconciliation_cleared.v1 on resolution; both exactly-once.
28. FR-M63-028: Presiding Officer signs reconciliation record via M31 TPM; CRO counter-signs constituency aggregate via web interface.
29. FR-M63-029: Signed reconciliation record delivered to IAB p99 < 2 hours post-station-close (NFR-63.06).
30. FR-M63-030: Alert on unresolved hold at T+2 h after poll close; escalate to ECC operations (FR-M63-17).
31. FR-M63-031: Reconciliation submission button on M31 disabled until arithmetic balances; prevents PO from submitting an imbalanced record.
32. FR-M63-032: Reconciliation arithmetic correct at boundary conditions: zero spoilt (all cast), all spoilt (zero deposited), all unused; property test.

Event sourcing and integrity:

33. FR-M63-033: All ballot events append-only; compensating events used for corrections; no in-place modification or deletion at any layer (FR-M63-10).
34. FR-M63-034: All emitted events signed with Ed25519; consumers verify signature before applying (FR-M63-09).
35. FR-M63-035: Kani append-only property proof passes in CI; property checked at every merge (FR-M63-11).
36. FR-M63-036: Merkle-anchor all events in P05 shared sub-tree; hourly verification; simulated break confirms P0 alert within 5 minutes.
37. FR-M63-037: Event stream replay produces identical reconciliation state deterministically regardless of replay order within causal constraints.
38. FR-M63-038: Kafka exactly-once transactions via eel-outbox-rs exactly-once mode; no duplicate events on producer restart.
39. FR-M63-039: Sequence counter on event stream; out-of-sequence events rejected; replayed events rejected on counter.

Publication and transparency:

40. FR-M63-040: Publish constituency-level serial range summaries as part of M10 results transparency stream (FR-63.07).
41. FR-M63-041: Station-level detail available to party agents (M30) and observers (M34) immediately after CRO signature.
42. FR-M63-042: National serial manifest available for post-election audit download within 24 hours of T+0; manifest signed SHA-256 (FR-63.07).
43. FR-M63-043: Bulk audit export with P05 Merkle inclusion proofs per event (FR-M63-12).

Evidentiary extract:

44. FR-M63-044: On-demand signed PDF extract per station: serial range, spoilt log with individual serials, deposited list, reconciliation arithmetic (FR-63.08).
45. FR-M63-045: Evidentiary extract p99 < 2 hours from authenticated request via judicial JWT or IAB JWT (NFR-63.06).
46. FR-M63-046: Require purpose_code on every judicial extract per DPA 2019 s.26; logged.
47. FR-M63-047: Pre-generate signed station-level PDFs at T+4h post-close; stored for petition-window readiness; load-tested at 100 concurrent judiciary downloads.

Admin and operations:

48. FR-M63-048: Admin reconciliation dashboard per station and constituency: traffic-light status, discrepancy count, hold queue (FR-M63-16).
49. FR-M63-049: CRO constituency overview: station grid with reconciliation status; drill-in to any station; counter-sign aggregate from web.
50. FR-M63-050: ECC discrepancy feed via M16 integration; P1 alerts with station detail and suggested operator action.
51. FR-M63-051: RTO <= 30 minutes, RPO <= 5 minutes; DR drill completed before election-day rehearsal; drill result documented (FR-M63-15).
52. FR-M63-052: Bulk export available to Auditor-General within 24 hours of T+0; audit trail confirms export timestamp.

## Acceptance criteria

1. AC-M63-01: Serial mismatch raises explicit result-hold flag in M10; hold never silently ignored; result-hold integration test with M10 double confirms publication blocked until resolved.
2. AC-M63-02: Kani append-only proof passes in CI; no mutation path exists at API level; compensating event correctly issued for corrections; formal proof checked at every merge.
3. AC-M63-03: All emitted events exactly-once via Kafka transactions; consumer idempotency test with simulated broker restart; zero duplicate events in event log.
4. AC-M63-04: Reconciliation arithmetic correct at all three boundary conditions: zero spoilt (all cast), all spoilt (zero cast), all unused; proptest with 10,000 random distributions.
5. AC-M63-05: Offline M31 spoilt recording synced on reconnect; zero events lost in 4-hour partition test; de-duplicated on spoilt_serial + station_code; network-partition integration test.
6. AC-M63-06: Hold resolution requires dual-officer approval (Presiding Officer and CRO); SpiceDB policy blocks unilateral resolution; access-control integration test with single-officer credentials.
7. AC-M63-07: API p99 within election-critical 99.99% SLA under sustained load of 900 events/s with 3,000/s burst; k6 load test.
8. AC-M63-08: RTO <= 30 minutes, RPO <= 5 minutes confirmed in staging DR drill; restore process logged to tlog; drill result documented before election-day rehearsal.
9. AC-M63-09: Out-of-range serial number rejected at API layer with 422; cargo-fuzz corpus of invalid serials confirms no bypass.
10. AC-M63-10: Merkle sub-tree verified every hour; simulated break confirmed by P0 alert within 5 minutes; chaos test with Merkle hash corruption.
11. AC-M63-11: Reconciliation record delivered to IAB p99 < 2 hours post-station-close; measured in staging simulation across 55,393 concurrent station closes.
12. AC-M63-12: Evidentiary extract p99 < 2 hours on authenticated request; load test at 100 concurrent judiciary downloads confirms SLA.
13. AC-M63-13: National serial manifest available within 24 hours of T+0; signed SHA-256 independently verifiable; staging simulation.
14. AC-M63-14: 99.9% of stations with complete reconciliation by T+2h post-election-close; metric measured in staging full-cycle test.
15. AC-M63-15: Arithmetic discrepancy count post-election: 0 for all stations in staging full-cycle test; any unresolved hold generates SRE alert within 5 minutes.
16. AC-M63-16: Ed25519 signature verified on every consumed event; signature mismatch rejected and published to tamper topic within 30 s; chaos test with byte-flip injection.

## Applicable edge cases

Time (all rows required for event-sourced module):

EC-TIM-01: M31 kit clock skew > 5 min at boot; tablet refuses ballot session until NTS/GNSS time resolved; offline spoilt events buffered with low time_confidence flag.
EC-TIM-02: NTP/NTS unreachable all day; M31 uses last-authenticated time + monotonic counter; time_confidence=low on all events; events accepted with annotation on reconnect.
EC-TIM-03: backend clock jumps backward; Hybrid Logical Clock on ballot serial event stream never regresses; skew alert fired; kani HLC merge proof.
EC-TIM-04: M31 tablet configured in non-EAT timezone (diaspora kit or mis-configured device); all ballot event timestamps stored in UTC; EAT displayed to Presiding Officer in UI.
EC-TIM-05: leap second (23:59:60) appears in submitted event timestamp; fuzz confirms parser tolerates or normalises correctly; no crash or wrong ordering.
EC-TIM-06: event time < causation time (spoilt event arrives before corresponding receipt event in stream); accepted; audit skew anomaly logged; reconciliation projection independent of wall-clock arrival order.
EC-TIM-07: s.14A close time moves from 17:00 to 19:00; HSM-signed extension reopens reconciliation window; M63 reconciliation deadline updated accordingly; forged extension rejected.
EC-TIM-08: petition evidentiary extract deadline at T+7 midnight; extract deadline computed using Elections Act s.84 inclusive semantics; boundary test at T+7 23:59:59.
EC-TIM-09: serial range assignment freeze race at T-60/T-30; OPA hard-close enforced at transaction time; all-or-nothing assignment; concurrent writers tested with TLA+ freeze model.

Data (all rows required for event-sourced module):

EC-DAT-01: duplicate spoilt ballot event with same idempotency key returns cached response; idempotent on spoilt_serial + station_code; no duplicate record.
EC-DAT-02: same idempotency key with different spoilt ballot body returns 422 idempotency-key-reuse; spoilt event rejected.
EC-DAT-03: out-of-order ballot events on stream (deposit event arrives before receipt event); buffer until causally prior event present or DLQ after timeout; shuffled replay produces same reconciliation state.
EC-DAT-04: replayed signed serial_assigned event with reused counter; rejected on sequence counter check; event published to tamper topic.
EC-DAT-05: poison message (deposited_count > assigned range size, impossible value); DLQ with validation reason; partition continues.
EC-DAT-06: schema v1/v2 ballot serial events coexist during rolling deployment; both accepted by consumer; Karapace compat check passes.
EC-DAT-07: partial batch failure in 30M synthetic ballot events; per-event report generated; valid events committed; failed events in DLQ with row-level reason.
EC-DAT-08: 38.8 million ballot events in 12-hour election window (900/s sustained, 3,000/s burst); streaming processing; checkpoint-resumable; bounded memory; kill-resume test confirms no loss.
EC-DAT-09: Unicode characters in spoilt ballot reason field (rare locale or non-Latin script); NFC normalised; grapheme-safe truncation applied.
EC-DAT-10: zero spoilt ballots at a station (all votes cast cleanly); reconciliation renders spoilt_count = 0; no division errors in totals; valid report generated.
EC-DAT-11: deposited_count exceeds assigned range (ballots appear to exceed issued); blocked at ingest with 422 and P1 alert; TLA+ invariant checked.
EC-DAT-12: Debezium slot lag or WAL bloat on ballot serial event table; alert at 60 s; outbox retained; disk guard active; no event loss.
EC-DAT-13: soft-deleted serial assignment row (e.g. cancelled station) appears in audit search; tombstone honoured; purge removes from search index.
EC-DAT-14: oversized event exceeding 1 MB (bulk spoilt list upload); outbox rejects; blob offloaded to MinIO with hash reference in event.
EC-DAT-15: M39 manifest returns two overlapping serial ranges for same station; AdjudicationCase raised; no auto-merge; P0 alert to procurement.
EC-DAT-16: legacy M31 pre-CloudEvents payload from older tablet firmware; shim upgrades envelope; Ed25519 signature preserved; golden field corpus test.

Security:

EC-SEC-01: signature mismatch on consumed ballot serial event; event never applied; tamper topic published; SRE page within 30 s.
EC-SEC-03: HSM unavailable during bulk signing; sign queue accumulates; events counter-signed on restore; degraded-mode flag set on queued events.
EC-SEC-05: injection on spoilt ballot reason, replacement serial, and notes fields; parameterised queries only; ZAP + Semgrep at CI.
EC-SEC-13: split-view on P05 Merkle transparency log detected during hourly verification; witness divergence alert; consistency probe fails; P0 alert within 5 minutes.
EC-SEC-16: serial numbers, officer UUIDs, and station codes must not appear unredacted in debug logs or traces; log-scanner fixture; CI regex.

Identity:

EC-IDN-02: JWT expires mid-reconciliation submission; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-10: SpiceDB down; writes fail closed; read-only class-C queries from replica; 503 problem+json returned to caller.

Scale:

EC-SCL-04: Kafka consumer lag > 60 s on ballot serial chain topic; SRE paged; backpressure applied; no OOM condition.
EC-SCL-07: 60,000 concurrent station reconciliation submissions at election close (55,393 stations with overlap); SLAs met; <= 10% p99 regression at 5x load; simulator pre-prod run.

Connectivity:

EC-CON-04: site partition during reconciliation window; Patroni fencing applied; no split-brain writes; MirrorMaker lag alert; RPO <= 5 min confirmed.
EC-MOB-01: OS kills M31 mid-spoilt recording; WAL intact; exact resume on relaunch at last committed event; no duplicate event on reconnect.

Kit:

EC-KIT-01: RFID GNSS jam or spoof at distribution point; multi-constellation + IMU fallback; lot scan falls back to manual serial entry with CRO countersign.

Operations:

EC-OPS-01: M31 kit battery death mid-reconciliation; journal-before-act; exact resume from last journal checkpoint; no double submit of reconciliation record.

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
