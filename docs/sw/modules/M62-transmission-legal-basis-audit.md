# M62 — Transmission Legal-Basis Audit

**Module ID:** M62
**Name:** Transmission Legal-Basis Audit
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4

**Strategy traceback:** Part II §PM-15, Part VII §VII.2, Part IX §IX.2.E, Part XII §XII.4, Part XXI.A.21

---

## 1. Purpose

Elections Act §39 requires that electronic transmission of results complies with statutory requirements: prescribed form, authorised signatories, timing within the prescribed window, and complete chain of custody. §44A governs the declaration workflow. The Elections (Technology) Regulations prescribe format and authentication requirements for each transmission event. While M26 detects statistical anomalies in transmission data, M62 is a distinct legal-compliance audit: it verifies that each individual transmission event satisfies its statutory requirements, produces a per-transmission compliance verdict, and generates a complete §39 compliance record for every station. This record is admissible as pre-emptive evidence in a Supreme Court petition (Strategy Part XII.4) and is the technical basis for any the Commission response to a §39 challenge. Distinct from M26: M26 asks "is this statistically anomalous?"; M62 asks "did this transmission comply with the law?"

---

## 2. User Personas

| Persona | Role | Access |
|---|---|---|
| the Commission Legal Counsel | Primary compliance consumer | Read-all compliance records; generate evidence bundles |
| the Commission RTS Operations Lead | Operational monitoring | Read compliance dashboard; escalate non-compliance |
| Datacraft Compliance Lead | Audit rule maintenance | RW on audit rules; read all verdicts |
| IAB Technical Member | Statutory compliance attestation | Read compliance summary; attestation sign-off |
| M40 IT Liaison | Petition evidence preparation | Read compliance records for specified stations |
| the Commission CISO | Security compliance layer | Read signature-verification verdicts |
| Party Agent (accredited) | Transparency consumer | Read own-party aggregate compliance summary only |

---

## 3. Functional Requirements

**FR-M62-01 — Transmission event ingestion:** Consume every §39 transmission event from M10 (RTS Backend) via event stream. Each event: station code, transmission type (Form 32A/32B/34A/34B/34C), transmission timestamp, transmitting officer ID, digital signature reference (P06 HSM), party-agent notification status, chain-of-custody metadata. Ingest within 30 s of M10 event publication.

**FR-M62-02 — Statutory compliance rule engine:** A configurable rule set mapping each statutory requirement to a verification check. Rules cover:
- Form compliance: correct form type for the contest level (§39 + reg. 6)
- Transmission window: within the prescribed hours per Elections Regulations (reg. 7)
- Authorised signatory: transmitting officer accredited and active in M12 CRMS (§39(3))
- Digital signature: valid P06 HSM signature present and verified (Elections Technology Regulations reg. 10)
- Party-agent notification: at least one accredited party agent acknowledged before transmission (§44A)
- Chain-of-custody: custody log complete from polling close to transmission (M20 link)
- Sequence integrity: transmission sequence consistent with station-close time (no pre-close transmission)

**FR-M62-03 — Per-transmission verdict:** For each transmission event, the rule engine produces a structured compliance verdict: overall pass/fail/partial, per-rule result (pass/fail/not-applicable), failure reason codes, evidence references (hashes of source records). Verdicts immutable once generated; stored in P05 Merkle chain.

**FR-M62-04 — Aggregated compliance dashboard:** Real-time aggregate view: total transmissions received, compliance rate by rule, non-compliant transmission queue, county-level compliance heat map. Updates in real time during election-day transmission window.

**FR-M62-05 — Non-compliance escalation:** Any transmission verdict with overall fail or any critical-rule fail (signatory, signature, pre-close transmission) generates an immediate alert to the Commission RTS Operations Lead and the Commission Legal Counsel. Alert includes station code, failing rules, evidence hashes. Operations Lead can trigger a re-transmission request or initiate a manual verification workflow.

