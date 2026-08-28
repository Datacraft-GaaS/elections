# M16 — Election Command Centre

**Module ID:** M16
**Name:** Election Command Centre (E-SOC / COC / CAOC Unified Platform)
**Owner:** Datacraft Limited — Operations Engineering Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part II §PM-04, §PM-05, §PM-11, Part XVI §XVI.5, Part XVIII §XVIII.6, Part XIX §XIX.4

---

## Purpose

The Election Command Centre is the unified operational nerve centre for the 2027 general election. It consolidates three previously distinct functions — the Election Security Operations Centre (E-SOC), the Compliance Operations Centre (COC), and the Compliance and Awareness Operations Centre (CAOC) — into one operator platform with role-specific workstation views. The ECC runs 24x7 from T-90 (11 May 2027) through result announcement and the 30-day petition window. It receives real-time telemetry from every KIEMS kit, all M17-M20 sub-systems, field-engineer reports, party-agent alerts, and observer feeds. It drives escalation, dispatches field resources, tracks statutory compliance, and feeds the public-facing status dashboard. The ECC is the human decision layer that connects automated detection to authorised human response.

---

## User Personas

| Persona | Workstation | Primary function |
|---|---|---|
| ECC Director (IEBC Deputy CEO) | Executive console | Overall operational authority; Section 44A authorisation |
| ECC Operations Manager (Datacraft) | Operations console | Resource dispatch; kit-failure response; field coordination |
| E-SOC Analysts (x4) | Security workstations | Kit telemetry, network anomaly, SIEM alert triage |
| COC Analysts (x3) | Compliance workstations | Statutory compliance tracking; s.44A log; party-agent compliance |
| CAOC Analysts (x3) | Awareness workstations | Public information channels; rapid-response comms; myth-busting |
| County Coordinators (x47, remote) | Web console | County-level kit status; field-team dispatch |
| Field Support Engineers (x200, field) | Mobile app | Kit maintenance; geolocation; status updates |
| IAB Liaison | Read-only console | Sanitised real-time feed to IAB Dashboard (M15) |
| Journalist Desk (media-adjacent briefing room) | Sanitised public feed | Real-time status for accredited media |
| Party Agent Portal (read-only) | Web | Per-station status; transmission progress; SOC health |

---

## Functional Requirements

### FR-16.01 — Unified operator dashboard
Single web application with role-based views. Each workstation type has a curated view: E-SOC sees kit health + network + SIEM; COC sees compliance status + s.44A log + party-agent sign-in; CAOC sees media channels + public sentiment + myth-busting queue. All views share a common alert inbox with severity-classified notifications.

### FR-16.02 — Real-time kit telemetry
Ingests heartbeat telemetry from all 55,393 KIEMS kits every 15 minutes: online/offline status, battery level, GPS fix (vs geofence), firmware version, transmission queue depth, authentication throughput rate, biometric first-try success rate. Aggregated to county and national views. Per-station drill-down available within 2 clicks.

### FR-16.03 — Kit-failure escalation protocol (PM-04)
Pre-configured thresholds with automatic escalation:

| Threshold | Action | Authority |
|---|---|---|
| > 2% kits offline before 07:00 E-Day | ECC Level 2 activation; all county coordinators notified | ECC Operations Manager |
| > 5% kits offline | Automatic SMS notification to all accredited party agents + scheduled media briefing | ECC Director |
| > 10% kits offline | Complementary Mechanism declaration process initiated; Commissioner involvement | ECC Director + IEBC Chair |

Every threshold trigger is timestamped, logged, hash-chained, and HSM-signed.

### FR-16.04 — Section 44A workflow
Structured s.44A declaration process: (1) Presiding Officer reports via kit; (2) County coordinator verifies and escalates; (3) ECC confirms failure; (4) If kit unrecoverable within 30 minutes, ECC requests declaration from duty commissioner; (5) Commissioner issues declaration via ECC; (6) Presiding officer notified; (7) Party agents at station notified via SMS within 5 minutes. Every step timestamped and logged. SMS notification log is legally admissible evidence per Elections Act s.44A compliance requirement.

### FR-16.05 — Field engineer dispatch
GPS-tracked field engineer fleet management. ECC can locate nearest available engineer to a failing kit, dispatch via push notification, track en-route progress, and close ticket on arrival. 2-hour response-to-site SLA for L1 county spares; 30-minute response for air-freight emergency spares. Dispatch log is part of the IMS M20 audit trail.

### FR-16.06 — RTS transmission progress board
Live Form 34A transmission progress: per-constituency % transmitted, time-to-expected-close, stations with transmission queue backlog. Alerts when any constituency falls below 80% transmitted 2 hours after expected close. Form 34C aggregation progress shown.

