# M15 — IAB Dashboard

**Module ID:** M15
**Name:** Independent Assurance Board Dashboard
**Owner:** Datacraft Limited — Platform Engineering Team (infrastructure); IAB (operations)
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4; IAB operates under separate MoU

**Strategy traceback:** Part II §PM-08, Part III §PB-01, Part IX §IX.2.F3, Part XII §XII.3, Part XIX §XIX.4, Part XXI.A.14

---

## Purpose

The IAB Dashboard provides the Independent Assurance Board with continuous, signed read-only access to every consequential system event in the KIEMS ecosystem — from provisioning through post-election disposition. The IAB is the standing technical witness that neutralises the "declined server access" ground from Raila Odinga v IEBC [2017] eKLR para 272. The board publishes quarterly public reports that are admissible as expert evidence in a Supreme Court petition. This module is the technical infrastructure for those assurance functions: a single authenticated read-only console integrating audit logs, reproducibility outputs, anomaly alerts, compliance status, inventory events, and a structured escalation ladder from IAB to IEBC Chair and, in extremis, to the Supreme Court IT Registrar.

---

## User Personas

| Persona | Role | Access level |
|---|---|---|
| IAB Chair | Board leader | Full IAB read access; escalation trigger authority |
| IAB Technical Member (x2-3) | Electoral technology specialists | Full IAB read access; drill-down on technical logs |
| IAB Legal Member | Constitutional law specialist | Legal-basis logs; compliance mapping; s.44A invocations |
| IAB ODPC Representative | Data-protection oversight | DPA 2019 compliance events; DPIA status; breach log |
| IAB Civil Society Nominee (x2) | ELOG / Kenyan civil society | Summary views; public-report drafting tool |
| IAB IFES/IDEA Nominee | International observer | Same as civil society |
| IAB Secretariat (Datacraft-funded) | Administrative support | Board scheduling; report publication; alert routing |
| IEBC Chair (read-only observer) | Commission Chair | Summary dashboard; escalation notifications |

---

## Functional Requirements

### FR-15.01 — Authentication and access
Every IAB member authenticates with MFA (TOTP + hardware key). Sessions are time-limited (4 hours) and re-authenticated for sensitive log views. All access is logged with timestamp, member identity, action, and purpose-of-access code per DPA 2019 s.26.

### FR-15.02 — Audit log viewer
Real-time and historical view of all HSM-signed KIEMS events across all modules. Filterable by: module, event type, time range, station code, officer identity, severity. Hash-chain integrity indicator: any break in the chain highlighted immediately. Cross-referenced to Merkle root for independent verification.

### FR-15.03 — Reproducibility audit feed
Live status of M14 reproducibility bundle: input manifest hash, per-stage expected vs actual hash (green/amber/red), second-implementation agreement status, Gazette publication confirmation, radio bulletin delivery confirmation. IAB can trigger a manual re-run of the pipeline from the dashboard.

### FR-15.04 — Anomaly alert inbox
Receives all Layer 1-7 anomaly alerts from M26 (anomaly detection stack, owned by parallel agent). IAB members can mark alerts as reviewed, request investigation, or escalate. Alert history retained permanently.

### FR-15.05 — Compliance dashboard
Per-module compliance status mapped to statutory obligations. Shows: kit-side guardrail coverage, s.44A invocation log, DPA 2019 breach log, Elections Act compliance violations detected, legal-clearance certificate status. IAB can query "show me every event logged under Elections Act s.44A."

### FR-15.06 — Inventory audit feed
Read-only access to M20 IMS events: custody chain per asset, geofence alerts, kit-status distribution, reverse-logistics reconciliation. IAB can request a signed inventory snapshot for any date.

### FR-15.07 — Escalation ladder (structured)
Four-level escalation mechanism:

| Level | Trigger | IAB action | IEBC response required |
|---|---|---|---|
| L1 — Advisory | Minor anomaly, process deviation | Logged advisory note, shared with IEBC Chair | Acknowledge within 24 h |
| L2 — Concern | Repeated advisory pattern, statistically significant anomaly | Formal IAB Concern Notice (signed, hash-logged) | Written response within 48 h + remediation plan |
| L3 — Public Alert | L2 unresolved after 72 h or critical integrity threat | IAB Public Alert published on IAB domain + social media | IEBC public statement within 24 h; IAB public hearing |
| L4 — Emergency | Active fraud or systemic failure threatening result integrity | IAB Emergency Declaration: notifies Supreme Court IT Registrar + AG + AU observer missions directly | IEBC Chair + Cabinet Secretary immediate response |

