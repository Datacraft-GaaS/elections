# M20 — Inventory Management

**Module ID:** M20
**Name:** Inventory Management System (IMS)
**Owner:** Datacraft Limited — Operations Engineering Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part II §PM-12, Part XV §XV.4, Part XXI.A.4, INVENTORY-MANAGEMENT.md §§1-11

---

## Purpose

The Inventory Management System tracks every physical KIEMS asset — 65,287 kits, 718,000+ sub-components, 60,932 ballot-box smart-lids, 1.9 million tamper-evident seals, and 3,500 consumable lots — from OEM factory to post-election disposition, with a Merkle-anchored chain of custody at every handoff. The OAG 2024 report found 200 KIEMS kits missing and 3,433 faulty after 2022, with IEBC unable to trace movements (Biometric Update March 2025). IMS makes that failure impossible: every item has a globally unique ID, every transfer is signed by both parties, every geofence deviation triggers an alert, and every Merkle root is published every 10 minutes to five independent channels. The Auditor-General, IAB, and Supreme Court IT Registrar have read-only access at all times.

---

## User Personas

| Persona | Access level | Primary task |
|---|---|---|
| Mechrevo factory (OEM) | Own output only (write) | Birth records, FAT results, component serials |
| Datacraft provisioning team | All kits at Nairobi centre (write) | Provisioning events, acceptance testing |
| Datacraft field engineer | Assigned county (write) | Condition events, custody transfers |
| IEBC procurement officer | All kits, no PII (write: receipt confirm) | Receipt confirmations, payment milestone triggers |
| County Returning Officer | Own county (write: county receipts) | County custody receipt |
| Constituency Returning Officer | Own constituency (write) | Constituency custody receipt |
| Presiding Officer | Own station's kit only (write: activation) | Station activation event |
| IEBC Operations (ECC) | All kits, real-time (write: alert acknowledge) | Monitoring, alert triage |
| Independent Assurance Board | All events, read-only | Audit; quarterly report |
| Auditor-General | All events, batch extract | Annual audit |
| Security services (KNPS/DCI) | Flagged assets only, on-request, read-only | Theft investigation |
| Public | Aggregated, no PII, read-only | Transparency |

---

## Functional Requirements

### FR-20.01 — Master asset register
Append-only source of truth for every asset. Every asset record: UID (immutable), SKU, parent UID (kit-component relationship), state, condition, factory serial, batch ID, all lifecycle events as linked event chain. No record can be deleted or modified; only new state-transition events appended. Stored in PostgreSQL with WORM-capable append log.

### FR-20.02 — Identifier architecture
Globally unique UIDs assigned at birth. Formats:
- Kit: `KE27-KIT-{BATCH:02d}-{SEQ:06d}`
- Component: `KE27-KIT-{BATCH:02d}-{SEQ:06d}-{TYPE:4s}-{INSTANCE:02d}`
- Ballot: `KE27-BAL-{CONST:3d}-{STN:5d}-{SEQ:7d}`
- Seal: `KE27-SEL-{TYPE:3s}-{SEQ:08d}`
- Consumable lot: `KE27-CSM-{TYPE:4s}-{MFGYYMM}-{LOT:06d}`

Every UID encoded on three simultaneous carriers: Code 128 barcode (ISO/IEC 15417), HF RFID tag (ISO 14443A/ISO 15693, 13.56 MHz), and printed human-readable label.

### FR-20.03 — Chain-of-custody tracking (8 defined workflows)
Eight choreographed custody workflows per INVENTORY-MANAGEMENT.md §5:
1. OEM birth to Mombasa port
2. Port clearance to Nairobi provisioning centre
3. Nairobi provisioning and acceptance testing
4. Cascade to 47 counties (T-30 to T-21)
5. County to 290 constituencies (T-20 to T-14)
6. Constituency to 55,393 polling stations (T-13 to T-7)
7. Reverse logistics (T+1 to T+7)
8. Post-petition disposition (T+30 to T+90)

Every custody event: signed by both transferor and transferee (mobile app + biometric where available), timestamped, geo-stamped, linked to seal UIDs photographically.

### FR-20.04 — RFID infrastructure
Gate readers at all warehouse entry/exit points perform passive reads without manual handling. Handheld RFID readers for ad-hoc scans. Kit RFID tag embedded in chassis at PCB level by Mechrevo (INVENTORY-MANAGEMENT.md §2.6). RFID overwritten at Nairobi provisioning with final station assignment.

