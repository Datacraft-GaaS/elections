# M39 — Ballot Printing Chain of Custody

**Module ID:** M39
**Name:** Ballot Printing Chain-of-Custody System
**Runs on:** Central servers (Kenyan cloud) + printer MES integration + RFID scan infrastructure
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Track every ballot paper from print-run authorisation through serial-range gazettement, RFID chip embedding, printer MES lot-data integration, port clearance, Nairobi central warehouse, constituency distribution, and individual polling-station scan-in/scan-out. Make ballot-chain tampering computationally detectable and immediately auditable. This module was designed with direct awareness that chain-of-custody weakness in Kenya's 2017 election contributed to the environment in which Chris Msando was killed; integrity here is non-negotiable.

Strategy reference: §XV.4 Element 2 (ballot security), §XVI.3 (named chain-of-custody threat).

## 2. User Personas

- **IEBC Print Authorisation Officer** — issues signed print-run order; specifies serial ranges
- **Printer MES Integration (Al Ghurair UAE or approved Kenyan alternative)** — system-to-system: sends lot-data (serial ranges, quantities, RFID chip IDs) per batch
- **IEBC Customs Clearance Officer** — records port-of-entry scan event; attaches C-63 customs form
- **IEBC Warehouse Manager (Nairobi)** — scans incoming pallets; confirms serial-range match; flags discrepancies
- **County Logistics Officer** — scans and signs for constituency allocations; records vehicle and seal number
- **Presiding Officer** — scans ballot packet at polling station on opening; scans unused ballot return at close
- **IEBC Audit Lead** — runs reconciliation reports; flags count discrepancies
- **IAB Observer** — views aggregate chain-of-custody events; raises concerns
- **Supreme Court IT Registrar (M40)** — receives signed chain-of-custody bundle for petition use

## 3. Functional Requirements

- **FR-M39-001** Print-run authorisation: IEBC Print Authorisation Officer issues a signed digital print order specifying: election ID, contest ID, constituency code, quantity authorised, serial-range start/end, paper specification hash. Order requires dual sign-off (officer + IEBC Commissioner). Print order anchored in P05 Merkle chain before transmission to printer.
- **FR-M39-002** Serial-range gazettement: authorised serial ranges gazetted as a Kenya Gazette supplement (PDF + structured JSON); gazette hash anchored in Merkle chain; public verifiable.
- **FR-M39-003** Printer MES integration: printer sends back lot-data JSON (serial ranges actually printed, RFID chip IDs per range, print date, quantity, reject count) via mTLS-secured API. System reconciles lot-data against authorised print order; any quantity variance > 0.01% triggers immediate IEBC alert.
- **FR-M39-004** RFID embedding verification: printer attests that each ballot in the lot has an embedded RFID chip with a chip ID in the declared range. IEBC may conduct spot-check RFID scan at printer facility; spot-check result uploaded to the chain-of-custody record.
- **FR-M39-005** Airfreight / sea-freight manifest: IEBC Customs Officer records: carrier, AWB/BL number, seal numbers, container ID, port of entry, arrival date, C-63 customs form ID. Manifest attached to shipment record.
- **FR-M39-006** Port clearance scan: on customs clearance, IEBC Customs Officer scans a random sample of ballot RFID chips (minimum 200 per lot) and records match against declared chip ID list. Discrepancies trigger hold + IEBC Commissioner notification.
- **FR-M39-007** Nairobi warehouse receipt: Warehouse Manager scans all pallet RFID tags on arrival; system confirms serial-range completeness; any missing range segment flagged; signed warehouse receipt generated and anchored.
- **FR-M39-008** Constituency allocation: County Logistics Officer creates an allocation record (constituency code, serial sub-range, quantity); officer and a witness scan-out ballot packets from warehouse; vehicle registration and physical seal number recorded; allocation record signed by both officer and witness.
- **FR-M39-009** Polling-station receipt: Presiding Officer scans packet RFID on delivery; system confirms serial range matches allocation record; signed receipt generated; M08 kit-side RFID scanner used.
- **FR-M39-010** Election-day usage tracking: Presiding Officer records ballots issued (from M08 kit BVI turnout); at close, scans remaining unused ballot bundle RFID; system computes: issued = received - unused - spoilt; discrepancy vs BVI turnout count flagged immediately to M16 Election Command Centre.
- **FR-M39-011** Post-election return: unused ballots returned to constituency store; RFID scan-in recorded; system generates a reconciliation report per station (received / issued / unused / spoilt / discrepancy).
- **FR-M39-012** Audit trail immutability: every scan event, allocation, and reconciliation record is hashed and added to P05 Merkle chain within 60 seconds of creation.
- **FR-M39-013** Public chain-of-custody summary: aggregate view (not individual serial ranges) published on M11 public portal showing lot quantities, gazette reference, warehouse receipt date, and constituency allocation status. No individual RFID chip IDs published.

