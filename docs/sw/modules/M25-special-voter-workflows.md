# M25 — Special Voter Workflows

**Module ID:** M25
**Name:** Special Voter Category Workflows (Prisoners, Hospital, KDF/GSU, Refugees, IDPs)
**Runs on:** Kit-side (special-configuration mode) + central backend
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Provide legally compliant, auditable registration and voting workflows for five special voter categories that cannot use standard polling-station procedures: remand prisoners (Constitution Art. 83), hospital and infirmary voters (Elections Act s.14), KDF/GSU/Prison Service voters (Elections Act s.5), refugees (Refugees Act 2021 — activation pending legal clarity), and internally displaced persons. Each category has a dedicated kit mode, signed-witness-chain protocol, and post-event audit sample.

Strategy reference: §XXI.A.3.

## 2. User Personas

- **Remand Prisoner (Category A voter)** — at-risk voter; right protected under Constitution Art. 83 and Kituo cha Sheria v AG [2013] eKLR
- **Prison Governor / Superintendent** — acts as de facto presiding officer at remand facility
- **Hospital Patient (Category B voter)** — eligible voter unable to travel to polling station
- **Attending Clinician / Hospital Administrator** — impartial witness; not a family member
- **KDF / GSU Service Member (Category C voter)** — votes at designated barracks station
- **Commission-Seconded Returning Officer (Category C)** — civilian-controlled; runs the barracks kit
- **Refugee (Category D voter, activation conditional)** — registered per Refugees Act 2021 and any court-ordered eligibility
- **IDP (Category E voter)** — registered at IDP-designated station near displacement site
- **the Commission Special Voter Coordinator** — central oversight of all special-category operations
- **IAB Special Voter Auditor** — audits a sample of special-category events post-election
- **Party Agent (Special Category)** — one per party per facility; witnesses procedures

## 3. Functional Requirements

### Category A — Remand Prisoners

- **FR-M25-001** Kit provisioned in REMAND_PRISON mode: voter roll scoped to the registered prison's remand population (pre-trial detainees only; convicted prisoners currently excluded per Elections Act unless a court order extends the right). Roll sourced from Kenya Prisons Service CRS cross-reference under MoU.
- **FR-M25-002** Prison Governor acts as Presiding Officer; Commission-assigned deputy Presiding Officer present; kit cannot open without dual biometric sign-in of both.
- **FR-M25-003** Party-agent complement reduced per the Commission regulation: minimum one party agent for the two leading presidential candidates; additional agents accommodated where facility permits.
- **FR-M25-004** Prisoner identity verification: fingerprint match against ABIS (same threshold as standard BVI); if fingerprints degraded (documented occupational damage), fallback to iris + face combination with Presiding Officer signed authorisation.
- **FR-M25-005** Physical security protocol: kit arrives sealed with RFID tag; custody log shows prison gate entry; party agents sign custody certificate at gate.
- **FR-M25-006** Voting schedule: published per facility in Kenya Gazette >= 14 days before election day.
- **FR-M25-007** Results transmission: same pipeline as domestic station; Form 34A-R (remand variant) published within 30 minutes of close.
- **FR-M25-008** Post-vote audit sample: IAB auditor reviews a random 5% sample of remand-prisoner authentication logs within T+7.

### Category B — Hospital / Infirmary Voters

- **FR-M25-009** Mobile KIEMS unit: tablet + portable fingerprint scanner + portable iris scanner + battery pack; same OS and firmware as standard kit; provisioned in HOSPITAL mode.
- **FR-M25-010** Hospital voter roll: derived from standard voter roll by cross-referencing patients confirmed as registered voters by the hospital administration; approved by the relevant county returning officer.
- **FR-M25-011** Consent and witness protocol per Elections Act s.72: patient must give informed consent; attending clinician (or hospital administrator if clinician unavailable) signs as impartial witness; party agent of patient's choice may be present.
- **FR-M25-012** Physical mobility accommodation: if patient cannot present biometrics in the standard way (limb injury, sedation, cognitive impairment), the Presiding Officer may authorise the complementary mechanism per Elections Act s.44A with reason code MEDICAL_INABILITY; dual party-agent witness required.
- **FR-M25-013** Assisted voting per Elections Act s.72: patient may appoint an assistant of their own choice; assistant must not be a candidate or party agent; kit logs assistant identity and patient consent.
- **FR-M25-014** Hospital kit operates offline; syncs Form 34A-H on return to base station.
- **FR-M25-015** Hospital voter list published (anonymised — patient names not published; voter count per facility published) in Kenya Gazette at T-14.

