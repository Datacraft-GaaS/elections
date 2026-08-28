# M18 — Boundary Transparency

**Module ID:** M18
**Name:** Boundary Transparency System
**Owner:** Datacraft Limited — Geo-Intelligence Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4; boundary dataset and simulator released as open data under CC BY 4.0

**Strategy traceback:** Part XVI §XVI.1, Part XVI §XVI.6

---

## Purpose

The Boundary Transparency System operationalises Kenya's constitutional obligation of fair delimitation (Constitution 2010 Art. 89; Sixth Schedule s.41) by publishing all boundary data as open, machine-readable, versioned geodata, providing a public simulator for alternative configurations, computing fairness metrics automatically, and maintaining a hash-chained audit trail of every proposed boundary change. It addresses the legal risk established in Katiba Institute v Attorney General [2017] eKLR (procedural delays in delimitation review found unconstitutional) and the advocacy demand for transparent, challengeable delimitation decisions. The system is a pro-bono Datacraft contribution at KES 0, and doubles as a geospatial data service for all other KIEMS modules requiring constituency or station coordinates.

---

## User Personas

| Persona | Channel | Primary task |
|---|---|---|
| Citizens / voters | Web browser, M13 app | Verify their constituency boundary; understand delimitation |
| Data journalists | Web browser, API | Download GeoJSON; run efficiency-gap analysis; write stories |
| Civil society researchers (Katiba Institute, KHRC, AfriCOG) | Web browser, API | Monitor for gerrymandering; submit boundary challenges |
| Legal counsel / petitioners | Web browser | Evidence for redistricting petition |
| IEBC Delimitation Officers | Internal console | Propose boundary changes; document justification |
| IEBC Commissioners | Executive view | Approve proposed changes; sign change records |
| Parliament (Boundaries Committee) | Read-only web | Review IEBC proposals with supporting fairness metrics |
| IAB | Read-only | Audit boundary change history; monitor fairness metrics |

---

## Functional Requirements

### FR-18.01 — Open boundary data platform
All constituency, ward, county, and polling-station boundaries published as GeoJSON (EPSG:4326) with: boundary ID, name, population (2019 Census), registered voter count (live, updated each CVR cycle), area km², type (constituency / ward / county / station), effective date, gazette notice reference, and Merkle-anchored version hash. Updated within 24 hours of any IEBC action.

### FR-18.02 — Historical snapshots
Boundary dataset published for every delimitation event since 2013. Historical snapshots downloadable individually. Each snapshot has a SHA-256 hash enabling independent verification that the published data matches the gazette. Provenance chain: gazette notice PDF hash, IEBC Commissioner signature, IMS event.

### FR-18.03 — Public API
RESTful GeoJSON API and bulk download endpoint. OpenAPI 3.1 spec published. WMS/WFS endpoint for GIS clients. Rate-limited public access; authenticated higher-rate access for civil society partners.

### FR-18.04 — Boundary-redraw simulator
Interactive web tool: user draws alternative boundary configurations subject to Constitutional Sixth Schedule population-parity constraint (+/-40% of national quota). System computes automatically:
- Population parity score (deviation from quota)
- Compactness index (Polsby-Popper: 4πA/P²)
- Efficiency gap (Stephanopoulos and McGhee 2015)
- Majority-minority ward preservation flag
- Cracking/packing detection (constituencies with > 80% single-ethnicity composition flagged)

Simulator outputs are clearly labelled as hypothetical; they are not IEBC proposals. Export: PDF report + GeoJSON of proposed configuration.

### FR-18.05 — Ex-post boundary impact analysis
After every election: automatically compute and publish per-constituency metrics: vote-to-seat ratio, wasted-vote count (votes for losing candidate + votes above winning threshold), efficiency gap. Historical comparison chart (2013, 2017, 2022, 2027). Published to the open data platform within 30 days of Form 34C publication.

### FR-18.06 — Redistricting audit trail
Every proposed boundary change by any IEBC officer logged with: author (IEBC staff ID), timestamp, change description, justification text, before/after fairness metrics (auto-computed), before-boundary hash, after-boundary hash. Append-only. Hash-chained and HSM-signed. Public comment portal: any registered user can submit a comment on a proposed change during a 30-day public notice period. Comments are logged and visible to IEBC Commissioners and the Boundaries Committee. Audit trail is readable by IAB in real time.

