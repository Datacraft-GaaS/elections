# M22 — Campaign Finance Transparency Portal

**Module ID:** M22
**Name:** Campaign Finance Transparency Portal
**Runs on:** Central servers (Kenyan cloud) + the Commission back-office + party-facing portal
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Enforce the Election Campaign Financing Act 2013 (ECFA) disclosure obligations for all candidates and political parties. Provide a public real-time dashboard of declared campaign spending vs statutory ceilings, integrate with CBK and FRC for aggregated cash-movement monitoring, detect POCAMLA 2009 suspicious-activity patterns, and route evidence packages for Election Offences Act bribery referrals to DCI/ODPP.

Strategy reference: §XX / §XVIII.4.

## 2. User Personas

- **Candidate** — files weekly spending disclosures; views own compliance status
- **Party Finance Officer** — files party-level disclosures on behalf of candidates; manages M-Pesa Paybill uploads
- **the Commission Campaign Finance Officer** — reviews filings, flags ceiling breaches, issues compliance notices
- **Registrar of Political Parties** — receives aggregate compliance reports
- **ODPP Prosecutor** — receives evidence packages for ceiling-breach and bribery referrals
- **FRC / CBK Analyst** — views aggregated cash-movement heat maps under MoU; submits SAR triggers
- **Public User** — reads the public spending dashboard; no login required
- **Whistleblower** — submits tip via protected channel (Whistleblower Protection Act)
- **the Commission Commissioner (Chair)** — signs off on compliance reports; triggers ODPP referrals
- **Datacraft DPO** — oversees DPA 2019 compliance for all personal data processed

## 3. Functional Requirements

- **FR-M22-001** Candidate registration: every gazetted candidate must register on the portal within 7 days of gazettement; name, constituency, party, and ECFA spending-ceiling tier auto-populated from M12 CRMS.
- **FR-M22-002** Weekly disclosure upload: candidates upload (a) bank statement PDF/OFX, (b) M-Pesa Paybill activity report (CSV), (c) itemised expense receipts PDF per ECFA 2013 s.22 format. System extracts totals via OCR + structured parser; candidate confirms before submission.
- **FR-M22-003** Spending ceiling enforcement: compute cumulative declared spending per candidate per contest; compare to ECFA 2013 s.22 ceiling for that contest tier. Flag in real time when any candidate exceeds 80% of ceiling (amber), 100% (red breach). Red breach triggers automatic the Commission notification + ODPP referral queue entry.
- **FR-M22-004** Aggregated cash-movement heat map: receive daily aggregated cash-out-volume data per constituency from CBK/FRC under MoU (no individual transactions; no personal data). Render as choropleth map on public dashboard. Flag constituencies with >2 standard-deviation spike from 30-day baseline during campaign period. Spikes generate FRC SAR alert within 30 minutes.
- **FR-M22-005** POCAMLA 2009 pattern detection: apply SAR trigger rules (bulk cash-out near polling stations, rapid round-trip transactions, structuring patterns) to the aggregated CBK/FRC data. Auto-generate SAR draft for FRC analyst review; analyst confirms before submission.
- **FR-M22-006** Foreign-money detection: flag any candidate bank statement showing inbound cross-border transfers; cross-reference against ECFA 2013 ss.32-34 prohibition; auto-generate referral for CBK/FRC review.
- **FR-M22-007** Whistleblower tip line: protected channel per Whistleblower Protection Act 2022; anonymous submission of cash-bribery incidents with photo, geolocation, and optional statement; verified tips routed to DCI + ODPP within 6 hours with evidence package.
- **FR-M22-008** Evidence package generation: for each ceiling-breach or bribery tip, assemble a structured evidence package (HSM-signed PDF + JSON) containing: candidate identity, relevant disclosure extracts, uploaded bank statements, computed totals, statutory ceiling value, excess amount, referral basis, timestamp chain. Package hash anchored in P05 Merkle chain.
- **FR-M22-009** Compliance rate tracking: dashboard shows percentage of registered candidates who have filed a complete weekly disclosure; the Commission sends automated non-compliance reminder at T+3 past deadline; formal notice at T+7.
- **FR-M22-010** Public spending dashboard: per-candidate spending bar chart vs ceiling; constituency comparison view; weekly trend chart; downloadable dataset in CSV + JSON; no raw bank statements published (those are Commission-internal).
- **FR-M22-011** Post-election analytics: per-candidate cost-per-vote regression once tallies are published; published as an aggregate research dataset (no individual voter data).
- **FR-M22-012** Bank and Sacco disclosure integration: via adapter, aggregate financial-institution disclosures at constituency level for the election-cycle window; cross-reference against candidate declared filings.
- **FR-M22-013** ECFA prosecution registry: public-facing list of Election Offences Act s.12 cases filed, outcome, and judgment date; updated within 14 days of each judgment.

