# M39 ballot-printing-chain

| field | value |
|---|---|
| M-number | M39 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M39-ballot-printing-chain.md |

## Purpose

Track every ballot paper from print-run authorisation through serial-range gazettement, RFID chip embedding, printer MES lot-data integration, port clearance, Nairobi central warehouse, constituency distribution, and individual polling-station scan-in and scan-out. Make ballot-chain tampering computationally detectable and immediately auditable. The 2017 Kenya election environment in which chain-of-custody weakness was exploited makes integrity here non-negotiable; this module is an explicit corrective response.

Strategy reference: §XV.4 Element 2 (ballot security), §XVI.3 (chain-of-custody named threat).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Print Authorisation Console | Phoenix LiveView | services/inventory-svc/ex/ballot_chain_web/ |
| RFID Scanner App (M08 integration) | Flutter | services/inventory-svc/apps/rfid-scanner/ |
| Audit Dashboard | Phoenix LiveView | services/inventory-svc/ex/ballot_chain_web/ |
| API | Rust (axum) | services/inventory-svc/crates/inventory-api/ |

Languages: EN only (internal operational system).

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PrintOrder | id, election_id, contest_id, constituency_code, quantity_authorised, serial_range_start, serial_range_end, paper_spec_hash, issued_by_officer_id, countersigned_by_commissioner_id, issued_at, merkle_node_id, gazette_notice_id | A | T1-audit |
| LotData | id, print_order_id, printer_facility_id, serial_range_start, serial_range_end, quantity_printed, quantity_rejected, rfid_chip_id_list_hash (SHA-256 of sorted chip ID list; full list stored encrypted at rest), print_date, received_at, variance_pct, variance_alert_sent | A | T1-audit |
| CustodyEvent | id, lot_id, event_type (PORT_CLEARANCE/WAREHOUSE_RECEIPT/CONSTITUENCY_ALLOCATION/STATION_RECEIPT/STATION_CLOSE_RETURN/RFID_SPOT_CHECK), location_code, officer_id, witness_id, quantity_scanned, rfid_match_count, rfid_mismatch_count, seal_number, vehicle_registration, scanned_at, merkle_node_id | A | T1-audit |
| StationReconciliation | id, station_code, election_id, ballots_received, ballots_issued, ballots_unused, ballots_spoilt, discrepancy, discrepancy_flag, reconciled_at, reconciled_by, merkle_node_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/inventory/ballot-chain.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/ballot-chain/print-orders | 2 | JWT+SpiceDB (COMMISSION_OFFICER) + HSM dual-sign | yes | issue signed print order; dual sign-off enforced |
| POST /v1/ballot-chain/lot-data | internal gRPC | PRINTER_SYSTEM mTLS | yes | printer MES lot-data submission |
| POST /v1/ballot-chain/events | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | yes | record custody event (scan-in or scan-out) |
| GET /v1/ballot-chain/events | 2 | JWT+SpiceDB (COMMISSION_AUDIT_LEAD) | n/a | list custody events; filterable by lot and location |
| GET /v1/ballot-chain/events/{id}/proof | 2 | JWT+SpiceDB (COMMISSION_AUDIT_LEAD) | n/a | Merkle inclusion proof for event |
| POST /v1/ballot-chain/reconciliation | 2 | JWT+SpiceDB (PRESIDING_OFFICER) | yes | submit station end-of-day reconciliation |
| GET /v1/ballot-chain/reconciliation/{station_code} | 2 | JWT+SpiceDB (COMMISSION_AUDIT_LEAD) | n/a | get station reconciliation record |
| GET /v1/ballot-chain/public/summary | 1 | public | n/a | aggregate chain-of-custody status; no serial IDs or chip IDs exposed |
| GET /v1/ballot-chain/court-bundle/{constituency_code} | 2 | JWT+SpiceDB (COURT_REGISTRAR) | n/a | signed chain-of-custody bundle for M40; rate limited 5/hr |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.inventory.lot_data_received.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | at-least-once idempotent |
| eel.inventory.custody_event_recorded.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | at-least-once idempotent |
| eel.inventory.quantity_variance_alert.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |
| eel.inventory.rfid_mismatch_alert.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |
| eel.inventory.station_reconciliation_discrepancy.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Printer MES (Al Ghurair UAE / approved alternative) | external mTLS | integrations/printer-mes/ | yes (stub) | lot-data submission; if unavailable, IEBC officer uploads scanned attestation |
| M08 RTS Device | internal | integrations/rts-device/ | yes | RFID scanner at station; offline scan queue; sync on reconnect |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | BVI turnout figure for station reconciliation |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | discrepancy alert consumer; circuit breaker; alert persists in queue |
| M40 Election Court IT Interface | internal | integrations/court/ | yes | court chain-of-custody bundle delivery |
| M11 Public Portal | internal | integrations/portal/ | yes | public aggregate summary publication |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | audit summary; non-critical |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | all events anchored within 60 s |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | print order dual signing |

