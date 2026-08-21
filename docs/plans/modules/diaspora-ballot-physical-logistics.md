# M65 diaspora-ballot-physical-logistics

| field | value |
|---|---|
| M-number | M65 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M65-diaspora-ballot-physical-logistics.md |

## Purpose

Manages the physical custody chain for diaspora ballot materials from IEBC headquarters to embassy stations and back, including pre-clearance manifests, offline packing app, customs and airline cargo integration, and reconciliation with the M24 diaspora voting module.

## Surfaces and languages

| surface | technology | path under services/inventory-svc/ |
|---|---|---|
| Admin logistics console | Django | django/inventory-admin/ |
| Offline packing mobile app | Flutter | apps/custody-mobile/ |
| Internal API | Rust (axum) | crates/diaspora-logistics/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| BallotContainer | id, container_code, station_id (embassy), contents_manifest_url, seal_id, status (packed/cleared/in-transit/received/reconciled), packed_at | C | T1-consensus |
| CustodyEvent | id, container_id, event_type (packed/customs-cleared/dispatched/received/seal-broken), actor_id, location, timestamp | C | T1-consensus |
| PreClearanceManifest | id, container_id, mfa_ref, kra_ref, airline_awb, status (submitted/approved/rejected), submitted_at | C | T1-audit |
| ReconciliationRecord | id, container_id, expected_count, received_count, discrepancy, resolved_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/diaspora-logistics/containers | 3 accredited | JWT+SpiceDB | yes | create ballot container |
| PUT /v1/diaspora-logistics/containers/{id}/pack | 3 accredited | JWT+SpiceDB | yes | record packing with manifest |
| POST /v1/diaspora-logistics/containers/{id}/preclearance | 3 accredited | JWT+SpiceDB | yes | submit MFA/KRA pre-clearance |
| PUT /v1/diaspora-logistics/containers/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | record dispatch with AWB |
| PUT /v1/diaspora-logistics/containers/{id}/receive | 3 accredited | JWT+SpiceDB | yes | record embassy receipt |
| POST /v1/diaspora-logistics/containers/{id}/custody-events | 3 accredited | JWT+SpiceDB | yes | record custody event |
| POST /v1/diaspora-logistics/containers/{id}/reconcile | 3 accredited | JWT+SpiceDB | yes | trigger reconciliation |
| GET /v1/diaspora-logistics/containers | 3 accredited | JWT+SpiceDB | n/a | list containers with status |
| GET /v1/diaspora-logistics/containers/{id}/custody-chain | 3 accredited | JWT+SpiceDB | n/a | full custody chain |

OpenAPI: packages/openapi-contracts/inventory/diaspora-ballot-physical-logistics.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.custody.diaspora_container_dispatched.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.diaspora_container_received.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.seal_broken.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.diaspora_reconciliation_complete.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.diaspora.embassy_station_registered.v1 | consumes | eel.diaspora.station.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| MFA (Ministry of Foreign Affairs) | mfa (tier B double) | integrations/ | yes | circuit breaker; clearance deferred |
| KRA e-Clearance | kra-eclearance | integrations/kra-eclearance/ | yes | circuit breaker; clearance deferred |
| Kenya Airways / IATA cargo | airline-cargo | integrations/airline-cargo/ | yes | circuit breaker; AWB manual entry fallback |
| M24 diaspora-voting | internal | crates/diaspora-logistics/ | yes | advisory; station register handoff |
| M20 inventory-management | internal | crates/diaspora-logistics/ | yes | advisory; seal lifecycle |
| M16 election-command-centre | internal | crates/diaspora-logistics/ | yes | advisory; seal-break alert |

## ML and AI

none

## Dependencies

- Modules: M24 (embassy station source), M20 (inventory seal), M16 (ECC alerts)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Embassy packing

