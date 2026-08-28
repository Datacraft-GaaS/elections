# M19 — Queue Transparency

**Module ID:** M19
**Name:** Queue Transparency System
**Owner:** Datacraft Limited — Operations Engineering Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part XV §XV.3, Part XV §XV.2, Part XVI §XVI.4, Part XVIII §XVIII.2

---

## Purpose

The Queue Transparency System tracks real-time voter queue status at all 55,393 polling stations, triggers escalation when wait times exceed thresholds, drives field dispatch decisions in M16, and publishes live queue data to M13's voter-facing Queue tab. Long queues that close before the last voter votes are a disenfranchisement mechanism and a petition ground. The system converts queue management from a subjective presiding officer judgment into a data-driven, automatically escalating process with a statutory path to extended voting hours under Elections Act s.14A. Attendance counting (XV.2) is co-located in this module.

---

## User Personas

| Persona | Channel | Primary task |
|---|---|---|
| Registered voter | M13 app, SMS, web | Check wait time at own station; decide when to arrive |
| Presiding Officer | KIEMS kit | Submit manual 30-min queue estimate; receive alert when queue critical |
| ECC Operations Manager | M16 console | Receive escalation alerts; dispatch roving engineers; recommend extended hours |
| County Coordinator | Web console | Monitor all stations in county; dispatch county spares |
| Field Support Engineer | Mobile app | Navigate to assigned station; update queue status after arrival |
| IEBC Chair | Executive console | Receive s.14A extended-hours recommendations; authorise declarations |
| Party Agents | M16 party portal | Monitor queue status for stations in their constituency |
| Journalists | M13 / journalist API | Report on national queue picture |

---

## Functional Requirements

### FR-19.01 — Queue status inputs (multiple redundant sources)
Per station, three independently tracked inputs updated on different schedules:

**Input 1 — Presiding Officer manual estimate (mandatory):**
Every 30 minutes during operating hours, the KIEMS kit prompts the Presiding Officer for a queue-length estimate (text field + visual slider: 0, 1-10, 11-30, 31-60, 61-100, 100+, cleared). The estimate is timestamped, signed by the kit, and transmitted. This is the legally authoritative input — zero technology dependency.

**Input 2 — Kit authentication throughput (computed):**
Kit reports authenticated voters per 15-minute interval. Queue-length estimation: (voters-in-queue-at-station-open + cumulative-arrivals-estimate) - authenticated. Smoothed with exponential moving average.

**Input 3 — Voluntary QR-code queue register:**
Voter optionally scans a QR code at station entrance (no ID required; generates an anonymous timestamp token). Aggregated count updated in real time. Reconciliation: QR scans >= authentication rate expected; anomaly if QR count << authentication count (suggests voters entering without scanning, which is fine — this is a voluntary system).

### FR-19.02 — Derived queue metrics per station
Every 5 minutes from three inputs: current queue length (estimate), current throughput rate (voters/minute, from Input 2), estimated wait time (queue length / throughput), estimated clear time (absolute EAT), alert flag.

### FR-19.03 — Live public queue map
Public-facing map at M13 Queue tab and `queue.iebc.go.ke`: all 55,393 stations colour-coded by estimated wait time (green < 45 min, amber 45-120 min, red > 120 min). Updated every 5 minutes. Per-station detail page with wait-time trend chart. Opt-in SMS alert when a voter's home station drops below 30-minute wait.

### FR-19.04 — ECC escalation triggers (PM-04)
Automatic alerts to M16 ECC on wait-time thresholds:

| Wait time | ECC action | Authority |
|---|---|---|
| > 90 min | Dispatch alert to County Coordinator | ECC Ops Manager |
| > 180 min | Deploy roving engineer with spare kit to increase throughput | ECC Ops Manager |
| > 240 min | Escalate to IEBC Chair + notify all accredited party agents for that station | ECC Director |
| > 300 min | Activate s.14A extended-voting-hours recommendation with documented evidence | IEBC Chair |

Each threshold trigger creates a timestamped, HSM-signed ECC alert appended to the audit log. The > 300-min escalation includes a pre-formatted s.14A recommendation document with supporting data, ready for IEBC Chair signature.

