# M65 — Diaspora Ballot Physical Logistics

**Module ID:** M65
**Name:** Diaspora Ballot Physical Logistics
**Owner:** Datacraft Limited — Operations Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Constitution Art. 82(1)(e); Elections Act §44A; M24 (Diaspora Voting) integration boundary

---

## Purpose

Manages the physical return leg of diaspora ballots from overseas embassies/high commissions to the Nairobi tally centre. Distinct from M24 Diaspora Voting, which covers the voting mechanic and electronic result reporting. M65 handles tamper-evident container packing at embassy, customs pre-clearance coordination, diplomatic pouch where applicable, hub-routing, chain of custody at each handoff, and physical arrival at the Nairobi tally centre. Implements Constitution Art. 82(1)(e) physical-return requirement.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| Embassy Election Officer | Write (own post) | Pack containers, initiate shipment, record custody |
| Diplomatic Bag Officer (MFA Kenya) | Write (diplomatic pouch leg) | Confirm pouch manifest, customs clearance |
| Hub Logistics Coordinator (IEBC) | Write (assigned hub) | Receive from multiple posts, consolidate, dispatch to Nairobi |
| Nairobi Tally Centre Officer | Write | Receive and log final delivery |
| IEBC Operations (ECC) | Read-only + alert acknowledge | Real-time shipment tracking |
| IAB Member | Read-only | Audit custody chain |
| Customs Authority (KRA) | Read-only, pre-clearance endpoint | Verify diplomatic manifest |
| Party Agent | Read-only (public summary) | Confirm delivery of ballots from each post |

---

## Functional Requirements

### FR-65.01 — Embassy packing event
After close of poll at each post: Embassy Election Officer scans all completed ballot envelopes, records count, seals tamper-evident container (ISO 17712 H-class security seal), photographs container + seals, records GPS coordinates of packing location. Container UID assigned (format: `KE27-DIA-{POST:5s}-{SEQ:04d}`). Packing event signed by embassy officer + IAB-appointed observer (remote witness via signed confirmation if local IAB observer not present).

### FR-65.02 — Routing determination
Each post assigned a routing policy at T-90: (A) diplomatic pouch via MFA, (B) scheduled airline cargo with security escort, (C) charter flight for high-volume posts (London, Nairobi same-day). M65 resolves the route from post configuration; routing policy approved by IEBC Commissioner and stored immutably before T-30.

### FR-65.03 — Customs pre-clearance
For airline-cargo routes: M65 generates a pre-clearance manifest per container (post, content description, ballot count, diplomatic status), submitted to KRA electronic pre-clearance portal at least 6 hours before estimated arrival. KRA acknowledgement (electronic) recorded against container UID.

### FR-65.04 — Hub receipt and consolidation
Designated regional hubs (Johannesburg, London, Dubai, Washington, Nairobi direct): Hub Coordinator scans container UIDs on arrival, verifies seals intact, records hub receipt event. Consolidation: multiple post containers packed into hub-outbound unit. New container UID assigned to outbound unit with manifest linking all component container UIDs.

### FR-65.05 — Nairobi tally centre receipt
Final delivery: Nairobi Tally Centre Officer scans all container UIDs, breaks seals in view of IAB observer and party agents, photographs seal-break, records receipt event. Ballot envelopes transferred to custody of National Tally returning process.

### FR-65.06 — Real-time shipment tracking
ECC tracks every container UID: current location tier (embassy / in-transit-to-hub / at-hub / in-transit-to-nairobi / delivered). Tracking source: airline cargo API integration (IATA Cargo2000), diplomatic pouch status from MFA API, manual GPS scan at each handoff. ECC dashboard shows ETA per container.

### FR-65.07 — Missed or delayed container alert
Any container not confirmed at hub within 2 hours of expected ETA triggers P1 alert to ECC, IEBC Commissioner, and MFA (if diplomatic route). Any container not confirmed at Nairobi by T+12h triggers P0 escalation including KNPS liaison. Protocol pre-agreed and documented.

### FR-65.08 — Public shipment summary
Publicly accessible summary (no container-content detail): posts dispatched, containers in transit, containers delivered. Updated every 30 minutes from ECC. Surfaced on M11 Public Portal.

### FR-65.09 — Chain of custody Merkle anchoring
All custody events appended to M20 Merkle tree (diaspora sub-tree). Root published with each M20 cycle (every 10 minutes during election window).

---

## Non-Functional Requirements

### NFR-65.01 — Availability
M65 tracking backend: 99.9% during T+0 to T+5. Embassy packing app: offline-capable; events queue and sync on next available connectivity.

### NFR-65.02 — Geographic coverage
50+ posts across 40+ countries. Airline cargo integrations: IATA, DHL, Kenya Airways. Diplomatic pouch: MFA Kenya REST API. All routes covered; no post relies solely on a single carrier.