### FR-20.05 — GPS and geofence enforcement
GPS self-location on powered kits (every 15 minutes when powered). Geofence enforcement: kit outside assigned station polygon triggers alert. Container-tracker (satellite/cellular) on sea-freight containers: door-open events outside Mombasa port geofence trigger immediate alert. Truck GPS tracking on all road-leg movements.

### FR-20.06 — Condition state machine
Kit lifecycle states: `new` → `in_transit` → `in_warehouse` → `provisioned` → `deployed` → `returned` → `faulty` → `in_repair` → `decommissioned` → `destroyed` (terminal). Additional states: `donated` (post-election) and `evidentiary_hold` (petition window). Every state transition signed by the responsible officer.

### FR-20.07 — Tamper-evident seal tracking
Every seal UID registered in IMS against the item it seals at moment of sealing: seal UID + item UID + sealing officer identity + timestamp + photograph hash. Seal breaking events recorded; any sealed item whose seal is broken outside a custody transfer event triggers a P0 alert.

### FR-20.08 — Consumable tracking and reconciliation
Consumable lots tracked by lot UID, expiry date, issue location, and quantity. At station close, reconciliation: issued quantity vs used + unused + spoilt. Any variance outside tolerance triggers a flag. Lot-level RFID for ballot-paper bundles (read by ballot-box smart-lid, per XV.4 Element 3).

### FR-20.09 — Ballot-box smart-lid integration (XV.4)
Each ballot has a passive RFID chip (embedded at printer, Al Ghurair or equivalent). Ballot-box smart-lid reads each chip on deposit. IMS receives: ballot UID deposited, station code, timestamp. Cross-check: ballots deposited vs authentications. Discrepancy > 0.1% triggers automatic result hold. Coordination with Tender 02 (ballot printing) on RFID chip compatibility.

### FR-20.10 — Merkle-chain publication
Every CustodyEvent, ConditionEvent, and LocationEvent appended to the Merkle tree. Root hash published every 10 minutes to five channels: IEBC.or.ke, GitHub, IAB domain, Twitter/X, IPFS. Any channel showing a different root triggers IAB immediate alert. Root hash also delivered to M15 for IAB Dashboard display.

### FR-20.11 — Alert engine
Rule-based + ML anomaly baseline alerting for: missing kit (not returned on schedule), late delivery (off SLA), tampered item (seal broken outside custody event), out-of-geofence, expired consumable, weight variance at pallet load (INVENTORY-MANAGEMENT.md §3.8), door-open event on sea container outside expected location. Alert severity P0/P1/P2/P3; P0 routes immediately to ECC and IEBC Ops Director.

### FR-20.12 — Issue management (6 sections)
Full ticket lifecycle per INVENTORY-MANAGEMENT.md §6: issue taxonomy (hardware fault, missing, tampered, consumable issue, environmental damage, DOA), severity SLA (P0: 2h, P1: 8h, P2: 24h, P3: 72h), ticket lifecycle (open → assigned → in-progress → resolved → closed), escalation ladder, and public-facing issue channels.

### FR-20.13 — Auditor-General read-only access
Daily signed event-log extract delivered to Auditor-General's Office via SFTP + PGP. Batch extract covers all events in prior 24 hours; cumulative dataset accessible via authenticated REST. Extract hash published on IAB domain daily.

### FR-20.14 — Post-petition disposition tracking
Three tracks per INVENTORY-MANAGEMENT.md §8: (A) refurbishment for by-election use (cryptographic erasure + TPM key destruction ceremony), (B) donation to schools (DPA 2019 s.41 erasure certificate), (C) physical destruction (licensed e-waste facility, witnessed shredding, destruction certificate). Every track produces a signed terminal state record. Biometric data cryptographic erasure: TPM key destruction ceremony witnessed by Datacraft DPO + IAB representative, executed per SECURITY.md SR-PRIV-008 (media sanitisation procedure).

### FR-20.15 — Reserve pool management
Reserve pool tiers per INVENTORY-MANAGEMENT.md §11.1: L1 county spare (~100 per county), L2 regional hub spare (5 hubs), air-freight emergency spare (500 at 5 hubs). Any deployed kit failure triggers automatic lookup of nearest spare, dispatch alert to county coordinator, 2-hour response SLA for L1, 30-min air-freight activation for critical failures. Reserve pool status visible to ECC in real time.

---