### Category C — KDF / GSU / Prison Service Voters

- **FR-M25-016** Kit provisioned in BARRACKS mode: voter roll scoped to service-personnel registered at the barracks station under Elections Act s.5 special enrolment.
- **FR-M25-017** Civilian control: Commission-seconded Returning Officer acts as Presiding Officer; no officer in the military chain of command has authority over the kit or the vote. Kit's Presiding Officer biometric must match the Commission-seconded officer, not any military credential.
- **FR-M25-018** Ballot secrecy enforcement: physical booth placed away from line-of-sight of any superior officer; party agents present as witnesses; any officer who approaches the booth is logged as a violation event.
- **FR-M25-019** Special enrolment window: service-personnel registered during the barracks-specific BVR window (Elections Act s.5); roll compiled by the Commission from commanding officer's approved list; the Commission does not delegate roll compilation authority to military.
- **FR-M25-020** Results transmission: Form 34A-B (barracks variant) published within 30 minutes of close.
- **FR-M25-021** Post-vote audit sample: IAB auditor reviews a 5% random sample of barracks authentication logs at T+7; any ballot-secrecy violation events reviewed at 100%.

### Category D — Refugees (Conditional Activation)

- **FR-M25-022** Module built but activation-gated behind a configuration flag: REFUGEE_VOTING_ENABLED (default: false). Datacraft monitors litigation and court orders; if judicial expansion of refugee eligibility is confirmed before 2027, flag is activated via Commission-authorised configuration change.
- **FR-M25-023** When activated: refugee voter roll derived from UNHCR Kenya registration data cross-referenced with the Commission criteria (minimum residence period, no criminal conviction); roll compiled with UNHCR + Department of Refugee Services cooperation.
- **FR-M25-024** Verification: UNHCR biometric registration system (ProGres v4 or equivalent) provides a biometric match response via API; ABIS does not store refugee biometrics independently.
- **FR-M25-025** Voting at designated refugee station within or adjacent to refugee camp.

### Category E — IDPs

- **FR-M25-026** IDP register: compiled from UNHCR Kenya, NDOC, Kenya Red Cross displacement data cross-referenced with the standard voter register; any registered voter in an IDP camp who cannot travel to their registered station is offered re-assignment to an IDP-designated station.
- **FR-M25-027** Pre-election IDP verification window: IDP-designated the Commission centres open for 15 days before election day for roll re-assignment; re-assignment recorded on the voter register and reflected in the BVI kit roll.
- **FR-M25-028** IDP station provisioned in IDP mode: voter roll contains re-assigned IDPs; standard BVI workflow applies.
- **FR-M25-029** IDP station results published as Form 34A-I variants; included in the relevant constituency aggregation based on the voter's original registered constituency.

### Cross-Category

- **FR-M25-030** All special-category events appended to the Merkle audit chain within 60 minutes of the event (or on sync if offline).
- **FR-M25-031** Special-category kit dispatch is tracked with the same RFID chain-of-custody as standard kits (M10).
- **FR-M25-032** Special-category voter count per facility published on the public portal at T-14.

## 4. Non-Functional Requirements