### NFR-65.03 — Security
Container UIDs, seal numbers, and ballot counts classified IEBC-RESTRICTED. Not disclosed in public API. Embassy packing app: device-level encryption; TPM-signed events.

### NFR-65.04 — DPA 2019
Ballot envelopes themselves are voter-identifying material at embassy level; M65 tracks only container-level data (counts, seal IDs, custody). No voter-identifying data enters M65 records.

### NFR-65.05 — SLA
Hub receipt confirmation: within 2 hours of expected ETA. Nairobi tally centre receipt: within T+5 days for all posts. P1 alert delivery: p99 < 5 minutes.

---

- **NFR-M65-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M65`; alert ruleset ID `iems-M65-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As an Embassy Election Officer, I want to scan all completed ballot envelopes into a tamper-evident container and generate a packing manifest, so that the ballot count is recorded before the container leaves my custody.
2. As a Diplomatic Bag Officer, I want to confirm the pouch manifest in M65 before dispatch, so that the custody chain is unbroken and KRA pre-clearance is triggered automatically.
3. As a Hub Logistics Coordinator, I want to scan arriving containers and verify seals, so that any tampering in transit is flagged before consolidation into the outbound shipment.
4. As the ECC Operations Manager, I want to see a live map of all containers with ETAs, so that I can identify any delayed shipment and activate the contingency protocol.
5. As an IAB Member, I want to see the Merkle-anchored custody chain for each container, so that I can confirm no custody gap between embassy and tally centre.
6. As a Nairobi Tally Centre Officer, I want to scan incoming container UIDs and record seal-break in front of party agents, so that the handoff to the tally process is witnessed and documented.
7. As a Party Agent, I want to see a real-time summary of how many containers have been delivered from each post, so that I can confirm no post's ballots are missing.
8. As a KRA Customs Officer, I want to receive the pre-clearance manifest electronically at least 6 hours before landing, so that I can process diplomatic clearance without holding the shipment at the airport.
9. As an IEBC Commissioner, I want a P0 alert if any container is not confirmed at Nairobi by T+12h, so that I can escalate to KNPS and MFA immediately.
10. As a Civil Society Observer, I want the public summary to show how many posts have dispatched and how many have been received, so that I can independently verify ballot completeness before tally.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime

class Post(BaseModel):
    model_config = ConfigDict(extra='forbid')
    code: str  # 5-char ISO post code
    name: str
    country: str
    routing_policy: str  # diplomatic_pouch | airline_cargo | charter
    hub_code: str  # assigned regional hub
    expected_ballot_count: int

class Container(BaseModel):
    model_config = ConfigDict(extra='forbid')
    uid: str  # KE27-DIA-{POST}-{SEQ}
    post_code: str | None
    parent_uid: str | None  # hub consolidation parent
    seal_uids: list[str]
    ballot_count: int
    current_tier: str  # packed | in_transit_hub | at_hub | in_transit_nairobi | delivered
    routing_policy: str
    packing_photo_hash: str | None
    created_at: datetime