## 4. Non-Functional Requirements

- **NFR-M22-001** Candidate portal p99 response time <= 1 second under 5,000 concurrent users.
- **NFR-M22-002** Public dashboard p99 <= 2 seconds under 100,000 concurrent users.
- **NFR-M22-003** Weekly disclosure parsing (OCR + structured extract) completes within 5 minutes of upload.
- **NFR-M22-004** SAR-flag generation and FRC routing: < 30 minutes from triggering event.
- **NFR-M22-005** WCAG 2.2 AA for candidate portal, the Commission back-office, and public dashboard.
- **NFR-M22-006** Languages: EN + SW (candidate portal and public dashboard); CN, Arabic, French for foreign-linked entity cross-references (the Commission internal only).
- **NFR-M22-007** DPA 2019 lawful basis: candidate financial disclosures processed under s.30(1)(b) (legal obligation); bank statements stored encrypted at rest (AES-256); retention 7 years post-election per ECFA; DPIA registered with ODPC.
- **NFR-M22-008** Availability: >= 99.9% during campaign period (T-90 to T+30).
- **NFR-M22-009** Audit trail: every the Commission officer action on a candidate record logged with actor, action, timestamp, IP, and purpose code.
- **NFR-M22-010** No individual bank transaction data visible to any party other than the candidate who filed it and the Commission Compliance Officer under a logged access event.

## 5. User Stories

1. As a candidate, I upload my weekly M-Pesa Paybill CSV and the system extracts total expenditure automatically, so I do not need to manually compute totals.
2. As a Commission Campaign Finance Officer, I see a real-time red banner when any candidate crosses their spending ceiling, so I can issue a formal notice without delay.
3. As a member of the public, I can view a map showing aggregated cash-movement spikes near polling stations, so I can identify potential vote-buying hotspots.
4. As an FRC analyst, I receive a SAR draft with all required statutory fields pre-populated from the system's pattern detection, so I only need to confirm and submit.
5. As a whistleblower, I can submit a photo, location, and description of a cash-for-vote exchange anonymously without logging in.
6. As an ODPP Prosecutor, I receive a signed evidence package via secure portal within 6 hours of a verified whistleblower tip.
7. As a researcher, I can download the post-election per-candidate spending dataset as CSV to run independent analysis.
8. As a Registrar of Political Parties, I receive a weekly compliance-rate summary showing which parties have outstanding disclosure filings.
9. As a party finance officer, I can file disclosures on behalf of multiple candidates under the same login, with separate submission records per candidate.
10. As the Commission Chair, I sign off on the ODPP referral package with my HSM-backed digital signature before it is transmitted.

## 6. Data Model Additions

