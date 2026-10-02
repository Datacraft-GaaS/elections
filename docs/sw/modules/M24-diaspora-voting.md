# M24 — Diaspora Voting Module

**Module ID:** M24
**Name:** Diaspora Voting Architecture (Embassy-Hosted BVR/BVI + Chain of Custody)
**Runs on:** Embassy-hosted KIEMS kits + central diaspora aggregation backend (Kenyan cloud)
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Extend BVR and BVI workflows to embassy-hosted environments for Kenyan citizens outside Kenya, in compliance with Constitution Art. 82(1)(e) and Elections Act s.5. Provide a distinct architecture handling delayed-batch transmission, embassy-staff-operated kit administration, cross-border chain of custody for physical ballots, and embassy-specific voter roll management. Integrate diaspora results into national aggregation without disrupting the mainline pipeline.

Strategy reference: §XXI.A.2 / Constitution Art. 82(1)(e).

Primary legal basis: Constitution Art. 82(1)(e) (Parliament may provide for voting of citizens not resident in Kenya); Katiba Institute v the Commission [2020] eKLR (judicially mandated expansion of diaspora registration); Nyoro & 2 Others v the Commission [2017] eKLR.

## 2. User Personas

- **Diaspora Voter** — registered Kenyan citizen at an embassy; participates in BVR and BVI
- **Embassy Presiding Officer** — Commission-appointed; operates the kit; no diplomatic-service line of command
- **Embassy Party Agent** — one per registered party; witnesses all procedures
- **the Commission Diaspora Operations Officer** — coordinates schedule, kit dispatch, and transmission across missions
- **IAB Diaspora Representative** — IAB-appointed auditor per mission cluster; observes process
- **Returning Officer (Diaspora)** — the Commission officer responsible for diaspora Form 34A and aggregation into Form 34B
- **Foreign Ministry Liaison** — facilitates diplomatic clearances; no operational authority over kit

## 3. Functional Requirements

### Registration (BVR — Diaspora Mode)

- **FR-M24-001** Kit provisioned in DIASPORA_BVR mode: voter roll scoped to embassy-jurisdiction-registered Kenyans only; restricted by embassy code and diplomatic-mission country code.
- **FR-M24-002** Capture same 22 biographic fields as M01; additionally capture: host-country city of residence, Kenyan ID or Passport number (mandatory), embassy registration date.
- **FR-M24-003** Biometric capture identical to M01 (10 fingerprints ISO/IEC 19794-2, iris ISO/IEC 19794-6, face ICAO 9303). All biometric data stored encrypted on-device; synced to central ABIS via encrypted delayed batch — not streamed live given embassy connectivity.
- **FR-M24-004** On-device dedup against embassy-local cache; full ABIS dedup via delayed sync; dedup result published before the voter roll is frozen for that embassy.
- **FR-M24-005** Embassy registration window enforced: kit refuses new BVR enrolments outside the Commission-gazetted diaspora registration window. Window dates are gazetted separately from domestic window.
- **FR-M24-006** Voter roll freeze: 14 days before diaspora voting day per mission, the embassy roll is frozen and a signed snapshot published to the IAB and the public portal.
- **FR-M24-007** Multi-language: EN + SW baseline; local-jurisdiction language as an additional display option (Arabic for Middle East missions, French for Francophone Africa missions, etc.) for voter-facing screens only.

### Voting (BVI — Diaspora Mode)