Every escalation event is HSM-signed and appended to the public record.

### FR-15.08 — Quarterly report builder
Structured report template mapping: audit findings, compliance status, anomaly summary, reproducibility certification, escalation history, recommendations. Export to PDF (signed) and HTML. Published on IAB domain and submitted to IEBC Chair + Parliament + Auditor-General.

### FR-15.09 — Pre-election evidentiary bundle status
Tracks the 15-item pre-emptive evidentiary bundle (Part XII.4): real-time completion status, missing items highlighted, sign-off workflow for each item, publication confirmation to Supreme Court IT Registrar.

### FR-15.10 — Party-agent complaint intake
IAB direct complaint channel (FR-15 of Strategy §IX.2.F3): any accredited party agent or civil society observer can submit a complaint via the IAB public portal. Complaint logged, hashed, acknowledged within 48 hours per SLA, routed to appropriate module owner.

### FR-15.11 — Export for legal proceedings
IAB can generate a signed, court-ready evidence package for any event or time range: includes log extracts, hash-chain proof, Merkle inclusion proofs, chain-of-custody certificates, and an IAB witness affidavit template. Package is produced within 24 hours of request.

### FR-15.12 — Sanitised public view
A read-only public sub-set of the IAB dashboard is accessible without authentication at `iab.iebc.go.ke/public`: aggregate anomaly counts, compliance summary per module, escalation history (excluding investigative details during active investigations), quarterly report archive. No PII. No operational details that would aid an attacker.

### FR-15.13 — No write path
The IAB has zero write access to any KIEMS production system. Every IAB action that produces a record (advisory, concern, alert, escalation) is a new record appended to the IAB's own log, not a modification of any KIEMS data. This is enforced at the API gateway, not just the application layer.

---

## Non-Functional Requirements

### NFR-15.01 — Availability
99.9% during T-365 through T+365. Hosted on dedicated infrastructure separate from production KIEMS to prevent IAB monitoring from being affected by the events it monitors.

### NFR-15.02 — Latency
Audit log queries p99 < 3 s for up to 7-day range. Anomaly alert delivery to IAB dashboard p99 < 60 s from event origination in source module.

### NFR-15.03 — Security
Zero write path to KIEMS production enforced at API gateway. IAB credentials cannot be shared; each member has a unique hardware key. All IAB sessions recorded (screen capture optional, keystroke log mandatory) for post-incident analysis. Dashboard hosted on IAB-controlled subdomain (`iab.iebc.go.ke`) with IEBC DNS delegation.

### NFR-15.04 — Integrity
Every log page displayed to IAB includes a live hash-chain integrity indicator. If any gap or hash mismatch is detected in the audit trail being viewed, the dashboard displays a CHAIN BREAK alert immediately and notifies all IAB members.

### NFR-15.05 — Privacy (DPA 2019)
IAB members sign confidentiality agreements covering any voter PII encountered. Dashboard redacts voter biometric references and national ID numbers from log displays by default; IAB Chair can request a legal-basis-justified unredacted view with per-record DPA s.26 logging.

### NFR-15.06 — Independence
IAB dashboard infrastructure is managed by the IAB Secretariat (Datacraft-funded but IAB-directed). Datacraft operational staff cannot access IAB member sessions or read IAB internal deliberations. The separation is enforced by a separate authentication domain and separate hosting instance.

---

## User Stories

1. As an IAB Technical Member, I want to view the hash-chain integrity of all audit logs in real time, so that I can immediately detect any tampering attempt.
2. As an IAB Legal Member, I want to query all s.44A invocation events, so that I can confirm every declaration was made according to the pre-agreed protocol.
3. As an IAB Chair, I want to issue a Formal Concern Notice and track IEBC's response, so that the process is documented and accountable.
4. As an IAB ODPC Representative, I want to see all DPA 2019 events including any breach log entries, so that I can confirm statutory compliance.
5. As an IAB Civil Society Nominee, I want to draft and publish the quarterly report from the dashboard, so that the public has timely independent assurance.
6. As an IAB member, I want to generate a signed court-ready evidence package within 24 hours of a petition filing, so that IEBC's legal team has immediate expert support.
7. As an accredited party agent, I want to file a complaint through the IAB portal and receive an acknowledgement ticket, so that my concern is independently recorded outside IEBC's direct control.
8. As an international observer, I want to access the sanitised public view and download the quarterly reports, so that my organisation can publish an informed independent assessment.
9. As an IAB Technical Member, I want to trigger a manual re-run of the reproducibility pipeline from the dashboard, so that I can independently confirm the published result is arithmetically correct.
10. As an IAB Chair, I want to invoke the L4 Emergency Declaration and notify the Supreme Court IT Registrar directly from the dashboard, so that an integrity threat is escalated through official channels immediately.

