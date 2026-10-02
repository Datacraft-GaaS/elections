# M61 — Ballot Secrecy Assurance and Testing Platform

**Module ID:** M61
**Name:** Ballot Secrecy Assurance and Testing Platform
**Owner:** Datacraft Limited — Security Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4

**Strategy traceback:** Part II §PM-14, Part VII §VII.3, Part IX §IX.2.F5, Part XII §XII.3, Part XXI.A.20

---

## 1. Purpose

Constitution Art. 138(2) guarantees that voting is by secret ballot. Elections Act §54 and §72 impose secrecy obligations on every electoral officer. Technically, ballot secrecy can be broken by: physical coercion (voter photographing ballot), kit spoofing (fake BVI device recording choices), RTS metadata leaks (vote order inferrable from transmission sequence), or party-agent-app leaks (M30 agent app inadvertently exposing voter choices). Each of these attack surfaces requires dedicated adversarial testing. Existing modules handle anomaly detection (M26) and AI monitoring (M28); M61 is distinct: a continuous, red-team-driven, structured pen-testing programme whose sole focus is proving that ballot secrecy holds under all attacks, and whose findings feed a formal IAB attestation. This module manages the red-team engagement lifecycle, finding tracking, remediation workflow, and IAB attestation record.

Distinct from M28 (AI monitoring of content): M28 monitors social media and content signals. M61 tests the KIEMS technology stack adversarially for secrecy vulnerabilities. Distinct from M23 (grievance): M23 handles complaints. M61 is a proactive internal assurance function.

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| Datacraft Security Lead | Programme owner; red-team manager | Full RW |
| Red Team Operator | External pen-test contractor | RW on assigned engagement findings |
| the Commission CISO | Security oversight | Read-all; approve remediation closures |
| IAB Technical Member | Attestation consumer | Read finding summaries; attestation workflow |
| Module Owner (M01-M10, M30) | Responsible for remediation | Read findings assigned to their module; update remediation status |
| the Commission DPO | DPA 2019 compliance overlay | Read findings with privacy implications |
| IAB Secretariat | Attestation logistics | Read IAB attestation records |

---

## 3. Functional Requirements

**FR-M61-01 — Red-team engagement registry:** Register each red-team engagement: type (physical coercion scenario, kit spoofing, RTS metadata analysis, party-agent-app leak, full-stack adversarial), target modules, engagement scope, red-team lead, start and end dates, rules of engagement. Rules of engagement must be signed by Datacraft Security Lead and the Commission CISO before engagement begins.

**FR-M61-02 — Attack surface taxonomy:** A maintained taxonomy of ballot-secrecy attack surfaces, each mapped to the constitutional/statutory obligation it threatens. Taxonomy updated with each new engagement. Current taxonomy covers: physical coercion (Art. 138(2)), kit spoofing (Elections Act §54), RTS metadata leak (Elections Technology Regulations reg. 11), party-agent-app leak (M30 scope), social-engineering of polling officials, metadata in result-transmission packets.

**FR-M61-03 — Finding lifecycle management:** Each pen-test finding: unique ID, attack surface taxonomy tag, affected module(s), description, proof-of-concept reference (hash-linked, not stored in plain), severity (critical, high, medium, low), discovery date, discoverer (red-team operator ID). Lifecycle: open, assigned, remediation-in-progress, remediation-complete, verified-closed, accepted-risk. Accepted-risk findings require the Commission CISO and IAB Technical Member dual sign-off.

**FR-M61-04 — Remediation workflow:** Each finding assigned to the relevant module owner with remediation deadline (severity-based: critical 48 h, high 7 days, medium 30 days, low 90 days). Module owner updates remediation status. Datacraft Security Lead verifies fix with a re-test. Re-test result logged. Finding moves to verified-closed only after passing re-test.

**FR-M61-05 — Continuous testing cadence:** Minimum testing cadence per module: quarterly for M01-M10 (kit devices and backends), monthly from T-90 for M30 (party agent app). Ad-hoc engagement triggered by: new module release, KIEMS configuration change, IAB advisory, or external intelligence indicating a specific attack vector. Cadence calendar maintained in M61; overdue engagements escalate to the Commission CISO.

**FR-M61-06 — IAB attestation workflow:** At T-30 and T-7, Datacraft Security Lead generates an attestation report for the IAB: all findings by severity, remediation status, open findings with accepted-risk justifications, testing coverage per module. IAB Technical Member reviews and counter-signs the attestation. Attestation is HSM-signed, logged in P05, and shared with the public IAB report (M15). Any critical finding open at T-7 triggers a mandatory IAB escalation.

