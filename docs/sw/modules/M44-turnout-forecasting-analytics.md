# M44 — Turnout Forecasting & Analytics

**Module ID:** M44
**Name:** Turnout Forecasting & Analytics
**Runs on:** Central analytics backend (Kenyan cloud) + public dashboard
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

---

## 1. Purpose

Pre-election turnout forecasting from CVR demographics, historical turnout, macro signals (weather, transport), and polling. Election-day turnout tracking with variance alerts. Post-election analytical dataset for M41 post-election review and academic / journalist access. Integrates anomaly signals with M41 (integrity review) per Strategy §XIX.1.

Strategy reference: §XIX.1 (Anomaly Detection); §XIX.3 (Post-Election Review). Data Protection Act 2019.

---

## 2. User Personas

- **the Commission Operations Analyst** — monitors election-day turnout trajectory; receives variance alerts
- **the Commission Commissioner** — views executive summary dashboard; acts on low-turnout alerts
- **M16 Election Command Centre Operator** — sees live turnout feed as operational context layer
- **Journalist / Media** — reads public constituency-level dashboard; downloads aggregated data
- **Academic / Researcher** — requests post-election dataset via data-access API (registration required)
- **M41 Post-Election Review Officer** — queries turnout analytics for integrity correlations
- **Presiding Officer** — submits hourly turnout count (feeds live tracking)

---

## 3. Functional Requirements

### Pre-Election Forecasting

- **FR-M44-001** Ingest CVR demographic extracts per constituency (age-band, gender, disability flag, registration date) from M02 BVR backend; no individual records retained beyond the aggregation step.
- **FR-M44-002** Ingest historical turnout per constituency (2013, 2017, 2022 general elections + by-elections from M50) as a baseline time series.
- **FR-M44-003** Ingest macro signals: KMD weather forecast per county (API), constituency-level road-accessibility index (KURA static dataset), registered-voter density per polling station.
- **FR-M44-004** Train and serve a constituency-level turnout forecast model (gradient-boosted ensemble; retrained each election cycle). Outputs: point estimate + 80% / 95% prediction interval per constituency.
- **FR-M44-005** Forecast published to public dashboard T-30, T-14, T-7 before election day; each publication versioned and archived.
- **FR-M44-006** Forecast methodology document published alongside each release (plain-language EN + SW).

### Election-Day Turnout Tracking

- **FR-M44-007** Presiding Officers submit hourly voter count per polling station via M08 RTS device (structured field in existing hourly report); data flows to M44 aggregation pipeline.
- **FR-M44-008** Constituency-level turnout computed every hour; compared against forecast trajectory.
- **FR-M44-009** Variance alert: if observed turnout deviates from forecast by > 2 standard deviations (or > 15 percentage points) at any constituency at any hour mark, generate an alert to M16 Command Centre and the Commission Operations Analyst within 5 minutes.
- **FR-M44-010** Public turnout dashboard updated every 30 minutes during polling hours (07:00-17:00 EAT); constituency-level only; no station-level data during polling hours.
- **FR-M44-011** Final turnout figure per polling station published after polls close and RO Form 34A data is received; reconciled against CVR registered voter count.

### Post-Election Analytics

- **FR-M44-012** Post-election dataset: per-constituency turnout, demographic breakdown (age-band, gender), weather index, historical delta, anomaly flags. Available for download T+7 via registered data-access API.
- **FR-M44-013** Journalist API: read-only; aggregated per constituency; no individual voter data; rate-limited.
- **FR-M44-014** Academic dataset API: same schema as public; additional sub-county breakdown available on request with the Commission DPO approval; DPA 2019 s.30(1)(e) (research) lawful basis; data-sharing agreement required.
- **FR-M44-015** Anomaly correlation report: cross-reference turnout anomaly flags with M41 post-election integrity events; delivered to M41 Review Officer as a structured JSON report within T+30.

---

## 4. Non-Functional Requirements

- **NFR-M44-001** Forecast model training pipeline completes within 4 hours on updated CVR extract.
- **NFR-M44-002** Election-day aggregation pipeline: hourly turnout computed within 10 minutes of Presiding Officer submission deadline.
- **NFR-M44-003** Variance alert delivery: <= 5 minutes from trigger.
- **NFR-M44-004** Public dashboard p99 <= 2 s under 50,000 concurrent users during polling hours.
- **NFR-M44-005** WCAG 2.2 AA for public dashboard; EN + SW.
- **NFR-M44-006** DPA 2019: individual CVR records aggregated before storage in M44 schema; no individual voter data stored; academic dataset governed by data-sharing agreement; DPIA registered.
- **NFR-M44-007** Forecast model versioned with training dataset hash and feature list stored; reproducible from archived inputs.
- **NFR-M44-008** Post-election dataset available T+7 (7 days after election day).
- **NFR-M44-009** Availability of public dashboard >= 99.5% during polling hours.

