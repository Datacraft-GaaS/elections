# M66 — Election Financial Forensics

**Module ID:** M66
**Name:** Election Financial Forensics
**Owner:** Datacraft Limited — Analytics Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Strategy §XX.5; M22 Campaign Finance Transparency integration

---

## Purpose

Post-election published analytics estimating cost-per-vote conversion ratios by candidate, constituency, and electoral cycle. Combines M22 disclosed campaign spending, FRC aggregate cash-movement data, KEPSA field estimates, and demographic/spatial covariates. Applies econometric methods (regression discontinuity, instrumental variables, difference-in-differences) to estimate the marginal cost of an additional vote per candidate. Published as an annual report and open API. Objective: surface patterns consistent with vote-buying for regulatory follow-up by ODPP and FRC. Strategy §XX.5.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| IEBC Research Officer | Write (analysis run) | Trigger analysis pipelines, review outputs |
| FRC Analyst | Read (aggregate data) | Contribute cash-movement estimates, consume forensic outputs |
| Academic Researcher | Read (public API + download) | Replicate analysis, cite in peer-reviewed work |
| Data Journalist | Read (public API) | Derive and publish constituency-level rankings |
| ODPP Investigator | Read (restricted tier, on authorisation) | Access candidate-level detail for prosecution referrals |
| IAB Member | Read (all tiers) | Audit methodology and result integrity |
| KEPSA Analyst | Write (estimate upload) | Submit field cash-flow estimates per constituency |
| Civil Society | Read (public tier) | Monitor candidate-level ratios in their constituency |

---

## Functional Requirements

### FR-66.01 — Data ingestion pipeline
Ingests four source streams at end of each election cycle:
1. M22 (Campaign Finance) — per-candidate disclosed expenditure by category and date
2. FRC aggregate cash-movement — anonymised constituency-level unusual-cash-flow indicators (per MOU with FRC; no individual account data)
3. KEPSA field estimates — constituency-level estimated vote-buying cash flow, with confidence intervals
4. Electoral outcome data — per-station vote totals from M10, margin of victory, turnout rate
5. Demographic/spatial covariates — population density, poverty index, unemployment rate (KNBS sourced), historical vote-share from prior cycles

### FR-66.02 — Regression discontinuity design
For each candidate at each constituency: run RD using vote margin as running variable; treatment = candidate's disclosed spend tier; outcome = vote share. Bandwidth selected by CCT algorithm (Calonico, Cattaneo, Titiunik). Outputs: local average treatment effect estimate, standard error, 95% CI, bandwidth, polynomial order. Minimum 200 constituencies required for national-level RD validity.

### FR-66.03 — Instrumental variables estimation
IV approach: instrument for actual spending using FRC cash-flow anomaly index (assumed correlated with actual vote-buying spend, uncorrelated with innate candidate quality). Two-stage least squares: first stage predicts effective spend from IV; second stage estimates vote-share impact. F-statistic for instrument strength reported; weak-instrument test (Kleibergen-Paap) required.

### FR-66.04 — Difference-in-differences (panel)
Across electoral cycles (2017, 2022, 2027): DID estimating change in cost-per-vote by constituency, controlling for time-fixed effects and constituency-fixed effects. Requires cleaned panel dataset from prior cycles (historical data import from M22 and legacy IEBC records).

### FR-66.05 — Cost-per-vote derivation
Point estimate: `cost_per_vote = effective_spend_estimate / marginal_votes`. Marginal votes derived from LATE estimate scaled to constituency. Confidence interval propagated. Flagging rule: candidate whose cost-per-vote point estimate exceeds KES 2,000 and whose CI lower bound exceeds KES 1,000 is flagged for FRC/ODPP referral review.

### FR-66.06 — Referral pipeline
Flagged candidates: referral packet generated (candidate name, party, constituency, cost-per-vote estimate, CI, data sources, methodology note). Packet transmitted to ODPP Investigations Unit via authenticated SFTP + PGP. ODPP acknowledgement logged. M66 does not determine guilt; it surfaces statistical anomaly.

### FR-66.07 — Published annual report
Aggregated national and county-level cost-per-vote distribution (kernel density, box plots by party, time-series comparison). Candidate-level estimates suppressed to constituency-level aggregates in public tier (individual candidate-level available only to ODPP and IAB). Report released within 90 days of election. Available as PDF + machine-readable JSON.

### FR-66.08 — Open replication package
Per Strategy §XI.6 open-data commitment: analysis code (Python, open-source), input data (M22 public disclosures + KNBS covariates + anonymised FRC index), and output tables published on GitHub within 90 days of election. Researchers can replicate all public-tier results.

### FR-66.09 — Historical panel maintenance
Panel dataset maintained across cycles. Each new election adds a new observation layer. Accumulated panel supports increasingly precise DID estimates. Data-governance policy: panel kept for 15 years per M69 long-term archive integration.

---

## Non-Functional Requirements

### NFR-66.01 — Compute
Full national analysis pipeline (290 constituencies, 3 methods): < 4 hours on 16-core compute instance. Parallelised by constituency.