- **NFR-M25-001** Kit offline operation: >= 8 hours for all special modes.
- **NFR-M25-002** Mobile hospital kit battery life: >= 6 hours continuous.
- **NFR-M25-003** WCAG 2.2 AA for all voter-facing screens; audio guidance mandatory for hospital voters.
- **NFR-M25-004** DPA 2019 lawful basis: special-category health data (hospital medical status) processed under s.30(1)(c) vital interests; biometric data under s.30(1)(b) legal obligation; DPIA registered per category.
- **NFR-M25-005** Remand-prisoner voter roll: no criminal-charge data stored in KIEMS; only prison-registration cross-reference.
- **NFR-M25-006** Refugee module: no refugee biometric data stored in KIEMS ABIS without explicit UNHCR data-sharing agreement; ProGres v4 API used as query endpoint only.
- **NFR-M25-007** KDF ballot secrecy: kit logs any proximity alert (officer within defined booth perimeter) as a mandatory violation event; violation events reviewed at 100% by IAB.
- **NFR-M25-008** Languages: EN + SW; Somali and Oromo added for refugee/IDP category facilities in North Eastern and Rift Valley.

## 5. User Stories

1. As a remand prisoner on election day, I authenticate with my fingerprint at the prison station, receive one ballot per contest I am registered for, and deposit it in the sealed ballot box without any prison official knowing how I voted.
2. As a bedridden hospital patient, I consent to vote with the mobile kit at my bedside; the attending nurse witnesses my consent; I vote with biometric authentication without leaving my bed.
3. As a KDF private, I vote at the barracks station with a Commission officer — not my commanding officer — as the presiding officer, and no superior can see my ballot.
4. As a Commission Special Voter Coordinator, I see the live status of all special-category kits (deployed/voting/closed/synced) on a single dashboard.
5. As a party agent at a remand prison, I witness the kit arrival with intact RFID seals, the voting session, and the sealed result transmission.
6. As an IAB Special Voter Auditor, I select a random 5% sample of remand-prison authentication events and verify each against the ABIS log at T+7.
7. As an IDP in Eldoret camp, I visited the Commission re-assignment centre 10 days ago; today I vote at the IDP station assigned to me; my vote is counted in my original Nairobi constituency.
8. As a hospital administrator, I receive advance notice from the Commission of the mobile kit visit schedule so I can prepare a private space for voting.
9. As the Commission Chair, I see an alert immediately if any KDF ballot-secrecy violation event is logged.
10. As a member of the public, I can see the total voter count per special-category facility on the portal (no patient or prisoner names), verifying no category is silently disenfranchised.

## 6. Data Model Additions