### FR-19.05 — Stations-clear-before-close tracking
Kit records: at 17:00 EAT (or extended hours), did any voter in the queue fail to vote? A "queue-not-cleared" event is logged if Input 1 estimate shows > 0 queue at station close AND the station closed before all queued voters voted. Any such event is a disenfranchisement flag routed to the IAB and the M23 grievance channel.

### FR-19.06 — Attendance counting (XV.2)
Three independent footfall counters per station:
- **Counter 1:** Presiding Officer mechanical clicker tally, logged every 30 minutes as part of FR-19.01 prompt (logged in Form 32B sync field).
- **Counter 2:** Kit camera anonymised headcount (computer-vision people detection, no face recognition, no image stored, DPA 2019 lawful basis via visible signage). Count logged locally every 10 minutes.
- **Counter 3:** QR-code voluntary roster (FR-19.01 Input 3). Aggregated count real-time.

Reconciliation checks (automated):
- Entries >= Authentications (must hold; if not, investigation)
- Authentications >= Ballots issued (must hold)
- Ballots issued >= Ballots cast (must hold)
- If authentications > entries × 1.02: automatic result hold pending review
- If entries > authentications × 1.5: systemic-disenfranchisement investigation flag

### FR-19.07 — SMS opt-in alert
Voters who opt in via M13 (phone number required for this feature only; DPA 2019 consent lawful basis) receive SMS when their home station's estimated wait drops below 30 minutes. Phone number stored hashed (SHA-256 + salt); decrypted only to send SMS; deleted 30 days after election.

### FR-19.08 — Historical queue analytics
Post-election: per-station queue data (wait time by 30-minute interval, attendance counts, alert events, s.14A triggers) published as open dataset within 30 days. Analysed for: peak hours by county/region, correlation with turnout, disenfranchisement events. Report to Parliament and Auditor-General.

---

## Non-Functional Requirements

### NFR-19.01 — Availability
Queue map and public API: 99.9% during E-Day. ECC alert delivery: 99.99% during polling window. Kit-side queue prompt: operates entirely offline; data syncs when connectivity available.

### NFR-19.02 — Latency
Queue map update: every 5 minutes from latest data. ECC alert delivery after threshold breach: p99 < 2 minutes. M13 queue display: p99 < 3 s.

### NFR-19.03 — Scalability
55,393 station telemetry streams; 1M concurrent M13 queue map users; 5M SMS alert subscribers (estimated maximum opt-in).

### NFR-19.04 — Privacy (DPA 2019)
Counter 2 (computer-vision headcount): no images stored; no faces identified; no data transmitted; count only, aggregated locally. DPA 2019 s.26 public interest basis confirmed in DPIA; visible signage at station entrance mandatory.

Opt-in SMS: explicit consent; phone number hashed at rest; deleted at T+30. Processed under DPA 2019 s.30 (consent basis).

QR roster: no personal data linked to QR token. Token is a random UUID7 generated at scan time; not linked to voter identity.

### NFR-19.05 — Offline resilience
All three inputs operate without connectivity. Queue estimates, headcounts, and QR logs queue locally on the KIEMS kit and sync when connected. ECC receives queued data on reconnection; interpolates gaps for the public map.

---

## User Stories

1. As a registered voter, I want to check the estimated wait time at my station before leaving home, so that I can time my arrival to minimise queuing.
2. As a voter, I want to receive an SMS when the queue drops below 30 minutes, so that I can arrive at the right time without repeatedly checking the app.
3. As a Presiding Officer, I want a 30-minute reminder to update my queue estimate on the kit, so that the ECC has current data without me needing to remember.
4. As an ECC Operations Manager, I want an automatic alert when any station exceeds 240 minutes wait, so that I can escalate to the IEBC Chair before voters are turned away.
5. As the IEBC Chair, I want to receive a pre-formatted s.14A recommendation when wait exceeds 300 minutes at a significant number of stations, so that I can authorise extended hours with documented justification.
6. As a party agent, I want to see queue status for all stations in my constituency, so that I can identify and report stations where voters may be disenfranchised.
7. As a journalist, I want to see the national queue map in real time, so that I can report accurately on voter access during election day.
8. As an IAB Member, I want to see all disenfranchisement flags (queue-not-cleared events) in the M15 audit feed, so that I can assess whether voters were turned away and include findings in the quarterly report.
9. As a researcher, I want to download the post-election queue analytics dataset, so that I can study the relationship between queue times, turnout, and demographics.