```python
class CandidateDisclosure(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    candidate_id: str               # FK to M12 CRMS candidate record
    contest_id: str
    period_start: date
    period_end: date
    bank_statement_file_id: str     # encrypted blob reference
    mpesa_statement_file_id: str | None
    receipts_file_id: str | None
    declared_total_kes: Decimal
    ocr_extracted_total_kes: Decimal | None
    discrepancy_flag: bool
    ceiling_kes: Decimal            # from ECFA 2013 s.22 lookup
    ceiling_pct_used: float
    status: Literal["DRAFT","SUBMITTED","UNDER_REVIEW","ACCEPTED","QUERIED"]
    submitted_at: datetime | None
    reviewed_by: str | None
    created_at: datetime

class CashMovementAggregate(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    constituency_code: str
    period_date: date
    total_cashout_kes: Decimal      # aggregated from CBK/FRC; no individual data
    zscore_from_baseline: float
    sar_triggered: bool
    source: Literal["CBK","FRC","SACCO"]
    received_at: datetime

class FinanceEvidencePackage(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    candidate_id: str
    referral_type: Literal["CEILING_BREACH","FOREIGN_MONEY","BRIBERY_TIP","POCAMLA_SAR"]
    statutory_basis: str            # e.g. "ECFA 2013 s.22", "EOA 2016 s.12"
    package_hash: str               # SHA-256 of signed PDF
    merkle_inclusion_proof: str | None
    routed_to: Literal["the Commission","ODPP","DCI","FRC","CBK"]
    routed_at: datetime | None
    commission_chair_signed_at: datetime | None
    status: Literal["DRAFT","SIGNED","DELIVERED","ACKNOWLEDGED"]
    created_at: datetime

class BriberyTip(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    channel: Literal["VERIFIER_APP","SHORT_CODE","WHATSAPP","SMS","WEB"]
    anonymous: bool
    reporter_phone_hash: str | None   # SHA-256 of phone; not stored in plain
    latitude: float | None
    longitude: float | None
    constituency_code: str | None
    description_text: str
    photo_file_id: str | None
    received_at: datetime
    severity: Literal["P0","P1","P2"]
    evidence_package_id: str | None
    status: Literal["RECEIVED","TRIAGED","VERIFIED","REFERRED","CLOSED"]
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/finance/candidates/{id}/disclosures | CANDIDATE JWT | 10/day | Submit weekly disclosure |
| GET | /v1/finance/candidates/{id}/disclosures | CANDIDATE JWT | 100/hr | List own disclosures |
| GET | /v1/finance/candidates/{id}/compliance | COMMISSION_OFFICER JWT | 200/hr | Full compliance status |
| GET | /v1/finance/public/spending | Public | 1000/min | Public spending dashboard data (no PII) |
| GET | /v1/finance/public/heat-map | Public | 500/min | Aggregated cash-movement heat map GeoJSON |
| POST | /v1/finance/tips | Public (anon) | 50/hr per IP | Submit bribery tip |
| GET | /v1/finance/tips/{id} | COMMISSION_OFFICER JWT | 100/hr | View tip detail |
| POST | /v1/finance/evidence-packages | COMMISSION_OFFICER JWT | 20/hr | Create evidence package |
| POST | /v1/finance/evidence-packages/{id}/sign | COMMISSION_CHAIR HSM | 5/hr | HSM sign and route to ODPP/DCI |
| GET | /v1/finance/compliance-report | COMMISSION_ADMIN JWT | 10/hr | Aggregate filing compliance rate |
| POST | /v1/finance/sar-triggers | FRC_ANALYST JWT | 20/hr | Record SAR action on a flagged aggregate |
| GET | /v1/finance/prosecution-registry | Public | 500/min | ECFA prosecution outcomes |

All endpoints: TLS 1.3; request logging with actor, IP, purpose code.

## 8. Integration Points

- **M12 CRMS** — candidate registry; spending ceiling by contest tier
- **M23 Grievance Channels** — bribery tips from short-code and WhatsApp ingested here
- **CBK / FRC** (external, MoU) — aggregated cash-movement data feed (no individual transactions)
- **ODPP** (external, MoU) — evidence package delivery endpoint
- **DCI** (external, MoU) — bribery-tip referral endpoint
- **P05 Merkle/Audit Chain** — evidence package hashes anchored
- **P06 HSM** — the Commission Chair digital signature on referral packages
- **M11 Public Portal** — public spending dashboard embedded

## 9. UI/UX Key Screens

1. **Candidate Dashboard** — compliance status card, ceiling-usage progress bar, upcoming disclosure deadline, upload wizard.
2. **Weekly Disclosure Upload** — three-step wizard: (a) upload files, (b) confirm OCR-extracted totals, (c) submit with e-signature. Inline help citing ECFA filing requirements.
3. **the Commission Compliance Console** — candidate table sortable by ceiling-pct-used; red/amber/green badges; one-click "issue formal notice"; referral queue.
4. **Aggregated Cash-Movement Heat Map** — choropleth of Kenya constituencies; colour scale tied to z-score from baseline; click-through to constituency trend chart; legend includes statutory ceiling values per tier.
5. **Evidence Package Builder** — step-by-step: select candidate, choose referral type, auto-attach relevant disclosures, preview PDF, HSM-sign, confirm route to ODPP/DCI.
6. **Public Spending Dashboard** — bar chart per candidate in selected constituency; national league table; downloadable data; last-updated timestamp.
7. **Prosecution Registry** — table with candidate name, contest, charge, case number, outcome, judgment date; search by county or party.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Spending ceilings and disclosure obligation | ECFA 2013 s.22, s.28, s.29 |
| Foreign campaign funding prohibition | ECFA 2013 ss.32-34 |
| Bribery criminalisation | Election Offences Act 2016 s.12 |
| SAR reporting obligation | POCAMLA 2009; FRC Regulations 2013 |
| Whistleblower protection | Whistleblower Protection Act 2022 |
| Personal data processing lawful basis | DPA 2019 s.30(1)(b) — legal obligation |
| Financial institution disclosure | Kenya Banking Act Cap 488; Sacco Societies Act 2008 |
| Access to information (public dashboard) | Access to Information Act 2016 s.4 |
| Prosecution registry | Election Offences Act 2016; ODPP Act 2011 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| ECFA disclosure filing compliance | > 95% of candidates | Weekly compliance report | the Commission + Registrar of Political Parties |
| Ceiling-breach flags routed to ODPP | 100% detected | Compliance dashboard log | the Commission Chair |
| SAR-flag response time (FRC + DCI) | < 30 minutes | FRC incident log | FRC Director |
| Whistleblower tip to ODPP/DCI routing | < 6 hours for verified tips | Tip routing timestamp | the Commission Chair |
| Public dashboard uptime during campaign period | >= 99.9% | Uptime monitor | Datacraft SRE |
| Evidence package hash in Merkle chain | 100% of issued packages | Merkle reconciliation | Datacraft Technical |
| Post-election spending dataset published | <= T+30 | Publication timestamp | the Commission Chair |

## 12. Test Strategy

- **Unit:** ECFA ceiling calculator; OCR extraction parser; POCAMLA rule engine; SAR pattern detector.
- **Integration:** end-to-end disclosure submission to Merkle anchor; FRC data feed ingestion; ODPP evidence package delivery.
- **E2E:** simulated candidate filing sequence with ceiling breach; verify notification chain to the Commission officer, ODPP referral, and public dashboard update.
- **Security:** attempt to access another candidate's bank statement; attempt to modify submitted disclosure after acceptance.
- **Accessibility:** WCAG 2.2 AA on candidate portal and public dashboard (axe-core + manual).
- **Privacy:** confirm no individual transaction data appears in public outputs; confirm bank statement files are encrypted at rest and not exposed via any public API.
- **Load:** 10,000 concurrent candidate uploads on the weekly deadline.
- **Chaos:** CBK/FRC feed unavailability — verify graceful degradation (heat map shows last-known data with staleness warning).

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| CBK/FRC MoU signed | 2026-12-01 | Post-award Commission-facilitated; Datacraft technical spec delivered |
| Candidate portal v1 live | 2027-02-01 | Candidate registration open; mock disclosures accepted |
| ODPP/DCI integration tested | 2027-03-01 | Evidence package delivery dry run |
| Campaign period opens | 2027-05-10 | Weekly disclosure obligations begin; public dashboard live |
| Full heat-map live | 2027-06-01 | CBK/FRC feed active; SAR detection running |
| Post-election analytics published | 2027-09-10 | T+31; spending dataset and cost-per-vote regression |

## 14. Dependencies

- M12 CRMS — candidate registry and contest-ceiling tier lookup
- M23 Grievance Channels — bribery tip ingestion
- CBK/FRC MoU — external; the Commission to negotiate post-award
- ODPP MoU — external; required for evidence package routing
- DCI MoU — external; required for bribery-tip referral
- P05 Merkle chain — for evidence package anchoring
- P06 HSM — for the Commission Chair signing

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CBK/FRC MoU not signed before campaign period | Medium | High | Datacraft builds the aggregated heat-map module independently; heat map launches with whatever data MoU provides; the Commission escalates MoU as post-award priority |
| Low candidate compliance with weekly filing | High | High | Automated reminder sequence; the Commission formal-notice workflow built in; escalation to Registrar of Political Parties for party-level pressure |
| OCR extraction error on non-standard bank statement formats | Medium | Medium | Manual override for the Commission officer; candidate can correct extracted totals with audit trail; new bank format added to OCR training set within 5 days |
| Whistleblower identity exposure | Low | Critical | Phone hash only; no plain-text identity stored; tip ingestion via Tor-accessible endpoint; legal-privilege annotation on tip records |
| Evidence package challenged as fabricated by defendant | Low | High | Merkle inclusion proof + HSM signature + the Commission Chair countersignature constitute a cryptographic chain of custody; pre-registered with Supreme Court IT Registrar |