---

- **NFR-44-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-44` (replace 44 with the module number); alert ruleset ID `iems-44-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Commission Operations Analyst, I see a live constituency-level map on election day showing turnout trajectory vs forecast, with red alerts for constituencies deviating > 15 percentage points, so I can escalate resource deployment.
2. As a journalist, I can view and download hourly turnout figures per constituency during election day without creating an account.
3. As a Commission Commissioner, I receive a push notification when any county-level turnout drops more than 2 standard deviations below forecast at the 12:00 mark.
4. As a presiding officer, my hourly voter count submitted on the M08 device is automatically factored into the national turnout dashboard without any additional step.
5. As an academic researcher, I apply for the post-election sub-county dataset and receive access within 5 working days of DPO approval.
6. As the M41 Post-Election Review Officer, I receive a structured anomaly-correlation report linking turnout variance flags to form-verification events.
7. As a member of the public, I read the forecast methodology document published 30 days before election day, which explains in plain Swahili how the model was built.
8. As an M16 Command Centre operator, I see the turnout anomaly feed as a layer on the operational map, so I can link deployment decisions to predicted low-turnout areas.
9. As a researcher, I can verify that the published post-election dataset matches the Merkle-anchored aggregation snapshot.
10. As a Commission IT analyst, I can replay the forecast model with updated inputs on any past election-cycle dataset for audit purposes.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import date, datetime
from enum import Enum

