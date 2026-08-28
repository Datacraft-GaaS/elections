# M63 — Ballot Serial Chain

**Module ID:** M63
**Name:** Ballot Serial Chain of Custody
**Owner:** Datacraft Limited — Operations Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part XV §XV.4 Element 2, §XVII.3 Element 3; Elections Act §54

---

## Purpose

Tracks every individual ballot paper from warehouse inclusion to insertion in ballot box or spoilt/replaced/reconciled disposition. Distinct from M20 IMS (tracks kit-level assets) and M39 Ballot Printing Chain (covers printer-to-warehouse leg). M63 begins where M39 ends: the moment a ballot bundle is broken out of warehouse storage and becomes station-assigned. Closes Elections Act §54 spoilt-ballot recovery gap and provides serial-range traceability for petition proceedings.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| Presiding Officer | Own station ballots (write) | Receive serial range, record spoilt/replaced |
| Constituency Returning Officer | Own constituency (write: consolidate) | Reconcile serial ranges across stations |
| IEBC Operations (ECC) | All stations, read-only monitor | Track discrepancy alerts |
| Independent Assurance Board | All, read-only | Audit serial continuity |
| Auditor-General | Batch extract, read-only | Post-election reconciliation |
| Party Agent (via M30) | Own station result, read-only | Verify serial range matches issued count |
| Judicial officer / petitioner | Station subset on order | Evidentiary extract |

---

## Functional Requirements

### FR-63.01 — Serial range assignment
Warehouse manager assigns a contiguous serial range per station at pack-out. Each assignment: station code, serial range `[low, high]`, book count, bundle count, assignment timestamp, assigning officer UUID, and seal UIDs on each bundle. Assignment ingested from M39 ballot manifest via signed API call.

### FR-63.02 — Station receipt confirmation
Presiding Officer scans bundle seals on arrival at station. M63 records: station code, received serial range, condition at receipt (`intact` / `damaged` / `short`), receiving officer UUID, timestamp. Any short or damaged bundle triggers immediate P1 alert to CRO and ECC.

### FR-63.03 — Spoilt ballot workflow (Elections Act §54)
Spoilt ballot events: voter presents spoilt ballot, Presiding Officer records serial number, reason code (ink-bleed / mis-mark / double-mark / voter-request), replacement serial issued. Spoilt ballot placed in separate envelope. Replacement serial must fall within the station's assigned range. Spoilt count included in reconciliation totals.

### FR-63.04 — Unused ballot reconciliation
At close of poll: issued count = authenticated voters. Unused serials = `[low, high]` minus serials deposited and minus spoilt. Any arithmetic mismatch > 0 triggers P1 result-hold flag against that station, surfaced to ECC and CRO.

### FR-63.05 — Ballot box deposition tracking
M63 receives per-ballot deposit events from M20 ballot-box smart-lid RFID reads (FR-20.09). Correlates deposited serial set against station assigned range. Discrepancy threshold: 0 (every deposited serial must be in assigned range).

### FR-63.06 — Reconciliation report (per-station, per-constituency)
At station close: signed reconciliation record containing assigned range, received count, spoilt count (with individual serials), deposited count, unused count, and arithmetic check. CRO counter-signs constituency aggregate. IAB receives signed aggregate within 2 hours of close of poll per station.

### FR-63.07 — Serial chain publication
Constituency-level serial range summaries published as part of M10 results transparency stream. Station-level detail available to party agents (M30) and observers (M34) immediately after CRO signature. Full national serial manifest available for post-election audit download within 24 hours of T+0.

### FR-63.08 — Evidentiary extract
On-demand signed PDF extract per station: serial range, spoilt log, deposited list, reconciliation arithmetic. Pre-formatted for court filing. Available within 2 hours of order via authenticated endpoint (judicial JWT, IAB JWT).

---

## Non-Functional Requirements

### NFR-63.01 — Availability
99.99% during T-3 to T+1. Offline queue on Presiding Officer tablet (M31 device) stores events and syncs when connectivity restored; zero events lost.

### NFR-63.02 — Throughput
55,393 stations, each generating O(700) ballot events on election day. Peak: 38.8 million events in 12-hour window (~900/s sustained, 3,000/s burst). API gateway and time-series ingest tier sized accordingly.

### NFR-63.03 — Integrity
All ballot events append-only, Merkle-anchored (shared M20 Merkle tree, separate subtree). No modification or deletion. Every event signed by Presiding Officer TPM (via M31 device).

### NFR-63.04 — Privacy (DPA 2019)
Serial numbers carry no voter identity. Deposited-serial logs are non-linkable to voter. Access scoped by station/constituency. Judicial extracts require logged-purpose code per DPA 2019 §26.

### NFR-63.05 — WCAG 2.2 AA
Reconciliation report UI (used by Presiding Officer and CRO) meets WCAG 2.2 AA. EN and SW language support throughout.

### NFR-63.06 — SLA
Reconciliation report delivery to IAB: p99 < 2 hours post-station-close. Evidentiary extract: p99 < 2 hours on authenticated request.

---

