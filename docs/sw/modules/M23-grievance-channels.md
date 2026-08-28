# M23 — Grievance Channels

**Module ID:** M23
**Name:** Multi-Channel Grievance and Case-Officer Triage System
**Runs on:** Central servers (Kenyan cloud) + kit-side kiosk mode + Public Verifier app
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Provide a unified, multi-channel grievance intake system spanning the Public Verifier app, *IEBC# USSD short-code, WhatsApp Business, SMS, kit-side kiosk, and IAB in-person office. Route cases to IEBC case officers with defined SLAs per severity. Generate court-admissible Election Offences Act 2016 evidence packages for DCI/ODPP referral. Protect whistleblowers under the Whistleblower Protection Act 2022. Provide transparent case-status tracking for complainants.

Strategy reference: §XIV.2 / §XV / §XVI.3 / §XVIII.5 / §XX.

## 2. User Personas

- **Voter / Citizen Complainant** — reports an election violation or problem; tracks status
- **Whistleblower** — reports bribery, fraud, or misconduct under legal protection; may be anonymous
- **IEBC Case Officer** — receives, triages, investigates, and closes cases
- **IEBC Supervisor** — monitors SLA compliance; escalates overdue cases
- **Election Offences Investigation Officer** — receives DCI/ODPP evidence packages
- **IAB Observer** — views aggregated grievance metrics and any case affecting the audit chain
- **Party Agent** — may submit grievances about process violations at a specific station
- **Observer (KEOG/ELOG/EU EOM)** — receives read-only anonymised grievance heat map
- **Presiding Officer (on kit)** — records local complaints at the station via kit-side kiosk mode

## 3. Functional Requirements

### Intake Channels

- **FR-M23-001** Public Verifier app intake: structured form with category selector, optional location, optional photo/video attachment, optional contact. Creates a case record and returns a case-reference number and tracking URL.
- **FR-M23-002** *IEBC# USSD short-code intake: IVR menu in EN + SW; structured question tree for common complaint categories; records caller MSISDN (hashed for whistleblower submissions); sends SMS acknowledgement with case reference.
- **FR-M23-003** WhatsApp Business intake: verified IEBC WhatsApp number; natural-language intake with structured extraction via LLM (M28); category confirmation before case creation; returns case reference in-chat.
- **FR-M23-004** SMS intake: free-text to the IEBC SMS short-code; auto-categorised by rule + ML; acknowledgement SMS with case reference.
- **FR-M23-005** Kit-side kiosk intake: locked mode on KIEMS kit at polling station; voter or party agent enters complaint category + optional voice note (transcribed on-device via M28 ASR); Presiding Officer counter-signs receipt; case submitted to central on sync.
- **FR-M23-006** In-person IAB office intake: IEBC case officer records complaint directly in the case management console; produces a printed receipt for complainant.

### Case Management

- **FR-M23-007** All intake channels create a unified case record with: case_id (uuid7), intake_channel, category, severity auto-classified (P0/P1/P2 — see below), station_code (if determinable), constituency, reported_at, description, attachments (encrypted), contact (optional, encrypted).
- **FR-M23-008** Severity auto-classification:
  - P0 (critical, 15-minute SLA): physical violence or threat at station; kit failure causing voter exclusion; ballot-box tampering; impersonation by official; double-voting in progress.
  - P1 (high, 4-hour SLA): bribery / undue influence; voter turned away without s.44A; party agent expelled; Form 34A not posted; queue exceeding 300 minutes.
  - P2 (standard, 24-hour SLA): voter information complaints; voter roll query; process confusion; general feedback.
- **FR-M23-009** Case-officer assignment: P0 assigned immediately to on-duty senior officer via push notification; P1 to next-available officer in constituency queue; P2 to general queue.
- **FR-M23-010** SLA tracking: system displays time-to-breach per case; supervisor sees SLA-at-risk queue; auto-escalation to supervisor 75% through SLA window with no case update.
- **FR-M23-011** Case-officer investigation tools: evidence upload, note-taking, status updates, inter-officer hand-off with reason, resolution with reason code.
- **FR-M23-012** Complainant status tracking: complainant queries case status via any intake channel using case reference; receives status + last-updated timestamp.
- **FR-M23-013** Protected whistleblower channel: submissions marked WHISTLEBLOWER get a privilege flag; case file visible only to IEBC Supervisor + IEBC Commissioner + IAB legal representative; no case officer other than the assigned supervisor can access the underlying identity even if contact was provided.
- **FR-M23-014** Evidence-package generation for DCI/ODPP referral: for cases categorised under Election Offences Act 2016 ss.6-12, system assembles a structured evidence package (HSM-signed PDF + JSON): case record, attachments, station telemetry extract, kit audit log extract, officer investigation notes, statutory basis, timestamp chain, Merkle inclusion proof. Package routed to DCI/ODPP endpoint within SLA.
- **FR-M23-015** Cross-reference with M26 anomaly detection: P0 and P1 cases at a specific station automatically cross-referenced with Layer 7 (human-adjudicated tip line) of the anomaly-detection stack; case officer sees current anomaly flags for that station alongside the case.
- **FR-M23-016** Grievance heat map: public-facing anonymised map showing case volume per constituency by severity tier, updated every 15 minutes.
- **FR-M23-017** Post-election case analytics: published report of total cases by category, channel, severity, SLA compliance, resolution code, and ODPP referrals; published within T+30.
- **FR-M23-018** Case closure requires: resolution code, summary note, complainant notification (via intake channel), and supervisor sign-off for P0 and P1.
- **FR-M23-019** Multi-language: intake channels offer EN + SW minimum; WhatsApp and IVR offer 5 additional Kenyan languages (Kikuyu, Luo, Kamba, Kalenjin, Luhya).

