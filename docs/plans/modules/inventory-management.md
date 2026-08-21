# M20 inventory-management

| field | value |
|---|---|
| M-number | M20 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M20-inventory-management.md |

## Purpose

End-to-end asset lifecycle management for all election materials (kits, ballot papers, seals, consumables): track custody from warehouse to polling station and back, enforce seal integrity, manage reserve pools, and integrate with OEM warranty and logistics partners to ensure zero untracked asset on election day.

## Surfaces and languages

| surface | technology | path under services/inventory-svc/ |
|---|---|---|
| Admin web interface | Django on packages/eel-web-py | django/inventory-admin/ |
| Mobile field app (MDM) | Flutter on packages/kit-sdk-flutter | apps/inventory-mobile/ |
| Inventory API | Rust (axum + utoipa) | crates/inventory-api/ |
| Domain + event store | Rust | crates/inventory-domain/ |
| Batch reconciliation | Rust | crates/inventory-batch/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Asset | asset_id, type (kit/ballot/seal/consumable), serial, model, status, station_code, last_seen_at | C | T1-audit |
| CustodyEvent | event_id, asset_id, from_custodian, to_custodian, location, ts_hlc, sig, seq | C | T1-audit |
| Seal | seal_id, asset_id, seal_type, affixed_at, affixed_by, broken_at, broken_reason, sig | C | T1-audit |
| Consumable | item_id, type, quantity, unit, station_code, replenishment_threshold | C | T2 |
| ReservePool | pool_id, scope, asset_type, count, alert_threshold, updated_at | C | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/inventory/assets.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /inventory/v1/assets | 3 accredited | JWT+SpiceDB | yes | Register asset |
| PATCH /inventory/v1/assets/{id}/custody | 3 accredited | JWT+SpiceDB | yes | Transfer custody; HSM-signed |
| POST /inventory/v1/seals | 3 accredited | JWT+SpiceDB | yes | Affix seal; HSM-signed |
| PATCH /inventory/v1/seals/{id}/break | 3 accredited | JWT+SpiceDB | yes | Break seal with reason; dual-officer |
| GET /inventory/v1/assets | 3 accredited | JWT+SpiceDB | n/a | Asset list with filters |
| GET /inventory/v1/assets/{id}/history | 3 accredited | JWT+SpiceDB | n/a | Full custody chain |
| GET /inventory/v1/reserves | 3 accredited | JWT+SpiceDB | n/a | Reserve pool status |
| GET /inventory/v1/summary | 3 accredited | JWT+SpiceDB | n/a | Election-day asset status summary |
| POST /inventory/v1/reconcile | 3 accredited | JWT+SpiceDB | yes | Trigger batch reconciliation |
| GET /inventory/v1/reconcile/{id}/report | 3 accredited | JWT+SpiceDB | n/a | Reconciliation report |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.inventory.asset_registered.v1 | emits | eel.inventory.asset.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.custody_transferred.v1 | emits | eel.inventory.asset.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.seal_affixed.v1 | emits | eel.inventory.seal.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.seal_broken.v1 | emits | eel.inventory.seal.v1 | event-source | yes (HSM dual) | exactly-once |
| eel.inventory.asset_status_changed.v1 | emits | eel.inventory.asset.v1 | event-source | yes | exactly-once |
| eel.inventory.reserve_alert.v1 | emits | eel.inventory.reserve.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.kit_revoked.v1 | consumes | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on custody + seal events |
| RFID/barcode readers | I10 | integrations/rfid/ | yes (stub) | manual entry fallback |
| OEM warranty / MES | I10 | integrations/oem/ | yes (stub) | async; alert on API fail |
| Boundary svc M18 | internal | integrations/boundary/ | yes | station geofence for location validate |
| ECC M16 | internal | integrations/ecc/ | yes | reserve alert + asset alert push |
| IAB dashboard M15 | internal | integrations/iab/ | yes | inventory status feed |
| Logistics partners | external | integrations/logistics/ | yes (stub) | shipment tracking; async |

## ML and AI

none

## Dependencies