1. FR-M65-001: After close of poll at each post, Embassy Election Officer initiates packing workflow in offline-capable mobile app. (FR-65.01)
2. FR-M65-002: Scan all completed ballot envelopes and record count per envelope type; total recorded in packing manifest. (FR-65.01)
3. FR-M65-003: Seal tamper-evident container using ISO 17712 H-class security seal; record seal UID sourced from M20 inventory. (FR-65.01)
4. FR-M65-004: Capture photograph of sealed container and all seals; store SHA-256 of photo in packing event. (FR-65.01)
5. FR-M65-005: Record GPS coordinates of packing location in the packing event. (FR-65.01)
6. FR-M65-006: Assign container UID in format KE27-DIA-{POST:5s}-{SEQ:04d} at creation time. (FR-65.01)
7. FR-M65-007: Packing event signed by Embassy Election Officer and IAB-appointed observer; remote witness via signed confirmation when local IAB observer is absent. (FR-65.01)
8. FR-M65-008: Packing event stored in TPM-signed, device-encrypted WAL on offline app; synced to backend on reconnect. (NFR-65.03, FR-65.01)
9. FR-M65-009: Offline packing sync idempotent; double-sync produces no duplicate custody events on the backend. (EC-DAT-01)
10. FR-M65-010: Offline app supports EN, FR, SW, and AR locale with ICU plurals. (UI/UX spec)

Routing policy

11. FR-M65-011: Resolve routing policy per post at T-90: diplomatic pouch, airline cargo, or charter flight. (FR-65.02)
12. FR-M65-012: Routing policy stored immutably after Commissioner approval; modification attempts after T-30 return 409. (FR-65.02)
13. FR-M65-013: Routing policy audit trail records policy version, approver UUID, and approval timestamp. (FR-65.02)
14. FR-M65-014: No post relies solely on a single carrier; routing policy enforces at least two carrier options per post. (NFR-65.02)

Customs pre-clearance

15. FR-M65-015: Generate pre-clearance manifest per container for airline-cargo-route posts. (FR-65.03)
16. FR-M65-016: Submit pre-clearance manifest to KRA e-Clearance at least 6 hours before estimated arrival time. (FR-65.03)
17. FR-M65-017: Record KRA electronic acknowledgement reference against container UID. (FR-65.03)
18. FR-M65-018: Alert if KRA pre-clearance is not acknowledged within 2 hours of submission. (FR-65.03)
19. FR-M65-019: Diplomatic Bag Officer confirms MFA pouch manifest in M65 before dispatch; triggers pre-clearance flag. (FR-65.03)
20. FR-M65-020: Circuit breaker on KRA e-Clearance and MFA API; clearance deferred to queue when upstream unavailable. (Integrations)
21. FR-M65-021: Customs embargo or refusal triggers alternative-routing alert to legal team and ECC. (FR-65.R2)

Hub receipt and consolidation

22. FR-M65-022: Hub Coordinator scans container UIDs on arrival at regional hub; verifies seals intact before accepting. (FR-65.04)
23. FR-M65-023: Record hub receipt event with officer UUID, hub location code, seal status, and timestamp. (FR-65.04)
24. FR-M65-024: Consolidate multiple post containers into hub-outbound unit; assign new container UID to outbound. (FR-65.04)
25. FR-M65-025: Hub-outbound manifest links all component container UIDs; any missing UID blocks hub dispatch. (FR-65.04)
26. FR-M65-026: Seal-broken event at hub triggers P1 alert to ECC and IAB; container quarantined pending inspection. (EC-OPS-03)
27. FR-M65-027: Re-sealing of inspected container requires new seal UID from M20 and IAB observer witness record. (FR-65.04)

Nairobi tally centre receipt

28. FR-M65-028: Nairobi Tally Centre Officer scans all container UIDs on delivery; checks against expected list. (FR-65.05)
29. FR-M65-029: Break seals in view of IAB observer and party agents; photograph seal-break; record receipt event with photo hash. (FR-65.05)
30. FR-M65-030: Record handoff officer UUID when ballot envelopes transfer to tally process custody. (FR-65.05)
31. FR-M65-031: Reconcile received ballot count against packing manifest; flag and alert on any count discrepancy. (FR-65.05)