## Non-Functional Requirements

### NFR-20.01 — Availability
IMS primary and mirror: 99.99% during election window. County edge caches operate in read-only offline mode if connectivity lost; events queue locally and sync on reconnect.

### NFR-20.02 — Throughput
Provisioning throughput: 686 kits/day (20 parallel lines, 2 shifts, 28 minutes per kit). Cascade to counties: ~5,935 kits/day. Station deployment: ~7,913 stations/day. IMS must handle these batch rates without latency degradation.

### NFR-20.03 — Data integrity
Every event is append-only. No record can be modified or deleted. Hash chain integrity verified on every read and monitored continuously. Any break triggers P0 alert to IAB and ECC.

### NFR-20.04 — Privacy (DPA 2019)
Purpose limitation: asset tracking only; no voter data in IMS. Biometric data is never stored in IMS; only kit serial and station assignment. Access is geographically scoped (county officer cannot read another county). Every access logged with purpose-of-access code per DPA 2019 s.26. Mechrevo (factory) access limited to own output; DPA 2019 s.36 (processor obligation).

### NFR-20.05 — Deployment topology
Primary (Nairobi Tier III DC) + mirror (second-site Nairobi DC, active-active) + escrow (read-only WORM cold replica, second jurisdiction). County edge caches (SQLite offline replica) at all 47 county stores. Mobile app (offline-first) for field engineers and county officers. Per INVENTORY-MANAGEMENT.md §4.5.

---

## User Stories

1. As the IEBC Procurement Officer, I want to see a live count of kits at each custody level at all times, so that I can confirm delivery milestones and trigger payment authorisations.
2. As an Auditor-General auditor, I want to download a daily signed event log and verify its Merkle hash independently, so that I can confirm the audit trail has not been tampered with.
3. As an IAB Member, I want to see any seal-break alert in real time on the M15 Dashboard, so that I can investigate potential tampering before it becomes a petition ground.
4. As a Datacraft Field Engineer, I want to scan a kit's RFID and update its condition in the mobile app, so that the master record reflects the physical reality without paperwork.
5. As a County Returning Officer, I want to sign digital receipt for all kits delivered to my county, so that my responsibility for those assets is documented and my signature is cryptographically verified.
6. As the ECC Operations Manager, I want to see the reserve pool status for each county in real time, so that I can dispatch spare kits before a threshold is reached.
7. As a Presiding Officer, I want to activate my kit on arrival at the polling station and have the location confirmed against the geofence, so that the system records my station as properly established.
8. As the Datacraft DPO, I want to generate a cryptographic erasure certificate for each decommissioned kit, so that IEBC can demonstrate DPA 2019 s.41 compliance for data disposal.
9. As a Civil Society Observer, I want to see the aggregated public inventory summary, so that I can verify that all kits are accounted for before and after the election.
10. As an IEBC Commissioner, I want a post-election reconciliation report showing all 65,287 kits with final disposition, so that the OAG audit will find zero missing assets.

---

## Data Model Additions