## ML and AI

- None.

## Dependencies

- Modules: M06, M08, M11, M15, M16, M40
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

Print-run authorisation:
1. FR-M39-001: IEBC Print Authorisation Officer issues signed digital print order; dual sign-off required (officer + IEBC Commissioner); order anchored in P05 Merkle chain before printer receives it
2. FR-M39-015: print order specifies paper specification hash; printer must use approved paper stock matching hash; deviation triggers alert to Commissioner
3. FR-M39-016: print order requires separate contest-level quantities (presidential, gubernatorial, senatorial, women rep, MP, MCAs); distinct serial ranges per contest
4. FR-M39-035: print order transmission to printer encrypted: TLS 1.3 transport + PGP-encrypted payload; print order cannot be intercepted and replayed

Gazettement:
5. FR-M39-002: authorised serial ranges gazetted as Kenya Gazette supplement (PDF + structured JSON); gazette hash anchored in Merkle chain; publicly verifiable
6. FR-M39-049: serial-range gazette supplement published to Kenya Gazette format; Attorney General sign-off reference included; gazette notice ID stored on PrintOrder

Printer MES integration:
7. FR-M39-003: printer MES submits lot-data JSON (serial ranges, RFID chip IDs, print date, quantity, reject count) via mTLS; system reconciles against authorised print order; quantity variance > 0.01% triggers immediate IEBC alert
8. FR-M39-020: lot-data reconciliation report generated within 5 minutes of printer MES submission; report includes: authorised vs printed quantities, variance_pct, RFID chip count vs declared
9. FR-M39-024: variance alert emits eel.inventory.quantity_variance_alert.v1 when lot quantity differs from authorised by > 0.01%; alert delivered within 1 minute of lot-data submission
10. FR-M39-034: printer MES must authenticate via mTLS certificate issued to approved printer facility; requests from unknown certificates rejected with 401 and logged

RFID embedding:
11. FR-M39-004: printer attests RFID embedding per ballot in declared range; IEBC spot-check result uploaded to custody record
12. FR-M39-021: RFID chip ID list stored as SHA-256 hash reference in LotData; full chip ID list stored encrypted AES-256 in MinIO; access restricted to COMMISSION_AUDIT_LEAD; no chip IDs in API responses to non-audit roles
13. FR-M39-022: RFID spot-check at printer facility: IEBC officer scans random sample; minimum 50 chips; result uploaded to custody record with officer ID and timestamp
14. FR-M39-023: RFID mismatch alert emits eel.inventory.rfid_mismatch_alert.v1 on any discrepancy; exactly-once; M16 alert consumer; shipment flagged for hold

Port clearance:
15. FR-M39-005: IEBC Customs Officer records carrier, AWB/BL number, seal numbers, container ID, port of entry, arrival date, C-63 customs form ID on arrival manifest
16. FR-M39-006: port clearance RFID spot-check minimum 200 chips per lot; discrepancy triggers hold and IEBC Commissioner notification
17. FR-M39-043: seal integrity check at every custody transfer; seal number recorded; if seal number differs from issued, custody event flagged and alert raised

Nairobi warehouse:
18. FR-M39-007: Warehouse Manager scans all pallet RFID tags on arrival; system confirms serial-range completeness; missing range segment flagged; signed warehouse receipt anchored
19. FR-M39-041: printer MES unavailable at lot-data submission time; IEBC officer uploads scanned paper attestation as fallback; system flags manual attestation in lot record
20. FR-M39-042: warehouse pallet scanner failure; IEBC officer records serial ranges manually; manual entry flagged for audit review; anchored as manual custody event