- Modules: M08 (kit asset source events), M15 (IAB status consumer), M16 (ECC alert consumer), M18 (location boundary validation)
- Platform: P05 (audit), P06 (HSM), RFID infra
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-py, kit-sdk-flutter

## Feature checklist

Asset register:

FR-M20-001: Append-only master asset register: every asset has immutable UID, SKU, parent UID, state, condition, factory serial, and batch ID; WORM-capable append log.
FR-M20-002: No record deletable or modifiable; only new state-transition events appended; any attempt to modify returns 405 with audit log entry.
FR-M20-003: UID formats: kit KE27-KIT-{BATCH}-{SEQ}, component KE27-KIT-{BATCH}-{SEQ}-{TYPE}-{INSTANCE}, ballot KE27-BAL-{CONST}-{STN}-{SEQ}, seal KE27-SEL-{TYPE}-{SEQ}, consumable lot KE27-CSM-{TYPE}-{MFGYYMM}-{LOT}.
FR-M20-004: Triple carrier encoding per asset: Code 128 barcode (ISO/IEC 15417), HF RFID tag (ISO 14443A/15693, 13.56 MHz), and printed human-readable label.
FR-M20-005: Asset registration: serial number, type, model, SKU, initial custodian, and batch ID recorded at birth; birth record received from OEM MES or entered by Datacraft provisioning.
FR-M20-006: Kit component hierarchy: parent UID links kit to all sub-components; component lineage queryable in full event chain.

Chain of custody:

FR-M20-007: Eight choreographed custody workflows: OEM birth to Mombasa port, port clearance to Nairobi, Nairobi provisioning and acceptance testing, cascade to 47 counties (T-30 to T-21), county to 290 constituencies (T-20 to T-14), constituency to 55,393 stations (T-13 to T-7), reverse logistics (T+1 to T+7), post-petition disposition (T+30 to T+90).
FR-M20-008: Every custody event: signed by both transferor and transferee (mobile app + biometric where available), timestamped with HLC, geo-stamped, linked to seal UIDs and optional photograph hash.
FR-M20-009: Custody transfer HSM-signed p95 <=2 s; dual-officer required for high-value transfers and seal breaks.
FR-M20-010: Bulk custody transfer endpoint for county and constituency cascade batches; streaming ingest; checkpoint-resumable; bounded memory.
FR-M20-011: RFID gate reader: passive read at all warehouse entry/exit points without manual handling; events appended automatically.
FR-M20-012: Handheld RFID scan for ad-hoc condition checks; manual barcode entry fallback if reader fails; alert when manual rate exceeds threshold (indicating reader outage).
FR-M20-013: Mobile field app: offline-first; WAL-backed local event queue (100,000-event capacity, exceeding 7 days of county activity); drains on reconnect with exactly-once delivery.
FR-M20-014: County edge cache: read-only SQLite offline replica at all 47 county stores; events queue locally; sync on reconnect with zero loss.
FR-M20-015: Full custody chain query for any asset from birth to current location; p99 <1 s.
FR-M20-016: Geofence validation: custody transfer location validated against M18 station polygon; out-of-geofence event triggers P0 alert.

Condition state machine:

FR-M20-017: Kit lifecycle states: new, in_transit, in_warehouse, provisioned, deployed, returned, faulty, in_repair, decommissioned, destroyed (terminal); plus donated and evidentiary_hold.
FR-M20-018: Every state transition signed by responsible officer; invalid or retrograde transitions rejected with reason.
FR-M20-019: Faulty kit: automatic nearest-reserve lookup; dispatch alert to County Coordinator; L1 county spare 2-hour response SLA.
FR-M20-020: DOA kit (dead on arrival): P0 issue ticket auto-created; assigned to field engineer within 2 h; P0 SLA countdown shown.

Seal tracking:

FR-M20-021: Seal lifecycle: register, affix (officer identity + timestamp + photograph hash), verify, break; every state HSM-signed.
FR-M20-022: Seal break requires dual-officer HSM auth; single officer cannot complete; both officers re-validated at commit.
FR-M20-023: Sealed item with broken seal outside a custody transfer event triggers P0 alert to ECC within 5 min.
FR-M20-024: All seal events appended to Merkle chain; seal break events use dual-HSM signature.