```python
class Asset(BaseModel):
    model_config = ConfigDict(extra='forbid')
    uid: str  # UUID7 or KE27-format
    sku: str
    parent_uid: str | None
    state: str
    condition: str  # new | good | faulty | decommissioned
    factory_serial: str
    batch_id: str
    created_at: datetime
    current_facility_id: str | None

class Facility(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    facility_type: str  # factory | port | warehouse | rdc | constituency | station
    name: str
    gps_lat: float
    gps_lng: float
    county_code: str | None
    constituency_code: str | None
    station_code: str | None
    jurisdiction_level: str

class CustodyEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    asset_uid: str
    from_party: str  # officer UUID
    to_party: str
    from_facility: str
    to_facility: str
    timestamp: datetime
    seal_uids: list[str]
    photo_hash: str | None
    signed_by_tpm: str | None
    merkle_seq: int

class ConditionEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    asset_uid: str
    timestamp: datetime
    old_state: str
    new_state: str
    technician_id: str
    defect_code: str | None
    notes: str | None
    merkle_seq: int

class LocationEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    asset_uid: str
    timestamp: datetime
    lat: float
    lng: float
    accuracy_m: float
    source: str  # gps | ble | manual | rfid_gate
    signed_by: str  # kit TPM or officer UUID

class IMSTicket(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    asset_uids: list[str]
    severity: str  # P0 | P1 | P2 | P3
    issue_type: str  # missing | tampered | faulty | doa | consumable | environmental
    state: str  # open | assigned | in_progress | resolved | closed
    created_by: str
    assigned_to: str | None
    sla_deadline: datetime
    resolved_at: datetime | None
    resolution_hash: str | None
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/assets` | Role JWT (scoped) | 100/s | Filterable asset list; geographic scoping enforced |
| GET | `/api/v1/assets/{uid}` | Role JWT (scoped) | 200/s | Asset detail with full event chain |
| POST | `/api/v1/assets/{uid}/custody` | Role JWT + biometric | 20/s | Record custody transfer |
| POST | `/api/v1/assets/{uid}/condition` | Role JWT | 20/s | Record condition change |
| POST | `/api/v1/assets/bulk-custody` | Role JWT (Datacraft/IEBC Ops) | 5/s | Batch custody transfer (county cascade) |
| GET | `/api/v1/merkle/root/latest` | None (public) | 1000/s | Current Merkle root + publication confirmations |
| GET | `/api/v1/merkle/events/{after_seq}` | IAB JWT / AG JWT | 50/s | Event batch for escrow replication |
| GET | `/api/v1/tickets` | Role JWT (scoped) | 50/s | Issue ticket list |
| POST | `/api/v1/tickets` | Role JWT | 10/s | Create issue ticket |
| GET | `/api/v1/reserve-pool/status` | Operator JWT | 50/s | Reserve pool by county/tier |
| GET | `/api/v1/public/summary` | None (public) | 500/s | Aggregated kit-status summary |
| GET | `/api/v1/audit/export` | AG JWT / IAB JWT | 2/h | Signed daily event-log extract |

All endpoints: OpenAPI 3.1, mTLS for service-to-service. RBAC enforced at API gateway with geographic jurisdiction binding at data layer.

---

## Integration Points

**Upstream (data sources):**
- Mechrevo factory MES (Suzhou) — birth records, FAT results, component serials (REST/mTLS)
- KRA eCargo (Mombasa port) — import declaration, inspection outcome
- KEPHIS (Kenya Plant Health Inspectorate Service) — phytosanitary import inspection clearance for kit shipments containing plant-based packaging or organic materials; clearance certificate required before customs release; integration via email/PDF initially, API pending KEPHIS digital roadmap
- KEBS PVoC registry — certificate verification per batch
- Al Ghurair ballot printer — ballot lot data, serial ranges, RFID chip manifest (SFTP + PGP)
- DHL / FedEx / G4S / Kenya Airways Cargo — shipment milestone events
- IEBC procurement system — receipt confirmation, payment milestone triggers

**Downstream (consumers):**
- M16 (ECC / E-SOC dashboard) — real-time kit status, geofence alerts (WebSocket push)
- M15 (IAB Dashboard) — read-only event log access
- M19 (Queue Transparency) — kit GPS for station activation confirmation
- M18 (Boundary Transparency) — station geofence polygons (shared service)
- Auditor-General audit endpoint — daily SFTP + PGP extract
- Public transparency dashboard — CDN-cached aggregated county summary

---

## UI/UX — Key Screens

1. **National asset map** — Kenya map; county-level kit-count summary; click county for constituency drill-down; click station for individual kit detail
2. **Asset search** — search by UID, serial, station code, officer name; result shows full event chain timeline
3. **Custody transfer screen** (mobile app) — RFID scan or QR scan; select destination officer; biometric sign; seal UID entry; photo capture; submit
4. **Condition report** (mobile app) — kit selection; condition dropdown; defect code; photo; notes; submit
5. **Reserve pool dashboard** (ECC) — tier breakdown per county; dispatch button with nearest-spare calculation
6. **Ticket inbox** — severity-sorted; SLA countdown timer; assign; update; close
7. **Merkle root status** — five-channel publication status; timestamp; hash; discrepancy highlight if any channel differs
8. **Post-election reconciliation** — all-asset final-state report; missing count (target 0); disposition breakdown (refurbished / donated / destroyed); signed PDF export

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Asset tracking as contractual obligation | IEBC/OIT/01/2026-2027 SoW; OAG recommendation 2024 |
| RFID tracking specified in tender | 2027-preparatory-specs.md §2.4 |
| Chain of custody for evidentiary purposes | Elections Act s.84; Constitution Art. 140 |
| Biometric data erasure on decommission | DPA 2019 s.41 |
| Auditor-General access | Public Finance Management Act s.68 |
| PPADA procurement audit trail | PPADA 2015 s.55 |
| Geographic data residency | DPA 2019 s.18 (no data leaves Kenya) |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Kit tracking coverage (all kits with UID from birth) | 100% | Datacraft / IEBC Procurement |
| Missing kits at T+30 post-election | 0 | Auditor-General |
| Seal-break alerts delivered to ECC | p99 < 5 min | Datacraft SRE |
| Merkle root published every 10 min | 100% of windows during election period | IAB |
| Custody transfer signed-both-parties rate | 100% | IEBC Procurement |
| DOA kit replacement SLA (L1 county spare) | 100% within 2 hours | Datacraft Ops |
| Biometric erasure certificates generated | 100% of decommissioned kits | Datacraft DPO |
| IMS availability during election window | >= 99.99% | Datacraft SRE |