Real-time shipment tracking

32. FR-M65-032: Track every container UID across tiers: embassy, in-transit-hub, at-hub, in-transit-nairobi, delivered. (FR-65.06)
33. FR-M65-033: Pull shipment milestones from IATA Cargo2000 and Kenya Airways Cargo API on 5-minute poll. (FR-65.06)
34. FR-M65-034: Pull diplomatic pouch status from MFA Kenya API on 5-minute poll. (FR-65.06)
35. FR-M65-035: Supplement API tracking with manual GPS-scan custody events at each handoff point. (FR-65.06)
36. FR-M65-036: ECC dashboard shows ETA per container; auto-refreshes every 5 minutes. (FR-65.06)
37. FR-M65-037: Circuit breaker on airline cargo API; fallback to manual AWB entry when API is unavailable. (Integrations)

Alerts and escalation

38. FR-M65-038: P1 alert when container not confirmed at hub within 2 hours of expected ETA. (FR-65.07)
39. FR-M65-039: P0 alert when container not confirmed at Nairobi by T+12 hours. (FR-65.07)
40. FR-M65-040: P1 alert delivered p99 under 5 minutes to ECC, Commissioner, and MFA on diplomatic-route delay. (NFR-65.05)
41. FR-M65-041: P0 escalation includes KNPS liaison notification and Commissioner page. (FR-65.07)
42. FR-M65-042: Alert acknowledge-and-resolve workflow; unresolved P0 alerts auto-escalate after 30 minutes. (FR-65.07)

Public summary

43. FR-M65-043: Publicly accessible summary showing posts dispatched, containers in transit, and containers delivered; no container-content detail. (FR-65.08)
44. FR-M65-044: Public summary refreshed every 30 minutes from ECC live state. (FR-65.08)
45. FR-M65-045: Public summary embeddable as widget on M11 Public Portal. (FR-65.08)

Merkle anchoring

46. FR-M65-046: Append all custody events to M20 Merkle tree diaspora sub-tree in sequence. (FR-65.09)
47. FR-M65-047: M20 Merkle root published every 10 minutes during election window. (FR-65.09)
48. FR-M65-048: Inclusion proof available for each custody event via M20 API. (FR-65.09)

Security and access control

49. FR-M65-049: Container UIDs, seal numbers, and ballot counts classified restricted; excluded from public-summary API response. (NFR-65.03)
50. FR-M65-050: SpiceDB RBAC: Embassy Officer write scope limited to own post; Hub Coordinator to assigned hub. (NFR-65.03)
51. FR-M65-051: SpiceDB-down: fail closed on writes; class-C reads only; return 503 problem+json. (EC-IDN-10)
52. FR-M65-052: All inputs parameterised; no injection on manifest fields, custody event location, or photo URL. (EC-SEC-05)
53. FR-M65-053: No voter PII enters M65 records; only container-level data stored and transmitted. (NFR-65.04)
54. FR-M65-054: Seal UIDs and ballot counts excluded from logs and telemetry; log-scanner CI gate. (EC-SEC-16)
55. FR-M65-055: JWT silent refresh mid-custody-event POST; next unauthenticated call returns 401. (EC-IDN-02)

Statutory and legal compliance

56. FR-M65-056: Constitution Art. 82(1)(e) physical-return enforcement: all posts must dispatch before T+24 hours; system blocks tally-centre reconciliation until all expected containers are accounted for.
57. FR-M65-057: Vienna Convention diplomatic pouch inviolability: M65 records only sealed handoff to MFA; no content inspection by M65 system.
58. FR-M65-058: Elections Act §84 chain-of-custody record: full custody chain exportable as HSM-signed PDF for audit with Merkle proof reference. (FR-65.17)
59. FR-M65-059: KRA Customs Act §98 pre-clearance: 6-hour lead-time compliance report available per container. (FR-65.03)