---

## Data Model Additions

```python
class IABEscalation(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    level: int  # 1-4
    triggered_by: str  # IAB member UUID
    triggered_at: datetime
    trigger_description: str
    linked_event_ids: list[str]  # audit log event UUIDs
    iebc_response_due: datetime
    iebc_response_at: datetime | None
    iebc_response_hash: str | None
    public: bool
    signed_hash: str  # HSM signature of this record

class IABComplaint(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    submitted_at: datetime
    submitter_type: str  # party_agent | civil_society | observer | individual
    submitter_ref: str  # accreditation number or pseudonym
    subject_module: str | None
    station_code: str | None
    description: str
    evidence_hash: str | None
    status: str  # new | acknowledged | under_investigation | closed
    acknowledgement_at: datetime | None
    resolution_hash: str | None

class IABReport(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    period: str  # Q1-2027 etc.
    published_at: datetime
    report_pdf_hash: str
    report_html_hash: str
    published_to: list[str]  # URLs
    signed_by: list[str]  # IAB member UUIDs
    findings_count: dict[str, int]  # severity -> count
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/audit/events` | IAB JWT (MFA) | 50/s | Filterable audit log query |
| GET | `/api/v1/audit/chain/status` | IAB JWT | 20/s | Hash-chain integrity status |
| GET | `/api/v1/reproducibility/status` | IAB JWT | 10/s | Bundle and second-impl status |
| POST | `/api/v1/reproducibility/trigger-rerun` | IAB Chair JWT | 1/10min | Trigger manual pipeline re-run |
| GET | `/api/v1/anomalies` | IAB JWT | 50/s | Anomaly alert feed |
| POST | `/api/v1/escalations` | IAB JWT | 5/s | Create escalation |
| GET | `/api/v1/escalations/{id}` | IAB JWT | 20/s | Escalation detail + IEBC response |
| GET | `/api/v1/compliance/status` | IAB JWT | 20/s | Per-module compliance summary |
| GET | `/api/v1/inventory/snapshot` | IAB JWT | 5/s | Signed inventory snapshot |
| POST | `/api/v1/evidence/package` | IAB JWT | 1/min | Generate court-ready package |
| POST | `/api/v1/complaints` | None (public) | 5/s per IP | Party-agent / observer complaint intake |
| GET | `/api/v1/public/summary` | None (public) | 100/s | Sanitised public status |
| GET | `/api/v1/reports` | None (public) | 20/s | Published quarterly reports |

---

## Integration Points

**Upstream data feeds (all read-only, push-to-IAB):**
- P05 (Audit log) — all KIEMS signed events
- M14 (Reproducibility Bundle) — bundle status, second-impl results
- M20 (IMS) — inventory and custody events
- M16 (Election Command Centre) — anomaly alerts, kit-status summary
- M26 (Anomaly Detection, parallel agent) — statistical and ML anomaly alerts
- P06 (HSM) — signing service for IAB escalation records

**Downstream (IAB pushes to):**
- Supreme Court IT Registrar — L4 emergency notification, evidence packages
- IEBC Chair — escalation notifications
- Public (IAB domain) — quarterly reports, L3 public alerts
- M23 (Grievance channels) — complaint routing

---

## UI/UX — Key Screens

