# M51 — Result Declaration and Certification

**Module ID:** M51
**Name:** Result Declaration and Certification
**Runs on:** Central servers (Kenyan cloud) + IEBC Constituency Tallying Centres + National Tallying Centre
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Formalise the Returning Officer declaration workflow distinct from tally publication (M10/M11). Covers winner determination, Certificate of Election issuance, formal Gazette notice preparation (routed to M52), and handoff notices to the Judiciary and Attorney General. Statutory basis: Elections Act §39, §85, §86. Also supports referenda and by-elections (M49, M50).

## 2. User Personas

- **Constituency Returning Officer (RO)** — declares winner; signs certificate; triggers gazette notice
- **County Returning Officer (CRO)** — for county-level seats (Governor, Senator, CWR)
- **Presidential Returning Officer (IEBC Chair)** — declares presidential result at national level
- **IEBC Legal Officer** — reviews declaration for statutory compliance before signature
- **Judiciary (Registrar, Supreme Court / High Court)** — receives formal certification handoff
- **Attorney General** — receives gazette notice trigger
- **Gazette Government Printer** — receives gazette notice via M52
- **Losing Candidate** — receives formal notification of result; petition window tracked
- **IEBC DPO** — oversees DPA 2019 compliance for candidate personal data in certificates

## 3. Functional Requirements

- **FR-M51-001** Declaration trigger: RO initiates declaration after all Forms 34A / 37A for their scope are confirmed in M10; system blocks declaration if outstanding unconfirmed forms exceed configurable threshold (default: 0 permitted outstanding).
- **FR-M51-002** Winner determination: system computes winner from signed tally per Elections Act §39 rules (simple plurality for most seats; >50% first-round rule for Presidential with run-off trigger).
- **FR-M51-003** Tie-breaking: if votes are equal, system prompts RO to conduct statutory draw; draw outcome recorded with witness signatures.
- **FR-M51-004** Pre-declaration checklist: mandatory review checklist before declaration — all forms confirmed, total votes reconciled against turnout, quorum of agents present, IEBC legal sign-off status.
- **FR-M51-005** Declaration statement generation: system generates a statutory declaration statement (EN + SW) pre-populated with candidate names, vote totals, and winning margin; RO reviews and signs via HSM-backed e-signature or wet-ink with scan upload.
- **FR-M51-006** Certificate of Election: auto-generate Form 39 (Certificate of Election) per IEBC format with winner details, seat, constituency, election date, RO signature, IEBC seal. PDF signed with HSM; UUID anchored in P05 Merkle chain.
- **FR-M51-007** Gazette notice preparation: push notice package to M52 for formal Gazette publication; include election type, seat, winner name, vote totals, date of declaration.
- **FR-M51-008** Judiciary handoff: generate formal notification letter (PDF, HSM-signed) for the relevant court — High Court for most seats, Supreme Court for Presidential; delivered via secure API endpoint (MoU).
- **FR-M51-009** Attorney General notification: formal notice within 24 hours of Presidential declaration per Constitution Art 138(5); automated delivery with read-receipt.
- **FR-M51-010** Losing-candidate notification: automated email + SMS to all non-winning candidates with final vote totals and petition window deadline.
- **FR-M51-011** Petition window tracking: system sets a 28-day countdown from declaration date; alerts IEBC legal at T-7, T-3, T-1; if petition lodged via court integration, status updated.
- **FR-M51-012** Run-off trigger: for Presidential result where no candidate passes 50%+1 threshold and 25%-in-24-counties rule, system flags run-off requirement and activates M49 planning (treated as a second election event).
- **FR-M51-013** Declaration log: immutable, cryptographically sealed log of every declaration event, actor, timestamp, and output document hash.

## 4. Non-Functional Requirements

- **NFR-M51-001** Pre-declaration checklist completion to RO signing <= 15 min (system processing; RO review time excluded).
- **NFR-M51-002** Certificate of Election generation <= 60 s after RO signature.
- **NFR-M51-003** Gazette notice package delivered to M52 <= 5 min after RO signature.
- **NFR-M51-004** Judiciary notification delivery acknowledged <= 30 min (automated retry up to 3 times).
- **NFR-M51-005** WCAG 2.2 AA for all RO-facing declaration screens.
- **NFR-M51-006** Languages: EN + SW for all generated documents.
- **NFR-M51-007** DPA 2019 lawful basis: candidate personal data in certificates processed under s.30(1)(b) (legal obligation); certificates retained 7 years; personal data in losing-candidate notifications minimised to name + totals.
- **NFR-M51-008** Availability: >= 99.95% during tallying and declaration window (T-0 to T+3 days).
- **NFR-M51-009** Declaration events immutable: no soft-delete; correction process requires a formal RO correction notice with reason, signed, appended to original — original never overwritten.