## 4. Non-Functional Requirements

- **NFR-M23-001** P0 case intake to officer notification: <= 2 minutes.
- **NFR-M23-002** All intake channels available >= 99.9% during T-7 to T+7 (election week).
- **NFR-M23-003** Case management console: p99 <= 500ms.
- **NFR-M23-004** WCAG 2.2 AA on the Public Verifier intake form and case-status page.
- **NFR-M23-005** DPA 2019 lawful basis: complaint data processed under s.30(1)(c) (vital interests) and s.30(1)(b) (legal obligation for evidence packages); complainant contact data encrypted at rest; consent obtained for follow-up communications.
- **NFR-M23-006** Whistleblower submissions: contact identity never logged in plain text; only SHA-256 hash of MSISDN stored; legal-privilege flag prevents standard discovery.
- **NFR-M23-007** Evidence packages: every ODPP/DCI package has a Merkle inclusion proof verifiable by the receiving party.
- **NFR-M23-008** Scalability: system handles 50,000 concurrent incoming cases on election day without degradation.
- **NFR-M23-009** Offline resilience: kit-side kiosk stores cases locally (AES-256 encrypted); syncs to central within 60 minutes of connectivity restore.
- **NFR-M23-010** Retention: cases retained for 7 years post-election; DPIA registered with ODPC.

## 5. User Stories

1. As a voter, I dial *IEBC# on my feature phone and report in Swahili that a presiding officer is demanding payment; I receive an SMS case reference within 30 seconds.
2. As a party agent, I submit a complaint via WhatsApp that a candidate's supporters are distributing cash outside the station; the system categorises it P0 and notifies an IEBC officer within 2 minutes.
3. As an IEBC Case Officer, I see an SLA countdown for each case in my queue and receive a push alert 75% through the window so I never miss a deadline unknowingly.
4. As a whistleblower, I submit a voice note about a bribery event anonymously via the Public Verifier app; my identity is never stored and the case is visible only to the IEBC Commissioner and IAB.
5. As an Election Offences Investigation Officer at DCI, I receive a signed evidence package with the full kit audit log, station telemetry, and statutory reference within 6 hours of a P1 bribery case being verified.
6. As a voter, I share my case reference with a friend; my friend queries it on WhatsApp and sees "Case resolved — complainant notified" without seeing any personal details.
7. As an IAB observer, I view the real-time grievance heat map and notice a spike of P0 cases in one constituency; I alert the IAB Commissioner.
8. As a Presiding Officer, I record a voter's verbal complaint via the kit kiosk, counter-sign the receipt, and hand the voter a printed slip with the case reference.
9. As an IEBC Supervisor, I see all P0 cases approaching their 15-minute SLA in a dedicated at-risk panel and reassign to an available officer with one click.
10. As an ELOG observer, I access the anonymised grievance feed via read-only API to cross-reference with my PVT data.

## 6. Data Model Additions