RFID and GPS:

FR-M20-025: Kit GPS self-location reported every 15 min when powered; deviation from assigned station polygon triggers geofence alert.
FR-M20-026: Container-tracker (satellite/cellular) on sea-freight containers: door-open event outside Mombasa port geofence triggers P0 alert to Datacraft Ops and DCI within 5 min.
FR-M20-027: Truck GPS tracking on all road-leg movements; GPS milestones linked to custody events.

Consumables and ballot reconciliation:

FR-M20-028: Consumable tracking by lot UID, expiry date, issue location, and quantity; expiry enforcement; expired lot flagged before issue.
FR-M20-029: At station close: reconciliation of issued vs used plus unused plus spoilt consumables; variance outside tolerance auto-flags.
FR-M20-030: Ballot-box smart-lid integration (XV.4): reads passive RFID chip on each ballot deposit; ballots deposited vs authentications cross-checked; discrepancy >0.1% triggers automatic result hold.
FR-M20-031: RFID chip compatibility confirmed with ballot printer (Al Ghurair) at procurement; chip manifest received via SFTP + PGP.

Reserve pool:

FR-M20-032: Reserve pool tiers: L1 county spare (~100/county), L2 regional hub (5 hubs), L3 air-freight emergency (500 at 5 hubs).
FR-M20-033: Reserve pool status visible to ECC in real time; ECC reserve-pool dashboard shows tier breakdown per county with dispatch button.
FR-M20-034: L3 air-freight emergency spare: 30-min activation SLA for critical failures.
FR-M20-035: Reserve alert delivered to ECC within 60 s of pool dropping below threshold; alert includes nearest-spare calculation.

Merkle chain:

FR-M20-036: Every CustodyEvent, ConditionEvent, and LocationEvent appended to Merkle tree; Merkle root computed and published every 10 min.
FR-M20-037: Merkle root published every 10 min to five independent channels: IEBC.or.ke, GitHub, IAB domain, Twitter/X, IPFS; 100% of windows during election period.
FR-M20-038: Any channel showing a different Merkle root triggers IAB P0 alert within 5 min.
FR-M20-039: Merkle root delivered to M15 IAB Dashboard for live display.
FR-M20-040: Merkle chain integrity verified on every read and monitored continuously; any break triggers P0 alert within 5 min.

Alert engine:

FR-M20-041: Rule-based alerts: missing kit (not returned on schedule), late delivery (off SLA), tampered item (seal broken outside custody event), out-of-geofence, expired consumable, weight variance at pallet load, container door-open outside expected location.
FR-M20-042: Alert severity tiers: P0/P1/P2/P3; P0 routes immediately to ECC and IEBC Ops Director.
FR-M20-043: Seal-break P0 alert delivered to ECC p99 <5 min.

Issue management:

FR-M20-044: Issue taxonomy: hardware fault, missing, tampered, consumable issue, environmental damage, DOA.
FR-M20-045: Issue SLAs: P0 2 h, P1 8 h, P2 24 h, P3 72 h; SLA countdown timer visible in ticket; automated escalation on breach.
FR-M20-046: Ticket lifecycle: open, assigned, in_progress, resolved, closed; every transition audit-logged.
FR-M20-047: Escalation ladder: unacknowledged P0 escalates to IEBC Ops Director at 30 min; further escalation to IEBC Commissioner at 1 h.

Auditor-General access:

FR-M20-048: Daily signed event-log extract delivered to Auditor-General via SFTP + PGP; batch covers all events in prior 24 h.
FR-M20-049: Cumulative dataset accessible to Auditor-General and IAB via authenticated REST at all times.
FR-M20-050: Daily extract hash published on IAB domain; IAB can independently verify extract integrity.

OEM and logistics integrations:

FR-M20-051: Mechrevo MES integration: birth records, FAT results, component serials via REST/mTLS; fallback to offline batch file if API unavailable.
FR-M20-052: KRA eCargo integration: import declaration and inspection outcome linked to custody events.
FR-M20-053: KEBS PVoC registry: certificate verification per batch; batch blocked if certificate missing.
FR-M20-054: Al Ghurair ballot printer: lot data, serial ranges, RFID chip manifest received via SFTP + PGP.
FR-M20-055: Logistics partner tracking: DHL/FedEx/G4S/Kenya Airways Cargo shipment milestone events linked to custody events.
FR-M20-056: OEM warranty: asset registration triggers warranty record creation; fallback to manual CSV import if OEM API unavailable before G4.

Post-election disposition:

FR-M20-057: Three disposition tracks: refurbishment (cryptographic erasure + TPM key destruction ceremony), donation (DPA 2019 s.41 erasure certificate), physical destruction (licensed e-waste, witnessed shredding, destruction certificate).
FR-M20-058: TPM key destruction ceremony: witnessed by Datacraft DPO and IAB representative; signed ceremony record produced; biometric data cryptographic erasure per SECURITY.md SR-PRIV-008.
FR-M20-059: Biometric erasure certificates generated for 100% of decommissioned kits; DPA 2019 s.41 compliant.
FR-M20-060: Post-election reconciliation report: all 65,287 kits with final disposition; signed PDF export; delivered to Auditor-General at T+30; target zero missing assets.

Access control and admin UI:

FR-M20-061: RBAC: geographic scoping enforced at data layer; county officer cannot read or write another county's assets.
FR-M20-062: Every data access logged with actor, purpose code, and timestamp per DPA 2019 s.26.
FR-M20-063: Mechrevo factory access limited to own output; DPA 2019 s.36 processor obligation enforced.
FR-M20-064: Supreme Court IT Registrar: read-only access; pre-produced evidentiary extract per kit on-demand; signed PDF; IAB co-signs.
FR-M20-065: National asset map: county-level kit-count summary; click county for constituency drill-down; click station for individual kit detail.
FR-M20-066: Asset search: by UID, serial, station code, officer name; full event-chain timeline rendered.
FR-M20-067: Custody transfer screen (mobile): RFID/QR scan, destination officer select, biometric sign, seal UID entry, photo capture, submit; offline queue if no connectivity.
FR-M20-068: Reserve pool dashboard (ECC): tier breakdown per county; dispatch button with nearest-spare calculation.
FR-M20-069: Merkle root status panel: five-channel publication status; timestamp; hash; discrepancy highlighted.
FR-M20-070: Post-election reconciliation screen: all-asset final-state breakdown; missing count (target 0); disposition breakdown; signed PDF export.

Provisioning:

FR-M20-071: Provisioning throughput: 686 kits/day (20 parallel lines, 2 shifts, 28 min/kit); IMS handles batch rate without latency degradation (NFR-20.02).
FR-M20-072: Kit 1 birth record ingested by T-180; all 65,287 UIDs provisioned and assigned by T-30 (full-fleet dry run milestone).

## Acceptance criteria