### FR-16.07 — Party-agent portal (read-only)
Accredited party agents access a sanitised view: per-constituency kit-online %, RTS transmission %, alert count by severity (no operational details), s.44A event log. Updated every 5 minutes. No PII, no operational data that could aid an attacker.

### FR-16.08 — Sanitised journalist feed
Media-adjacent briefing room receives: national kit-online %, RTS transmission %, Form 34A published count, active alerts (count only), comparison to prior election benchmarks. Auto-updated slide deck for briefing room screens. Journalist-specific API endpoint for direct ingestion by NMG / Standard / Citizen data teams.

### FR-16.09 — Public status dashboard
Public-facing status page at `status.iebc.go.ke`: national kit-online %, RTS transmission progress bar, Form 34A publication count, last updated timestamp, system status (operational / degraded / incident). Served as signed static snapshot from CDN; updates every 5 minutes. Degraded-mode fallback: last known good snapshot with timestamp.

### FR-16.10 — Communication cadence management (PM-05)
Built-in meeting scheduler: daily standups from T-90 to T-30, twice-daily from T-30, hourly during 72-hour polling window. Auto-generates meeting agenda from current alert queue. Records are hash-chained and visible to IAB.

### FR-16.11 — Rapid-response comms cell (Part IV-A)
Pre-drafted holding statements for 12 scenario types. ECC Operations Manager can release a statement (requiring ECC Director counter-signature) via: IEBC Twitter/X (verified), WhatsApp official channel, email to registered media contacts, M13 app notification. All releases logged with author, timestamp, approval chain.

### FR-16.12 — Safety panic button routing (M13 integration)
Anonymous geo-tagged panic button submissions from M13 voter app route to CAOC workstation. Analyst triages: ignore (false positive), flag (notify County coordinator), escalate (notify KNPS + IEBC Chair). Response logged and visible to IAB.

### FR-16.13 — Observer access (sanitised SOC feed)
Accredited international observer missions (KEOG, ELOG, EU EOM, Carter Center, IFES) have read-only access to the sanitised observer dashboard: same content as party-agent portal plus a field-incident map (no PII). Observer access confirmed and logged; visible to IAB.

### FR-16.14 — Post-election debrief mode
After result announcement: ECC transitions to debrief mode. Stores all ECC events, dispatch logs, alert timelines, and comms in a signed archive. Generates post-election operational report within 7 days. Archive available to IAB and, via evidence request, to courts.

---

## Non-Functional Requirements

### NFR-16.01 — Availability
ECC platform: 99.99% during T-90 through T+30. **Active-active** deployment across primary (Nairobi DC) and mirror site — active-passive is insufficient for a hub with 14+ downstream dependents (M13, M17, M19, M20, M23, M26, M32, M33, M43, M46, M48, M56, M57, M60, M65). Both nodes serve write and read traffic; a failed primary must not leave the ECC dark during polling-window incidents. ECC platform failure does not affect kit operations (kits operate autonomously). Failover RTO ≤ 30 s.

### NFR-16.02 — Latency
Kit telemetry ingestion to dashboard display: p99 < 30 s. Alert notification to operator inbox: p99 < 10 s. Public status dashboard update: every 5 minutes.

### NFR-16.03 — Scalability
60,000 concurrent kit telemetry streams; 200 concurrent field-engineer mobile sessions; 10,000 concurrent party-agent portal users; 1M concurrent public dashboard views.

### NFR-16.04 — Security
ECC internal network: zero-trust mTLS. Every operator action logged with 5-tuple (who, what, which entity, from where, when). No external write access to ECC data from kit side. Party-agent and journalist portals are read-only at API gateway level. Sanitised feeds contain no operational data usable by an adversary.

### NFR-16.05 — Audit
Every ECC action (dispatch, escalation, statement release, s.44A workflow step) appended to HSM-signed audit log. Log is read-accessible to IAB in real time via M15.

---

## User Stories

1. As an ECC Operations Manager, I want to see a real-time map of every kit's status, so that I can identify and respond to failure clusters before they affect voter access.
2. As the ECC Director, I want to initiate the s.44A declaration workflow with a single authenticated action, so that the statutory process is followed consistently and documented automatically.
3. As a County Coordinator, I want to dispatch the nearest available field engineer with one click, so that kit failures are resolved within the SLA.
4. As a party agent, I want to see transmission progress for all stations in my constituency, so that I can identify stations that have not transmitted and raise concerns promptly.
5. As a journalist, I want to see the national kit-online percentage in real time, so that I can report accurately on election-day technology performance without relying on IEBC press releases.
6. As an ELOG observer, I want to access the sanitised observer dashboard, so that my organisation can publish an independent real-time assessment of system performance.
7. As the ECC Director, I want to release a pre-drafted holding statement with counter-signature, so that the response to an incident is fast, accurate, and auditable.
8. As an E-SOC Analyst, I want to receive anomaly alerts from M26 in my alert inbox, so that I can triage them and escalate to the ECC Director within the 15-minute SLA.
9. As a CAOC Analyst, I want to see voter panic-button submissions on a field map, so that I can route safety concerns to county coordinators immediately.
10. As the IAB Liaison, I want to confirm that my sanitised feed matches the IAB Dashboard data in M15, so that IAB monitoring is not dependent on ECC staff cooperation.