---

## Test Strategy

- **Unit:** State machine transitions, Merkle root computation, geofence polygon containment checks, UID format validation
- **Integration:** Mechrevo MES birth-record ingest; KRA eCargo inspection intake; RFID gate-reader events; M16 ECC push; M15 IAB feed
- **Load:** 65,287 concurrent asset telemetry streams at kit-activation peak; 200 concurrent mobile-app custody transfers
- **Chain integrity:** Automated hourly check that all events hash-chain correctly to current root; simulated chain-break confirms alert fires within 5 minutes
- **Offline resilience:** County edge cache disconnected for 8 hours; events queued; sync on reconnect; zero events lost
- **DPA compliance:** Independent audit of data access logs confirming geographic scoping enforced; no cross-county data leak
- **Penetration test:** IEBC Procurement JWT used to attempt write to a kit outside its jurisdiction; must be blocked at data layer
- **Full-fleet dry run:** T-60 dry-run election: all 65,287 UIDs provisioned and cascade tracked end-to-end

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| IMS deployed to production (Nairobi DC) | 2026-12-01 | Primary + mirror active |
| Mechrevo MES integration live | 2027-01-15 | Birth records flowing from factory |
| Nairobi provisioning centre RFID gates installed | 2027-03-01 | Gate reader test |
| County edge caches deployed (all 47) | 2027-06-01 (T-70) | Offline capability confirmed |
| Kit 1 birth record ingested | 2027-02-01 | First production kit |
| All 65,287 kits provisioned and UIDs assigned | 2027-07-11 (T-30) | Full fleet ready |
| T-30 to T-21 county cascade tracked | 2027-07-11 to 2027-07-20 | Workflow 4 live |
| Reverse logistics tracked (T+1 to T+7) | 2027-08-11 to 2027-08-17 | Workflow 7 live |
| Post-election reconciliation report | 2027-09-10 (T+30) | To Auditor-General |

---

## Dependencies

- **M08** (RTS Device Client) — kit GPS telemetry; activation event
- **M15** (IAB Dashboard) — Merkle event feed
- **M16** (ECC) — real-time alert push; reserve pool dispatch
- **M18** (Boundary Transparency) — station geofence polygons
- **M19** (Queue Transparency) — kit activation confirmation
- **P05** (Audit log) — all IMS events appended
- **P06** (HSM) — Merkle root signing; biometric erasure certificates
- **P04** (IAM) — role-based access, geographic scoping
- External: Mechrevo MES; KRA eCargo; KEBS; Al Ghurair SFTP; shipping carrier APIs; RFID gate hardware (procured separately)

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Mechrevo MES API unavailable for birth records | Mechrevo contractually obligated to provide API access; offline batch alternative (FAT data file) specified as fallback; Datacraft field engineer attends FAI for Batch 1 |
| R2 | RFID tag cloning by sophisticated adversary | Multi-modal redundancy (RFID + barcode + label + photograph); tamper-evident seal adds physical layer; any clone triggers weight and photograph discrepancy |
| R3 | Sea-container door-open event in transit (tampering) | Container-tracker satellite/cellular alert to Datacraft Ops < 5 min; coast-guard and DCI coordination pre-agreed for Mombasa port leg |
| R4 | County edge cache loses events during extended offline | Event queue on device with WAL (write-ahead log); maximum capacity 100,000 events (>> 7 days of county activity); tested in chaos exercise |
| R5 | Petition window: court requests kit custody records within 24 hours | Pre-produced evidentiary extract function in IMS; signed PDF per kit on-demand; IAB co-signs; tested at T-14 tabletop exercise |