### NFR-66.02 — Reproducibility
All analysis runs are containerised with pinned OCI image digests (same pattern as M14). Any IAB member can reproduce outputs from published inputs. Random seed fixed where any randomisation used.

### NFR-66.03 — DPA 2019 lawful basis
FRC data: received under statutory MOU (Elections Act §77A lawful basis); no individual account holder data enters M66. KEPSA estimates are aggregate. Candidate-level estimates published only where candidate is a public figure (DPA 2019 §31 public-interest exception). DPIA completed before any analysis run.

### NFR-66.04 — Statistical validity
All published estimates carry 95% CI and effect size. Null-effect findings published equally. No cherry-picking of method that produces highest cost-per-vote estimate. IAB methodological review before public release.

### NFR-66.05 — Availability
Analysis pipeline: batch, no uptime SLA. Public report API: 99.5%.

### NFR-66.06 — Security
Candidate-level raw outputs and ODPP referral packets: IEBC-RESTRICTED. TLS 1.3 + mTLS for FRC and ODPP integrations. Audit log on every data access.

---

- **NFR-M66-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M66`; alert ruleset ID `iems-M66-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As an IEBC Research Officer, I want to run the full forensics pipeline on this cycle's data and have outputs ready for IAB review within 4 hours, so that the report can be published within the 90-day window.
2. As an FRC Analyst, I want to upload the constituency-level cash-flow anomaly index via authenticated API, so that it feeds into the IV estimation without sharing individual account data.
3. As an IAB Member, I want to review the methodology report and reproduce the public-tier outputs, so that I can sign off on statistical validity before publication.
4. As a Data Journalist, I want to download the constituency-level cost-per-vote distribution as JSON, so that I can build an interactive map for my readers.
5. As an Academic Researcher, I want to access the replication package on GitHub, so that I can replicate the findings and build on the methodology in peer-reviewed work.
6. As an ODPP Investigator, I want to receive a structured referral packet for flagged candidates, so that I can open a preliminary inquiry without needing to re-derive the analysis.
7. As a Civil Society Observer, I want to see which constituencies have statistically significantly higher cost-per-vote estimates, so that I can prioritise monitoring in those areas in the next cycle.
8. As a KEPSA Analyst, I want to submit field cash-flow estimates with confidence intervals, so that my organisation's data improves the IV instrument quality.
9. As a Candidate's Campaign Team, I want to understand the methodology used to flag candidates, so that I can verify the data inputs attributed to my campaign are correctly sourced from M22.
10. As a DPO (IEBC), I want to confirm that no individual voter or account-holder data enters any M66 output, so that the DPIA obligations are satisfied before publication.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime

class AnalysisCycle(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    election_year: int
    run_by: str  # officer UUID
    run_at: datetime
    status: str  # pending | running | complete | failed
    method: str  # rd | iv | did | all
    constituency_count: int
    flagged_count: int
    report_published_at: datetime | None
    replication_package_url: str | None

class ConstituencyEstimate(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    cycle_id: str
    constituency_code: str
    method: str  # rd | iv | did
    cost_per_vote_estimate: float  # KES
    ci_lower: float
    ci_upper: float
    effective_spend_estimate: float
    marginal_votes: int
    flagged: bool
    flag_reason: str | None

class ReferralPacket(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    cycle_id: str
    constituency_code: str
    candidate_name: str
    party: str
    cost_per_vote_estimate: float
    ci_lower: float
    ci_upper: float
    data_sources: list[str]
    methodology_note: str
    transmitted_at: datetime | None
    odpp_acknowledgement: str | None

class FRCCashFlowEstimate(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    cycle_id: str
    constituency_code: str
    anomaly_index: float  # 0-1 scale
    confidence: float
    submitted_by: str  # FRC officer UUID
    submitted_at: datetime
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M66 Election Financial Forensics API
  version: "1.0.0"
paths:
  /api/v1/financial-forensics/cycles:
    post:
      summary: Trigger analysis run for a cycle
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/AnalysisCycle"}
      responses:
        "202": {description: Analysis queued}
  /api/v1/financial-forensics/cycles/{cycle_id}/status:
    get:
      summary: Get analysis run status
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Cycle status and progress}
  /api/v1/financial-forensics/public/report/{cycle_id}:
    get:
      summary: Public aggregated report (county/national level)
      responses:
        "200": {description: Aggregated cost-per-vote distributions}
  /api/v1/financial-forensics/public/report/{cycle_id}/download:
    get:
      summary: Download public report PDF + JSON
      responses:
        "200":
          description: Report file
          content:
            application/pdf:
              schema: {type: string, format: binary}
  /api/v1/financial-forensics/frc-estimates:
    post:
      summary: Submit FRC cash-flow anomaly index
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {type: array, items: {$ref: "#/components/schemas/FRCCashFlowEstimate"}}
      responses:
        "201": {description: Estimates ingested}
  /api/v1/financial-forensics/referrals/{cycle_id}:
    get:
      summary: Get flagged-candidate referral packets (ODPP, IAB only)
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Referral packet list}
```

---

## Integrations

**Upstream:**
- M22 (Campaign Finance Transparency) — per-candidate disclosed expenditure
- M10 (RTS Back-End) — per-constituency and per-station vote totals
- FRC (Financial Reporting Centre) — anomaly index via authenticated SFTP/API (MOU-governed)
- KEPSA — field estimates via authenticated upload
- KNBS API — demographic and poverty covariates
- M66 historical panel — prior-cycle data

**Downstream:**
- M69 (Long-Term Archive) — panel dataset and reports for 15-year retention
- M15 (IAB Dashboard) — methodology review feed
- ODPP — referral packet SFTP delivery
- M11 (Public Portal) — published report and replication package links
- GitHub — open replication package
- P05 (Audit log), P06 (HSM)

---

## UI/UX — Key Screens

1. **Analysis trigger dashboard** (IEBC Research) — cycle selection, method checkboxes, data source status indicators, run button; progress bar during run
2. **Constituency estimate map** — Kenya map with constituency colour-coded by cost-per-vote quartile; click for full estimate card with CI
3. **Flagged-candidate list** (ODPP/IAB restricted) — table with candidate, constituency, estimate, CI, flag reason; download referral PDF button
4. **Public report page** — national distribution chart, county-level bar chart, methodology summary, download links; EN and SW
5. **Replication package page** — GitHub link, Docker instructions, input manifest download

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Vote-buying detection and referral | Strategy §XX.5; Elections Act §65 |
| FRC data sharing authority | Elections Act §77A; AML Act §36 |
| DPA 2019 public-interest exception for candidate data | DPA 2019 §31 |
| DPIA requirement | DPA 2019 §31 |
| Open-source publication | Strategy §XI.6 |
| 15-year data retention | Elections Act §44; M69 integration |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Report published within 90 days of election | 100% | IEBC Chair |
| Replication package available within 90 days | 100% | IAB |
| IAB methodology sign-off obtained before publication | 100% | IAB |
| ODPP referral packet delivery | 100% of flagged candidates | IEBC Legal |
| Null-effect findings published (no suppression) | 100% | IAB |
| Analysis pipeline runtime | < 4h | Datacraft SRE |

---

## Test Strategy

- **Unit:** RD bandwidth selection, IV first/second stage, DID fixed-effects; each method unit-tested against simulated datasets with known true effect sizes
- **Reproducibility:** Two independent runs from identical inputs; outputs bit-identical (fixed random seeds)
- **Integration:** M22 data ingest, M10 vote-total ingest, FRC estimate upload, ODPP SFTP delivery
- **Statistical:** Simulated datasets with planted known effects; each method must recover the planted effect within 95% CI
- **DPA compliance:** Automated scan confirming no individual account data in any M66 database table

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| FRC MOU finalised | 2026-12-01 | Data-sharing authority in place |
| Historical panel import (2017, 2022 cycles) | 2027-02-01 | Panel populated |
| Analysis pipeline built and IAB-reviewed | 2027-04-01 | Methodology approved |
| KEPSA estimate upload interface live | 2027-06-01 | Field estimates available post-election |
| Post-election analysis run | 2027-09-01 (T+22) | Pipeline triggered |
| Report published | 2027-11-08 (T+90) | Public release |

---

## Dependencies

- **M22** (Campaign Finance Transparency) — disclosed expenditure source
- **M10** (RTS Back-End) — electoral outcome data
- **M69** (Long-Term Archive) — panel and report retention
- **M15** (IAB Dashboard) — methodology review
- **P05** (Audit log), **P06** (HSM)
- External: FRC, KEPSA, KNBS, ODPP

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | FRC refuses to share anomaly index | MOU with IEBC under Elections Act §77A negotiated at T-365; fall back to IV exclusion restriction test if FRC data unavailable |
| R2 | Methodology challenged as politically biased | IAB sign-off on methodology pre-publication; null findings published equally; independent academic peer review of methods before T-0 |
| R3 | KEPSA estimate quality too low for IV instrument | Instrument strength test (Kleibergen-Paap F > 10) before using KEPSA data; exclude weak instruments from published results |
| R4 | Referral packet challenged in court as hearsay | Methodology documentation at expert-witness standard; IAB co-signs; statistical expert retained as ODPP technical advisor |
| R5 | ODPP SFTP delivery fails | Encrypted email fallback; delivery confirmed within 48h; retry with human escalation if delivery fails twice |

---

## Cross-References

- **M22** Campaign Finance Transparency — primary data source
- **M10** RTS Back-End — vote-total source
- **M14** Reproducibility Bundle — pattern for open replication package
- **M15** IAB Dashboard — methodology audit
- **M69** Long-Term Archive — 15-year panel retention

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| §XX.5 | Post-election financial forensics and cost-per-vote analytics |
| §XI.6 | Open-source replication package |
| Elections Act §65 | Vote-buying offence and referral |
| DPA 2019 §31 | Public-interest exception for candidate data |

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
