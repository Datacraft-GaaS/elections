# M68 — Referendum Explainer Platform

**Module ID:** M68
**Name:** Referendum Explainer Platform
**Owner:** Datacraft Limited — Civic Participation Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Constitution Art. 255-257; Elections Act §49; distinct from M37 Voter Education CMS (general) and M49 Referendum Mechanics (voting workflow)

---

## Purpose

Voter-facing platform providing multi-language, multi-format public explainers for each question on a constitutional referendum ballot. M37 handles general voter education; M49 handles the referendum voting workflow. M68 handles the per-question explanatory content layer: structured explanations of what each question asks, what a Yes and No vote means, background constitutional context, fact-check partner review status, and distribution to all channels. WCAG 2.2 AAA target for referendum content (elevated from AA, given referendum's constitutional significance). EN and SW baseline; additional languages via community translation pipeline.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| IEBC Content Editor | Write | Draft and publish question explainers |
| Fact-Check Partner (e.g. PesaCheck) | Write (review annotations) | Flag inaccurate content; submit verified corrections |
| Legal Reviewer (IEBC Legal) | Write (legal accuracy check) | Confirm legal accuracy of constitutional citations |
| Translator | Write (assigned language) | Translate approved EN content to SW / other languages |
| Voter (mobile/web) | Read | Read explainer for any referendum question |
| Voter (voice/audio) | Read (audio channel) | Listen to TTS-rendered explainer |
| Disability-support user | Read (augmented) | Access sign-language video, high-contrast, simplified text |
| M11 Public Portal | API consumer | Embed question explainers |
| M49 Referendum Mechanics | API consumer | Show contextual explainer at point of ballot interaction |
| IAB Member | Read + audit | Monitor fact-check status, editorial process |

---

## Functional Requirements

### FR-68.01 — Question registry
Each referendum ballot question registered as an entity: question ID (UUID7), official question text (verbatim per ballot), question type (constitutional amendment / repeal / approval), ballot-paper position, constitutional provision affected. Questions seeded from official Gazette notice; any change requires Commissioner sign-off and audit log entry.

### FR-68.02 — Explainer content structure
Each question has a structured explainer with mandatory sections:
- What the question asks (plain language, < 150 words)
- What a Yes vote means (concrete legal effect)
- What a No vote means (concrete legal effect)
- Constitutional context (relevant Art. citation + 100-word summary)
- Frequently asked questions (up to 10 Q&A pairs)
- Fact-check status (reviewed / pending / flagged)
- Source citations (numbered, linked)

### FR-68.03 — Editorial workflow
Draft → Legal Review → Fact-Check Review → IEBC Chair Approval → Published. Each state transition logged with reviewer UUID and timestamp. Regression to earlier state permitted with mandatory reason. No explainer published without all three review stages cleared. Target T-60 for all questions published in EN and SW.

### FR-68.04 — Fact-check partner API
Fact-check partners (PesaCheck, Code for Africa, others registered by IEBC) access content via API. Partners annotate specific claims: claim text, verdict (accurate / inaccurate / unverifiable), evidence URL. Annotations visible to IEBC editors. Any `inaccurate` verdict blocks publication until editor resolves. Resolution: update content or override with written determination.

### FR-68.05 — Multi-language pipeline
EN master content approved first. Translation workflow: translator assigned per language; translation submitted; IEBC editor + community reviewer (bilingual) signs off. SW mandatory. Additional languages (Kikuyu, Luo, Kalenjin, Somali, Kamba) on best-effort basis via community volunteer platform. Each language version has its own publication state.

### FR-68.06 — Multi-format output
From approved content: system auto-generates:
- Web page (responsive HTML, WCAG 2.2 AAA)
- PDF (print-ready, A4, branded)
- Audio (TTS via M17 Speech Integrity integration; SSML annotations for natural pacing)
- Plain text (SMS-compatible, < 160 chars summary per question)
- Easy-read version (simplified language, Makaton-compatible symbols)
- Sign-language video (Kenya Sign Language; commissioned video for each question, embedded)

### FR-68.07 — Distribution channels
From M68 API: (1) M11 Public Portal embedded, (2) M49 Referendum Mechanics contextual link, (3) M29 Mobile Citizen Verify in-app tab, (4) SMS push via M23 (opt-in), (5) WhatsApp Bot (IEBC official channel), (6) Printed PDF for physical distribution, (7) Radio bulletin export (plain text for KBC broadcasters).

### FR-68.08 — WCAG 2.2 AAA compliance
All web explainer pages: WCAG 2.2 AAA (elevated from standard AA for referendum content). Requirements include: contrast ratio >= 7:1 (AAA), no automatic audio, sign-language video, extended audio description. Automated + manual accessibility audit per question before publication.

### FR-68.09 — Versioning and correction log
Each published explainer version immutable. Corrections create new versions. Change log publicly visible per question: version number, change description, date, approver. Voters notified of material changes via the same channels that delivered the original explainer.

### FR-68.10 — Analytics and reach
Page-level analytics (anonymous): page views per question, audio plays, PDF downloads, language distribution, device type. No voter-identifying tracking. Used by IEBC to prioritise translation and distribution effort. Published as part of post-referendum transparency report.

---

## Non-Functional Requirements

### NFR-68.01 — WCAG 2.2 AAA
All referendum explainer web pages: WCAG 2.2 AAA. Third-party accessibility audit required before public launch. Audit report published.

### NFR-68.02 — Availability
99.9% from publication date to T+30. Content served from CDN; origin failover.

### NFR-68.03 — Performance
Page load < 2s on 3G connection (50% of rural Kenya connectivity baseline). Images optimised; TTS audio streamed; PDF generated server-side.

### NFR-68.04 — DPA 2019
Anonymous analytics only. No voter identification, no cookie consent required for explainer read. Fact-check partner contact details: personal data; scoped access only.

### NFR-68.05 — Languages
EN and SW: mandatory, 100% coverage before publication. Five additional languages: best-effort, 60-day lead time. Sign-language video: 100% of questions before T-30.

### NFR-68.06 — SLA
Editorial workflow: target T-60 for all EN and SW explainers published. Fact-check turnaround: partners given 7-day window per question. Sign-language video: commissioned at T-90, delivered T-30.

---

- **NFR-M68-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M68`; alert ruleset ID `iems-M68-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As a voter, I want to read a plain-language explanation of each referendum question in Swahili, so that I can make an informed decision before I vote.
2. As a voter with a visual impairment, I want to listen to an audio version of the referendum explainer, so that the information is accessible to me without a screen.
3. As a voter who uses Kenya Sign Language, I want to watch a sign-language video for each referendum question, so that the content is fully accessible to me.
4. As an IEBC Content Editor, I want to draft a structured explainer per question and submit it for legal review, so that the workflow enforces all quality gates before publication.
5. As a Fact-Check Partner (PesaCheck), I want to annotate specific claims in an explainer and have my verdict block publication until resolved, so that voters only see verified content.
6. As a community translator (Luo), I want to translate the approved EN explainer for my language community, so that Luo-speaking voters have access to referendum information in their language.
7. As an M49 Referendum Mechanics system, I want to fetch the current explainer for a given question via API when a voter arrives at the ballot interface, so that contextual information is shown at the point of decision.
8. As an IAB Member, I want to audit the editorial history of each explainer, so that I can confirm all review stages were completed before publication.
9. As a deaf voter, I want to see the WCAG 2.2 AAA sign-language video embedded on the explainer page without needing to navigate away, so that my experience is equivalent to a hearing voter's.
10. As a rural voter receiving the SMS summary, I want the 160-character question summary to tell me what Yes and No mean in one sentence each, so that I have the minimum information even on a basic phone.
11. As an IEBC Commissioner, I want to be required to sign off every explainer before publication, so that political accountability rests with the Commission, not editorial staff alone.
12. As a data journalist, I want access to anonymous reach analytics per question, so that I can report on which referendum questions were least understood and most queried.
13. As a radio broadcaster (KBC), I want to receive a plain-text radio bulletin export for each question, so that I can read explainers on-air without reformatting.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime

class ReferendumQuestion(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    ballot_position: int
    official_text: str
    question_type: str  # constitutional_amendment | repeal | approval
    constitutional_provisions: list[str]  # list of Article references
    gazette_reference: str
    created_at: datetime
    commissioner_signed_at: datetime | None

class ExplainerVersion(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    question_id: str
    language: str  # BCP47: en | sw | ki | luo | ...
    version: int
    status: str  # draft | legal_review | factcheck_review | commissioner_approval | published | superseded
    plain_language_summary: str  # < 150 words
    yes_meaning: str
    no_meaning: str
    constitutional_context: str
    faqs: list[dict]  # [{question: str, answer: str}]
    source_citations: list[str]
    fact_check_status: str  # pending | reviewed | flagged
    created_by: str
    legal_reviewer: str | None
    factcheck_cleared: bool
    commissioner_approved_by: str | None
    published_at: datetime | None
    change_log_entry: str | None

class FactCheckAnnotation(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    explainer_id: str
    partner_org: str
    claim_text: str
    verdict: str  # accurate | inaccurate | unverifiable
    evidence_url: str | None
    annotated_by: str
    annotated_at: datetime
    resolved: bool
    resolution_note: str | None

class ExplainerFormat(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    explainer_id: str
    format: str  # web | pdf | audio | sms | easy_read | sign_language_video
    language: str
    url: str
    generated_at: datetime
    file_size_bytes: int | None
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M68 Referendum Explainer Platform API
  version: "1.0.0"
paths:
  /api/v1/referendum/questions:
    get:
      summary: List all referendum questions
      responses:
        "200": {description: List of ReferendumQuestion}
  /api/v1/referendum/questions/{question_id}/explainer:
    get:
      summary: Get current published explainer for a question
      parameters:
        - name: question_id
          in: path
          required: true
          schema: {type: string}
        - name: lang
          in: query
          schema: {type: string, default: en}
        - name: format
          in: query
          schema: {type: string, enum: [web, pdf, audio, sms, easy_read, sign_language_video]}
      responses:
        "200": {description: ExplainerVersion or ExplainerFormat}
  /api/v1/referendum/explainers:
    post:
      summary: Create explainer draft (IEBC Editor)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/ExplainerVersion"}
      responses:
        "201": {description: Draft created}
  /api/v1/referendum/explainers/{id}/state:
    patch:
      summary: Advance or revert editorial state
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                new_state: {type: string}
                reason: {type: string}
      responses:
        "200": {description: State updated}
  /api/v1/referendum/factcheck-annotations:
    post:
      summary: Submit fact-check annotation (partner)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/FactCheckAnnotation"}
      responses:
        "201": {description: Annotation recorded}
  /api/v1/referendum/public/analytics:
    get:
      summary: Anonymous reach analytics per question
      responses:
        "200": {description: Page views, audio plays, downloads by question and language}
```

---

## Integrations

**Upstream:**
- Kenya Gazette — official question text and gazette reference
- M17 (Speech Integrity) — TTS audio generation for audio format
- Fact-check partners (PesaCheck, Code for Africa) — annotation API
- Community translation volunteers — translation submission portal
- Sign-language video production (contracted; video upload to M68 store)
- P06 (HSM) — version signing

**Downstream:**
- M11 (Public Portal) — embedded explainer widgets
- M49 (Referendum Mechanics) — contextual explainer link at ballot point
- M29 (Mobile Citizen Verify) — in-app explainer tab
- M23 (Grievance Channels) — SMS push opt-in
- KBC Radio — plain-text bulletin export
- WhatsApp Business API — bot content delivery
- P05 (Audit log)

---

## UI/UX — Key Screens

1. **Voter explainer page** (web) — question card per ballot question; tabs: Plain Language / Yes Means / No Means / Context / FAQs; fact-check status badge; audio player; PDF download; sign-language video embedded; EN/SW language toggle; WCAG 2.2 AAA; dark-mode support
2. **Mobile explainer view** (M29 integration) — single-question card; swipe between questions; large text option; audio button prominent
3. **SMS summary format** — 160-char per question; "Question 1: [short text]. YES=[effect]. NO=[effect]."
4. **Editor dashboard** — question list with status per language; workflow action buttons; fact-check annotation inbox
5. **Fact-check partner portal** — claim annotation form; verdict selection; evidence URL; question-level explainer text with inline annotation anchors
6. **Translation workspace** — EN source on left, target language on right; section-by-section; submit for review button
7. **Commissioner approval screen** — full explainer preview per language; sign-off button (requires 2FA); digital signature appended

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Public information on referendum questions | Constitution Art. 255(2) |
| IEBC mandate to educate voters | Elections Act §49 |
| WCAG 2.2 AAA (referendum content) | Strategy accessibility policy |
| Multi-language accessibility | Constitution Art. 7(3)(b) |
| DPA 2019 anonymous analytics | DPA 2019 §26 |
| Content immutability and change log | Elections Act §44 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| All questions published EN + SW by T-60 | 100% | IEBC Chair |
| Sign-language videos published by T-30 | 100% | IEBC Chair |
| WCAG 2.2 AAA audit pass | 100% | Accessibility auditor |
| Fact-check clearance before publication | 100% | IAB |
| Commissioner sign-off on every published version | 100% | IEBC Chair |
| Audio format available for all questions | 100% by T-60 | IEBC ICT |
| Additional language coverage (5 languages) | >= 80% of questions | IEBC Comms |

---

## Test Strategy

- **Unit:** Editorial state machine, fact-check block logic, SMS truncation to 160 chars, multi-version publish/supersede
- **Integration:** M17 TTS audio generation, M49 contextual link, M11 embed, KBC bulletin export, WhatsApp API delivery
- **Accessibility:** Automated axe-core for WCAG 2.2 AAA; manual screen-reader (NVDA, VoiceOver, TalkBack); manual keyboard-navigation audit; colour contrast >= 7:1 verified programmatically
- **Load:** 2 million concurrent page views at referendum vote-day peak; p99 < 2s on 3G
- **Sign-language video:** Manual review of each video for accuracy against EN text by KSL interpreter
- **Translation accuracy:** Bilingual community reviewer sign-off workflow tested end-to-end

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Platform built and accessibility-audited | 2027-01-01 | AAA audit pass |
| Fact-check partner API live and partners onboarded | 2027-02-01 | 3 partners active |
| All questions registered from Gazette | 2027-03-01 | Question registry seeded |
| EN and SW explainers published (T-60 target) | 2027-06-10 (T-60) | All questions published |
| Sign-language videos published (T-30 target) | 2027-07-11 (T-30) | All videos live |
| M49 contextual link live | 2027-08-01 (T-9) | Integration confirmed |

---

## Dependencies

- **M37** (Voter Education CMS) — distinct module; M68 is referendum-specific and does not share a CMS instance
- **M49** (Referendum Mechanics) — contextual link consumer
- **M17** (Speech Integrity) — audio generation
- **M11** (Public Portal), **M29** (Mobile Citizen Verify), **M23** (Grievance Channels)
- **P05** (Audit log), **P06** (HSM), **P04** (IAM)
- External: Kenya Gazette, fact-check partners, KSL video production

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Fact-check partner disputes IEBC editorial content publicly | Written determination process documented; IAB arbitration available; public change-log shows all revisions transparently |
| R2 | Sign-language video production delayed | Commission at T-90; video production contracts include T-30 delivery penalty; placeholder accessible text version live from T-60 |
| R3 | Referendum question text changed after publication | Versioning system auto-archives prior version; change notification pushed to all distribution channels within 1h; audit log records who authorised change |
| R4 | WCAG 2.2 AAA fails audit for one or more components | External accessibility agency retained from T-150; iterative audit from T-120; hard gate: no publication without AAA pass |
| R5 | Rural voters cannot access web or audio explainers | SMS format (160 chars) covers basic phones; printed PDF for physical distribution via sub-county offices; radio bulletin for broadcast reach |

---

## Cross-References

- **M37** Voter Education CMS — general voter education (separate system; M68 is referendum-specific)
- **M49** Referendum Mechanics — voting workflow (M68 provides contextual content)
- **M11** Public Portal — distribution channel
- **M17** Speech Integrity — audio generation
- **M29** Mobile Citizen Verify — in-app consumer
- **M15** IAB Dashboard — editorial audit

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| Constitution Art. 255(2) | Public information right on referendum questions |
| Elections Act §49 | IEBC voter education mandate |
| Constitution Art. 7(3)(b) | Multilingual access to public information |
| Strategy accessibility policy | WCAG 2.2 AAA for referendum content |

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
