# M60 — Medical and First-Response Coordination

**Module ID:** M60
**Name:** Medical and First-Response Coordination Interface
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part II §PM-13, Part V §V.4, Part IX §IX.2.G2, Part XXI.A.19

---

## 1. Purpose

General elections concentrate large numbers of voters, polling officials, and observers at thousands of sites simultaneously. Medical emergencies, crowd injuries, and epidemic surveillance signals during this concentration require a coordinated interface between IEBC election operations and the Ministry of Health (MoH), the National Disaster Management Unit (NDMU) under the Ministry of Interior, and the Kenya Red Cross Society (KRCS). This module is not a health information system. It is an election-operations coordination layer: station-level medical-incident reporting by IEBC field staff, a dispatch-coordination interface with KRCS ambulance units, epidemic-surveillance signal forwarding to MoH during the gathering window, and a cross-agency incident register that feeds situational awareness to M16 (ECC). Datacraft facilitates the interface; health management decisions remain with MoH, NDMU, and KRCS under their statutory mandates.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| IEBC Field Medical Liaison | Per-county first-response coordinator | RW on county incidents; read dispatch status |
| IEBC County Coordinator | Oversees all county operations | Read county incidents; escalate |
| M16 ECC Operator | Situational awareness monitor | Read incident feed (SSE) |
| MoH Surveillance Officer | Epidemic signal consumer | Read M60 surveillance signals only |
| NDMU Liaison | Disaster-management coordination | Read incidents; update NDMU response status |
| KRCS Dispatch Coordinator | Ambulance and first-aid dispatch | Read incident location; update dispatch status |
| IEBC Director of Operations | Oversight | Read-all; approve escalation |

---

## 3. Functional Requirements

**FR-M60-01 — Pre-election medical asset registration:** KRCS registers pre-positioned first-aid teams and ambulance units by county and sub-county before election day. Each unit: unit ID, unit type (first-aid team, ambulance, field hospital), GPS location, contact number, capacity. Registration closes at T-2. Read-only access for IEBC field coordinators.

**FR-M60-02 — Polling-station medical-incident reporting:** IEBC field staff report medical incidents at polling stations via M31 Mobile Poll Clerk app (integration point) or M60 web interface. Each incident: polling station code, incident type (cardiac, trauma, heat, crowd crush, other), severity (minor, moderate, critical), number of persons affected, timestamp. No personal health data of voters recorded — aggregate count and severity only.

**FR-M60-03 — KRCS dispatch coordination:** Each moderate/critical incident triggers a dispatch request visible to the county KRCS Dispatch Coordinator. KRCS updates dispatch status (en route, on scene, resolved). Estimated time of arrival (ETA) displayed to IEBC Field Medical Liaison. Dispatch confirmation and resolution timestamps logged.

**FR-M60-04 — NDMU escalation interface:** Critical incidents or multi-site cluster events (3+ moderate incidents in one sub-county within 2 hours) auto-generate an escalation notice to the county NDMU Liaison. NDMU Liaison acknowledges and updates response status. Unacknowledged escalations within 30 minutes re-escalate to IEBC Director of Operations and M16 ECC.

**FR-M60-05 — Epidemic surveillance signal forwarding:** During the 3-day election window (T-1 to T+1), aggregate incident counts by type per county are forwarded to MoH surveillance system at 2-hour intervals. Signal format: county code, incident-type breakdown, total persons affected, timestamp. No individual voter or officer health records forwarded. MoH uses this for situational awareness only; clinical decisions remain with MoH.

**FR-M60-06 — Real-time incident feed to M16 ECC:** All open incidents pushed via SSE to M16 ECC as a medical-overlay layer. ECC operators see incident count per polling station and county; severity colour-coded. No individual data in ECC view.

**FR-M60-07 — Post-election medical summary report:** Automated report generated at T+1: total incidents by type and severity per county, KRCS dispatch count, average ETA, unresolved incidents, MoH signals sent. Report submitted to IEBC Director of Operations and shared with MoH + KRCS coordinators as part of post-election debrief.