**FR-M62-06 — §39 compliance record generation:** After transmission window closes (T+1 at latest), generate a per-station §39 compliance record: all transmissions for the station, verdict per transmission, final compliance status. Records signed by P06 HSM. Aggregate §39 compliance certificate for the full election generated on the Commission Legal Counsel request.

**FR-M62-07 — Evidence bundle for petition:** On the Commission Legal Counsel request, generate a signed evidence bundle for specified stations or the full election: per-transmission verdicts, rule engine configuration at the time of audit, source event hashes, P05 Merkle inclusion proofs. Bundle generated within 4 hours of request. Feeds M40 for active-petition use.

**FR-M62-08 — Rule engine versioning:** Every change to the compliance rule set is version-controlled with author, change description, effective date, and the Commission Legal Counsel approval. Verdicts reference the rule-engine version active at the time of audit. Retrospective re-audit with a different rule version is possible for analysis; only the original-version verdict is authoritative.

**FR-M62-09 — Party-agent transparency:** Accredited party agents can query the aggregate compliance summary for their own party's contested stations (not individual transmission details). Summary: pass rate, most common failure reason, number of escalations. Supports M30 party-agent oversight without exposing internal operational details.

**FR-M62-10 — §44A invocation audit:** Every §44A declaration event (from M10) is tracked in M62: which officer invoked, at which station, at what time, with what reason code, and whether the invocation satisfied the pre-agreed protocol. Results feed the M15 IAB Dashboard compliance panel.

---

## 4. Non-Functional Requirements

**NFR-M62-01 — Performance:** Transmission event to compliance verdict p99 < 5 s. Dashboard refresh p99 < 1 s. Evidence bundle generation < 4 hours for full election.

**NFR-M62-02 — Throughput:** Handle peak of 2,000 transmission events per minute (election-day concentration when all stations transmit close results simultaneously).

**NFR-M62-03 — Reliability:** 99.9% uptime during T-1 to T+2 (transmission window). Verdicts immutable; no data loss acceptable.

**NFR-M62-04 — Immutability:** All verdicts appended to P05 Merkle chain immediately on generation. Retroactive modification impossible at API level; any attempt logged as security event.

**NFR-M62-05 — Security:** M62 has read-only access to M10 event stream. No write path to any RTS component. Rule engine changes require the Commission Legal Counsel approval (audit trail in P05). Party-agent read access scoped to aggregate summaries only.

**NFR-M62-06 — DPA 2019 lawful basis:** Transmission metadata processed under s.30(1)(b) — legal obligation (Elections Act §39). Transmitting officer IDs processed under s.30(1)(d). Voter-level data is not in scope. Retention: 7 years post-election.

**NFR-M62-07 — WCAG 2.2 AA:** Compliance dashboard and party-agent summary portal.

**NFR-M62-08 — SLA:** Verdict generated within 5 s. Non-compliance alert delivered to Legal Counsel within 30 s. Evidence bundle within 4 hours.

**NFR-M62-09 — Language:** EN + SW for party-agent summary portal and dashboard.

---

- **NFR-62-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-62` (replace 62 with the module number); alert ruleset ID `iems-62-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the Commission RTS Operations Lead, I see a real-time compliance rate dashboard during the election-day transmission window, with non-compliant transmissions highlighted immediately.
2. As the Commission Legal Counsel, I receive an immediate alert when a transmission from any station fails the digital-signature check, so I can trigger a re-transmission request before the window closes.
3. As the Datacraft Compliance Lead, I update the party-agent-notification rule to reflect a last-minute regulation amendment; the change is approved by the Commission Legal Counsel and takes effect at the specified time.
4. As the IAB Technical Member, I view the aggregate §39 compliance summary at T+1 and counter-sign the compliance certificate for inclusion in the IAB report.
5. As the Commission Legal Counsel, I request an evidence bundle for all 47 Nairobi county stations and receive a signed HSM-timestamped package within 4 hours.
6. As the M40 IT Liaison, I retrieve the compliance records for ten contested stations and pass them to the election-petition response team.
7. As an accredited party agent, I view the aggregate compliance rate for my party's contested seats and see that 98% of transmissions passed all statutory checks.
8. As the Commission Legal Counsel, I run a retrospective re-audit with the rule engine version active before a last-minute amendment, to produce an analysis supporting a petition response.
9. As the Datacraft Compliance Lead, I confirm that all §44A invocation events have been audited and that none violated the pre-agreed protocol.
10. As the Commission CISO, I review all transmissions where the signature-verification rule failed and confirm each was escalated and resolved before the transmission window closed.
11. As the IAB Technical Member, I use the M15 compliance panel to verify that M62's §44A audit results are consistent with the IAB's own anomaly review.