---

## Data Model Additions

```python
class QueueSnapshot(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    snapshot_at: datetime  # EAT
    input1_po_estimate: int | None  # presiding officer manual estimate
    input2_throughput_per_15min: int | None  # authenticated voters
    input3_qr_scans: int | None
    derived_queue_length: int | None
    derived_wait_min: int | None
    derived_clear_time_eat: datetime | None
    alert_level: str | None  # green | amber | red | critical
    kit_serial: str
    connectivity: bool  # was kit connected when snapshot generated?

class AttendanceSnapshot(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    snapshot_at: datetime
    counter1_clicker: int | None
    counter2_cv_headcount: int | None
    counter3_qr_scans: int | None
    authentications_total: int
    ballots_issued: int
    ballots_cast: int
    reconciliation_ok: bool
    reconciliation_flag: str | None

class DisenfranchisementFlag(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    detected_at: datetime
    flag_type: str  # queue_not_cleared | auth_gt_entries | entries_gt_auth_x15
    queue_at_close: int | None
    authentications: int
    entries_estimate: int | None
    escalated_to_iab: bool
    escalated_to_grievance: bool

class S14ARecommendation(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    generated_at: datetime
    triggered_by_station_count: int  # stations with > 300 min wait
    recommended_extension_hours: int
    stations: list[str]  # station codes
    supporting_data_hash: str
    iebc_chair_signed: bool
    signed_at: datetime | None
    declaration_gazette_ref: str | None
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/queue/map` | None (public) | 500/s | All stations colour-coded; compact GeoJSON |
| GET | `/api/v1/queue/{station_code}` | None (public) | 200/s | Per-station detail with trend |
| POST | `/api/v1/queue/{station_code}/alert-opt-in` | None (public, consent) | 5/s per IP | SMS opt-in; phone number hashed at rest |
| GET | `/api/v1/attendance/{station_code}` | Operator JWT | 100/s | Attendance counts; reconciliation status |
| GET | `/api/v1/disenfranchisement/flags` | Operator JWT | 50/s | All disenfranchisement flags |
| GET | `/api/v1/s14a/recommendations` | ECC Director JWT | 10/s | s.14A recommendation documents |
| POST | `/api/v1/s14a/{id}/sign` | IEBC Chair JWT + HSM | 2/s | Chair signs s.14A recommendation |
| GET | `/api/v1/analytics/queue-post-election` | None (public, post-election) | 20/s | Historical queue dataset |
| SSE | `/api/v1/queue/stream` | Operator JWT | 1 per session | Live queue update stream for ECC |

---

## Integration Points

**Upstream:**
- M08 (RTS Device Client) — kit telemetry (authentication throughput, QR scan count, PO manual estimate, CV headcount)
- M13 (Public Verifier App) — SMS opt-in intake; queue map display consumer
- P06 (HSM) — signing s.14A recommendation documents

**Downstream:**
- M16 (ECC) — escalation alerts (push via SSE + event bus)
- M13 (Public Verifier App) — queue map data, voter SMS alerts
- M15 (IAB Dashboard) — disenfranchisement flags, attendance reconciliation
- M23 (Grievance Channels) — automatic grievance creation for disenfranchisement flags
- SMS gateway — opt-in voter alerts

---

## UI/UX — Key Screens