---

## Data Model Additions

```python
class KitTelemetrySnapshot(BaseModel):
    model_config = ConfigDict(extra='forbid')
    kit_serial: str
    station_code: str
    snapshot_at: datetime
    online: bool
    battery_pct: int
    gps_lat: float
    gps_lng: float
    geofence_ok: bool
    firmware_version: str
    auth_queue_depth: int
    auth_throughput_per_hour: int
    biometric_first_try_success_pct: float
    tx_queue_depth: int
    last_heartbeat_utc: datetime

class ECCAlert(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    source_module: str
    severity: str  # P0 | P1 | P2 | P3
    category: str  # kit_offline | threshold_breach | s44a | safety | comms | anomaly
    station_code: str | None
    constituency_code: str | None
    county_code: str | None
    description: str
    created_at: datetime
    acknowledged_by: str | None
    acknowledged_at: datetime | None
    resolved_at: datetime | None
    escalation_id: str | None

class S44AEvent(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    station_code: str
    kit_serial: str
    failure_reported_at: datetime
    county_verified_at: datetime | None
    ecc_confirmed_at: datetime | None
    recovery_attempted: bool
    recovered: bool
    declaration_requested_at: datetime | None
    declaration_issued_at: datetime | None
    commissioner_id: str | None
    party_agents_notified_at: datetime | None
    notification_sms_count: int
    complementary_mechanism_activated: bool
    signed_hash: str
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/ecc/dashboard` | Operator JWT (role) | 100/s | Role-filtered dashboard data |
| GET | `/api/v1/ecc/kits` | Operator JWT | 50/s | Kit telemetry, filterable |
| GET | `/api/v1/ecc/kits/{serial}` | Operator JWT | 20/s | Per-kit drill-down |
| GET | `/api/v1/ecc/alerts` | Operator JWT | 50/s | Alert inbox, filterable |
| POST | `/api/v1/ecc/alerts/{id}/acknowledge` | Operator JWT | 20/s | Acknowledge alert |
| POST | `/api/v1/ecc/s44a/initiate` | ECC Director JWT | 5/s | Start s.44A workflow |
| POST | `/api/v1/ecc/dispatch` | Operator JWT | 10/s | Dispatch field engineer |
| GET | `/api/v1/ecc/rts/progress` | Operator JWT | 50/s | Transmission progress |
| POST | `/api/v1/ecc/comms/release` | Director + countersig | 2/h | Release holding statement |
| GET | `/api/v1/party-portal/status` | Party agent JWT | 20/s per IP | Sanitised party portal |
| GET | `/api/v1/observer/status` | Observer JWT | 20/s per IP | Sanitised observer feed |
| GET | `/api/v1/journalist/feed` | Journalist token | 60/s | Journalist data API |
| GET | `/public/status` | None | 1000/s | Public status page data |
| SSE | `/api/v1/ecc/stream` | Operator JWT | 1 per session | Live event stream |

---

## Integration Points

**Upstream feeds:**
- M08 (RTS Device Client) — kit heartbeat telemetry
- M10 (RTS Back-End) — transmission progress, Form 34A status
- M17 (Speech Integrity) — hate-speech / disruption alerts
- M18 (Boundary Transparency) — boundary alert feed
- M19 (Queue Transparency) — per-station queue metrics, extended-hours triggers
- M20 (IMS) — kit custody and geofence events
- M13 (Public Verifier App) — safety panic button submissions
- M26 (Anomaly Detection, parallel agent) — statistical anomaly alerts
- P08 (Monitoring) — infrastructure health metrics

**Downstream:**
- M15 (IAB Dashboard) — sanitised real-time feed
- M23 (Grievance Channels, parallel agent) — voter complaint routing
- Field engineer mobile app — dispatch notifications
- SMS gateway — s.44A party-agent notifications
- IEBC Twitter/X, WhatsApp — comms releases
- Media briefing room screens — journalist feed
- NIS (National Intelligence Service) — intelligence briefing channel; ECC receives threat assessments via IEBC Secretariat liaison; NIS does not have direct system access; integration is human-in-the-loop (secure email + verbal briefing at T-7, T-3, T-1, and on election day); formal coordination MoU required pre-election

---

## UI/UX — Key Screens