AC-M20-01: Custody chain for any kit verifiable end-to-end from factory birth record to station deployment; every event hash-chained (integration test).
AC-M20-02: Seal break requires dual-officer HSM auth; single-officer attempt returns 403; both officers re-validated at commit (UAT + security test).
AC-M20-03: Merkle root published within 10 min in all five channels; discrepancy on any one channel triggers P0 alert within 5 min (integration test).
AC-M20-04: Seal-break P0 alert delivered to ECC p99 <5 min; no missed alerts in chaos injection of 100 concurrent seal breaks (integration test).
AC-M20-05: Reserve alert delivered to ECC within 60 s of pool dropping below threshold; alert includes nearest-spare coordinates (integration test).
AC-M20-06: Offline mobile queue holds 100,000 events (7-day typical county volume); drains to server without loss or duplication after reconnect (chaos test; WAL verified).
AC-M20-07: Batch reconciliation for all 65,287 kits completes within 4 h (load test; per appendix SLA; checkpoint-resumable after kill).
AC-M20-08: Out-of-geofence custody transfer location triggers P0 alert; transfer blocked at data layer (integration test).
AC-M20-09: Container door-open outside Mombasa port geofence triggers P0 alert within 5 min (integration test with container-tracker stub).
AC-M20-10: Biometric erasure certificate generated for 100% of decommissioned kits; DPA 2019 s.41 attestation signed by DPO and IAB representative (process audit).
AC-M20-11: Auditor-General daily extract hash published and independently verifiable via IAB domain (integration test; hash comparison).
AC-M20-12: Geographic access scoping enforced: IEBC Procurement JWT blocked from writing to kit outside its jurisdiction (penetration test at data layer).
AC-M20-13: Ballot-box smart-lid discrepancy >0.1% between deposited ballots and authentications triggers result hold (integration test with injected mismatch at exactly 0.1%).
AC-M20-14: Provisioning throughput sustained at 686 kits/day across 20 parallel lines without IMS API p99 degradation (load test).
AC-M20-15: 99.9% availability during election cycle; county edge caches operational in read-only offline mode when primary unreachable (SLO measurement per appendix cycle-critical class).

## Applicable edge cases