---

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime
from typing import Literal

class TransmissionEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                  # uuid7str(); mirrors M10 event ID
    station_code: str
    form_type: Literal["32A","32B","34A","34B","34C"]
    transmitted_at: datetime
    officer_id: str
    signature_ref: str       # P06 HSM signature ID
    party_agent_notification_status: Literal["confirmed","pending","not_required"]
    custody_chain_complete: bool
    m10_event_hash: str      # hash of source M10 event
    ingested_at: datetime

class ComplianceRule(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    rule_code: str           # e.g. "SIG_VALID", "WINDOW_OK", "OFFICER_ACCREDITED"
    description: str
    statutory_basis: str     # e.g. "Elections Act s.39(3)"
    version: int
    effective_from: datetime
    approved_by: str         # the Commission Legal Counsel ID
    active: bool

class TransmissionVerdict(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    transmission_id: str
    rule_engine_version: int
    overall: Literal["pass","partial","fail"]
    rule_results: dict[str, dict]  # rule_code -> {"result": "pass"|"fail"|"na", "reason": str | None}
    generated_at: datetime
    merkle_leaf_hash: str    # P05 Merkle leaf
    merkle_inclusion_proof: str | None

class ComplianceEscalation(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    verdict_id: str
    station_code: str
    failing_rules: list[str]
    alerted_at: datetime
    alerted_to: list[str]    # officer IDs
    resolution: Literal["retransmission","manual_verification","accepted","unresolved"] | None
    resolved_at: datetime | None

class Section39ComplianceRecord(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    transmissions: list[str]  # verdict IDs
    final_status: Literal["compliant","non_compliant","partially_compliant"]
    generated_at: datetime
    signed_pdf_hash: str
    merkle_inclusion_proof: str

class EvidenceBundle(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    requested_by: str
    station_codes: list[str] | None  # None = full election
    generated_at: datetime
    bundle_hash: str
    merkle_root: str
    delivered_to: str | None
    delivered_at: datetime | None
```

---

## 7. API Surface (OpenAPI 3.1)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/transmission-audit/verdicts` | COMMISSION_LEGAL or COMMISSION_RTS JWT | 200/min | List verdicts; filter by station, status, rule |
| GET | `/api/v1/transmission-audit/verdicts/{id}` | COMMISSION_LEGAL JWT | 100/min | Verdict detail with rule results |
| GET | `/api/v1/transmission-audit/dashboard` | COMMISSION_RTS JWT | 200/min | Aggregate compliance metrics |
| GET | `/api/v1/transmission-audit/escalations` | COMMISSION_LEGAL JWT | 60/min | Non-compliance escalation queue |
| PATCH | `/api/v1/transmission-audit/escalations/{id}` | COMMISSION_RTS JWT | 30/min | Update escalation resolution |
| GET | `/api/v1/transmission-audit/s39-records` | COMMISSION_LEGAL or M40 JWT | 60/min | Per-station §39 compliance records |
| POST | `/api/v1/transmission-audit/evidence-bundles` | COMMISSION_LEGAL JWT | 5/hr | Request evidence bundle |
| GET | `/api/v1/transmission-audit/evidence-bundles/{id}` | COMMISSION_LEGAL or M40 JWT | 20/min | Bundle status and download hash |
| GET | `/api/v1/transmission-audit/rules` | COMMISSION_LEGAL JWT | 30/min | Rule engine versions |
| PATCH | `/api/v1/transmission-audit/rules/{id}` | COMMISSION_LEGAL JWT | 5/hr | Update rule (requires approval event) |
| GET | `/api/v1/transmission-audit/s44a-audit` | IAB or COMMISSION_LEGAL JWT | 30/min | §44A invocation audit records |
| GET | `/api/v1/transmission-audit/party-summary` | PARTY_AGENT JWT | 30/min | Aggregate compliance summary for own party |
| GET | `/api/v1/transmission-audit/stream` | M15 or IAB JWT | SSE | Real-time compliance event stream |

---

## 8. Integrations

- **M10 RTS Backend** — source of all transmission events (event stream; read-only)
- **M12 CRMS** — officer accreditation verification for signatory rule
- **M20 IMS** — chain-of-custody completeness check
- **M26 Anomaly Detection** — M62 non-compliance events can seed M26 anomaly rules
- **M40 Election Court IT** — evidence bundle consumer for active petitions
- **M15 IAB Dashboard** — compliance summary and §44A audit feed; SSE stream
- **P05 Audit Log** — all verdicts Merkle-anchored; all escalation events logged
- **P06 HSM** — evidence bundle signing; §39 compliance record signing

---

## 9. UI/UX Key Screens

1. **Compliance Dashboard** — live transmission count; overall pass rate; rule-by-rule breakdown; county compliance heat map; non-compliant queue with severity badges.
2. **Verdict Explorer** — searchable table: station code, form type, overall result, transmission time; click through to per-rule detail view with evidence hashes.
3. **Non-Compliance Queue** — escalation list; time-since-alert countdown; resolution workflow (retransmission / manual verification / accept / unresolved).
4. **§39 Record Browser** — per-station compliance record; all transmissions for that station; final compliance status; signed PDF download.
5. **Evidence Bundle Builder** — station code multi-select (or full-election toggle); generate request; status polling; bundle hash and download on completion.
6. **Rule Engine Manager** — version history; active rules; edit-and-approve workflow; effective-date scheduling.
7. **Party-Agent Summary Portal** — aggregate pass rate for own party's stations; most common failure reason; no per-transmission detail.

---

## 10. Compliance Mapping

| Requirement | Source |
|---|---|
| Electronic transmission compliance | Elections Act 2011 s.39 |
| Declaration procedure | Elections Act 2011 s.44A |
| Technology transmission standards | Elections (Technology) Regulations 2017 reg. 6, 7, 10 |
| Digital signature requirement | Kenya Information and Communications Act Cap 411A; Digital Security Act 2023 |
| Pre-emptive evidence bundle | Strategy Part XII.4; Raila v the Commission [2017] eKLR para 274-280 |
| DPA 2019 lawful basis | DPA 2019 s.30(1)(b) — legal obligation |
| Record retention | Public Archives and Documentation Service Act Cap 19 |

---

## 11. Success Metrics + SLA

| Metric | Target | Sign-off |
|---|---|---|
| Verdict generated per transmission | p99 < 5 s | Datacraft SRE |
| Non-compliance alert delivered | p99 < 30 s from verdict generation | Datacraft SRE |
| Evidence bundle for full election | < 4 hours from request | Datacraft Compliance Lead |
| §39 records for all stations | Generated by T+2 | the Commission Legal Counsel |
| All verdicts in P05 Merkle chain | 100% | Datacraft SRE |
| §44A audit coverage | 100% of §44A events audited | Datacraft Compliance Lead |
| M62 availability during transmission window (T-1 to T+2) | 99.9% | Datacraft SRE |

---

## 12. Test Strategy

- **Unit:** Each compliance rule in isolation: signatory check, window check, signature verification, custody-chain completeness, sequence-integrity.
- **Integration:** M10 event stream ingestion; P05 Merkle anchoring of verdicts; M12 officer accreditation lookup; M20 custody-chain query.
- **Load:** 2,000 transmission events per minute sustained for 2 hours; confirm all verdicts generated within 5-second p99.
- **Immutability:** Attempt to modify a generated verdict via API; verify blocked and logged.
- **Evidence bundle:** Generate full-election bundle in staging; verify all stations represented; confirm HSM signature valid.
- **Chaos:** M12 CRMS unavailable during transmission window; verify rule engine degrades gracefully (caches last-known accreditation state; flags degraded-mode verdicts).
- **Party-agent access:** Confirm party agent cannot see stations outside their registered party's contests.

---

## 13. Rollout (T-0 = 2027-08-10)

| Phase | Date | Milestone |
|---|---|---|
| Rule engine seeded with statutory requirements | T-365 (2026-10-10) | Legal Counsel approves initial rule set |
| M10 event stream integration tested | T-180 (2027-02-10) | Staging event ingestion confirmed |
| Full-election simulation drill | T-60 (2027-06-11) | 47,000 station simulation; verdict generation and evidence bundle tested |
| Pre-election §39 briefing for Legal Counsel | T-30 (2027-07-11) | Rule engine walk-through; party-agent portal demonstrated |
| Transmission window live | T-0 (2027-08-10) | All feeds active; Legal Counsel on standby |
| §39 compliance records complete | T+2 (2027-08-12) | All stations covered |
| Evidence bundle pre-generated | T+7 (2027-08-17) | Petition window pre-position complete |

---

## 14. Dependencies

- **M10 RTS Backend** — primary event source
- **M12 CRMS** — officer accreditation
- **M20 IMS** — custody-chain data
- **M40 Election Court IT** — evidence bundle consumer
- **M15 IAB Dashboard** — compliance stream consumer
- **P05, P06** — audit log and HSM signing
- the Commission Legal Counsel approval for rule engine changes — standing dependency

---

## 15. Risks + Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Last-minute regulation amendment changes statutory requirements after rule engine is locked | Rule engine supports effective-date scheduling; Legal Counsel can approve and schedule a rule update up to T-1; all verdicts reference rule-engine version at time of audit |
| R2 | M10 event stream lag causes verdicts to lag behind actual transmissions | Latency monitoring on M10 ingestion; alert to Datacraft SRE if ingestion lag exceeds 60 s; manual event re-push from M10 as fallback |
| R3 | High-volume transmission burst exceeds 2,000 events/minute | Horizontal scaling of verdict workers; queue-based ingestion with at-least-once delivery; load tested at 3,000 events/minute in staging |
| R4 | Compliance verdict challenged as algorithmically biased (rule applied inconsistently) | Rule engine is fully deterministic and version-controlled; same input always produces same verdict; Legal Counsel can produce rule-engine version and evidence hashes for any verdict in court |
| R5 | Evidence bundle generation fails during petition filing window | Bundle pre-generated at T+7 and stored; Datacraft SRE monitors bundle storage; fallback: per-station §39 records individually signed and retrievable within 1 hour each |

---

## 16. Cross-References

- **M10** — RTS Backend: source of all transmission events
- **M26** — Anomaly Detection: M62 non-compliance events can seed M26 rules; M26 asks statistical questions, M62 asks legal-compliance questions
- **M40** — Election Court IT: evidence bundle consumer; M58 pre-registers bundle structure with SC IT Registrar
- **M15** — IAB Dashboard: §44A audit feed; compliance stream
- **M20** — IMS: custody-chain completeness check
- **M61** — Secrecy Assurance Testing: M61 tests whether transmissions leak ballot secrecy; M62 tests whether they comply with statutory form

---

## 17. Strategy Traceability

| Strategy Obligation | Covered By |
|---|---|
| §39 transmission compliance verification | FR-M62-01, FR-M62-02, FR-M62-03 |
| Per-transmission compliance record | FR-M62-06 |
| Pre-emptive evidence bundle | FR-M62-07 |
| §44A invocation audit | FR-M62-10 |
| Rule engine versioning and auditability | FR-M62-08 |
| Party-agent transparency | FR-M62-09 |
| DPA 2019 compliance | NFR-M62-06 |

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