- **NFR-M63-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M63`; alert ruleset ID `iems-M63-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As a Presiding Officer, I want to scan bundle seals on arrival and have M63 confirm my serial range, so that I know exactly which ballot serials I am responsible for.
2. As a Presiding Officer, I want to record a spoilt ballot serial and the voter's replacement serial in one step, so that my reconciliation is complete and Elections Act §54 is satisfied.
3. As a Constituency Returning Officer, I want to see a real-time tally of reconciled vs pending stations in my constituency, so that I can chase outstanding stations before the close of count.
4. As an IAB Member, I want to receive signed reconciliation records within 2 hours of each station closing, so that I can raise discrepancies before results are declared.
5. As a Party Agent (M30), I want to verify that the serial range assigned to my station matches the count on the reconciliation form, so that I can raise an objection if there is a mismatch.
6. As an ECC Operations Manager, I want a P1 alert when any station's deposited count diverges from its authenticated voter count, so that I can hold that station's result pending investigation.
7. As a judicial officer, I want a signed PDF extract of all ballot events for a specific station within 2 hours of my order, so that petition proceedings can proceed on verified facts.
8. As the Auditor-General, I want the national serial manifest available within 24 hours of election day, so that I can verify total ballot accountability for the annual audit.
9. As an IEBC Commissioner, I want the national unused-ballot count to equal the difference between total assigned and total authenticated voters, so that I can demonstrate zero unaccounted ballots.
10. As a Civil Society Observer, I want constituency-level serial range summaries published openly, so that I can independently verify that ballot counts are arithmetically consistent.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime

class SerialRangeAssignment(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    serial_low: int
    serial_high: int
    book_count: int
    bundle_count: int
    assigned_at: datetime
    assigning_officer_id: str
    seal_uids: list[str]
    m39_manifest_ref: str  # reference to M39 ballot manifest record

class StationReceipt(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    assignment_id: str
    station_code: str
    received_at: datetime
    receiving_officer_id: str
    condition: str  # intact | damaged | short
    actual_count: int
    damaged_serials: list[int]
    officer_signature: str  # TPM-signed hash

class SpoiltBallotEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    spoilt_serial: int
    reason: str  # ink_bleed | mis_mark | double_mark | voter_request
    replacement_serial: int | None
    recorded_at: datetime
    presiding_officer_id: str

class DepositEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    serial: int
    deposited_at: datetime
    source: str  # rfid_smartlid | manual
    merkle_seq: int

class ReconciliationRecord(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    assignment_id: str
    assigned_count: int
    received_count: int
    spoilt_count: int
    deposited_count: int
    unused_count: int
    arithmetic_ok: bool
    discrepancy: int  # signed; 0 = clean
    po_signature: str
    cro_signature: str | None
    signed_at: datetime
    cro_signed_at: datetime | None
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M63 Ballot Serial Chain API
  version: "1.0.0"
paths:
  /api/v1/ballot-serial/assignments/{station_code}:
    get:
      summary: Get serial range assignment for a station
      security: [{BearerAuth: []}]
      parameters:
        - name: station_code
          in: path
          required: true
          schema: {type: string}
      responses:
        "200": {description: SerialRangeAssignment}
  /api/v1/ballot-serial/receipt:
    post:
      summary: Record station receipt of ballot bundle
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/StationReceipt"}
      responses:
        "201": {description: Receipt recorded}
  /api/v1/ballot-serial/spoilt:
    post:
      summary: Record spoilt ballot event
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/SpoiltBallotEvent"}
      responses:
        "201": {description: Spoilt event recorded}
  /api/v1/ballot-serial/reconciliation/{station_code}:
    get:
      summary: Get reconciliation record for a station
      security: [{BearerAuth: []}]
      parameters:
        - name: station_code
          in: path
          required: true
          schema: {type: string}
      responses:
        "200": {description: ReconciliationRecord}
    post:
      summary: Submit signed reconciliation record
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/ReconciliationRecord"}
      responses:
        "201": {description: Reconciliation submitted}
  /api/v1/ballot-serial/evidentiary-extract/{station_code}:
    get:
      summary: Signed PDF evidentiary extract for a station
      security: [{BearerAuth: []}]
      parameters:
        - name: station_code
          in: path
          required: true
          schema: {type: string}
        - name: purpose_code
          in: query
          required: true
          schema: {type: string}
      responses:
        "200":
          description: Signed PDF
          content:
            application/pdf:
              schema: {type: string, format: binary}
  /api/v1/ballot-serial/national-manifest:
    get:
      summary: National serial manifest (post-election)
      security: [{BearerAuth: []}]
      responses:
        "200": {description: National manifest JSON}
```

---

## Integrations

**Upstream:**
- M39 (Ballot Printing Chain) — initial serial range manifest, bundle-seal UIDs
- M20 (IMS) — ballot-box smart-lid deposit events (FR-20.09)
- M31 (Mobile Poll Clerk) — spoilt-ballot and receipt events from tablet
- M08 (RTS Device Client) — authenticated-voter count cross-reference
- P06 (HSM) — Presiding Officer and CRO signature verification

**Downstream:**
- M10 (RTS Back-End) — result hold flag when discrepancy detected
- M15 (IAB Dashboard) — reconciliation status feed
- M16 (ECC) — P1 discrepancy alerts
- M30 (Mobile Party Agent) — serial range read access
- M34 (Mobile Observer) — constituency-level summary
- P05 (Audit log) — all events

---

## UI/UX — Key Screens

1. **PO receipt screen** (M31 tablet) — scan seal UIDs, confirm serial range, flag damage; EN/SW
2. **Spoilt ballot entry** (M31 tablet) — serial lookup, reason selection, replacement issue; single-screen workflow; large touch targets (accessibility)
3. **Station reconciliation dashboard** (M31 + web) — live arithmetic check, traffic-light status; submit button disabled until arithmetic balances
4. **CRO constituency overview** (web) — station grid with reconciliation status; drill-in to any station; counter-sign aggregate
5. **ECC discrepancy feed** (M16 integration) — P1 alerts with station detail and suggested action
6. **National manifest download** (public portal) — available T+24h; signed SHA-256

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Spoilt ballot recovery procedure | Elections Act §54 |
| Chain of custody for evidentiary purposes | Elections Act §84; Constitution Art. 140 |
| Ballot accountability | Strategy §XV.4 Element 2, §XVII.3 Element 3 |
| Purpose limitation | DPA 2019 §26 |
| Records retention | Elections Act §44 (7 years) |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Stations with complete reconciliation by T+2h | >= 99.9% | IEBC Operations |
| Arithmetic discrepancy count post-election | 0 | Auditor-General |
| IAB reconciliation record delivery | p99 < 2h per station | IAB |
| Evidentiary extract delivery | p99 < 2h on order | IEBC Legal |
| National manifest availability | T+24h | IEBC ICT |
| Spoilt ballot serial recorded per §54 | 100% | IEBC Operations |

---

## Test Strategy

- **Unit:** Serial range arithmetic, spoilt-ballot state machine, reconciliation calculation
- **Integration:** M39 manifest ingest, M20 smart-lid deposit feed, M31 tablet event sync, M10 result-hold flag trigger
- **Load:** 55,393 concurrent reconciliation submissions at close of poll; p99 < 5s
- **Offline resilience:** M31 tablet disconnected 4h; events queued; sync confirms zero loss
- **Chain integrity:** Merkle sub-tree verified hourly; simulated break confirms P0 alert within 5 minutes
- **Adversarial:** Attempt to post deposit event with out-of-range serial; must be rejected at API layer

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| M39 integration spec agreed | 2027-01-15 | Serial manifest API contract signed |
| M31 tablet PO workflow built | 2027-03-01 | UI acceptance |
| Dry-run election (T-60) | 2027-06-10 | Full reconciliation cycle tested |
| Smart-lid RFID integration live | 2027-07-01 (T-40) | Deposit event feed confirmed |
| CRO training complete | 2027-07-25 (T-16) | All 290 CROs signed off |
| Election day (T-0) | 2027-08-10 | Live |

---

## Dependencies

- **M20** (IMS) — smart-lid deposit events
- **M31** (Mobile Poll Clerk) — event capture device
- **M39** (Ballot Printing Chain) — serial manifest (M35-M62 range; do not modify)
- **M10** (RTS Back-End) — result-hold trigger
- **M15** (IAB Dashboard) — reconciliation feed
- **M16** (ECC) — alert routing
- **P05** (Audit log), **P06** (HSM), **P04** (IAM)

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | M31 tablet offline during reconciliation | Offline queue with WAL; events sync on reconnect; PO can complete reconciliation offline and sync later |
| R2 | Smart-lid RFID chip on ballot fails to read | Manual serial entry fallback on M31; barcode scan backup; any manual entry flagged for CRO review |
| R3 | PO records wrong serial for spoilt ballot | Validation: spoilt serial must fall within assigned range; replacement must also be in range; both confirmed before commit |
| R4 | Petition forces rapid evidentiary extract at scale | Pre-generation of station-level signed PDFs at T+4h; judiciary endpoint load-tested at 100 concurrent downloads |
| R5 | M39 serial range arrives with gaps or overlaps | M63 ingestion validates range contiguity across all stations before election day; any gap triggers P0 alert to IEBC procurement |

---

## Cross-References

- **M20** IMS — kit-level chain of custody (upstream of this module for smart-lid events)
- **M39** Ballot Printing Chain — printer-to-warehouse leg; M63 begins at warehouse break-out
- **M10** RTS Back-End — result hold trigger on discrepancy
- **M15** IAB Dashboard — audit consumer
- **M16** ECC — alert routing
- **M30** Mobile Party Agent — serial range read access
- **M31** Mobile Poll Clerk — event capture
- **M34** Mobile Observer — summary read access

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| §XV.4 Element 2 | Ballot accountability at individual serial level |
| §XVII.3 Element 3 | Reconciliation record required per station |
| Elections Act §54 | Spoilt ballot recovery and serial logging |
| Elections Act §84 | Chain of custody for evidentiary use |
| Constitution Art. 140 | Petition evidentiary standard |

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