Audit and reporting

60. FR-M65-060: Full custody chain queryable for any container UID from creation to delivery. (FR-65.12)
61. FR-M65-061: Admin container list filterable by station, status, flight date, and alert state. (FR-65.13)

## Acceptance criteria

1. AC-M65-01: Seal-broken event pages ECC within 60 seconds; integration test with M16 stub confirms alert delivery time. (EC-OPS-03)
2. AC-M65-02: Offline packing sync idempotent; double-sync produces zero duplicate custody events on backend; property test over 100 random sync sequences.
3. AC-M65-03: Reconciliation count discrepancy flagged and operations alerted within 5 minutes; integration test with planted count mismatch.
4. AC-M65-04: Pre-clearance circuit breaker on MFA and KRA unavailability; clearance deferred to queue; no custody-chain gap; chaos test.
5. AC-M65-05: Full custody chain reconstructable from events alone for any container UID; property test with synthetic 10-hop chain.
6. AC-M65-06: API p99 within routine 99.5% SLA under simulated 50-post concurrent sync load; k6 load test.
7. AC-M65-07: KRA pre-clearance submitted at least 6 hours before estimated arrival for every airline-cargo container; compliance report check against 100% of containers.
8. AC-M65-08: P1 alert delivered p99 under 5 minutes for missed-hub-ETA event; load test with 50 synthetic simultaneous missed-ETA events.
9. AC-M65-09: P0 alert fires within 5 minutes for any container not confirmed at Nairobi by T+12 hours; integration test with simulated delayed container.
10. AC-M65-10: Routing policy immutable after T-30; attempt to modify after T-30 returns 409; test with backdated timestamp.
11. AC-M65-11: No voter PII in any M65 database table, API response, or telemetry; automated schema scan and Semgrep CI pass.
12. AC-M65-12: SpiceDB down triggers fail-closed on writes; class-C reads returned; PodChaos test confirms 503 problem+json on write attempts.
13. AC-M65-13: JWT expiry mid-custody-event POST: in-flight request completes; next call returns 401 or silent-refreshes; integration test with 5-second TTL realm.
14. AC-M65-14: Merkle inclusion proof valid for each custody event; M20 integration test verifies proof acceptance.
15. AC-M65-15: Offline app events survive 48-hour disconnect and sync without loss or duplication; offline-simulation test with WAL kill-and-resume.
16. AC-M65-16: No restricted-classification fields in public-summary API response; automated response-body scan on every build.
17. AC-M65-17: Duplicate custody event submission returns cached response; no second record created; idempotency test with same key and body.

## Applicable edge cases

Operations

EC-OPS-03: Seal broken in transit at hub; custody event raised with seal-broken status; P1 alert to ECC and IAB within 60 seconds; container quarantined pending re-seal inspection.
EC-OPS-06: Airport closure or flight rerouting mid-transit; alternate routing trigger fires; legal team and ECC notified; container tier updated to reflect new route.
EC-OPS-07: Embassy officer no-show or swap during packing; credential reassignment exactly-once; old credentials revoked immediately; no double custody event.

Security

EC-SEC-02: Tamper or chassis event on container seal detected at hub scan; quarantine stream event raised; ECC alert fired; inventory FSM transitions container to quarantined state.
EC-SEC-05: Injection on manifest URL, custody-event location, or photo-hash fields; parameterised queries only; Semgrep CI gate; ZAP scan on all write endpoints.
EC-SEC-07: Path traversal on manifest URL or photo upload path; input sanitised; fuzz extractor test with dotdot corpus.
EC-SEC-16: Seal UIDs and ballot counts excluded from all logs and traces; log-scanner CI fixture returns zero matches on Tier A and B field patterns.

Identity