Constituency allocation:
21. FR-M39-008: County Logistics Officer creates constituency allocation record; officer and witness scan-out ballot packets; vehicle registration and physical seal number recorded; allocation signed by both
22. FR-M39-025: constituency allocation signed by both County Logistics Officer and witness; allocation record includes vehicle registration, seal number, departure timestamp
23. FR-M39-026: constituency allocation record anchored in P05 Merkle chain; anchor must complete before transport vehicle departs; system blocks departure confirmation if anchor not confirmed
24. FR-M39-027: inter-constituency transfers require IEBC Commissioner sign-off; transfer event anchored and logged separately
25. FR-M39-028: ballot packet handover at polling station requires both Presiding Officer scan and returning officer acknowledgement; dual-scan recorded in CustodyEvent

Station receipt:
26. FR-M39-009: Presiding Officer scans ballot packet RFID on delivery at station; system confirms serial range matches allocation; signed receipt generated via M08 RFID scanner
27. FR-M39-029: station ballot receipt anchored within 60 s of scan; if anchor fails, M08 kit queues event locally; retry with exponential backoff; no station can open without confirmed receipt
28. FR-M39-040: M08 RFID scanner operates offline; local event queue persists up to 48 h; sync on reconnect; no duplicate events submitted on reconnect

Election-day tracking and close:
29. FR-M39-010: at close, Presiding Officer scans unused ballot bundle RFID; system computes issued = received - unused - spoilt; discrepancy vs BVI turnout count flagged immediately to M16
30. FR-M39-030: end-of-day reconciliation submitted by Presiding Officer within 2 hours of station close; if not submitted, M16 receives automatic alert
31. FR-M39-031: station reconciliation report includes: ballots received, ballots issued (from M06 BVI count), ballots unused (scan count), ballots spoilt (manual entry), discrepancy = received - issued - unused - spoilt
32. FR-M39-032: any non-zero discrepancy_flag raises alert to M16 within 2 minutes; Constituency Returning Officer must acknowledge within 1 hour or escalation triggered

Post-election return and reporting:
33. FR-M39-011: post-election return; unused ballots scanned in at constituency store; per-station reconciliation report generated (received/issued/unused/spoilt/discrepancy)
34. FR-M39-033: post-election batch reconciliation: IEBC Audit Lead generates national summary report; stations with unresolved discrepancies highlighted; report signed and anchored

Audit trail and cryptography:
35. FR-M39-012: every scan event, allocation, and reconciliation record hashed and anchored in P05 Merkle chain within 60 seconds of creation
36. FR-M39-017: Merkle inclusion proof available for every custody event; IEBC Audit Lead can verify any individual event without downloading full chain
37. FR-M39-036: all custody event records append-only; modification of committed record detectable via Merkle proof; system raises tamper alert on verification failure
38. FR-M39-019: discrepancy alert to M16 emits eel.inventory.station_reconciliation_discrepancy.v1; fires within 2 minutes of reconciliation; exactly-once delivery

Public and court access:
39. FR-M39-013: public summary (aggregate lot quantities, gazette reference, warehouse receipt date, constituency allocation status) on M11; no individual RFID chip IDs published
40. FR-M39-014: full RFID chip ID list restricted to COMMISSION_AUDIT_LEAD role; stored encrypted AES-256; printer-MES connection over mTLS + PGP-encrypted payload
41. FR-M39-018: court bundle endpoint: signed chain-of-custody bundle per constituency including all custody events and Merkle proofs; delivered to M40 within 2 hours of request; rate limited 5/hr
42. FR-M39-037: public chain-of-custody summary: no individual serial ranges, no RFID chip IDs, no officer names; aggregate quantities and status only
43. FR-M39-038: court bundle includes individual custody events with officer IDs (not names) and Merkle proofs; court registrar must authenticate with COURT_REGISTRAR JWT
44. FR-M39-046: M11 public portal updated with chain-of-custody summary at each major milestone (warehouse receipt, constituency allocation complete, all station receipts); update signed and anchored

Security and access:
45. FR-M39-039: custody event write requires COMMISSION_OFFICER JWT + SpiceDB; read of full serial ranges requires COMMISSION_AUDIT_LEAD; read of public summary: public
46. FR-M39-047: IAB observers can download aggregate reconciliation report per constituency; report signed by IEBC Audit Lead; no RFID chip IDs or officer PII included