### FR-18.07 — Gazette-linked version control
Every gazette notice for a boundary change is: PDF-linked, hash-computed, and attached to the corresponding audit trail record. The Gazette PDF hash is published alongside the GeoJSON hash. Any discrepancy between gazette content and IEMS boundary data is an immediate alert to the IAB.

### FR-18.08 — Station geofence service (shared infrastructure)
Boundary data service doubles as the geofencing engine for all KIEMS modules (M08 BVI device, M05 BVI back-end, M16 ECC kit-tracking). All station boundary polygons (55,393 stations + buffers) served as a low-latency geofence API consumed by other modules. Geofence queries: p99 < 20 ms.

### FR-18.09 — Fairness-metric monitoring
Continuous monitoring of the registered-voter distribution across constituencies against Sixth Schedule quota. Monthly public report showing: top 10 most over-represented constituencies (registered voters per seat above national mean), top 10 most under-represented, and trend since last delimitation. Alert to IEBC Chair and IAB when any constituency exceeds ±40% of national quota — triggering constitutional delimitation obligation.

### FR-18.10 — Parliament and public submissions channel
Structured submission portal where citizens, civil society organisations, or Parliament can formally submit boundary representations. Submissions logged, hashed, given reference numbers, and included in IEBC's statutory consultation record. Response from IEBC recorded per submission with reasoning.

---

## Non-Functional Requirements

### NFR-18.01 — Availability
Open data API and simulator: 99.9% annual availability. Geofence service (M08/M05 dependency): 99.99% during election window.

### NFR-18.02 — Performance
Public API p99 < 500 ms for boundary polygon queries. Geofence service p99 < 20 ms. Simulator fairness metric computation < 5 s for any proposed configuration.

### NFR-18.03 — Data accuracy
Boundary data matches corresponding Kenya Gazette notice within 24 hours of gazette publication. Any discrepancy between IEMS data and gazette triggers immediate IAB alert. Coordinate precision: 6 decimal places (approximately 0.1 m) matching Kenya National Land Commission standards.

### NFR-18.04 — Privacy (DPA 2019)
Boundary data: no personal data. Registered-voter count is an aggregate; individual voter data is not in this module. Public comment portal: optional name disclosure; comments retained for 7 years as part of statutory consultation record. DPA 2019 s.30 public interest basis.

### NFR-18.05 — Openness
Boundary dataset: Creative Commons Attribution 4.0 International (CC BY 4.0). API: no authentication required for public access up to rate limit. Simulator: open-source front-end (MIT licence). The commitment to open data is enforceable via the Access to Information Act 2016 s.4.

---

## User Stories

1. As a citizen, I want to view my constituency boundary on a map and see how many registered voters it contains, so that I understand my electoral unit.
2. As a civil society researcher, I want to download the complete GeoJSON boundary dataset and compute the efficiency gap myself, so that my analysis is independent of IEBC's published figures.
3. As a journalist, I want to use the boundary simulator to test what the map would look like under an alternative configuration, so that I can illustrate gerrymandering claims with factual alternatives.
4. As a legal counsel, I want to download the boundary change audit trail for constituency X, so that I have evidence of procedural compliance or failure for a petition.
5. As an IEBC Delimitation Officer, I want to log a proposed boundary change with my justification and see the automatic fairness metrics, so that I can assess compliance with Sixth Schedule before proposing to Commissioners.
6. As an IAB Member, I want to receive an alert whenever any constituency exceeds the ±40% population parity threshold, so that I can flag a constitutional delimitation obligation to IEBC.
7. As a member of the public, I want to submit a formal representation on a proposed boundary change and receive a reference number, so that my voice is part of the statutory consultation record.
8. As an IEBC Commissioner, I want to review all public comments on a proposed change alongside the fairness metrics before signing, so that my decision is fully informed and documented.

---

## Data Model Additions