**FR-M61-07 — RTS metadata analysis:** Specific continuous test: verify that the sequence or timing of RTS transmissions does not leak vote order. Automated analysis runs after each RTS simulation exercise and after each M10 configuration change. Analysis result (pass/fail) logged with evidence hash. Failures generate immediate critical finding.

**FR-M61-08 — Kit spoofing simulation:** Periodic simulation of a fake BVI device (M05) attempting to intercept or record voter choices. Simulation exercises conducted against staging-environment kits only. Results logged with proof-of-concept hash.

**FR-M61-09 — DPA 2019 overlay:** All finding descriptions must be reviewed by the Commission DPO if the finding implicates personal data. DPO annotates findings with privacy-impact rating. Accepted-risk findings with high privacy impact require ODPC notification before acceptance.

---

## 4. Non-Functional Requirements

**NFR-M61-01 — Security of M61 itself:** Finding records classified as RESTRICTED. Access strictly role-limited. Proof-of-concept details stored as encrypted blobs; accessible only to Datacraft Security Lead and the Commission CISO. Finding summaries (no PoC detail) available to module owners and IAB.

**NFR-M61-02 — Audit trail:** Every action (finding creation, assignment, remediation update, re-test, attestation sign-off) logged in P05 with actor, timestamp, and data hash.

**NFR-M61-03 — Availability:** 99.5% uptime. M61 must be operational at T-90 to begin intensified pre-election testing cadence.

**NFR-M61-04 — Remediation SLA enforcement:** Automated alert to the Commission CISO at 50% and 100% of remediation deadline if finding is not yet in remediation-in-progress. Critical findings generate hourly alerts after 48-hour deadline breach.

**NFR-M61-05 — WCAG 2.2 AA:** Internal-only; staff and IAB screens.

**NFR-M61-06 — DPA 2019:** Finding records may reference test environments only; no live voter data in any pen-test activity. DPA s.30(1)(b) — legal obligation (Elections Act §54 compliance assurance).

**NFR-M61-07 — Isolation:** M61 has no write path to any production KIEMS module. All adversarial testing against staging environments. Production-equivalent staging maintained by Datacraft SRE.

---

- **NFR-61-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-61` (replace 61 with the module number); alert ruleset ID `iems-61-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the Datacraft Security Lead, I register a new kit-spoofing engagement with signed rules of engagement before the red team begins testing.
2. As a Red Team Operator, I log a critical finding — RTS packet timing leaks vote order — with proof-of-concept hash and affected modules, so M10 owner is notified within minutes.
3. As an M10 Module Owner, I receive the finding assignment, implement a fix, and update the remediation status so Datacraft Security Lead can schedule a re-test.
4. As the Datacraft Security Lead, I run the re-test, confirm the fix holds, and move the finding to verified-closed with a re-test evidence hash.
5. As the IAB Technical Member, I review the T-30 attestation report and counter-sign it; the signed attestation is published to the IAB public report.
6. As the Commission CISO, I co-sign an accepted-risk decision for a low-severity finding that cannot be fixed before the election, with a written justification logged in P05.
7. As the Commission DPO, I review a finding that implicates party-agent-app data and annotate it with a high privacy-impact rating, triggering ODPC notification.
8. As the IAB Technical Member, I see a critical finding open at T-7 and the system triggers an automatic IAB escalation per FR-M61-06.
9. As the Datacraft Security Lead, I view the continuous RTS metadata analysis dashboard and see the last 10 simulation results, all passing.
10. As the Commission CISO, I verify that the testing cadence calendar shows no overdue engagements at T-90.
11. As an IAB Secretariat member, I download the T-7 signed attestation PDF for inclusion in the IAB pre-election public report.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date
from typing import Literal