1. **National queue map** (public, on M13 and standalone) — choropleth map by station wait time; legend; county-level aggregate summary
2. **Per-station queue detail** — estimated wait, throughput rate trend chart, last updated timestamp, alert level badge
3. **ECC queue panel** (within M16) — filtered to critical stations; s.14A trigger button when threshold met; dispatch integration
4. **Attendance reconciliation panel** (operator, within M16) — per-station input comparison, flag highlighting, trend over polling day
5. **s.14A recommendation document** — auto-populated with supporting data; IEBC Chair electronic signature workflow
6. **Post-election analytics** — interactive charts: wait time by hour, by county; disenfranchisement event map; turnout correlation

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Extended voting hours for queued voters | Elections Act s.14A |
| Voter's right to vote | Constitution Art. 38; Elections Act s.5 |
| No voter disenfranchisement by queue | Constitution Art. 81(e) (universal suffrage) |
| Attendance counting as Form 32 basis | Elections (General) Regulations 2012 reg. 62 |
| DPA 2019 — computer-vision headcount (no face recognition) | DPA 2019 s.30 public interest; DPIA required |
| DPA 2019 — SMS opt-in (consent) | DPA 2019 s.30 (consent basis) |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Median queue wait during peak 09:00-13:00 | <= 45 minutes | IEBC Chair |
| Stations with wait > 240 min at any point | <= 0.5% | IEBC Chair |
| Voters denied vote due to queue-at-close | 0 | IAB |
| Queue map update latency | < 5 min | Datacraft SRE |
| ECC escalation delivery after threshold | p99 < 2 min | Datacraft SRE |
| SMS alert delivery after queue drop | p99 < 60 s | SMS gateway SLA |
| Attendance reconciliation flag false-positive rate | < 1% | IAB |
| s.14A recommendation generation when triggered | 100% within 5 min | ECC Ops Manager |

---

## Test Strategy

- **Unit:** Queue estimation algorithm (queue length from inputs), attendance reconciliation logic, s.14A trigger conditions
- **Integration:** Kit telemetry ingestion; M16 alert routing; M13 map data feed; SMS gateway
- **Simulation:** Replay of 2022 election-day throughput data to validate queue estimation accuracy
- **Offline resilience:** Kit disconnected for 2 hours; queue data queued locally; sync on reconnect; ECC receives historical snapshots on reconnection
- **Privacy:** CV headcount confirmed to store no images and transmit no video; DPA DPIA sign-off; independent verification that no face data is retained
- **Load:** 55,393 concurrent station telemetry streams; 1M concurrent public queue map users

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Queue algorithm validated on 2022 data | 2027-01-15 | Model calibration |
| Public queue map deployed (mock data) | 2027-04-01 | UI validation |
| By-election pilot (1 constituency) | 2027-05-01 | End-to-end test with real kits |
| Dry-run election (T-60) | 2027-06-10 | All inputs active; s.14A rehearsed |
| SMS opt-in live | 2027-07-11 (T-30) | Voter registration for alerts |
| Election Day | 2027-08-10 | Full ops |
| Post-election dataset published | 2027-09-10 (T+30) | Open data release |

---

## Dependencies

- **M08** (RTS Device Client) — primary telemetry source (authentication count, CV headcount, PO estimate, QR scans)
- **M13** (Public Verifier App) — queue map display; SMS opt-in; Safety tab
- **M16** (ECC) — alert escalation; s.14A workflow; dispatch trigger
- **M15** (IAB Dashboard) — disenfranchisement flag feed
- **M23** (Grievance Channels) — automatic grievance creation
- **P05** (Audit log) — queue and attendance event logging
- **P06** (HSM) — s.14A document signing
- External: SMS gateway (Safaricom / Airtel aggregator)

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Presiding Officers skip the manual estimate prompt | Kit escalates to ECC if no estimate received for > 45 min; County Coordinator follows up; PO training curriculum includes this as mandatory step |
| R2 | CV headcount triggers DPA 2019 challenge | No face recognition; no image storage; no transmission; visible consent signage; DPIA published; IAB confirms compliance |
| R3 | Queue map causes voter pile-up at "green" stations | Data presented as estimated, not exact; app displays confidence indicator; media briefed on interpretation |
| R4 | s.14A trigger fires erroneously (false-positive wait time) | Multiple input cross-check required before trigger; human ECC Ops Manager reviews before recommendation is generated; IEBC Chair has final authority |
| R5 | SMS gateway capacity exceeded on election morning | Pre-negotiated higher-tier quota with both Safaricom and Airtel; alerts batched by county if volume exceeds quota |