class ForecastRelease(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    election_id: str
    published_at: datetime
    model_version: str
    training_dataset_hash: str       # SHA-256 of CVR extract used
    constituency_forecasts: list[str]  # FK list to ConstituencyForecast

class ConstituencyForecast(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    forecast_release_id: str
    constituency_code: str
    registered_voters: int
    forecast_turnout_pct: float
    interval_80_low: float
    interval_80_high: float
    interval_95_low: float
    interval_95_high: float

class HourlyTurnoutReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    polling_station_code: str
    constituency_code: str
    report_hour: int                 # 7-17 EAT
    voters_turned_out: int
    submitted_at: datetime
    presiding_officer_id: str

class TurnoutVarianceAlert(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    constituency_code: str
    alert_hour: int
    observed_pct: float
    forecast_pct: float
    deviation_pct: float
    sigma_deviation: float
    alerted_at: datetime
    acknowledged_by: str | None
    acknowledged_at: datetime | None
```

---

## 7. API (OpenAPI 3.1 excerpt)

```yaml
openapi: "3.1.0"
info:
  title: M44 Turnout Forecasting API
  version: "1.0.0"
paths:
  /api/v1/forecasts/latest:
    get:
      summary: Get latest pre-election forecast (all constituencies)
      responses:
        "200": { description: "Array of ConstituencyForecast" }
  /api/v1/turnout/live:
    get:
      summary: Live election-day turnout by constituency (SSE stream)
      responses:
        "200":
          content:
            text/event-stream:
              schema:
                type: object
                properties:
                  constituency_code: { type: string }
                  current_pct: { type: number }
                  report_hour: { type: integer }
  /api/v1/turnout/alerts:
    get:
      summary: Active variance alerts (the Commission internal)
      security: [{ BearerAuth: [operations_analyst, command_centre] }]
      responses:
        "200": { description: "Array of TurnoutVarianceAlert" }
  /api/v1/datasets/post-election:
    get:
      summary: Download post-election turnout dataset
      parameters:
        - name: format
          in: query
          schema: { type: string, enum: [csv, json] }
      security: [{ BearerAuth: [public, journalist, academic] }]
      responses:
        "200": { description: "Constituency-level post-election dataset" }
        "403": { description: "Academic sub-county requires DPO approval" }
```

---

## 8. Integrations

| System | Direction | Purpose |
|---|---|---|
| M02 BVR Backend | Inbound | CVR demographic aggregate extract |
| M08 RTS Device | Inbound | Hourly Presiding Officer turnout reports |
| M16 Election Command Centre | Outbound | Turnout layer + variance alerts |
| M41 Post-Election Review | Outbound | Anomaly-correlation report |
| M50 By-Election Module | Inbound | By-election historical turnout data |
| KMD Weather API | Inbound | County weather forecast for pre-election model |
| P05 Merkle Log | Outbound | Dataset publication hash |

---

## 9. UI/UX

- Public dashboard: choropleth map (constituency level) with animated hourly update; bar chart vs forecast; download button (CSV/JSON); methodology document link. No login required.
- the Commission internal dashboard: same map with variance alert overlays; alert panel with acknowledge button; M16 integration layer toggle.
- Forecast report page: data table with confidence intervals; chart per region; plain-language methodology section (EN + SW).
- All public screens WCAG 2.2 AA; EN + SW toggle; responsive mobile layout.

---

## 10. Compliance Mapping

| Requirement | Provision | Implementation |
|---|---|---|
| Individual voter data protection | DPA 2019 s.26 | CVR data aggregated before entering M44; no individual records |
| Research data sharing | DPA 2019 s.30(1)(e) | Data-sharing agreement required for academic sub-county dataset |
| Transparency of forecast methodology | the Commission Regulations | Methodology document published with each forecast release |
| Anomaly detection linkage | Strategy §XIX.1 | Variance alerts fed to M41 integrity pipeline |

---

## 11. Success Metrics + SLA

| Metric | Target |
|---|---|
| Forecast mean absolute error (constituency level) | <= 5 percentage points vs actual |
| Variance alert delivery time | <= 5 minutes from trigger |
| Public dashboard update frequency (election day) | Every 30 minutes |
| Post-election dataset availability | T+7 |
| Public dashboard p99 (polling hours) | <= 2 s |
| System availability during polling hours | >= 99.5% |

---

## 12. Test Strategy

- Unit: forecast model pipeline; variance alert computation; aggregation logic.
- Integration: M08 hourly report ingestion; M16 alert push; M41 report generation.
- Load: 50,000 concurrent public dashboard reads during polling hours.
- Back-test: run model against 2022 election data; evaluate MAE vs published turnout.
- UAT: the Commission Operations Analysts; M16 operators; pilot journalist access.

---

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date |
|---|---|
| CVR demographic aggregate extract agreed with M02 team | T-270 (2026-11-13) |
| Model training pipeline built + 2022 back-test complete | T-180 (2027-02-11) |
| Public dashboard UAT | T-90 (2027-05-12) |
| First public forecast published | T-30 (2027-07-11) |
| T-14 forecast update | T-14 (2027-07-27) |
| T-7 final forecast update | T-7 (2027-08-03) |
| Election-day live dashboard active | T-0 07:00 EAT |
| Post-election dataset published | T+7 (2027-08-17) |

---

## 14. Dependencies

- M02 BVR backend aggregate extract API available (T-270).
- M08 RTS device firmware updated to include hourly turnout field (T-90).
- M16 Election Command Centre alert-receive endpoint ready (T-60).
- M41 Post-Election Review anomaly-report ingest API ready (T+7).
- KMD weather API access agreement (T-180).

---

## 15. Risks + Mitigations (Top 5)

| Risk | Likelihood | Mitigation |
|---|---|---|
| Presiding Officers skip hourly turnout reports | High | M08 device prompts at each hour mark; RO supervisor receives non-submission alert |
| Forecast model underperforms in low-registration constituencies | Medium | Widen prediction intervals; flag low-sample constituencies explicitly on dashboard |
| Public dashboard overwhelmed during poll-close rush | High | CDN caching of 30-min snapshots; rate limiting on dataset downloads |
| Weather API unavailable at T-7 | Low | KMD static seasonal-average fallback dataset pre-loaded |
| Academic data sharing request exceeds DPO capacity | Medium | Batch DPO review slots published in advance; standard data-sharing agreement template reduces processing time |

---

## 16. Cross-References

- `M02-bvr-backend.md` — CVR demographic aggregate source
- `M08-rts-device.md` — hourly turnout submission channel
- `M16-election-command-centre.md` — alert consumer
- `M41-post-election-review.md` — anomaly-correlation report consumer
- `M50-by-election.md` — historical by-election turnout source
- `P05-merkle-audit.md` — dataset publication hash

---

## 17. Strategy Traceability

| Strategy Section | Requirement | Module Response |
|---|---|---|
| §XIX.1 Anomaly Detection | Turnout variance as integrity signal | FR-M44-009, FR-M44-015 |
| §XIX.3 Post-Election Review | Analytical dataset for review | FR-M44-012 to FR-M44-015 |
| §XVIII.4 Transparency | Public turnout dashboard | FR-M44-010, FR-M44-005 |
| DPA 2019 | Individual data protection in analytics | NFR-M44-006 |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if the Commission terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and the Commission DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to the Commission ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** the Commission ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