- **FR-M24-008** Kit provisioned in DIASPORA_BVI mode with the frozen embassy roll. BVI workflow identical to M05 except: no live central ABIS sync required (embassy connectivity may be intermittent); double-vote check performed against the embassy-local signed roll snapshot.
- **FR-M24-009** Delayed transmission: voting results (Form 34A equivalent — designated "Form 34A-D") stored in an encrypted signed queue on-device. Transmission scheduled within the Commission-published per-embassy transmission window (published 30 days before voting day).
- **FR-M24-010** Transmission window: if embassy connectivity does not permit transmission within the window, kit generates a DELAYED_TRANSMISSION event; data retained on device under continuous party-agent custody; retransmission attempted every 30 minutes until window closes (maximum 6 hours after poll close).
- **FR-M24-011** Physical ballot fallback: if electronic transmission fails within 6 hours of poll close, physical ballots are sealed in a tamper-evident pouch (RFID-tagged per M10 chain of custody). Pouch dispatched to Nairobi under the Commission observer escort. Embassy Presiding Officer and party agents sign a custody certificate. Central receive confirms physical ballot arrival and reconciles against Form 34A-D.
- **FR-M24-012** Cross-border chain of custody: physical-ballot pouches tracked via the same RFID system as domestic ballot boxes (M10). Custody-change events at embassy dispatch, transit waypoints, JKIA arrival, bonded warehouse, and Nairobi counting centre — all appended to the Merkle chain.
- **FR-M24-013** Time zone handling: kit runs on UTC; all timestamps converted to EAT for display to the Commission officers and to local time for display to voters. Transmission window published in both UTC and local time for each embassy.
- **FR-M24-014** Embassy kit assignment: each kit assigned to exactly one embassy code; cannot operate at a different embassy; geo-fence enforced at embassy GPS coordinates with 5 km tolerance (embassies may use external venues for large-volume voting days).
- **FR-M24-015** Form 34A-D publication: transmitted digitally-signed Form 34A-D published on the public portal within 30 minutes of central receipt. Physical-ballot Form 34A-D published within 24 hours of Nairobi receipt.
- **FR-M24-016** Diaspora result integration: diaspora Form 34A-Ds aggregated into a dedicated "Diaspora Constituency" in the national aggregation pipeline. Presidential contest only — diaspora voters do not vote in constituency contests per current Elections Act framework.
- **FR-M24-017** Embassy staffing language: all the Commission instructions to embassy presiding officers published in EN + SW. Embassy-country liaison language at the Commission discretion.

## 4. Non-Functional Requirements

- **NFR-M24-001** Kit operates fully offline for up to 24 hours; BVR and BVI workflows unaffected by network loss.
- **NFR-M24-002** Encrypted batch sync: batch transmitted over AES-256-encrypted channel with mTLS; certificate pinning to Kenyan cloud endpoint.
- **NFR-M24-003** Physical-ballot pouch RFID tracked; any chain-of-custody gap >= 6 hours triggers an automatic IAB alert.
- **NFR-M24-004** Diaspora voter roll frozen state reproducible from Merkle root; any subsequent modification detectable.
- **NFR-M24-005** WCAG 2.2 AA for all voter-facing screens; audio guidance available.
- **NFR-M24-006** Multi-language voter-facing: EN + SW + local-jurisdiction language where configured.
- **NFR-M24-007** DPA 2019 compliance: biometric data of diaspora voters processed under same framework as domestic voters (DPA 2019 s.26); cross-border data transfer to Kenyan ABIS falls within the definition of a transfer to a Kenyan controller — no third-country adequacy required.
- **NFR-M24-008** Availability of diaspora aggregation backend: >= 99.9% during per-embassy transmission window.
- **NFR-M24-009** Diaspora Form 34A-D submission to public portal: <= 30 minutes of digital receipt; <= 24 hours of physical ballot receipt.
- **NFR-M24-010** Kit provisioning for up to 60 diplomatic missions; post-award specification exercise confirms final kit count per mission (current planning estimate: 40 active missions).

## 5. User Stories