**FR-M60-08 — Pre-election coordination calendar:** Joint calendar of pre-election coordination meetings with MoH, NDMU, and KRCS: tabletop exercises, pre-position confirmation, communication-protocol rehearsals. Minutes and action items tracked.

---

## 4. Non-Functional Requirements

**NFR-M60-01 — Performance:** Incident submission p99 < 1 s (field conditions, low-bandwidth tolerant). Dispatch request visible to KRCS p99 < 15 s from incident submission.

**NFR-M60-02 — Reliability:** 99.5% uptime T-3 to T+1. Offline-capable incident submission (M31 integration) with sync on reconnect.

**NFR-M60-03 — Security:** MoH and KRCS users receive time-limited IEBC-issued read-scoped credentials. No voter PII in any record. NDMU access to incident list only, not dispatch communications.

**NFR-M60-04 — WCAG 2.2 AA:** IEBC field web interface. M31 mobile integration covered by M31's own WCAG compliance.

**NFR-M60-05 — DPA 2019 lawful basis:** Incident records contain no individual personal data (aggregate severity counts only). Processing under s.30(1)(d) (public interest — public health and election safety). Retention: 3 years post-election. DPIA filed with ODPC before T-90.

**NFR-M60-06 — Scale:** Up to 47,000 polling stations, up to 50 incidents per station per day in worst case. System must handle 10,000 concurrent incident submissions on election morning.

**NFR-M60-07 — SLA:** KRCS dispatch request visible within 15 s. NDMU escalation sent within 5 minutes of cluster trigger. ECC SSE feed latency p99 < 30 s.

**NFR-M60-08 — Language:** EN + SW for all field-facing screens and KRCS/MoH interfaces.

---

- **NFR-60-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-60` (replace 60 with the module number); alert ruleset ID `iems-60-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As an IEBC Field Medical Liaison, I report a cardiac arrest at a polling station from my phone and KRCS dispatch sees the request within 15 seconds.
2. As a KRCS Dispatch Coordinator, I assign the nearest ambulance to the incident and update the ETA visible to the IEBC field liaison.
3. As an IEBC County Coordinator, I see a cluster alert when three moderate incidents appear in the same sub-county within two hours, triggering automatic NDMU escalation.
4. As an NDMU Liaison, I acknowledge the escalation notice and update my response status so IEBC Director of Operations knows the incident is being managed.
5. As an M16 ECC Operator, I view the medical incident overlay on the county map and see severity colour-codes, so I can prioritise where to direct field support.
6. As a MoH Surveillance Officer, I receive the 2-hourly county aggregate signal during the election window and flag an unusual trauma cluster for further investigation.
7. As the IEBC Director of Operations, I view the post-election medical summary report and forward it to MoH and KRCS as part of the debrief package.
8. As an IEBC Field Medical Liaison, I submit an incident offline when network is unavailable; the incident syncs to the central system within 2 minutes of reconnection.
9. As a KRCS Dispatch Coordinator, I mark an incident as resolved after my team departs the scene, and the resolution timestamp is logged.
10. As the IEBC Director of Operations, I confirm all pre-positioned KRCS units are registered and located per the pre-election plan at T-1.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime
from typing import Literal