```python
class GrievanceCase(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    intake_channel: Literal[
        "VERIFIER_APP","SHORT_CODE","WHATSAPP","SMS","KIT_KIOSK","IN_PERSON"
    ]
    category: Literal[
        "VIOLENCE","KIT_FAILURE","BALLOT_TAMPERING","IMPERSONATION",
        "DOUBLE_VOTING","BRIBERY","VOTER_EXCLUSION","AGENT_EXPELLED",
        "FORM_NOT_POSTED","QUEUE","VOTER_ROLL_QUERY","PROCESS","FEEDBACK"
    ]
    severity: Literal["P0","P1","P2"]
    station_code: str | None
    constituency_code: str | None
    description_text: str
    attachments: list[str]          # encrypted blob IDs
    contact_hash: str | None        # SHA-256 of MSISDN/email; never plain
    is_whistleblower: bool
    language: str                   # BCP-47
    reported_at: datetime
    assigned_officer_id: str | None
    assigned_at: datetime | None
    sla_deadline: datetime          # computed from severity + reported_at
    status: Literal[
        "OPEN","ASSIGNED","INVESTIGATING","PENDING_EVIDENCE","RESOLVED","REFERRED","CLOSED"
    ]
    resolution_code: str | None
    resolution_note: str | None
    closed_at: datetime | None
    merkle_node_id: str | None
    evidence_package_id: str | None

class EvidencePackage(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    case_id: str
    statutory_basis: str            # e.g. "EOA 2016 s.12(1)"
    package_hash: str
    merkle_inclusion_proof: str | None
    routed_to: Literal["DCI","ODPP","IEBC_LEGAL"]
    routed_at: datetime | None
    signed_by_officer_id: str
    signed_at: datetime
    status: Literal["DRAFT","SIGNED","DELIVERED","ACKNOWLEDGED"]

class CaseOfficerAction(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    case_id: str
    officer_id: str
    action_type: Literal["NOTE","STATUS_CHANGE","HAND_OFF","EVIDENCE_UPLOAD","ESCALATE","CLOSE"]
    detail: str
    action_at: datetime
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/grievances | Public (anon or token) | 50/hr per IP | Create grievance case; returns case_id |
| GET | /v1/grievances/{id}/status | Public + case reference | 100/hr | Query case status (no PII returned) |
| GET | /v1/grievances | IEBC_OFFICER JWT | 200/hr | List cases in officer's queue |
| PATCH | /v1/grievances/{id} | IEBC_OFFICER JWT | 100/hr | Update case (status, note, assignment) |
| POST | /v1/grievances/{id}/evidence-package | IEBC_OFFICER JWT | 10/hr | Generate evidence package |
| POST | /v1/grievances/{id}/evidence-package/{pkg_id}/sign | IEBC_SUPERVISOR JWT + HSM | 5/hr | Sign and route evidence package |
| GET | /v1/grievances/heat-map | Public | 500/min | Anonymised constituency-level case volume |
| GET | /v1/grievances/analytics | IEBC_ADMIN JWT | 20/hr | Full analytics dataset |
| POST | /v1/grievances/intake/whatsapp | WEBHOOK_SECRET | 1000/min | WhatsApp Business webhook intake |
| POST | /v1/grievances/intake/sms | WEBHOOK_SECRET | 1000/min | SMS gateway webhook intake |
| POST | /v1/grievances/intake/ussd | WEBHOOK_SECRET | 1000/min | USSD gateway callback |

## 8. Integration Points

- **M28 AI Monitoring Stack** — WhatsApp intake NLP extraction; ASR transcription of kit-side voice notes; chatbot escalation
- **M26 Anomaly Detection** — Layer 7 tip-line; P0/P1 cases cross-referenced with station anomaly flags
- **M22 Campaign Finance** — bribery tips routed as FinanceEvidencePackage input
- **M08 RTS Device / M10 RTS Back-End** — kit-side kiosk intake; station telemetry extracted for evidence packages
- **P05 Merkle Chain** — evidence package hashes anchored
- **P06 HSM** — evidence package signing
- **M11 Public Portal** — grievance status tracker and heat map embedded
- **DCI / ODPP** (external, MoU) — evidence package delivery endpoints
- **WhatsApp Business API** (external) — intake channel
- **Africa's Talking / Safaricom USSD gateway** (external) — *IEBC# short-code
- **M15 SMS gateway** (or equivalent) — SMS intake and acknowledgement

## 9. UI/UX Key Screens

1. **Public Intake Form (Verifier App + Web)** — category selector with icons, optional station-code lookup by GPS, photo/video attach (compressed on-device), anonymous toggle, submit. Returns case reference with QR code.
2. **Case Status Tracker** — enter case reference; see status timeline (submitted, assigned, investigating, resolved); estimated resolution time per severity; contact IEBC link.
3. **IEBC Case Management Console** — kanban-style: OPEN / ASSIGNED / INVESTIGATING / RESOLVED; SLA countdown badges; severity colour coding; one-click assignment and hand-off; evidence-package builder.
4. **P0 Alert Panel** — dedicated supervisor view; live count of P0 cases; time-to-SLA-breach per case; push notification log.
5. **Grievance Heat Map (Public)** — Kenya constituency choropleth; colour by P0/P1/P2 volume; 15-minute refresh; legend and disclaimer ("cases are unverified reports").
6. **Evidence Package Builder** — case summary + auto-selected attachments + station telemetry extract + statutory basis picker + preview PDF + HSM-sign button.
7. **Kit Kiosk Intake** — simplified 4-step screen on kit: category (icon grid), description (voice note or text), Presiding Officer counter-sign, printed receipt with case reference.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Voter's right to report violations | Elections Act 2011 s.21A (observer and public reporting) |
| Bribery criminalisation and evidence collection | Election Offences Act 2016 ss.6-12 |
| Whistleblower protection | Whistleblower Protection Act 2022 |
| Data protection — complaint records | DPA 2019 s.30(1)(b) and (c); DPIA required |
| Evidence admissibility chain | Evidence Act Cap 80; Electronic Transactions Act 2007 |
| ODPP referral obligation | ODPP Act 2011 s.5 |
| General election offences referral pipeline | Elections Act 2011 §83-84 (offences against the Act); M23 FR-M23-014 evidence package covers all EOA 2016 ss.6-12 cases and Elections Act §83-84 general offences; structured case record routed to ODPP with statutory basis field set to the specific provision |
| DCI reporting obligation | National Police Service Act 2011 s.24 |
| Complainant access to case status | Access to Information Act 2016 s.4 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| P0 case intake-to-officer-notification | <= 2 minutes | Timestamp delta | IEBC Ops Director |
| P0 SLA compliance rate | >= 99% | SLA tracking log | IEBC Commissioner |
| P1 SLA compliance rate | >= 95% | SLA tracking log | IEBC Commissioner |
| P2 SLA compliance rate | >= 90% | SLA tracking log | IEBC Supervisor |
| Election Offences evidence packages routed to ODPP | 100% of verified EOA ss.6-12 cases | Referral log | IEBC Chair |
| Evidence packages delivered within SLA | 100% within 6 hours of P0 verification | Delivery timestamp | IEBC Chair |
| All intake channels available on election day | >= 99.9% | Uptime monitor per channel | Datacraft SRE |
| Cases with Merkle anchor | 100% | Merkle reconciliation | Datacraft Technical |
| Grievance heat map update latency | <= 15 minutes | Dashboard refresh log | Datacraft SRE |
| Whistleblower identity exposure incidents | 0 | Security audit | Datacraft CISO |

## 12. Test Strategy

- **Unit:** severity classifier; SLA deadline calculator; evidence-package assembler; statutory-basis lookup.
- **Integration:** intake via WhatsApp webhook, USSD callback, SMS gateway; case creation and notification pipeline; ODPP evidence delivery.
- **E2E:** simulated P0 bribery case from WhatsApp intake through to signed evidence package delivered to mock ODPP endpoint.
- **Security:** attempt to access whistleblower identity via API; attempt to modify a case after officer sign-off; confirm plain-text MSISDN never stored.
- **Accessibility:** WCAG 2.2 AA on all public-facing screens; audio-guided IVR testing.
- **Load:** 50,000 concurrent case submissions on election day.
- **Chaos:** WhatsApp API outage — verify SMS and USSD intake continue; kit kiosk offline — verify local queue persistence.
- **Penetration:** OWASP Top 10 scan on grievance API; attempt IDOR on case record access.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Channel integrations complete | 2026-12-01 | WhatsApp, SMS, USSD adapters live |
| Case management console UAT | 2027-01-15 | IEBC case officers UAT with synthetic cases |
| Kit kiosk mode deployed | 2027-05-01 | Included in kit firmware release |
| Dry-run on by-election | 2027-02-28 | Real intake across all channels |
| MoU with DCI/ODPP for evidence delivery | 2027-03-01 | External dependency |
| Full production live | 2027-07-01 | T-40 from election day |
| Post-election analytics published | 2027-09-10 | T+31 |

## 14. Dependencies

- M28 AI Monitoring Stack — NLP extraction, ASR, chatbot
- M26 Anomaly Detection — Layer 7 tip-line cross-reference
- M22 Campaign Finance — bribery tip routing
- M08 / M10 RTS — station telemetry for evidence packages
- P05 Merkle Chain — anchoring
- P06 HSM — signing
- M11 Public Portal — heat map and status tracker
- WhatsApp Business API approval — external (6-8 week approval lead time)
- Africa's Talking / Safaricom USSD gateway MoU — external
- DCI / ODPP delivery MoU — external; IEBC-facilitated

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| WhatsApp Business API approval delayed | Medium | High | SMS and USSD channels can carry all intake volume without WhatsApp; WhatsApp is a convenience channel, not single point of failure |
| P0 case officer unavailable at intake | Low | High | On-call escalation matrix; supervisor auto-assigned on officer non-response within 5 minutes; secondary on-call paged |
| Evidence package rejected by ODPP as procedurally defective | Low | High | Package format co-designed with ODPP legal team pre-election; mock delivery and sign-off at T-30 dry run |
| Grievance channel used to flood IEBC with false P0 reports (DoS) | Medium | Medium | Rate limiting per IP and MSISDN; P0 auto-classification reviewed by officer before escalation; anomaly detection on intake spike patterns |
| Whistleblower identified by correlation of case details | Low | Critical | Case details redacted in all shared views; officer investigation notes accessible only to assigned officer + supervisor; no case metadata exported to public heat map |