class CustodyEvent65(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    container_uid: str
    event_type: str  # packed | hub_received | hub_dispatched | tally_received | seal_broken
    location_name: str
    officer_id: str
    observer_id: str | None
    seal_status: str  # intact | broken | not_checked
    photo_hash: str | None
    timestamp: datetime
    merkle_seq: int

class ShipmentAlert(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    container_uid: str
    severity: str  # P0 | P1
    alert_type: str  # missed_hub_eta | missed_nairobi_eta | seal_broken | count_mismatch
    triggered_at: datetime
    acknowledged_by: str | None
    acknowledged_at: datetime | None
    resolved_at: datetime | None
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M65 Diaspora Ballot Physical Logistics API
  version: "1.0.0"
paths:
  /api/v1/diaspora-logistics/containers:
    post:
      summary: Create container (embassy packing event)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/Container"}
      responses:
        "201": {description: Container created}
  /api/v1/diaspora-logistics/containers/{uid}/custody:
    post:
      summary: Record custody event
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/CustodyEvent65"}
      responses:
        "201": {description: Event recorded}
  /api/v1/diaspora-logistics/containers/{uid}:
    get:
      summary: Get container status and custody chain
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Container with full custody chain}
  /api/v1/diaspora-logistics/pre-clearance/{uid}:
    post:
      summary: Submit KRA pre-clearance manifest
      security: [{BearerAuth: []}]
      responses:
        "200": {description: KRA acknowledgement}
  /api/v1/diaspora-logistics/public-summary:
    get:
      summary: Public shipment summary (no content detail)
      responses:
        "200": {description: Post-level dispatch and delivery counts}
  /api/v1/diaspora-logistics/alerts:
    get:
      summary: Active alerts (ECC, Commissioner)
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Active alert list}
```

---

## Integrations

**Upstream:**
- M24 (Diaspora Voting) — close-of-poll event triggers packing workflow
- MFA Kenya API — diplomatic pouch manifest and status
- IATA Cargo2000 / Kenya Airways Cargo API — airline shipment milestones
- KRA e-Clearance portal — pre-clearance submission
- Embassy packing mobile app (M65 sub-app, offline-capable) — custody events

**Downstream:**
- M20 (IMS) — Merkle tree append (diaspora sub-tree)
- M16 (ECC) — real-time dashboard, P0/P1 alerts
- M15 (IAB Dashboard) — custody chain audit feed
- M11 (Public Portal) — public summary
- P05 (Audit log), P06 (HSM)

---

## UI/UX — Key Screens

1. **Embassy packing app** — offline-capable; container UID + seal scan; ballot count entry; photo capture; observer confirmation (async remote); EN/FR/SW/AR locale
2. **ECC container map** — world map; container markers by tier; click for custody chain; ETA countdown; P1/P0 highlight
3. **Hub coordinator receipt screen** — container UID scan list; seal-status per container; consolidation UI; outbound manifest generation
4. **Tally centre receipt screen** — container UID scan; seal-break witness workflow; party agent sign-off; final count entry
5. **Public summary widget** — embeddable on M11; posts dispatched / in-transit / delivered; auto-refresh 30 min

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Physical return of diaspora ballots | Constitution Art. 82(1)(e) |
| Diplomatic pouch for ballot security | Vienna Convention on Diplomatic Relations Art. 27 |
| Chain of custody for election materials | Elections Act §84 |
| DPA 2019 — no voter PII in logistics records | DPA 2019 §26 |
| Customs pre-clearance for diplomatic materials | KRA Customs Act §98 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| All posts dispatched within T+24h | 100% | IEBC Commissioner |
| All containers at Nairobi by T+5d | 100% | IEBC Tally Director |
| Seal-intact on arrival at Nairobi | 100% | IAB |
| Custody chain completeness (no gaps) | 100% | IAB |
| P1 alert delivery | p99 < 5 min | Datacraft SRE |
| KRA pre-clearance submitted >= 6h before arrival | 100% | IEBC Legal |

---

## Test Strategy

- **Unit:** Routing policy resolution, ETA calculation, alert trigger logic
- **Integration:** MFA API diplomatic pouch status, IATA Cargo2000 milestone feed, KRA e-Clearance submission, M20 Merkle append
- **Offline:** Embassy app disconnected 24h; events sync confirms no loss
- **Alert:** Simulated missed-hub-ETA; confirms P1 fires within 5 minutes
- **Seal integrity:** Simulated seal-broken event at hub; confirms ECC alert and custody-chain flag
- **Dry run T-60:** Mock diaspora election with 3 test posts; full logistics cycle end-to-end

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Routing policy agreed for all 50+ posts | 2027-04-01 | Commissioner sign-off |
| MFA API integration tested | 2027-05-01 | Diplomatic pouch route live |
| IATA/airline cargo API integration tested | 2027-05-15 | Airline route live |
| KRA pre-clearance integration tested | 2027-06-01 | Customs route live |
| Embassy officer training complete | 2027-07-01 (T-40) | All posts trained |
| Election day (T-0) | 2027-08-10 | Live |

---

## Dependencies

- **M24** (Diaspora Voting) — close-of-poll trigger; voting mechanic
- **M20** (IMS) — Merkle tree
- **M16** (ECC) — alert dashboard
- **P04** (IAM), **P05** (Audit log), **P06** (HSM)
- External: MFA Kenya, IATA Cargo2000, KRA e-Clearance, airline cargo APIs

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Airline cargo embargo or airport closure at post | Charter-flight contingency pre-arranged for 10 highest-volume posts; diplomatic pouch fallback |
| R2 | Customs holds container despite diplomatic status | Pre-clearance submitted >= 6h before arrival; IEBC Legal has MFA-backed rapid-clearance protocol |
| R3 | Seal broken in transit (not tampered) | Seal-break at hub triggers container inspection + re-sealing with new seal UID; IAB observer witness required |
| R4 | Embassy officer unable to access M65 app (internet outage) | Offline packing app with WAL; paper manifest backup; manual data entry at hub on receipt |
| R5 | Hub consolidation loses a post's container in aggregate | Container UID manifest validated before hub-outbound seal; any missing UID blocks hub dispatch |

---

## Cross-References

- **M24** Diaspora Voting — voting mechanic and electronic result reporting (upstream)
- **M20** IMS — shared Merkle tree for custody events
- **M16** ECC — alert routing
- **M15** IAB Dashboard — audit consumer
- **M11** Public Portal — public summary

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| Constitution Art. 82(1)(e) | Physical ballot return for diaspora voters |
| Elections Act §44A | Diaspora ballot security in transit |
| Elections Act §84 | Chain of custody for election materials |
| Vienna Convention Art. 27 | Diplomatic pouch inviolability |

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