EC-TIM-01: Kit clock skew >5 min at boot; kit refuses custody session until NTS/GNSS time confirmed; event rejected by backend.
EC-TIM-02: NTP/NTS unreachable all day; HLC monotonic with time_confidence=low; custody events accepted but flagged; tolerance window applied.
EC-TIM-03: Backend clock jumps backward; HLC never regresses; skew alert fires; Merkle sequence number maintained in strictly increasing order.
EC-TIM-04: DST/TZ on diaspora kits: all custody event timestamps stored UTC; EAT displayed; cascade schedule computed in UTC.
EC-TIM-05: Leap second; timestamp parser tolerates 23:59:60; no custody event dropped.
EC-TIM-06: Event time < causation time (delayed sync from offline kit); accepted; audit skew anomaly logged; projection independent of wall order.
EC-TIM-07: Station assignment changes post-geofence-bundle; new polygon re-issued; kit must acknowledge; unacknowledged assignments listed in ECC.
EC-TIM-08: Post-petition disposition must complete by statutory T+90; scheduling constraint enforced; alert fires if disposition track falls behind.
EC-TIM-09: Asset freeze at T-60/T-30 provisioning milestone; OPA hard-close on retroactive disposition writes; all-or-nothing transaction.
EC-DAT-01: Duplicate custody event same idempotency key and body; cached response returned; no double-transfer; Merkle sequence not incremented.
EC-DAT-02: Same custody event key, different body; rejected with 422 idempotency-key-reuse; transfer not applied.
EC-DAT-03: Out-of-order CustodyEvent in stream; buffered until sequence gap filled or DLQ after timeout.
EC-DAT-04: Replayed signed CustodyEvent; rejected on Merkle sequence counter; tamper topic populated; alert fired.
EC-DAT-05: Poison consumable lot message (negative quantity); DLQ with reason; partition continues; ECC notified.
EC-DAT-06: CustodyEvent schema v1/v2 coexist during rolling deploy; both accepted; dual-emit; Karapace compat check.
EC-DAT-07: Partial bulk custody transfer failure (batch of 200 kits, 3 fail); per-row report; valid transfers committed; failed ones returned with reason.
EC-DAT-08: 65,287 kit operations during election cascade day; streaming ingest; checkpoint-resumable after kill; bounded memory.
EC-DAT-09: Unicode officer names and station names; NFC normalisation; ICU collation; grapheme-safe in all UI and reports.
EC-DAT-10: Zero reserve pool count is valid; no divide-by-zero; renders "0 units" with critical-level alert.
EC-DAT-11: Seal count exceeds expected (phantom seals scanned); blocked with reason; investigation flag auto-created.
EC-DAT-12: Debezium slot lag and WAL bloat on high-volume cascade day; alert at 60 s lag; outbox retained; disk guard prevents write-stop.
EC-DAT-13: Soft-deleted (decommissioned) asset appears in search; tombstone honoured; excluded from active counts; purge removes after legal hold.
EC-DAT-14: Oversized custody event (bulk photo attachments); outbox rejects >1 MB; photo blob stored in MinIO with hash reference.
EC-DAT-15: Mechrevo MES returns two serial numbers for the same physical kit; AdjudicationCase created; no auto-merge until resolved.
EC-DAT-16: Legacy kit pre-CloudEvents payload; shim upgrades envelope; original HSM signature preserved; field corpus tested.
EC-SEC-02: RFID tag cloned by adversary; multi-modal redundancy (RFID + barcode + photograph + weight) catches discrepancy; tamper alert; chain continues with quarantine flag.
EC-SEC-03: HSM unavailable; custody and seal events blocked; STH pauses; ingest continues in degraded mode; alert to ECC.
EC-SEC-05: Injection on asset search, condition-note, and resolution fields; parameterised queries only; Semgrep compile-time and ZAP dynamic scan required.
EC-SEC-16: Secrets or PII in custody-event notes refused at ingest; telemetry scrubbed; log-scanner fixture in CI.
EC-IDN-02: JWT expires mid-custody transfer; in-flight request completes; next request returns 401 with silent-refresh hint.
EC-IDN-09: Dual-officer: approver revoked between first and second sign; re-validate both at commit; transfer aborted if approver invalid.
EC-IDN-10: SpiceDB down; fail closed custody and seal writes; class-C reads (asset lookup, manifest download) allowed.
EC-SCL-04: Consumer lag >60 s on eel.inventory.asset.v1 topic; backpressure activated; page fired; no OOM.
EC-OPS-01: Power cut mid-reconciliation on field device; WAL journal-before-act; exact resume from last checkpoint; no double-transfer.
EC-OPS-08: RFID reader fails at warehouse gate; manual barcode entry logged; alert when manual rate exceeds 20% of total scans indicating systematic reader outage.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | custody FSM, seal lifecycle, reserve alert trigger, reconciliation diff, geofence validate | nextest + Flutter test | M20-UNIT- |
| property | custody chain append-only; seal state machine; no double-transfer | proptest | M20-PROP- |
| contract | M06 kit-revoked (consumer); M16 reserve alert (provider); M15 status feed (provider) | Pact | M20-PACT- |
| integration | SoftHSM custody + seal; RFID stub; logistics stub; offline drain | nextest + testcontainers | M20-INT- |
| e2e | asset register -> custody chain -> seal affix -> break (dual) -> reconcile | patrol + Playwright | M20-E2E- |
| fuzz | custody transfer payload; seal break reason; reconciliation input | cargo-fuzz | M20-FUZZ- |
| formal | custody chain append-only; no untracked asset in steady state | kani | M20-FORMAL- |
| load | 60k kit transfers + 40k station reconciliations; mobile drain | k6 | M20-LOAD- |
| chaos | HSM kill; SpiceDB kill; RFID reader fail; mobile offline 7 days | Chaos Mesh | M20-CHAOS- |
| security | RFID clone; signature replay; ASVS L3; Semgrep | Semgrep | M20-SEC- |

## Runbooks and dashboards

- docs/runbooks/inventory-svc-degradation.md (create): HSM fail; RFID outage; reconciliation failure; reserve alert escalation.
- docs/runbooks/inventory-seal-ceremony.md (create): seal affix, verify, and break dual-officer ceremony steps.
- platform/observability/inventory-svc/grafana-inventory.json: asset status heatmap, custody transfer rate, seal integrity count, reserve pool levels, reconciliation lag.
- platform/observability/inventory-svc/slo-alerts.yaml: 99.9% burn rate; reconciliation > 4 h; reserve below threshold unacknowledged > 30 min.

## Open questions

- OEM/logistics API availability: OEM warranty APIs and logistics partner tracking APIs are unconfirmed; if unavailable before G4, the integration degrades to manual CSV import.
- RFID lead time: RFID reader procurement and field deployment requires >= 12 months lead time; if not confirmed by T-18 months, barcode scanning is the primary scan method.