1. **Command overview** — live health summary across all modules, escalation queue, anomaly count by severity
2. **Audit log explorer** — searchable, filterable event table; hash-chain integrity badge; hash-chain break highlighted
3. **Reproducibility panel** — bundle status, per-stage hash status, second-impl agreement, trigger re-run button (Chair only)
4. **Compliance matrix** — per-module statutory obligations, red/amber/green status
5. **Inventory map** — county-level kit status from M20, geofence alert overlay
6. **Escalation workflow** — create escalation, track IEBC response, time-to-response counter
7. **Evidence package builder** — date-range picker, module filter, one-click signed PDF generation
8. **Complaint portal** (public-facing) — simple form with acknowledgement receipt
9. **Report builder** — guided report template with auto-populated statistics, review, sign, publish
10. **Public view** — stripped-down public summary page accessible without login

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Independent oversight of IEBC | Constitution Art. 88(4) |
| Server access — proactive neutralisation | Raila v IEBC [2017] eKLR para 272 |
| Independent audit and assurance | Elections (Technology) Regulations 2017 reg. 12 |
| DPA 2019 oversight | DPA 2019 s.15 (Data Commissioner) + IEBC DPO |
| Access to information | Access to Information Act 2016 s.4 |
| Evidence preservation for petition window | Constitution Art. 140; Elections Act s.84 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Audit log chain integrity | 100% — zero breaks | IAB Technical Member |
| Anomaly alert delivery to IAB dashboard | p99 < 60 s | IAB + Datacraft SRE |
| L1/L2 IEBC response SLA | 100% within defined windows | IAB Chair |
| Quarterly report publication SLA | Within 30 days of quarter end | IAB Chair |
| Evidence package generation | Within 24 hours of request | IAB Secretariat |
| Complaint acknowledgement | 100% within 48 hours | IAB Secretariat |
| IAB dashboard availability | >= 99.9% active-active (primary + mirror) | Datacraft SRE |
| Public view availability | >= 99.9% active-active (primary + mirror) | Datacraft SRE |

**HA deployment note:** M15 is a hub module with 16+ downstream dependents. Active-active across primary and mirror DCs is required; active-passive would leave downstream alert pipelines (M16, M20, M21, M26, M30, M34, M56, M57, M59, M62, M63, M65, M66, M67, M68) unable to confirm audit trail continuity during a primary-site failure. Verified at FAT.

---

## Test Strategy

- **Unit:** Escalation state machine, evidence package generator, report builder template
- **Integration:** Audit log consumption from P05; anomaly alert push from M26; inventory feed from M20
- **Access control:** Automated test that IAB credentials have zero write access to any KIEMS module; penetration test of read-only enforcement at API gateway
- **Chain-break detection:** Deliberately introduce a hash-chain gap in a staging copy; confirm dashboard detects and alerts within 60 seconds
- **Evidence package:** Generate a test package and verify all 15 items from Part XII.4 are present, signed, and verifiable
- **Load:** 50 concurrent IAB sessions during peak (election day); audit log queries returning 7-day windows
- **Security:** Independent pen test of IAB authentication flow (MFA bypass attempt); session isolation test

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| IAB membership chartered by IEBC | 2026-10-01 | MoU signed |
| Dashboard alpha (IAB Chair + Secretariat) | 2027-01-01 | Internal access |
| Full IAB access | 2027-02-01 | All members onboarded |
| Public view live | 2027-03-01 | `iab.iebc.go.ke/public` |
| Dry-run election monitoring | 2027-06-10 (T-60) | Escalation workflow tested |
| Election Day ops | 2027-08-10 | 24x7 IAB monitoring |
| Post-election final report | 2027-10-10 (T+60) | Published + submitted to Parliament |

---

## Dependencies

- **P05** (Audit log) — primary data source
- **P06** (HSM) — signing of IAB escalation records and evidence packages
- **P04** (IAM) — IAB MFA authentication; hardware key integration
- **M14** (Reproducibility Bundle) — bundle status feed
- **M20** (IMS) — inventory data
- **M26** (Anomaly Detection, parallel agent) — anomaly alerts
- External: IAB member hardware keys; Supreme Court IT Registrar contact details registered at T-90

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | IAB member conflict of interest undermines independence | Selection protocol with published COI declarations; civil society veto on IEBC nominees; COI disclosed triggers 30-day replacement process |
| R2 | IEBC non-compliance with L1/L2 escalations erodes IAB credibility | L3 public alert mechanism; IAB Secretariat independently publicises non-compliance; pre-agreed media engagement protocol |
| R3 | IAB dashboard itself is compromised | Separate hosting from KIEMS production; IAB-controlled subdomain; independent security audit; any IAB member can independently verify audit log hash chain without using the dashboard |
| R4 | IAB captures no useful signal during non-election period, loses institutional knowledge | Monthly standing meeting with KIEMS operations team; quarterly public reports; year-round complaint intake |
| R5 | Evidence package generation fails during petition window | 24-hour human-backed fallback: IAB Secretariat can produce manual evidence extract directly from HSM-signed logs; pre-rehearsed at T-14 dry run |