```python
class SpecialVoterFacility(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    category: Literal["REMAND_PRISON","HOSPITAL","BARRACKS","REFUGEE_CAMP","IDP_CAMP"]
    name: str
    county_code: str
    constituency_code: str
    latitude: float
    longitude: float
    kit_ids: list[str]
    presiding_officer_id: str
    languages: list[str]
    voting_date: date
    voting_start: time
    voting_end: time
    published_voter_count: int | None   # at T-14; no names
    status: Literal["CONFIGURED","ACTIVE","CLOSED","SYNCED"]

class SpecialVoterEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    facility_id: str
    category: Literal["REMAND_PRISON","HOSPITAL","BARRACKS","REFUGEE_CAMP","IDP_CAMP"]
    event_type: Literal[
        "KIT_ARRIVED","VOTING_OPENED","AUTHENTICATION","BALLOT_ISSUED",
        "BALLOT_CAST","ASSISTED_VOTE","S44A_OVERRIDE","SECRECY_VIOLATION",
        "VOTING_CLOSED","FORM_TRANSMITTED","FORM_SIGNED"
    ]
    voter_id: str | None         # encrypted; null for aggregate events
    officer_id: str | None
    witness_ids: list[str]
    reason_code: str | None
    event_at: datetime
    merkle_node_id: str | None

class SpecialForm34(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    facility_id: str
    form_variant: Literal["34A-R","34A-H","34A-B","34A-ID"]
    contest_id: str
    registered_voters: int
    ballots_issued: int
    ballots_cast: int
    rejected_ballots: int
    candidate_tallies: dict[str, int]
    presiding_officer_signed: bool
    party_agent_signatures: list[str]
    transmitted_at: datetime | None
    form_hash: str
    merkle_node_id: str | None
    published_at: datetime | None

class IDPReassignment(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    voter_id: str
    original_station_code: str
    original_constituency_code: str
    idp_facility_id: str
    reassigned_at: datetime
    verified_by_officer_id: str
    displacement_source: Literal["UNHCR","NDOC","RED_CROSS","SELF_DECLARATION"]
    effective_date: date
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| GET | /v1/special/facilities | COMMISSION_ADMIN JWT | 100/hr | List all special voter facilities |
| GET | /v1/special/facilities/{id} | COMMISSION_ADMIN JWT | 200/hr | Facility detail and status |
| POST | /v1/special/facilities/{id}/events | KIT_CERT mTLS | 500/min | Submit special voter event from kit |
| GET | /v1/special/facilities/{id}/events | COMMISSION_ADMIN JWT | 100/hr | Event log for a facility |
| POST | /v1/special/forms/34 | KIT_CERT mTLS | 50/hr | Submit special Form 34 |
| GET | /v1/special/forms/34 | Public | 500/min | List published special forms |
| GET | /v1/special/forms/34/{id} | Public | 500/min | Single form with Merkle proof |
| POST | /v1/special/idp/reassignments | COMMISSION_OFFICER JWT | 200/hr | Record IDP voter reassignment |
| GET | /v1/special/dashboard | COMMISSION_OPS JWT | 200/hr | Ops coordinator live status |
| GET | /v1/special/audit/sample/{category} | IAB_AUDITOR JWT | 10/hr | Pull IAB random-sample authentication logs |
| GET | /v1/special/public/counts | Public | 500/min | Published voter counts per facility (no PII) |

## 8. Integration Points

- **M01 BVR Device / M02 BVR Back-End** — special mode configuration for category-specific registration
- **M05 BVI Device / M06 BVI Back-End** — special mode BVI workflow
- **M03 ABIS** — biometric dedup and authentication for all categories
- **M10 RTS Back-End** — special Form 34 ingestion and aggregation
- **P05 Merkle Chain** — all special-category events anchored
- **M11 Public Portal** — published voter counts and special Form 34 viewer
- **Kenya Prisons Service** (external, MoU) — remand prisoner roll data
- **MoH / Hospital Authorities** (external, MoU) — hospital voter scheduling
- **KDF / GSU** (external, MoU) — barracks voter roll
- **UNHCR Kenya** (external, MoU, conditional) — refugee registration data for Category D
- **NDOC / UNHCR / Red Cross** (external) — IDP displacement data

## 9. UI/UX Key Screens

1. **Special Voter Operations Dashboard** — the Commission coordinator view; map of all special facilities; status badges; live event feed; violation-event highlight panel.
2. **Category-Specific Kit UI** — each mode has a mode identifier banner (colour-coded: orange for remand, blue for hospital, green for barracks, purple for IDP); otherwise identical to standard BVI UX to minimise officer training delta.
3. **Hospital Mobile Kit** — simplified large-text UI; audio guidance enabled by default; one-handed operation mode (tablet horizontal); "call for assistance" soft button triggers an alert to the presiding officer.
4. **Witness Signature Screen** — sequential signature collection: presiding officer, then each party agent, with biometric tap; party agent count displayed with waiting status.
5. **Violation Event Logger** — auto-triggered on secrecy-violation event; officer must acknowledge and add a note; auto-escalation to the Commission Chair within 5 minutes if unacknowledged.
6. **IDP Re-assignment Centre Console** — the Commission officer view; search by voter ID or name; pull current station assignment; log displacement source; confirm new IDP facility assignment.
7. **Public Special Voter Counts Page** — facility type filter; per-facility voter count and voting-day schedule; no patient or prisoner names.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Remand prisoner right to vote | Constitution Art. 83; Kituo cha Sheria v AG [2013] eKLR |
| Hospital voter assistance | Elections Act 2011 s.72 |
| Service personnel special enrolment | Elections Act 2011 s.5 |
| Ballot secrecy for all voters | Constitution Art. 38(3); Elections Act s.72 |
| Complementary mechanism | Elections Act 2011 s.44A |
| Refugee Act 2021 framework | Refugees Act 2021 (voting rights subject to judicial determination) |
| Data protection — sensitive health and detention data | DPA 2019 s.30(1)(b), (c); DPIA per category |
| Audit of special-category events | Elections Act s.39; IAB mandate |
| IDP voter access | Constitution Art. 38; Internal Displacement Act 2012 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Remand prisoners offered the vote at registered facilities | 100% of eligible remand facilities | Facility schedule vs gazette | the Commission Special Voter Coordinator |
| Hospital voter sessions completed per schedule | >= 95% | Session completion log | the Commission Ops |
| KDF ballot-secrecy violation events reviewed within 24 hours | 100% | Violation event log | IAB |
| Special Form 34 published within 30 minutes of close | >= 98% | Timestamp delta | Datacraft Technical |
| IAB 5% audit sample reviewed at T+7 | 100% of sample | Audit completion log | IAB Special Voter Auditor |
| IDP re-assignments reflected in correct constituency tally | 100% | Aggregation cross-check | Returning Officer |
| Special-category events in Merkle chain within 60 minutes | 100% | Merkle reconciliation | Datacraft Technical |

## 12. Test Strategy

- **Unit:** category-mode kit configuration; special Form 34 variant schema; IDP re-assignment logic; audit sample generator.
- **Integration:** end-to-end remand-prison session with synthetic roll; hospital mobile kit sync; barracks result transmission.
- **E2E:** simulated multi-category election day with all five categories active.
- **Security:** confirm barracks kit cannot be opened by military credential; confirm hospital voter records never expose patient health status; confirm remand roll contains no charge data.
- **Accessibility:** audio-guidance flow for hospital voter; large-text mode for elderly voters.
- **Offline:** 8-hour offline BVI at remand prison; verify sync integrity on reconnect.
- **Chaos:** hospital kit battery failure mid-session; remand prison network failure.
- **Audit:** simulate IAB 5% sample selection and verify reproducibility from Merkle chain.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| MoUs with Kenya Prisons Service, MoH, KDF signed | 2027-01-01 | Commission-facilitated; Datacraft technical spec supplied |
| Special mode kit firmware complete | 2027-02-01 | All five category modes in firmware |
| Facility gazettement (T-90) | 2027-05-11 | All special facilities gazetted with voter counts |
| Hospital voter roll cross-reference | 2027-07-01 | County returning officers confirm hospital rolls |
| IDP verification window opens | 2027-07-26 | 15-day pre-election window |
| Special category voting days | 2027-08-08 to 2027-08-09 | One or two days before national election day |
| IAB 5% audit | 2027-08-10 to 2027-08-17 | T to T+7 |
| Post-election special-voter report | 2027-09-10 | T+31 |

## 14. Dependencies

- M01/M02/M05/M06 — shared BVR/BVI code base; special-mode flags
- M03 ABIS — authentication backend
- M10 RTS Back-End — Form 34 ingestion
- Kenya Prisons Service MoU — remand roll data (external)
- MoH / Hospital MoU — hospital voter scheduling (external)
- KDF/GSU secondment MoU — the Commission officer deployment to barracks (external)
- UNHCR Kenya MoU — refugee category (conditional; external)
- NDOC/Red Cross data-sharing — IDP category (external)

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Kenya Prisons Service refuses to provide remand roll or kit access | Medium | High | Constitutional mandate is clear (Art. 83 + Kituo cha Sheria); the Commission escalates via AG's office; pre-election court order if necessary |
| Hospital patient votes under duress from family member | Low | Medium | Clinician impartial-witness requirement; party agent present; s.72 consent protocol; voter has the right to request a private space |
| KDF commanding officer attempts to observe individual votes | Low | High | Civilian presiding officer logs violation event immediately; IAB alert within 5 minutes; the Commission Chair escalation; DCI referral if conduct constitutes an offence under Election Offences Act |
| IDP register inaccurate — IDPs turned away at assigned station | Medium | Medium | 15-day verification window; day-of s.44A complementary mechanism as fallback; overcounting guard at both old and new station |
| Refugee eligibility judicially expanded after procurement deadline | Low | Medium | Module built and gated behind configuration flag; activation does not require a firmware update |