```python
class Boundary(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    code: str  # official IEBC code
    name: str
    boundary_type: str  # constituency | ward | county | station
    parent_code: str | None
    geojson: dict  # GeoJSON Feature
    population_2019: int | None
    registered_voters: int  # live, updated per CVR cycle
    area_km2: float
    effective_date: datetime
    gazette_notice_ref: str
    gazette_pdf_hash: str
    geojson_hash: str
    version: int
    signed_by: str  # IEBC Commissioner UUID
    merkle_seq: int

class BoundaryChange(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    boundary_id: str
    proposed_by: str  # IEBC officer UUID
    proposed_at: datetime
    justification: str
    before_hash: str
    after_geojson: dict
    after_hash: str
    population_parity_before: float
    population_parity_after: float
    polsby_popper_before: float
    polsby_popper_after: float
    efficiency_gap_before: float | None
    efficiency_gap_after: float | None
    public_notice_opened_at: datetime | None
    public_notice_closes_at: datetime | None
    comments_count: int
    approved_by: str | None  # Commissioner UUID
    approved_at: datetime | None
    gazette_ref: str | None
    signed_hash: str

class BoundarySubmission(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    change_id: str
    submitted_at: datetime
    submitter_name: str | None
    submitter_org: str | None
    submission_type: str  # support | object | comment
    content: str
    evidence_hash: str | None
    iebc_response: str | None
    iebc_response_at: datetime | None
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/boundaries` | None (public) | 100/s | List boundaries (filterable by type) |
| GET | `/api/v1/boundaries/{code}` | None (public) | 200/s | Single boundary with GeoJSON |
| GET | `/api/v1/boundaries/{code}/history` | None (public) | 20/s | Version history with change audit |
| GET | `/api/v1/boundaries/bulk.geojson.gz` | None (public) | 5/s | Full dataset bulk download |
| GET | `/api/v1/geofence/station/{code}` | Service JWT | 2000/s | Station polygon for kit geofence |
| POST | `/api/v1/simulator/compute` | None (public) | 2/s | Compute fairness metrics for proposed config |
| GET | `/api/v1/metrics/fairness` | None (public) | 20/s | Current fairness metric summary |
| GET | `/api/v1/metrics/impact/{election}` | None (public) | 10/s | Ex-post impact analysis for election year |
| GET | `/api/v1/changes` | None (public) | 50/s | Boundary change audit log |
| POST | `/api/v1/changes` | IEBC Officer JWT | 5/s | Propose boundary change |
| POST | `/api/v1/changes/{id}/approve` | Commissioner JWT | 2/s | Approve change (triggers gazette submission) |
| POST | `/api/v1/submissions` | None (public, rate-limited) | 2/s per IP | Submit public representation |
| GET | `/api/v1/submissions/{change_id}` | None (public) | 20/s | Read public submissions on a change |

---

## Integration Points

**Upstream:**
- Kenya National Land Commission — authoritative survey data (annual batch exchange)
- Kenya National Bureau of Statistics — population data (2019 Census; 2024 mini-census if available)
- M02 (BVR Back-End) — registered voter count per station (live feed)
- M10 (RTS Back-End) — election results for ex-post impact analysis
- Kenya Government Printer — Gazette PDF import

**Downstream (boundary as a service):**
- M05 (BVI Back-End) — station polygon for biometric identification geofencing
- M08 (RTS Device Client) — station geofence for kit activation
- M16 (ECC) — kit location vs geofence comparison
- M20 (IMS) — station geofence for kit custody events
- M13 (Public Verifier App) — station location display
- M15 (IAB Dashboard) — boundary change audit feed
- Parliament (Boundaries Committee) — structured data feed (authenticated)

---

## UI/UX — Key Screens

