# Medical first response (M60)

Module M60 (`medical-first-response`) owned by `training-svc`. Manages medical incident reporting and dispatch for election personnel on polling day, including field officer dispatch requests, polling-station cluster alerts when multiple incidents occur in proximity, and coordination with county health authorities. The schema enforces a strict no-PHI design: no personally identifiable health data is stored in IEMS.

## Contents

1. What Medical First Response does (vs M16 ECC)
2. Data plane (schema)
3. No-PHI schema enforcement
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Medical First Response does (vs M16 ECC)

- **M16** (Election Command Centre) = situational awareness for all election-day incidents; it receives escalated cluster alerts.
- **M60** = the operational layer that captures individual medical incidents at polling stations, coordinates immediate dispatch of medical response teams, and aggregates incidents into cluster alerts that M16 consumes.

M60 emits `eel.medical.cluster_alert.v1` events that M16 renders on its operational map. M16 does not store individual medical incident details.

## 2. Data plane

Three aggregates (per `services/training-svc/crates/medical/src/models.rs` and the M60 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **MedicalIncident** | Individual incident at a polling station: `incident_id`, `station_code`, `reported_by_officer_id`, `incident_type` (HEAT_EXHAUSTION / FAINTING / INJURY / CARDIAC / OTHER), `severity` (LOW / MEDIUM / HIGH / CRITICAL), `reported_at`, `status` (REPORTED / DISPATCHED / RESOLVED / TRANSFERRED), `transfer_destination` (county facility name, no patient ID), `resolved_at`. No patient name, national ID, or clinical details stored. | C |
| **DispatchRequest** | Request to county health / St John's / KRCS for a response unit: `dispatch_id`, `incident_id`, `dispatch_agency` (COUNTY_HEALTH / ST_JOHNS / KRCS / POLICE), `dispatched_at`, `eta_minutes`, `responded_at`. | C |
| **ClusterAlert** | Geographic cluster of incidents triggering ECC escalation: `cluster_id`, `constituency_code`, `incident_ids` (array), `cluster_radius_km`, `triggered_at`, `ecc_escalation_at`, `resolved_at`. Cluster detection uses DBSCAN (Ester et al., KDD 1996) over station GPS coordinates within a 30-minute rolling window. | B |

## 3. No-PHI schema enforcement

The schema is deliberately free of Protected Health Information. Design decisions:

- `MedicalIncident` stores only `incident_type` and `severity`, not clinical notes, diagnosis, or treatment records.
- `transfer_destination` is a facility name string, not a patient link; it does not identify the individual transferred.
- `reported_by_officer_id` identifies the reporting officer (a Commission staff member), not the patient.
- A Postgres `CHECK` constraint on `medical_incidents` enforces that columns for patient name, national ID, date of birth, and clinical notes are absent from the schema (enforced by confirming the constraint list in CI via `psql -c "\d medical_incidents"`).
- Data sharing with county health authorities is limited to incident location and severity; county facilities issue their own patient records outside IEMS.

## 4. Pipeline

Files on disk under `services/training-svc/crates/medical/src`: `lib.rs`, `models.rs`, `incident.rs`, `dispatch.rs`, `cluster.rs`, `routes/`.

```
field officer (mobile app M32 or M31) reports MedicalIncident
      -> validate: station_code exists in M18 boundary registry
      -> severity = CRITICAL: immediate SMS to county health emergency line
      -> create MedicalIncident with status REPORTED
      -> emit eel.medical.incident_reported.v1
      -> dispatch coordinator creates DispatchRequest
            -> agency selected based on station proximity + availability
            -> emit eel.medical.dispatch_requested.v1
            -> status updated to DISPATCHED
      -> response unit responds: DispatchRequest.responded_at filled
      -> incident resolved or transferred: status updated
      -> emit eel.medical.incident_resolved.v1

cluster detection (runs every 5 min via cron):
      -> DBSCAN scan over REPORTED / DISPATCHED incidents
         in past 30 min per constituency (Ester et al., KDD 1996)
      -> if cluster size >= 3 within 5 km radius:
            -> create ClusterAlert
            -> emit eel.medical.cluster_alert.v1 to ECC (M16)
```

## 5. API surface

Contract: `packages/openapi-contracts/training/medical.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/medical/incidents` | Field officer (JWT + SpiceDB `FIELD_OFFICER`) | Report medical incident |
| `GET /v1/medical/incidents` | Dispatch coordinator, ECC system | List incidents by station/status |
| `PATCH /v1/medical/incidents/{id}/status` | Dispatch coordinator | Update incident status |
| `POST /v1/medical/incidents/{id}/dispatch` | Dispatch coordinator (SpiceDB `DISPATCH_COORDINATOR`) | Create dispatch request |
| `PATCH /v1/medical/incidents/{id}/dispatch/{did}/respond` | Dispatch coordinator | Log dispatch response |
| `GET /v1/medical/clusters` | ECC system (SVID) | Active cluster alerts for M16 map |
| `POST /v1/medical/clusters/{id}/resolve` | Dispatch coordinator | Mark cluster resolved |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Medical dispatch console (LiveView) | `services/training-svc/ex/medical_web` | Real-time incident and dispatch management |
| ECC integration (M16) | Cluster alert feed via Kafka | ECC map overlay |
| Mobile field reporting (M32) | `services/field-mobile-svc/` | Field officers report incidents from mobile app |

Languages: EN + SW. WCAG 2.2 AA on the dispatch console.

## 7. Integrity + anti-tamper

- **No-PHI constraint in CI**: the test suite includes an integration test that connects to the testcontainer DB and asserts the column list of `medical_incidents` contains no PHI columns; this test fails the build if a migration inadvertently adds a prohibited column.
- **CRITICAL incident SMS**: Twilio (or Africa's Talking) outbound SMS to county health emergency line is fired synchronously before the incident row is committed; if SMS fails, the commit is rolled back and the officer is shown an error to re-attempt.
- **Cluster alert SLA**: Prometheus alert fires if `eel.medical.cluster_alert.v1` is not delivered to ECC within 60 seconds of `triggered_at`.
- **Audit shadow**: all incident and dispatch mutations logged to `eel_training_audit_shadow` with officer identity and timestamp.
- **DBSCAN stability**: the cluster detection job is idempotent; re-runs within the same 5-minute window produce the same clusters because DBSCAN with fixed epsilon and min-samples is deterministic over the same input set.

## 8. Kenya statute alignment

- **Kenya Data Protection Act §25 (lawful processing)**: medical incident data (type + severity) is processed under legitimate interest (election safety); no sensitive personal data is processed, so no ODPC notification is required for M60 specifically.
- **Kenya Data Protection Act §26 (minimisation)**: the no-PHI design means no health data about identifiable individuals is collected; the incident record is about the event at a station, not the individual patient.
- **Occupational Safety and Health Act 2007 §13 (employer duty of care)**: the Commission's duty of care to election-day workers is partially discharged through M60's dispatch coordination capability.
- **Public Health Act Cap 242 §36 (disease notification)**: if a cluster alert involves a communicable disease concern, the dispatch coordinator must notify the county public health officer via the standard MoH channels; M60 does not automate this notification, as it has no clinical diagnosis capability.

## 9. Honest caveats + external prerequisites

- County health emergency line SMS depends on a commercial SMS gateway (Africa's Talking or Twilio) with the Commission account credentials. The gateway and short sender ID must be provisioned before election day.
- Dispatch agency selection is manual: the system shows proximity and agency availability, but the dispatch coordinator selects the agency. Fully automated dispatch is not implemented.
- DBSCAN cluster detection runs every 5 minutes via cron; this means a cluster that forms and resolves within 5 minutes may not trigger an alert. The 5-minute window is a deliberate trade-off between alerting latency and false positive rate.
- GPS coordinates for polling stations come from M18 (boundary registry). Stations without GPS coordinates (approximately 3% of stations in M18 at last count) are excluded from cluster detection; this is a known gap.
- No integration with county ambulance dispatch systems (911 / 999 equivalent); all dispatch is via phone and is manually logged.

## 10. Test coverage + operational runbooks

Coverage under `services/training-svc/crates/medical/tests/`: no-PHI schema assertion (integration, fails build if PHI columns present), CRITICAL incident SMS (pytest-httpserver mock of SMS gateway), DBSCAN cluster detection (property-based, various station distributions), cluster alert ECC round-trip (integration), 5-minute cron regression (testcontainers + time-travel test using `tokio::time::advance`).

Runbooks under `docs/runbooks/`:

- `training-svc.md` (section: medical-first-response) -- deploy, rollback, SMS gateway failover.
- `cluster-alert-ecc-failure.md` -- manual ECC notification when Kafka is unavailable.
- `sms-gateway-outage.md` -- fallback to officer phone call when SMS gateway is unreachable.

Related documents: full spec at `docs/plans/modules/medical-first-response.md`; service capability catalogue at `services/training-svc/docs/CAPABILITIES.md`; M16 ECC at `docs/features/election-command-centre.md`; M32 field mobile at `docs/features/mobile-field-support.md`.

### DBSCAN cluster parameters

```
epsilon (eps)  = 5 km          (geographic radius)
min_samples    = 3             (minimum incidents to form a cluster)
time_window    = 30 min        (rolling window for incident inclusion)
scan_interval  = 5 min         (cron cadence)
```

Station GPS coordinates are sourced from M18 boundary registry at the start of each polling day and cached in `services/training-svc/caches/station_coords.json`; the cache is refreshed every 24 hours. DBSCAN uses the Haversine distance metric (Sinnott, Sky and Telescope 1984) for geographic distance calculation.

### Dispatch agency priority

When multiple agencies are available, the dispatch coordinator selects based on:

1. St John's Ambulance -- preferred for polling station medical emergencies (nationwide presence).
2. Kenya Red Cross Society (KRCS) -- secondary; activated when St John's is unavailable in the constituency.
3. County health facility -- tertiary; always available as last resort but not mobile-dispatched.
4. Police -- only for incidents with a security component (e.g., violent altercation with injury).

The priority is a guideline, not enforced by the system; the dispatch coordinator has full discretion.