Status tracking:
47. FR-M39-048: chain-of-custody completeness dashboard: for each lot, shows status at each custody stage (IN_PRINT/PRINTED/IN_TRANSIT/AT_WAREHOUSE/ALLOCATED/AT_STATION/RECONCILED)

Legal and retention:
48. FR-M39-050: chain-of-custody records retained 7 years per Elections Act; records transferred to M69 Long-Term Archive at decommission; disposal certificate issued

Observability:
49. FR-M39-044: OpenTelemetry traces exported to central collector; Grafana dashboard eel-iems-39; alert ruleset iems-39-alerts
50. FR-M39-045: SLO alerts: Merkle anchor latency <= 60 s; lot reconciliation <= 5 min; discrepancy alert <= 2 min to M16; all 290+ constituency allocations confirmed by T-10

## Acceptance criteria

1. AC-M39-01: print order dual-signed, transmitted to printer, Merkle-anchored; UAT with mock printer MES confirms all three steps complete
2. AC-M39-02: lot-data reconciliation completes within 5 minutes of MES submission; quantity variance alert fires within 1 minute of variance detection; integration test confirms
3. AC-M39-03: port RFID spot-check (200 chips) logged and anchored; mismatch triggers hold and Commissioner notification; integration test confirms
4. AC-M39-04: station receipt and close-of-poll reconciliation via M08 RFID scanner; discrepancy alert to M16 within 2 minutes of detection; end-to-end test confirms
5. AC-M39-05: court bundle (all custody events + Merkle proofs) generated within 2 hours of M40 request; integration test confirms
6. AC-M39-06: public summary exposes no serial ranges or chip IDs; penetration test with RFID extraction attempt confirms no data leak
7. AC-M39-07: every custody event anchored in P05 Merkle chain within 60 s; 100% anchoring confirmed by Merkle reconciliation; chaos test with P05 unavailable shows retry succeeds
8. AC-M39-08: warehouse receipt completeness check flags missing serial range segment within 5 minutes of scan completion; integration test confirms
9. AC-M39-09: printer MES submission without valid mTLS certificate rejected with 401 and logged; security test confirms
10. AC-M39-10: 290+ constituency allocations processed simultaneously during distribution week without service degradation; p99 <= 2 s on allocation endpoints; load test confirms
11. AC-M39-11: station reconciliation discrepancy alert reaches M16 within 2 minutes of discrepancy detection; end-to-end latency test confirms
12. AC-M39-12: chain-of-custody records retained 7 years; immutable audit trail confirmed; no modification detectable via Merkle proofs; long-term retention policy audited at OAT gate

## Applicable edge cases

data:
- EC-DAT-01: duplicate custody event submission with same idempotency key; cached response returned; Merkle chain not re-anchored for duplicate
- EC-DAT-02: same custody event key with different body (tamper attempt); 422 idempotency-key-reuse returned; tamper attempt logged to audit trail
- EC-DAT-04: replayed signed custody event; counter check rejects replay; tamper event emitted; logged to audit topic
- EC-DAT-07: partial batch failure during warehouse pallet scan; valid scanned records committed; error rows reported; warehouse manager must re-scan failed pallets
- EC-DAT-09: constituency names or officer identifiers with Unicode characters; NFC normalization applied; no truncation in audit trail or PDF reports
- EC-DAT-14: lot-data submission with chip ID list exceeding 1 MB outbox limit; full chip ID list stored in MinIO; event carries hash reference; no oversized event

security:
- EC-SEC-01: committed custody event byte-flipped; Merkle proof verification fails; tamper alert raised to M16 within 30 s; event locked in audit log
- EC-SEC-02: physical seal tampered on ballot box during transit; custody event flagged with seal mismatch; quarantine status applied; IEBC Commissioner and M16 notified
- EC-SEC-05: injection attempt in serial range or officer ID fields; parameterised queries only; Semgrep CI; ZAP scan at OAT gate
- EC-SEC-08: XXE injection attempt in printer MES lot-data XML/JSON payload; DTD disabled; strict JSON parsing; fuzz corpus in CI
- EC-SEC-16: RFID chip IDs and officer personal identifiers stripped from unscoped telemetry; log-scanner fixture in CI confirms Tier A/B fields absent
- EC-SEC-17: printer MES connection must use TLS 1.3; TLS 1.2 rejected; SPIFFE mTLS certificate required; testssl.sh validation in CI