1. **Boundary explorer** — Kenya map (Mapbox/Leaflet, self-hosted tiles); layer toggles (county, constituency, ward, station); click any polygon to see detail panel with registered voters, fairness metrics, change history
2. **Simulator** — draw mode on top of base map; real-time fairness metrics update as user draws; constraint warnings (parity violation highlighted); export button
3. **Change audit timeline** — chronological list of all boundary changes; click to see before/after map overlay, fairness metric comparison, public comments
4. **Fairness metrics dashboard** — national summary cards; per-constituency deviation bar chart; efficiency gap trend line; top-10 over/under-represented
5. **Public submission portal** — change listing; submission form; reference number confirmation; IEBC response tracker
6. **Ex-post impact report** — post-election per-constituency vote-to-seat ratio, wasted-vote count, efficiency gap; comparison to previous elections

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Delimitation every 8-12 years; population parity ±40% | Constitution Art. 89; Sixth Schedule s.41 |
| Procedural fairness in delimitation | Katiba Institute v AG [2017] eKLR |
| Public consultation on boundary changes | IEBC Act s.27; Constitution Art. 10 |
| Open access to government information | Access to Information Act 2016 s.4 |
| Audit trail of administrative decisions | PPADA 2015; IEBC Act |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Boundary data updated after IEBC action | Within 24 hours | IEBC Delimitation Office |
| Geofence API p99 latency | < 20 ms | Datacraft SRE |
| Boundary explorer availability | 99.9% | Datacraft SRE |
| Simulator fairness metric compute time | < 5 s | Datacraft Engineering |
| Unique simulator users per year | > 100,000 | IEBC Comms |
| Submission response rate | 100% of formal representations | IEBC Chair |
| Ex-post impact analysis publication | Within 30 days of Form 34C | IEBC Delimitation Office |
| ±40% parity alert delivery | Within 1 hour of threshold breach | IAB |

---

## Test Strategy

- **Unit:** Polsby-Popper calculation, efficiency gap calculation, population-parity check
- **Integration:** CVR voter count feed from M02; geofence service consumption by M08/M05; Gazette PDF import
- **Accuracy:** Boundary polygons cross-checked against Kenya National Land Commission shapefiles; discrepancy rate target 0
- **Simulator correctness:** Known-gerrymandered configurations must trigger appropriate flags; fair configurations must return passing metrics
- **Load:** Geofence service load test at 2,000 concurrent queries (peak kit activation period T-7 to T-1)
- **Open data compliance:** API anonymity test (no authentication required for public read); licence header check
- **History integrity:** SHA-256 of each historical boundary version recomputed and verified against published hash

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Historical data ingestion (2013-2027) | 2026-11-01 | All gazetted boundaries loaded |
| Open API and explorer live (pro-bono launch) | 2027-01-01 | Public access |
| Simulator published | 2027-02-01 | Pro-bono launch |
| Geofence service integrated with M05/M08 | 2027-03-01 | System test with kit provisioning |
| Ex-post 2022 impact analysis published | 2027-04-01 | Establishes methodology baseline |
| Election Day (geofence service in critical path) | 2027-08-10 | Full ops |
| Ex-post 2027 impact analysis | 2027-09-10 (T+30) | Published |

---

## Dependencies

- **M02** (BVR Back-End) — registered voter count per station
- **M05** (BVI Back-End), **M08** (RTS Device Client) — geofence service consumers
- **M16** (ECC) — kit geofence consumer
- **M20** (IMS) — custody geofence events
- **P05** (Audit log) — boundary change events
- **P06** (HSM) — boundary version signing
- External: Kenya National Land Commission shapefile data exchange; KNBS 2019 Census API; Kenya Government Printer Gazette API

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Geofence service outage during kit activation window (T-7 to T-1) | 99.99% SLA; kit caches geofence polygon at provisioning; kit operates on cached polygon if service unreachable |
| R2 | Boundary data dispute between IEBC dataset and gazette gazette | Gazette PDF hash stored alongside GeoJSON hash; any discrepancy triggers IAB alert and publication freeze until resolved |
| R3 | Simulator misused to generate redistricting propaganda | Outputs clearly labelled as hypothetical; no IEBC branding; open-source code allows independent verification of fairness metrics |
| R4 | Land Commission data is stale or incorrect | Annual data exchange MoU; IEBC Delimitation Office review of each boundary update before publication |
| R5 | Public submission portal overwhelmed by organised campaign | Rate limiting per IP + per organisation; CAPTCHA; submissions batched for review; malicious submissions flagged and removed with IAB oversight |