1. As a diaspora voter in London, I register my biometrics at the Kenyan High Commission in a BVR session, receive a registration confirmation, and know I am on the embassy voter roll.
2. As a diaspora voter in Dubai, I vote on election day at the embassy; my ballot is accepted via biometric authentication against the frozen embassy roll even if the internet is down.
3. As an Embassy Presiding Officer, I follow the same kit workflow as a domestic presiding officer, with embassy-specific configuration; I do not need to understand the underlying diaspora-mode architecture.
4. As a party agent at the Nairobi count centre, I can verify that each physical pouch arrived with an unbroken RFID chain of custody from the originating embassy.
5. As a member of the public, I can verify the diaspora Form 34A-D for each embassy on the public portal, with its Merkle path and transmission timestamp.
6. As an IAB Diaspora Representative, I receive a custody-chain event log for every embassy's ballot pouch in real time, so I can flag any gap >= 6 hours immediately.
7. As a Commission Diaspora Operations Officer, I see a live dashboard of all 40 missions with transmission status (transmitted / pending / delayed / physical-ballot-only) and can trigger manual escalation.
8. As a diaspora voter in Paris, I can choose French as the display language for voter-facing screens while all official records are maintained in English.
9. As a Returning Officer (Diaspora), I aggregate all embassy Form 34A-Ds into the Diaspora constituency Form 34B within 6 hours of the last embassy's transmission window close.
10. As a Supreme Court petitioner, I can download the Diaspora constituency Form 34B with Merkle proof and verify it against the published national Form 34C.

## 6. Data Model Additions