class RedTeamEngagement(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str()
    engagement_type: Literal["physical_coercion","kit_spoofing","rts_metadata","party_agent_app","full_stack","other"]
    target_modules: list[str]
    scope_description: str
    red_team_lead: str
    start_date: date
    end_date: date
    roe_signed_by_security_lead: bool
    roe_signed_by_ciso: bool
    status: Literal["planned","active","report_pending","complete"]
    created_at: datetime

class SecrecyFinding(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    engagement_id: str
    attack_surface_tag: str   # from taxonomy e.g. "rts_metadata_leak"
    affected_modules: list[str]
    description: str
    poc_blob_id: str | None   # encrypted blob reference; not plain text
    severity: Literal["critical","high","medium","low"]
    discovered_by: str
    discovered_at: datetime
    assigned_to: str | None   # module owner
    remediation_deadline: datetime | None
    status: Literal["open","assigned","remediation_in_progress","remediation_complete","verified_closed","accepted_risk"]
    retest_result: Literal["pass","fail"] | None
    retest_at: datetime | None
    privacy_impact: Literal["none","low","high"] | None
    accepted_risk_justification: str | None
    ciso_sign_at: datetime | None
    iab_sign_at: datetime | None

class SecrencyAttestation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    attestation_type: Literal["T-30","T-7","ad_hoc"]
    generated_at: datetime
    findings_summary: dict   # severity -> {total, open, closed, accepted_risk}
    open_critical_count: int
    coverage_summary: dict   # module -> last_test_date
    security_lead_signed_at: datetime | None
    iab_member_signed_at: datetime | None
    attestation_pdf_hash: str | None
    published_to_iab: bool

class RTSMetadataAnalysisRun(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    run_at: datetime
    trigger: Literal["scheduled","m10_config_change","ad_hoc"]
    result: Literal["pass","fail"]
    evidence_hash: str
    finding_id: str | None   # if fail, linked finding
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/secrecy/engagements` | SECURITY JWT | 60/min | List red-team engagements |
| POST | `/api/v1/secrecy/engagements` | SECURITY_LEAD JWT | 5/hr | Register engagement |
| GET | `/api/v1/secrecy/findings` | SECURITY or COMMISSION_CISO JWT | 60/min | List findings (no PoC detail) |
| POST | `/api/v1/secrecy/findings` | RED_TEAM JWT | 20/hr | Log new finding |
| PATCH | `/api/v1/secrecy/findings/{id}` | MODULE_OWNER JWT | 30/hr | Update remediation status |
| POST | `/api/v1/secrecy/findings/{id}/retest` | SECURITY_LEAD JWT | 10/hr | Log re-test result |
| POST | `/api/v1/secrecy/findings/{id}/accept-risk` | COMMISSION_CISO + IAB JWT | 3/hr | Dual sign-off accepted risk |
| GET | `/api/v1/secrecy/attestations` | SECURITY or IAB JWT | 30/min | List attestation reports |
| POST | `/api/v1/secrecy/attestations` | SECURITY_LEAD JWT | 5/hr | Generate attestation report |
| POST | `/api/v1/secrecy/attestations/{id}/sign` | SECURITY_LEAD or IAB JWT | 5/hr | Sign attestation |
| GET | `/api/v1/secrecy/rts-analysis` | SECURITY JWT | 30/min | RTS metadata analysis run history |
| GET | `/api/v1/secrecy/taxonomy` | SECURITY JWT | 60/min | Attack surface taxonomy |
| PATCH | `/api/v1/secrecy/taxonomy/{tag}` | SECURITY_LEAD JWT | 10/hr | Update taxonomy entry |

---

## 8. Integrations

- **M15 IAB Dashboard** — attestation reports published; IAB reads attestation summaries via M15
- **M10 RTS Backend** — RTS metadata analysis triggered on M10 config change
- **M05 BVI Device** — kit-spoofing simulation target (staging only)
- **M30 Mobile Party Agent** — party-agent-app leak test target (staging only)
- **M26 Anomaly Detection** — M61 findings can seed anomaly-detection rules in M26
- **P05 Audit Log** — all M61 actions logged and Merkle-anchored
- **P06 HSM** — attestation PDF signing; accepted-risk dual sign-off

---

## 9. UI/UX Key Screens

1. **Security Dashboard** — open findings by severity; overdue remediations; RTS analysis last-run status; next scheduled engagement; T-30 and T-7 attestation countdown.
2. **Finding Detail** — finding metadata; severity badge; attack surface tag; affected modules; remediation timeline; re-test history. PoC blob accessible only to CISO and Security Lead.
3. **Engagement Registry** — engagement list; rules-of-engagement sign-off status; active engagement progress.
4. **Attestation Builder** — auto-populated finding summary; coverage matrix; sign-off panel; PDF preview.
5. **RTS Analysis History** — timeline of analysis runs; pass/fail per run; linked findings for failures.
6. **IAB View (read-only)** — finding summary counts (no descriptions); attestation sign-off workflow; published attestation PDF download.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Secret ballot guarantee | Constitution Art. 138(2) |
| Polling officer secrecy obligation | Elections Act 2011 s.54 |
| Secrecy of the ballot — technology | Elections Act 2011 s.72 |
| Technology audit and pen-testing | Elections (Technology) Regulations 2017 reg. 12 |
| DPA 2019 lawful basis for security testing | DPA 2019 s.30(1)(b) — legal obligation |
| IAB attestation obligation | Strategy Part XII.3; Part IX §IX.2.F5 |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Zero open critical findings at T-7 | 100% | the Commission CISO |
| T-30 and T-7 attestations published to IAB | 100% on schedule | Datacraft Security Lead |
| Critical finding remediation within 48 hours | >= 95% | the Commission CISO |
| RTS metadata analysis run after every M10 config change | 100% | Datacraft Security Lead |
| Testing cadence coverage (all modules tested) | >= 1 full cycle by T-30 | Datacraft Security Lead |
| Accepted-risk dual sign-off for every accepted finding | 100% | the Commission CISO |

---

## 12. Test Strategy

- **Unit:** Severity-based deadline calculator; accepted-risk dual sign-off gate; attestation auto-summary logic.
- **Integration:** Attestation PDF signing via P06 HSM; M15 attestation publication; M10 config-change trigger to RTS analysis.
- **Access control:** Red Team Operator cannot access findings from other engagements; Module Owner sees only their assigned findings; IAB sees summaries only (no PoC detail).
- **Security of M61:** Attempt to read encrypted PoC blob without CISO credentials; verify blocked at API gateway.
- **E2E:** Full finding lifecycle from red-team submission through module remediation, re-test, and attestation publication.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| M61 platform live; taxonomy seeded | T-365 (2026-10-10) | Security Lead and CISO onboarded |
| First quarterly engagement (full-stack) | T-270 (2027-01-13) | Baseline findings established |
| Monthly cadence begins | T-90 (2027-05-10) | All M01-M10 and M30 in monthly cycle |
| T-30 attestation | T-30 (2027-07-11) | Signed by Security Lead + IAB Technical Member |
| T-7 attestation | T-7 (2027-08-03) | Zero critical findings confirmed |
| Post-election retrospective | T+30 (2027-09-10) | Final findings report; lessons learned |

---

## 14. Dependencies

- **M15 IAB Dashboard** — attestation publication
- **M10, M05, M30** — staging-environment test targets
- **M26** — can seed rules from M61 findings
- **P05, P06** — audit log and HSM signing
- External red-team contractor — contracted by T-365
- Production-equivalent staging environment — maintained by Datacraft SRE from T-270

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Critical finding discovered at T-7 cannot be remediated before election | IAB mandatory escalation; the Commission CISO and IAB assess whether election proceeds with accepted-risk controls or whether a mitigating operational instruction is issued to all ROs; decision logged and signed |
| R2 | Red-team contractor has insufficient KIEMS-specific knowledge | Contractor on-boarded with full staging-environment access from T-270 and supervised for first two engagements; finding quality reviewed by Datacraft Security Lead |
| R3 | PoC details leaked from M61 before election | Encrypted blob storage; access logs monitored daily by CISO; M61 itself subject to annual pen test separate from KIEMS programme |
| R4 | Module owner disputes finding severity | Datacraft Security Lead is severity authority; module owner can escalate to the Commission CISO for review; finding frozen at disputed severity during review; review closed within 5 business days |
| R5 | RTS metadata analysis false negative masks a real leak | Analysis methodology reviewed by IAB Technical Member at each attestation; second-opinion analysis by independent security reviewer at T-60 |

---

## 16. Cross-References

- **M15** — IAB Dashboard: attestation reports published here; IAB findings may reference M61 attestation
- **M26** — Anomaly Detection: M61 findings may seed new detection rules
- **M28** — AI Monitoring: separate function; M61 is pen-testing not content monitoring
- **M10, M05** — primary staging test targets
- **M30** — party-agent-app leak test target
- **M40** — Election Court IT: M61 attestations may be submitted as pre-emptive evidence

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| Ballot secrecy assurance under all attacks | FR-M61-02, FR-M61-07, FR-M61-08 |
| Structured pen-test programme | FR-M61-01, FR-M61-05 |
| Finding lifecycle and remediation | FR-M61-03, FR-M61-04 |
| IAB attestation | FR-M61-06, NFR-M61-02 |
| DPA 2019 compliance | FR-M61-09, NFR-M61-06 |

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