- **NFR-51-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-51` (replace 51 with the module number); alert ruleset ID `iems-51-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Returning Officer, I see a pre-declaration checklist confirming all 47 polling stations in my constituency have confirmed Form 34A before I can proceed to declaration, preventing premature announcement.
2. As a Returning Officer, I click "Generate Declaration" and the system produces a pre-filled statutory statement with candidate names and vote totals, so I verify rather than type.
3. As an IEBC Legal Officer, I receive a declaration for review 30 minutes before the RO is permitted to sign, so I can flag any discrepancy.
4. As a winning candidate, I receive a digitally signed Certificate of Election PDF within 5 minutes of the RO declaration, with a Merkle inclusion proof.
5. As a losing candidate, I receive an SMS and email with final vote totals and the exact date the 28-day petition window closes.
6. As the Supreme Court Registrar, I receive a formal presidential-result notification via secure API with a signed PDF within 1 hour of the IEBC Chair's declaration.
7. As the Attorney General, I receive an automated notice of the presidential result within 24 hours of declaration per Art 138(5).
8. As an IEBC Legal Officer, I see a countdown dashboard showing all active petition windows and their deadlines by constituency.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime

class Declaration(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    election_event_id: str
    seat_type: Literal["PRESIDENTIAL","GOVERNOR","SENATOR","MP","CWR","MCA","REFERENDUM"]
    constituency_code: str
    declared_winner_candidate_id: str | None  # None for referendum
    referendum_result: Literal["PASSED","FAILED"] | None
    declared_by: str                 # RO user ID
    declared_at: datetime
    declaration_statement_hash: str  # SHA-256 of signed PDF
    certificate_of_election_id: str | None
    gazette_notice_id: str | None    # FK to M52
    judiciary_notification_sent_at: datetime | None
    ag_notification_sent_at: datetime | None
    petition_window_closes: date
    petition_lodged: bool
    status: Literal["DRAFT","LEGAL_REVIEW","DECLARED","GAZETTE_PENDING","CERTIFIED","PETITIONED"]
    merkle_inclusion_proof: str | None

class CertificateOfElection(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    declaration_id: str
    candidate_id: str
    candidate_name_en: str
    seat: str
    constituency: str
    election_date: date
    votes_received: int
    total_valid_votes: int
    ro_signature_hash: str
    iebc_seal_hash: str
    pdf_file_id: str
    merkle_root: str
    issued_at: datetime

class DeclarationTieBreak(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    declaration_id: str
    tied_candidate_ids: list[str]
    draw_method: Literal["LOT","COIN"]
    winner_candidate_id: str
    witness_signatures: list[str]
    conducted_at: datetime
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/declarations:
    post:
      summary: Initiate declaration workflow for a constituency
      security: [{ROJwt: []}]
  /v1/declarations/{id}/checklist:
    get:
      summary: Pre-declaration checklist status
      security: [{ROJwt: []}]
  /v1/declarations/{id}/legal-review:
    post:
      summary: IEBC Legal Officer sign-off
      security: [{LegalOfficerJWT: []}]
  /v1/declarations/{id}/sign:
    post:
      summary: RO signs declaration with HSM
      security: [{ROHSM: []}]
  /v1/declarations/{id}/certificate:
    get:
      summary: Download Certificate of Election PDF
      security: [{PublicRead: []}]
  /v1/declarations/{id}/petition-window:
    get:
      summary: Petition window status and deadline
      security: [{IEBCLegalJWT: []}]
  /v1/declarations/{id}/petition:
    post:
      summary: Record petition lodged (court integration)
      security: [{JudiciaryAPIKey: []}]
```

## 8. Integrations

- **M10 RTS Back-End** — source of confirmed tally data; declaration blocked until tally confirmed
- **M11 Public Portal** — declaration events surfaced on public results page
- **M12 CRMS** — candidate identity for certificates and notifications
- **M49 Referendum Mechanics** — referendum-specific declaration path
- **M50 By-Election Operations** — by-election declaration variant
- **M52 Election Calendar + Gazette** — gazette notice package delivery
- **P05 Merkle/Audit Chain** — certificate and declaration hash anchoring
- **P06 HSM** — RO and IEBC Chair digital signatures
- **Judiciary API** (external, MoU) — formal notification delivery
- **AG Office API** (external, MoU) — presidential result notification

## 9. UI/UX Key Screens

1. **Pre-Declaration Checklist** — RO view; per-station form status; reconciliation totals; legal review status; each item with green/red badge.
2. **Declaration Statement Preview** — bilingual pre-filled document; edit-lock after legal review; sign button activates only when all checks pass.
3. **Certificate of Election Viewer** — candidate-facing page; PDF download; Merkle proof; QR code for verification.
4. **Petition Window Dashboard** — IEBC Legal Officer view; countdown per constituency; "Petition Lodged" button; expiry alerts.
5. **Losing Candidate Notification Log** — IEBC admin view; delivery status per candidate; retry button on failure.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Winner declaration procedure | Elections Act 2011 §39 |
| Certificate of Election form | Elections Act 2011 §85 |
| Gazette notice of result | Elections Act 2011 §86 |
| Presidential AG notification | Constitution Art 138(5) |
| Petition window | Constitution Art 87; Elections Act §75 |
| Run-off trigger | Constitution Art 138(4) |
| Candidate data protection | DPA 2019 s.30(1)(b) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Certificate issued after RO signature | <= 60 s | Certificate issuance log |
| Gazette notice delivered to M52 | <= 5 min | M52 ingestion log |
| Judiciary notification acknowledged | <= 30 min | Delivery receipt log |
| Petition window deadline alerts | 100% at T-7, T-3, T-1 | Alert system log |
| Declaration log tamper-evidence | 100% events in Merkle chain | P05 reconciliation |

## 12. Test Strategy

- **Unit:** Winner determination (plurality; 50%+1; tie-break); run-off trigger logic.
- **Integration:** Tally-to-declaration pipeline; M52 gazette package delivery; Judiciary API mock.
- **E2E:** Full constituency declaration from tally confirmation through certificate issuance and candidate notification.
- **Security:** Attempt to declare before all forms confirmed; attempt to modify signed certificate.
- **Accessibility:** WCAG 2.2 AA axe-core on RO declaration screens and public certificate viewer.
- **Chaos:** Judiciary API unavailable — verify retry and queue behaviour.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| RO declaration module UAT | 2027-03-01 | IEBC RO training cohort acceptance |
| Judiciary API MoU and integration | 2027-04-01 | Dry-run with Judiciary IT |
| AG notification integration | 2027-04-15 | AG Office API dry-run |
| Full E2E test declaration cycle | 2027-06-01 | Mock general election simulation |
| Production ready | T-14 (2027-07-27) | |

## 14. Dependencies

- M10 RTS Back-End (tally source)
- M12 CRMS (candidate identity)
- M52 Election Calendar + Gazette (gazette delivery)
- P05 Merkle/Audit Chain
- P06 HSM
- Judiciary API (external MoU)
- AG Office API (external MoU)

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| RO declares before all forms confirmed | Low | Critical | Hard system block; cannot bypass without CEO override with audit log |
| Certificate PDF challenged as unsigned | Low | High | Merkle proof + HSM signature constitutes statutory evidence; pre-registered with Judiciary |
| Judiciary API unavailable at declaration | Medium | High | Queue with 3 retries; fallback to secure email delivery with manual acknowledgement |
| Tie-break procedure not conducted correctly | Low | High | System records draw method, witnesses; procedural guide embedded in UI; IEBC Legal Officer required as witness |
| Petition window missed by IEBC legal | Medium | Medium | Three-level alert cascade; calendar entry pushed to M52 |

## 16. Cross-References

- `M10-rts-backend.md` — tally source for declaration
- `M11-public-portal.md` — declaration events on public portal
- `M12-crms.md` — candidate identity
- `M49-referendum-mechanics.md` — referendum declaration path
- `M50-by-election-operations.md` — by-election declaration variant
- `M52-election-calendar-gazette.md` — gazette notice delivery

## 17. Strategy Traceability

- Elections Act 2011 §39 — winner determination
- Elections Act 2011 §85 — Certificate of Election
- Elections Act 2011 §86 — gazette notice of result
- Constitution Art 138(4)(5) — presidential run-off and AG notification
- Constitution Art 87 — petition window

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