```python
class EmbassyStation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    embassy_code: str               # the Commission internal code, e.g. "GBR-LHR-001"
    host_country_iso3: str
    host_city: str
    latitude: float
    longitude: float
    geofence_radius_km: float       # default 5.0
    languages: list[str]            # BCP-47; EN + SW mandatory
    kit_ids: list[str]              # provisioned kit serial numbers
    bvr_window_start: datetime
    bvr_window_end: datetime
    bvi_date: date
    transmission_window_start: datetime
    transmission_window_end: datetime
    presiding_officer_id: str | None
    status: Literal["CONFIGURED","BVR_OPEN","BVR_CLOSED","ROLL_FROZEN","VOTING","TRANSMITTING","CLOSED"]

class DiasporaVoterRoll(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    embassy_code: str
    roll_version: int
    frozen_at: datetime | None
    voter_count: int
    roll_hash: str                  # SHA-256 of the signed voter-roll snapshot
    merkle_inclusion_proof: str | None
    published_at: datetime | None

class DiasporaForm34AD(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    embassy_code: str
    contest_id: str                 # presidential only
    registered_voters: int
    ballots_issued: int
    ballots_cast: int
    rejected_ballots: int
    candidate_tallies: dict[str, int]   # candidate_id -> vote_count
    transmission_mode: Literal["DIGITAL","PHYSICAL_BALLOT"]
    transmitted_at: datetime | None
    physical_pouch_rfid: str | None
    nairobi_receipt_at: datetime | None
    presiding_officer_signed: bool
    party_agent_signatures: list[str]   # officer ids
    form_hash: str
    merkle_node_id: str | None
    published_at: datetime | None

class BallotPouchCustody(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    pouch_rfid: str
    embassy_code: str
    event_type: Literal[
        "SEALED","EMBASSY_DISPATCH","TRANSIT","JKIA_ARRIVAL",
        "WAREHOUSE_INTAKE","COUNT_CENTRE_RECEIPT"
    ]
    custodian_id: str               # the Commission officer or escort
    party_agent_witness_id: str | None
    location_description: str
    latitude: float | None
    longitude: float | None
    event_at: datetime
    merkle_node_id: str | None
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| GET | /v1/diaspora/embassies | COMMISSION_ADMIN JWT | 50/hr | List all embassy stations |
| GET | /v1/diaspora/embassies/{code} | COMMISSION_ADMIN JWT | 100/hr | Embassy config and status |
| GET | /v1/diaspora/embassies/{code}/roll | COMMISSION_ADMIN JWT | 50/hr | Current voter roll snapshot |
| POST | /v1/diaspora/embassies/{code}/roll/freeze | COMMISSION_CHAIR HSM | 1/embassy | Freeze and publish embassy voter roll |
| POST | /v1/diaspora/forms/34ad | KIT_CERT mTLS | 100/hr | Transmit Form 34A-D from kit |
| GET | /v1/diaspora/forms/34ad | Public | 500/min | List all published diaspora forms |
| GET | /v1/diaspora/forms/34ad/{id} | Public | 500/min | Single Form 34A-D with Merkle proof |
| POST | /v1/diaspora/pouch-custody | RFID_SCANNER_CERT | 500/hr | Record pouch custody event |
| GET | /v1/diaspora/pouch-custody/{rfid} | COMMISSION_ADMIN JWT | 100/hr | Full custody chain for one pouch |
| GET | /v1/diaspora/dashboard | COMMISSION_OPS JWT | 200/hr | Live mission status dashboard |
| POST | /v1/diaspora/alert/custody-gap | SYSTEM | Internal | Auto-alert on >= 6-hour custody gap |

## 8. Integration Points

- **M01 BVR Device / M02 BVR Back-End** — diaspora BVR uses the same device software in DIASPORA mode; biometric data syncs to central ABIS via delayed batch
- **M05 BVI Device / M06 BVI Back-End** — diaspora BVI similarly uses the same device software in DIASPORA mode
- **M03 ABIS** — biometric dedup for diaspora registrations
- **M10 RTS Back-End** — diaspora Form 34A-D ingested and aggregated into the Diaspora constituency
- **P05 Merkle Chain** — all diaspora events anchored
- **P06 HSM** — presiding officer signing; the Commission Chair roll-freeze signing
- **M11 Public Portal** — diaspora Form 34A-D publication and embassy status dashboard
- **Foreign Ministry** (external, diplomatic clearance) — kit export authorisation and embassy access
- **RFID tracking system** (M10 shared) — physical-ballot pouch tracking

## 9. UI/UX Key Screens

1. **Embassy Operations Dashboard** — list of all missions; status badge per mission (green/amber/red); transmission-window countdown; transmission mode indicator (digital/physical/delayed); one-click escalation to the Commission Ops Officer.
2. **Diaspora BVR Wizard (on-kit)** — identical to M01 registration UX with embassy-specific header; language toggle with local-jurisdiction language option; confirmation screen shows embassy code prominently.
3. **Voter Roll Freeze Screen** — the Commission Chair view; shows roll count, pending dedup results, freeze timestamp; requires HSM card tap to execute.
4. **Diaspora BVI Flow (on-kit)** — identical to M05 BVI UX in diaspora mode; shows "offline mode — voting will be transmitted within [window close time]" if no connectivity.
5. **Physical Pouch Custody Screen** — RFID scanner interface; custody event type selector; party-agent witness QR scan; automatic GPS capture.
6. **Diaspora Form 34A-D Public View** — per-embassy result card with candidate tallies, transmission mode, timestamp, Merkle proof link; identical styling to domestic Form 34A view.
7. **Diaspora Constituency Aggregation** — Returning Officer view; all embassy Form 34A-Ds with acceptance status; aggregate Form 34B for Diaspora constituency; HSM-sign button.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Authority to extend voting rights to citizens outside Kenya | Constitution Art. 82(1)(e) |
| Right to vote | Constitution Art. 38; Elections Act s.5 |
| Judicially mandated diaspora expansion | Katiba Institute v the Commission [2020] eKLR |
| Kit operation outside Kenya | Election Technology Regulations 2017; ECOWAS/AU observer protocols |
| Physical ballot transport | Elections Act s.73 (ballot security); Customs Act for air freight |
| Biometric data cross-border | DPA 2019 s.26; transfer to Kenyan controller within Kenyan jurisdiction |
| Chain of custody | Elections Act s.39; Election Offences Act 2016 s.3 |
| Form 34A-D publication | Constitution Art. 86(a) — transparent and verifiable method |
| Observer access at embassy | Elections Act s.21A; bilateral agreements with host countries |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Embassy missions with digital transmission within window | >= 90% | Transmission log | the Commission Diaspora Ops |
| Physical pouch chain-of-custody gaps >= 6 hours | 0 | RFID log | IAB Diaspora Representative |
| Diaspora Form 34A-D published within 30 minutes of digital receipt | 100% | Timestamp delta | Datacraft Technical |
| Physical-ballot Form 34A-D published within 24 hours of Nairobi receipt | 100% | Timestamp delta | Returning Officer (Diaspora) |
| Diaspora voter roll frozen 14 days before voting day | 100% of missions | Freeze timestamp | the Commission Chair |
| Diaspora result integrated into national Form 34C | 100% | Aggregation log | Returning Officer |
| Kit offline operation across full voting day (8 hours) | 100% | Battery + storage test | Mechrevo QA |

## 12. Test Strategy

- **Unit:** diaspora mode kit config flags; transmission-window enforcement; Form 34A-D structure validation; custody-event schema.
- **Integration:** end-to-end diaspora BVR + BVI cycle with simulated embassy kit; delayed-batch transmission after connectivity restore; physical-pouch RFID scan chain.
- **E2E:** full simulated diaspora mission: registration, roll freeze, voting, digital transmission, portal publication.
- **Offline:** 24-hour full-offline BVI session; verify data integrity and transmission on reconnect.
- **Security:** confirm diaspora kit cannot activate outside configured geofence; confirm roll cannot be modified post-freeze; mTLS certificate rotation test.
- **Accessibility:** WCAG 2.2 AA on voter-facing kit screens and public portal.
- **Load:** 40 simultaneous embassy transmission events; confirm aggregation pipeline handles concurrent Form 34A-D ingestion.
- **Chaos:** embassy network failure during transmission window; physical-pouch RFID scanner failure at JKIA.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Embassy specification exercise complete | 2026-12-15 | Per-mission kit count, languages, geofence coordinates |
| Foreign Ministry diplomatic clearance framework agreed | 2027-01-15 | Commission-Foreign Ministry MoU |
| Kit provisioning for 40 missions | 2027-03-01 | Hardware shipped; customs cleared per mission |
| Embassy presiding-officer training | 2027-04-01 | Remote + in-person per cluster |
| Diaspora BVR window opens | 2027-04-15 | Per mission; registration closes 2027-06-15 |
| Roll freeze | 2027-07-26 | T-14 from diaspora voting day |
| Diaspora voting day | 2027-08-09 | T-1 before national election day |
| Transmission window | 2027-08-09 to 2027-08-10 18:00 EAT | Digital and physical-ballot routes |
| Diaspora Form 34A-D published | 2027-08-10 | Integrated into national tally |

## 14. Dependencies

- M01 BVR Device / M02 BVR Back-End — shared code base with diaspora mode flag
- M05 BVI Device / M06 BVI Back-End — shared code base with diaspora mode flag
- M03 ABIS — biometric dedup for diaspora
- M10 RTS Back-End — Form 34A-D aggregation
- Foreign Ministry MoU — diplomatic clearance per mission (external; Commission-facilitated)
- RFID tracking (M10 shared infrastructure) — physical ballot chain of custody
- Customs clearance per country — kit export; Datacraft handles documentation, the Commission provides diplomatic note

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Host country refuses entry to KIEMS kits (import restrictions) | Low-Medium | High | Pre-cleared diplomatic note from Kenyan Ministry of Foreign Affairs; kit classified as diplomatic equipment; contingency: voting at Kenyan diplomatic premises under Vienna Convention exemption |
| Embassy connectivity insufficient for digital transmission in window | Medium | Medium | 6-hour retry; physical-ballot fallback is a first-class pathway; presiding officer trained on both workflows |
| Low diaspora voter registration (less than 5,000 nationally) | High | Low (scale) | Architecture scales from 100 to 500,000 voters without change; small diaspora volume does not affect national result materially |
| Physical ballot pouch lost or tampered in transit | Low | High | RFID tracking + the Commission observer escort + party-agent custody certificate + air-freight with IATA chain-of-custody requirements; any gap triggers IAB alert and a DCI report |
| Host country legal challenge to in-country voting by Kenyan embassy | Low | Medium | Operations conducted on embassy premises (Kenyan sovereign territory under Vienna Convention Art. 22); legal opinion obtained per mission cluster pre-deployment |