1. **National overview** — map with county-level colour coding (kit health), RTS progress bar, active alert count, transmission count
2. **Kit explorer** — searchable/filterable table; per-kit drill-down; GPS map overlay; failure trend charts
3. **Alert inbox** — severity-sorted alert list; one-click acknowledge/escalate/dispatch; timer showing SLA countdown
4. **s.44A workflow** — step-by-step guided workflow with mandatory field validation; SMS preview before send; confirmation audit trail
5. **Field engineer tracker** — map of field engineers with GPS; nearest-to-failing-kit calculation; dispatch button
6. **Transmission progress** — per-constituency Gantt-style progress; ETA to 100%; red-flag for delayed constituencies
7. **Comms release** — pre-drafted statement selector; customisation fields; dual-signature requirement; release log
8. **Party agent portal** — separate login flow; read-only constituency view
9. **Journalist feed** — simplified national status with comparison to previous election benchmarks
10. **Public status page** — minimal, fast, CDN-served; green/amber/red system status

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| s.44A protocol with party-agent notification | Elections Act 2011 s.44A; ELOG 2017 report |
| Two-person integrity for critical decisions | IEMS security architecture |
| All actions logged and signed | Elections (Technology) Regulations 2017 reg. 12 |
| Observer access confirmation | Elections Act s.21A; IEBC accreditation regs |
| DPA 2019 — no PII in public/journalist feeds | DPA 2019 s.26 |
| Pre-agreed escalation ladder | PM-04 Strategy commitment |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| ECC activation by T-90 | 100% staffed and operational | IEBC Deputy CEO |
| Kit telemetry latency to dashboard | p99 < 30 s | Datacraft SRE |
| s.44A party-agent SMS notification | 100% within 5 min of declaration | ECC Director |
| Field engineer dispatch-to-site | 100% within 2 hours for county spares | Datacraft Ops |
| ECC platform availability during E-Day | >= 99.99% | Datacraft SRE |
| Public status page availability | >= 99.9% | Datacraft SRE |
| Active alert SLA: P0 acknowledged | 100% within 5 min | ECC Ops Manager |
| s.44A log completeness | 100% of declarations documented | IEBC Legal |

---

## Test Strategy

- **Unit:** Alert triage logic, s.44A workflow state machine, dispatch calculation (nearest engineer)
- **Integration:** Kit telemetry ingestion from M08; transmission progress from M10; anomaly feed from M26
- **Load:** 60,000 concurrent telemetry streams; 10,000 concurrent party-agent portal sessions
- **s.44A drill:** Full s.44A workflow rehearsed with real kits at T-60 dry-run election; all 5 steps timed and documented
- **Comms drill:** All 12 holding-statement scenarios rehearsed at T-30 tabletop exercise
- **Failover:** Primary ECC instance killed; secondary takes over within 5 minutes; operators resume from last state
- **Security:** Party-agent portal penetration test confirming zero write access; journalist feed data-leakage test confirming no PII

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| ECC software deployed to staging | 2027-04-01 | Internal testing |
| County coordinator training | 2027-05-01 | 47 coordinators onboarded |
| ECC activated (T-90) | 2027-05-11 | 24x7 staffing begins |
| Observer portal onboarding | 2027-06-01 (T-70) | All missions credentialed |
| Dry-run election monitoring | 2027-06-10 (T-60) | Full ECC ops exercise |
| Election Day operations | 2027-08-10 | Full ops |
| ECC petition-window mode | 2027-08-11 to 2027-09-07 | Evidence preservation |

---

## Dependencies

- **M08** (RTS Device Client) — telemetry source
- **M10** (RTS Back-End) — transmission progress
- **M13** (Public Verifier App) — panic button routing
- **M15** (IAB Dashboard) — sanitised feed consumer
- **M17-M19** — alert sources
- **M20** (IMS) — kit custody, geofence
- **M26** (Anomaly Detection) — statistical alerts
- **P04** (IAM) — role-based authentication for all ECC roles
- **P05** (Audit log) — event logging
- **P08** (Monitoring) — infrastructure health
- External: SMS gateway (Safaricom/Airtel); GPS tracking (field engineer devices)

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | ECC primary instance fails during peak | Hot-standby secondary ECC; automatic failover < 5 min; state replicated in real time |
| R2 | Operator error in s.44A workflow (premature or missing declaration) | Mandatory 3-step confirmation; second-officer counter-signature; pre-rehearsed at T-60 dry run |
| R3 | Party-agent portal access exploited to harvest operational intelligence | Read-only enforced at API gateway; sanitisation validated by independent pen test; no geolocation data in party portal |
| R4 | Comms release issued without proper counter-signature | Dual-signature enforced at API layer; no bypass path; audit log entry with both signatories |
| R5 | 200 field engineers insufficient for > 10% kit failure | Air-freight emergency spares pre-positioned; escalation to Section 44A complementary mechanism within 30-min SLA |