EC-IDN-02: JWT expires mid-custody-event POST; in-flight request completes with original token; next call returns 401 or triggers silent refresh.
EC-IDN-09: IAB co-sign approver revoked between dual-control packing steps; both roles re-validated at commit; partial approval rejected.
EC-IDN-10: SpiceDB down during custody write; fail closed; 503 problem+json returned; class-C reads served from replica.

Data

EC-DAT-01: Duplicate custody event submitted with same idempotency key and body; cached response returned; no second record created; no re-execution.
EC-DAT-02: Same idempotency key with different custody event body; 422 idempotency-key-reuse returned; original record unchanged.
EC-DAT-07: Partial batch sync failure from offline app; per-event report generated; valid events committed; failed events requeued.
EC-DAT-14: Oversized custody photo exceeds 1 MB outbox limit; photo stored in MinIO with SHA-256 hash reference; event carries hash only.

Scale

EC-SCL-04: Consumer lag exceeds 60 seconds on diaspora custody event topic; backpressure applied; page fired; no OOM on consumer pod.
EC-SCL-08: Valkey eviction removes custody idempotency key; PG-durable idempotency store falls back; no duplicate event created.

Connectivity

EC-CON-01: Embassy offline for up to 7 days; WAL queue holds packing events signed and sequenced; syncs without gap on reconnect.
EC-CON-05: Flapping embassy link during offline sync; backoff plus jitter; retry budget respected; no duplicates delivered.
EC-CON-07: Satellite 600 ms RTT at remote embassy; transport-tuned timeouts; MTU-safe frames; zstd compression applied.
EC-CON-09: IPsec tunnel drop mid-batch sync; resume at sequence number; idempotent re-send; no gap in custody chain.

Mobile

EC-MOB-01: OS kills offline packing app mid-sync; WAL intact; resumable from last committed event; no duplicate submitted.

Time

EC-TIM-04: DST in diaspora embassy country; UTC storage throughout; gazetted EAT timezone displayed in admin console.

Legal

EC-LEG-02: Court-ordered election postponement issued mid-transit; signed calendar amendment propagated; containers held at hub pending signed unlock bundle; portal notice displayed.
EC-LEG-03: s.44A extension invoked at diaspora post after close of polls; packing workflow deferred by extension duration; ECC notified of revised dispatch deadline; custody timeline updated with new expected-ETA.
EC-LEG-05: Presidential run-off triggers second diaspora logistics cycle; routing policies re-approved by Commissioner; all new container UIDs carry run-off prefix; M65 routes second cycle through same hub network.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Container FSM, custody chain reconstruction, reconciliation arithmetic | nextest | M65-UNIT- |
| property | Custody chain idempotency; reconciliation invariant | proptest | M65-PROP- |
| contract | MFA/KRA/airline doubles; M24 station; M16 ECC | Pact | M65-PACT- |
| integration | Full logistics lifecycle; offline sync; reconciliation | nextest + testcontainers | M65-INT- |
| e2e | Admin packing; pre-clearance; receive; reconcile | Playwright | M65-E2E- |
| fuzz | Manifest payload; custody event fields | cargo-fuzz | M65-FUZZ- |
| security | ZAP; seal-break alert; log scanner | ZAP + Semgrep | M65-SEC- |

Tags: `#[trace("FR-M65-007","AC-M65-01","EC-OPS-03")]`

## Runbooks and dashboards

- docs/runbooks/inventory-svc-degradation.md (extend with diaspora-logistics section)
- docs/runbooks/diaspora-container-seal-break.md (new)
- platform/observability/inventory-svc/diaspora-logistics-dashboard.json
- platform/observability/inventory-svc/slo-alerts.yaml

## Open questions

- Customs embargo risk: if a country refuses ballot imports, alternative routing (DHL/courier) requires separate MoU; MFA engagement needed.
- Return-journey chain: whether spoilt and unused ballots must be returned and reconciled or destroyed in-country; legal opinion from IEBC required.