identity:
- EC-IDN-02: IEBC officer JWT expires mid-custody event entry; in-flight write completes; next request 401 + silent refresh; no event lost
- EC-IDN-10: SpiceDB unavailable; all custody event writes fail closed 503; public summary continues from last confirmed state; alert raised to SRE

scale:
- EC-SCL-01: 290 constituency allocations processed in parallel during distribution week; inventory-svc horizontally scaled; p99 <= 2 s maintained
- EC-SCL-04: Kafka consumer lag > 60 s on eel.inventory.ballot-chain.v1 topic; alert raised; backpressure applied; no event loss

connectivity:
- EC-CON-01: M08 RFID scanner offline at station receipt; local queue persists up to 48 h; sync on reconnect; no duplicate events; station receipt confirmed only after sync
- EC-CON-05: flapping MES connection during lot-data submission; eel-outbox-rs queues with backoff+jitter; no duplicate lot record created; exactly-one lot confirmed by server

operations:
- EC-OPS-03: seal broken in transit; custody event flagged with seal mismatch; quarantine status applied to lot; IEBC Commissioner notified; re-verification required before station distribution
- EC-OPS-04: dual-control officer absent during print order sign-off; time-boxed remote co-sign via ECC; no print order transmitted without both signatures
- EC-OPS-05: power outage at Nairobi warehouse during pallet scan; SQLite WAL on scanner device resumes; no duplicate scan records; reconciliation continues from last committed position

security (crypto):
- EC-SEC-04: KEK rotation for AES-256 chip ID list encryption in MinIO; online re-wrap applied; old key readable until rotation complete; no asset access gap during rotation

legal:
- EC-LEG-01: candidate withdraws after ballots printed for that contest; print order status frozen at PRINTED; affected serial range annotated in lot record; reconciliation proceeds normally
- EC-LEG-04: printer substitution without IEBC approval; lot-data submitted from unapproved mTLS cert; request rejected 401; immediate Commissioner notification; print order on hold
- EC-TIM-08: court bundle request arrives at T+6d 23h before petition deadline; 24-hour production clock applies; not freeze-blocked; bundle delivered within statutory window

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | serial-range completeness checker, RFID chip ID list hash verifier, discrepancy calculator, Merkle anchor trigger, variance alert threshold | nextest | M39-UNIT- |
| property | any committed scan record modification detectable via Merkle proof | proptest | M39-PROP- |
| contract | printer MES mTLS, M08 RFID scan event, M06 BVI turnout, M16 alert, M40 court bundle | Pact | M39-PACT- |
| integration | print order to lot-data reconciliation; station receipt to close-of-poll reconciliation; discrepancy alert; court bundle assembly | integration_test | M39-INT- |
| e2e | full lifecycle: print order to station reconciliation; Merkle proof for every event; court bundle verification | nextest e2e | M39-E2E- |
| security | MES without mTLS cert (reject); officer modifies committed event (Merkle detects); non-audit-lead accesses full chip ID list (deny) | ZAP + red team | M39-SEC- |
| load | 290 constituency allocations concurrently during distribution week | k6 | M39-LOAD- |
| chaos | M08 offline at station receipt; local queue; reconnect; no duplicates | netem | M39-CHAOS- |

Tags: `#[trace("FR-M39-012","AC-M39-04","EC-CON-01")]`

## Runbooks and dashboards

- `docs/runbooks/inventory-svc-degradation.md` - extend with ballot-chain offline queue paths and printer MES unavailability
- `docs/runbooks/ballot-discrepancy-response.md` - new runbook; discrepancy alert escalation; IEBC Commissioner notification; station hold procedure
- Grafana dashboard: `eel-iems-39` (alert ruleset: `iems-39-alerts`)
- SLO alerts: Merkle anchor latency <= 60 s; lot reconciliation <= 5 min; discrepancy alert <= 2 min to M16

## Open questions

- Printer MES API contract: Al Ghurair (UAE) or approved Kenyan alternative; API spec must be co-designed at contract signature; fallback to manual scanned attestation if MES API not delivered by T-270.
- Gazettement format: Attorney General sign-off on serial-range gazette format required by T-180; legal confirmation needed before FR-M39-002 is implemented.
- RFID firmware on M08 kit: M08 hardware must support RFID scanner; confirm scanner firmware update timeline (target T-90) with M08 team.