## 4. Non-Functional Requirements

- **NFR-M39-001** RFID scan throughput: warehouse scanner must process >= 500 tags/minute; station scanner >= 50 tags/minute.
- **NFR-M39-002** Lot-data reconciliation: <= 5 minutes from printer MES submission to IEBC reconciliation report.
- **NFR-M39-003** Merkle anchor latency: every scan event anchored within 60 seconds.
- **NFR-M39-004** Availability: chain-of-custody API >= 99.9% from print-run to T+30; offline scan queue on kit must persist if connectivity lost.
- **NFR-M39-005** Security: print order transmission to printer over mTLS + PGP-encrypted payload; all RFID chip ID lists stored encrypted at rest (AES-256); access to full serial-range data restricted to IEBC_AUDIT_LEAD role.
- **NFR-M39-006** DPA 2019: no personal data on ballot papers; chain-of-custody records retain for 7 years per Elections Act records requirement.
- **NFR-M39-007** Tamper evidence: any attempt to modify a committed scan record must be detectable via Merkle proof verification.
- **NFR-M39-008** Scalability: handles 290+ constituency allocations concurrently during distribution week.

- **NFR-39-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-39` (replace 39 with the module number); alert ruleset ID `iems-39-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Print Authorisation Officer, I issue a dual-signed print order for Nairobi County presidential ballots; the order is anchored in the Merkle chain before the printer receives it.
2. As the printer MES system (Al Ghurair), I submit lot-data JSON for the completed print run; the system reconciles quantity and RFID ranges against the signed print order within 5 minutes.
3. As an IEBC Customs Officer, I scan 200 random RFID chips at the port; all match the declared chip ID list; I attach the C-63 form and the shipment is cleared.
4. As a Warehouse Manager, I scan all incoming pallets; the system highlights that serial range 0045000-0045500 is absent; I immediately alert the IEBC Commissioner.
5. As a County Logistics Officer, I scan-out ballot packets for Westlands constituency; the system issues a signed allocation record with the vehicle and seal number.
6. As a Presiding Officer, I scan the ballot packet on delivery at Lavington Primary; the system confirms the serial range matches my allocation record and issues a receipt.
7. As a Presiding Officer at close of poll, I scan the unused ballot bundle; the system computes issued=480, unused=120, spoilt=3 and confirms this matches my BVI turnout figure of 480.
8. As an IEBC Audit Lead, I run the post-election reconciliation report for Westlands; I can see every scan event from printer to station for every ballot packet, with Merkle proofs.
9. As an IAB Observer, I request the signed chain-of-custody summary for a contested constituency and compare gazette quantities to station receipts.
10. As a petitioner, I request the chain-of-custody bundle for Kibra constituency from M40; I receive a signed package with Merkle proofs for every scan event.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime

class PrintOrder(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    election_id: str
    contest_id: str
    constituency_code: str
    quantity_authorised: int
    serial_range_start: int
    serial_range_end: int
    paper_spec_hash: str
    issued_by_officer_id: str
    countersigned_by_commissioner_id: str
    issued_at: datetime
    merkle_node_id: str | None
    gazette_notice_id: str | None

class LotData(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    print_order_id: str
    printer_facility_id: str
    serial_range_start: int
    serial_range_end: int
    quantity_printed: int
    quantity_rejected: int
    rfid_chip_id_list_hash: str     # SHA-256 of sorted chip ID list; full list stored encrypted
    print_date: datetime
    received_at: datetime
    variance_pct: float
    variance_alert_sent: bool

class CustodyEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    lot_id: str
    event_type: Literal[
        "PORT_CLEARANCE","WAREHOUSE_RECEIPT","CONSTITUENCY_ALLOCATION",
        "STATION_RECEIPT","STATION_CLOSE_RETURN","RFID_SPOT_CHECK"
    ]
    location_code: str
    officer_id: str
    witness_id: str | None
    quantity_scanned: int
    rfid_match_count: int
    rfid_mismatch_count: int
    seal_number: str | None
    vehicle_registration: str | None
    scanned_at: datetime
    merkle_node_id: str | None

class StationReconciliation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    election_id: str
    ballots_received: int
    ballots_issued: int             # from BVI turnout
    ballots_unused: int
    ballots_spoilt: int
    discrepancy: int                # received - issued - unused - spoilt
    discrepancy_flag: bool
    reconciled_at: datetime
    reconciled_by: str
    merkle_node_id: str | None
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/ballot-chain/print-orders | IEBC_OFFICER JWT + HSM dual-sign | 5/day | Issue signed print order |
| POST | /v1/ballot-chain/lot-data | PRINTER_SYSTEM mTLS | 20/day | Printer MES lot-data submission |
| POST | /v1/ballot-chain/events | IEBC_OFFICER JWT | 500/hr | Record custody event (scan-in/out) |
| GET | /v1/ballot-chain/events | IEBC_AUDIT_LEAD JWT | 200/hr | List custody events (filterable by lot, location) |
| GET | /v1/ballot-chain/events/{id}/proof | IEBC_AUDIT_LEAD JWT | 100/hr | Merkle inclusion proof for event |
| POST | /v1/ballot-chain/reconciliation | PRESIDING_OFFICER JWT | 100/hr | Submit station end-of-day reconciliation |
| GET | /v1/ballot-chain/reconciliation/{station_code} | IEBC_AUDIT_LEAD JWT | 100/hr | Get station reconciliation |
| GET | /v1/ballot-chain/public/summary | Public | 200/min | Aggregate chain-of-custody status (no serial IDs) |
| GET | /v1/ballot-chain/court-bundle/{constituency_code} | COURT_REGISTRAR JWT | 5/hr | Signed chain-of-custody bundle for petitioners |

## 8. Integration Points

- **Printer MES (Al Ghurair UAE / approved Kenyan alternative)** — lot-data API (external, mTLS)
- **M08 RTS Device** — RFID scanner on KIEMS kit; station receipt and close-of-poll scan
- **M06 BVI Back-End** — BVI turnout figure for ballot-count reconciliation
- **M16 Election Command Centre** — real-time alert on discrepancy events
- **M40 Election Court IT Interface** — court chain-of-custody bundle
- **M11 Public Portal** — public summary publication
- **M15 IAB Dashboard** — audit summary
- **P05 Merkle Chain** — all events anchored
- **P06 HSM** — print order dual signing
- **Kenya Revenue Authority Customs** — C-63 form reference (external)

## 9. UI/UX Key Screens

1. **Print Order Builder** — form with election, contest, constituency, quantity, serial-range inputs; paper-spec hash auto-computed; dual-sign button for officer and Commissioner.
2. **Lot Reconciliation Dashboard** — per-print-order: authorised vs printed quantities; RFID match rate; variance alert badges; drill-through to lot-data detail.
3. **Custody Event Log** — chronological list of events per lot; map view of custody chain from printer to station; event type icons; Merkle proof button.
4. **RFID Scanner App (M08 integration)** — simple scan screen: scan packet, confirm serial range match, confirm officer ID; works offline; syncs on connect.
5. **Station Reconciliation Screen** — Presiding Officer: ballots received (pre-filled), issued (from BVI), unused (scan count), spoilt (manual entry); computed discrepancy shown; submit button.
6. **Audit Dashboard** — IEBC Audit Lead: national reconciliation summary; constituencies with discrepancies highlighted; export to PDF for IAB.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Ballot paper security and chain of custody | Elections (General) Regulations 2012 r.65-r.72 |
| Gazettement of ballot serial ranges | Elections Act 2011 §63 |
| Ballot paper return and reconciliation | Elections (General) Regulations 2012 r.83 |
| Chain-of-custody records retention | IEBC records management policy; 7 years |
| Customs clearance documentation | East African Community Customs Management Act 2004 |
| Evidence admissibility | Evidence Act Cap 80; Electronic Transactions Act 2007 |
| Chain-of-custody for court petitions | Elections Act 2011 §75-§83; Constitution Art 140 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Lot quantity variance (printer vs authorised) | <= 0.01% per lot | Lot reconciliation report | IEBC Audit Lead |
| Port RFID spot-check match rate | >= 99.9% | Spot-check log | IEBC Customs Officer |
| All custody events anchored in Merkle chain | 100% | Merkle reconciliation | Datacraft Technical |
| Station reconciliation completion by T+1 | >= 99% of stations | Reconciliation records | IEBC Audit Lead |
| Discrepancy alert to Command Centre | <= 2 minutes from detection | Alert timestamp delta | IEBC Ops Director |
| Court chain-of-custody bundle generation | <= 2 hours from request | Bundle creation timestamp | Datacraft Technical |

## 12. Test Strategy

- **Unit:** serial-range completeness checker; RFID chip ID list hash verifier; discrepancy calculator; Merkle anchor trigger.
- **Integration:** printer MES lot-data submission to reconciliation; M08 RFID scan to custody event; M06 BVI turnout to station reconciliation; M16 alert on discrepancy.
- **E2E:** simulate full lifecycle from print order to station reconciliation; verify Merkle proof for every event; verify court bundle includes all events.
- **Security:** printer system submits lot-data without valid mTLS certificate (must reject); officer attempts to modify committed custody event (must fail — Merkle proof detects); attempt to access full RFID chip ID list without IEBC_AUDIT_LEAD role.
- **Load:** 290 constituency allocations processed simultaneously during distribution week.
- **Chaos:** M08 offline during station receipt — verify local event queue; sync on reconnect; no duplicate events.
- **Penetration:** OWASP Top 10 on chain-of-custody API; attempt RFID chip ID list extraction via public summary endpoint.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Printer MES integration contract finalised | 2026-10-01 | Al Ghurair or approved alternative; API spec delivered |
| Printer MES integration tested | 2026-12-01 | Synthetic print run |
| RFID scanner hardware provisioned | 2027-03-01 | M08 kit RFID scanner firmware update |
| Dry-run print order (by-election) | 2027-02-28 | End-to-end custody test |
| Gazettement format approved by AG | 2027-04-01 | Attorney General sign-off on serial-range gazette format |
| Pre-election print run | 2027-06-01 | T-70: main print run begins |
| Nairobi warehouse receipt complete | 2027-07-01 | T-40 |
| Constituency distribution complete | 2027-07-31 | T-10 |
| Station receipt scan-in | 2027-08-08 | T-2 |
| Post-election reconciliation reports | 2027-08-17 | T+7 |

## 14. Dependencies

- Printer MES API (Al Ghurair or alternative) — external; contract required
- M08 RTS Device — RFID scanner hardware + firmware
- M06 BVI Back-End — turnout data for reconciliation
- M16 Election Command Centre — discrepancy alert consumer
- M40 Election Court IT Interface — court bundle delivery
- P05 Merkle Chain — event anchoring
- P06 HSM — print order dual signing
- Kenya Revenue Authority — customs documentation

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Printer MES API incompatible or unavailable | Medium | Critical | API spec co-designed with printer at contract signature; fallback: manual lot-data upload by IEBC officer with scanned attestation |
| RFID chip tampering in transit | Low | Critical | Port spot-check minimum 200 chips per lot; any mismatch triggers hold and full re-scan before clearance |
| Serial-range discrepancy between printer data and warehouse receipt | Medium | High | Automatic alert at > 0.01% variance; warehouse hold until IEBC Commissioner authorises release or printer re-submits |
| RFID scanner failure at station on election day | Medium | High | M08 falls back to manual serial-range record; reconciliation possible post-election via manual tally |
| Printer substitution without IEBC approval | Low | Critical | Print order specifies approved printer facility ID; lot-data must originate from that facility's mTLS certificate; any deviation triggers immediate Commissioner notification |

## 16. Cross-References

- `M06-bvi-backend.md` — turnout data for ballot reconciliation
- `M08-rts-device.md` — RFID scanner hardware
- `M11-public-portal.md` — public summary
- `M15-iab-dashboard.md` — audit metrics
- `M16-election-command-centre.md` — discrepancy alert
- `M40-election-court-it-interface.md` — court chain-of-custody bundle

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XV.4 Element 2 — ballot security | FR-M39-001 through FR-M39-013 |
| §XVI.3 — chain-of-custody named threat | FR-M39-006, FR-M39-010, NFR-M39-007 |
| Chris Msando context — integrity commitment | Design rationale; dual sign-off (FR-M39-001); Merkle anchoring (FR-M39-012) |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if IEBC terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and IEBC DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to IEBC ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** IEBC ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