class MedicalUnit(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    unit_type: Literal["first_aid_team","ambulance","field_hospital"]
    county_code: str
    sub_county_code: str
    gps_lat: float
    gps_lon: float
    contact_number: str
    capacity_persons: int
    registered_by: str       # KRCS user
    registered_at: datetime
    active: bool

class MedicalIncident(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    polling_station_code: str
    county_code: str
    sub_county_code: str
    incident_type: Literal["cardiac","trauma","heat_exhaustion","crowd_crush","respiratory","other"]
    severity: Literal["minor","moderate","critical"]
    persons_affected: int    # aggregate count only
    reported_by: str         # IEBC field officer ID
    reported_at: datetime
    status: Literal["open","dispatch_requested","dispatch_en_route","on_scene","resolved"]
    krcs_unit_id: str | None
    dispatch_requested_at: datetime | None
    eta_minutes: int | None
    on_scene_at: datetime | None
    resolved_at: datetime | None

class NDMUEscalation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    trigger_type: Literal["critical_incident","cluster_alert"]
    linked_incident_ids: list[str]
    county_code: str
    escalated_at: datetime
    ndmu_liaison_id: str | None
    acknowledged_at: datetime | None
    ndmu_response_status: str | None
    reescalated_at: datetime | None

class EpidemicSignal(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    county_code: str
    signal_period_start: datetime
    signal_period_end: datetime
    incident_counts: dict[str, int]  # incident_type -> count
    total_persons_affected: int
    forwarded_to_moh_at: datetime
    moh_acknowledged: bool
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/api/v1/medical/incidents` | IEBC_FIELD JWT | 500/min | Submit incident |
| GET | `/api/v1/medical/incidents` | IEBC_COUNTY or KRCS JWT | 200/min | List / filter incidents |
| PATCH | `/api/v1/medical/incidents/{id}/dispatch` | KRCS JWT | 100/min | Update dispatch status and ETA |
| PATCH | `/api/v1/medical/incidents/{id}/resolve` | KRCS JWT | 100/min | Mark resolved |
| GET | `/api/v1/medical/incidents/stream` | M16 SERVICE JWT | SSE | Real-time ECC feed |
| GET | `/api/v1/medical/units` | IEBC JWT | 60/min | List registered KRCS units |
| POST | `/api/v1/medical/units` | KRCS JWT | 200/hr | Register medical unit |
| GET | `/api/v1/medical/escalations` | NDMU JWT | 60/min | NDMU escalation queue |
| POST | `/api/v1/medical/escalations/{id}/acknowledge` | NDMU JWT | 30/min | NDMU acknowledge escalation |
| GET | `/api/v1/medical/surveillance/signal` | MOH JWT | 30/min | Latest epidemic signals |
| GET | `/api/v1/medical/reports/post-election` | IEBC_OPS JWT | 10/hr | Post-election summary report |

---

## 8. Integrations

- **M31 Mobile Poll Clerk** — incident submission from field devices (offline-capable sync)
- **M16 Election Command Centre** — SSE medical incident overlay
- **M56 County Coordination** — county health-facility location data (pre-positioned unit siting)
- **MoH surveillance system** (external, MoU) — 2-hourly epidemic signal forwarding
- **NDMU** (external, MoU) — escalation notification endpoint
- **KRCS dispatch system** (external or M60-hosted) — dispatch coordination interface
- **P05 Audit Log** — all incident and dispatch events logged

---

## 9. UI/UX Key Screens

1. **Field Incident Report Form** — minimal fields for fast entry: station code (auto-prefilled from field login), incident type, severity, persons affected; submit button; offline-queued indicator.
2. **County Medical Dashboard** — live incident map; severity colour layer; open vs resolved count; cluster alert banner.
3. **KRCS Dispatch Console** — incoming dispatch requests; unit assignment; ETA entry; on-scene and resolved confirmation.
4. **NDMU Escalation Queue** — open escalations with time-since-trigger countdown; acknowledge button; response-status update.
5. **MoH Signal Feed** — county aggregate table; 2-hour interval view; download CSV.
6. **Post-Election Summary Report** — auto-generated PDF with county-by-county breakdown; KRCS response statistics; unresolved incidents.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Election-day safety of voters and staff | IEBC Act 2011 s.4(j); Constitution Art. 83(3) |
| NDMU mandate for disaster coordination | National Disaster Response Plan 2021; National Policy for Disaster Management 2011 |
| MoH epidemic surveillance obligation | Public Health Act Cap 242 s.16 |
| KRCS cooperation | Kenya Red Cross Society Act Cap 256 |
| DPA 2019 lawful basis | DPA 2019 s.30(1)(d) — public interest / public health |
| No individual health data processing | DPA 2019 s.31(1) — sensitive data prohibition unless lawful basis |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Dispatch request visible to KRCS | p99 < 15 s | Datacraft SRE |
| NDMU escalation sent from cluster trigger | < 5 minutes | Datacraft SRE |
| ECC SSE incident feed latency | p99 < 30 s | Datacraft SRE |
| MoH 2-hourly signals delivered on schedule | 100% during election window | Datacraft SRE |
| Post-election summary report generated | By T+2 | IEBC Director of Operations |
| Incident system availability T-3 to T+1 | >= 99.5% | Datacraft SRE |

---

## 12. Test Strategy

- **Unit:** Cluster-alert trigger logic (3 moderate incidents, 2-hour window, same sub-county); epidemic-signal aggregation; dispatch ETA computation.
- **Integration:** M31 offline sync; M16 SSE feed; MoH signal API; NDMU escalation endpoint.
- **Load:** 10,000 concurrent incident submissions (election-morning scenario); verify all incidents processed within 60 s.
- **Chaos:** KRCS dispatch system offline; verify IEBC field liaison sees degraded-mode alert; incidents continue to log; dispatch resumes on reconnect.
- **Access control:** MoH can read surveillance signals only; KRCS cannot access NDMU escalation queue.
- **Offline sync:** Submit 50 incidents with no connectivity; reconnect; verify all appear in system with correct timestamps.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| MoU templates with MoH, NDMU, KRCS | T-365 (2026-10-10) | Legal sign-off |
| Pre-position database build | T-90 (2027-05-10) | KRCS unit registration open |
| Tabletop exercise with MoH + KRCS + NDMU | T-60 (2027-06-11) | End-to-end incident drill |
| KRCS unit pre-position confirmed | T-2 (2027-08-08) | Registration locked |
| Election-day live | T-0 (2027-08-10) | All feeds active |
| Post-election summary report | T+2 (2027-08-12) | Report generated and shared |

---

## 14. Dependencies

- **M31 Mobile Poll Clerk** — offline incident submission
- **M16 ECC** — medical overlay consumer
- **M56 County Coordination** — health-facility location data
- **P05 Audit Log** — event logging
- MoU with MoH, NDMU, KRCS — required before T-365

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | KRCS dispatch coordinator not monitoring portal on election day | Dual-channel alert: M60 portal + SMS to KRCS duty phone; IEBC Field Medical Liaison has KRCS regional phone number as fallback |
| R2 | Field officer network outage prevents incident submission | M31 offline cache syncs on reconnect; county coordinator can submit on behalf of offline field officer via M60 web interface |
| R3 | MoH surveillance system API unavailable during election window | M60 queues signals and retries every 15 minutes; signals stored locally; manual email dispatch if queue exceeds 6 hours |
| R4 | Personal health data inadvertently captured in incident description field | Free-text field scanned for PII patterns (name, ID number) before storage; field stripped with alert to submitter |
| R5 | NDMU escalation ignored during active disaster event | Re-escalation to IEBC Director of Operations at 30-minute mark; Director has direct NDMU Director-General contact; Datacraft SRE monitors escalation acknowledgement SLA |

---

## 16. Cross-References

- **M16** — ECC: medical incident overlay
- **M31** — Mobile Poll Clerk: offline incident submission integration
- **M56** — County Coordination: health-facility siting data
- **M23** — Grievance Channels: medical complaints by voters (distinct from operational incidents)

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| Election-day first-response coordination | FR-M60-02, FR-M60-03, FR-M60-04 |
| Pre-positioning of medical assets | FR-M60-01, FR-M60-08 |
| Epidemic surveillance signal to MoH | FR-M60-05 |
| Situational awareness integration with ECC | FR-M60-06 |
| Post-election health debrief | FR-M60-07 |
| DPA 2019 compliance — no individual health data | NFR-M60-05, FR-M60-02 |

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
