# M37 — Voter Education CMS

**Module ID:** M37
**Name:** Voter Education Content Management System
**Runs on:** Central servers (Kenyan cloud) + CDN edge
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Manage, version-control, fact-check, translate, and distribute voter-facing educational materials across 11 Kenyan languages and four formats (video, radio/audio, print-PDF, WhatsApp-ready card). Gate every asset behind a fact-check-partner review and the Commission approval before publication. Distribute to M13 Public Verifier App, M40 Media Portal, and physical partners (county offices, CSO networks). Ensure no outdated or unreviewed content reaches voters.

Strategy reference: §XVI.4 (voter education programme), §XVIII.5 (public communication).

## 2. User Personas

- **the Commission Voter Education Officer** — creates and edits content; submits for fact-check review
- **Fact-Check Partner Reviewer** — external CSO or academic; reviews content for accuracy; approves or returns with notes
- **the Commission Approver (Director of Voter Education)** — final approval gate; signs off content for publication
- **Translator** — external contractor; receives EN master; returns localised version for each target language
- **County Education Coordinator** — receives approved materials; manages physical distribution tracking
- **M13 App User (Voter)** — consumes educational content in-app
- **Media Partner (Radio/TV)** — accesses M40 Media Portal for audio/video files and broadcast scripts
- **WhatsApp Campaign Manager** — pulls approved card-format assets for WhatsApp broadcast
- **the Commission Digital DPO (Datacraft)** — audits any analytics data collected on content consumption

## 3. Functional Requirements

- **FR-M37-001** Content creation: structured article editor (rich text + embedded media) with mandatory fields: title, language, format, topic category, target audience, election-cycle tag, author.
- **FR-M37-002** Asset upload: video (MP4 ≤ 500 MB), audio (MP3 ≤ 100 MB), PDF (≤ 20 MB), image card (PNG/JPG ≤ 5 MB). All assets virus-scanned on upload; stored in encrypted object store.
- **FR-M37-003** Version control: every save creates an immutable version; editor can branch from any prior version; diff view shows changes between versions. Published version cannot be edited — a new version must be created.
- **FR-M37-004** Fact-check workflow: on submission, system assigns to a fact-check partner reviewer from the rotation list; reviewer has 5 business days to approve or return with annotated notes; author must address each note before resubmission.
- **FR-M37-005** the Commission approval gate: after fact-check pass, content enters the Commission Director approval queue; Director can approve, return, or escalate to Commissioner. Approved content is immutably marked with approver ID and timestamp.
- **FR-M37-006** Translation workflow: approved EN master triggers translation tasks per configured target languages; translator uploads translated text file; the Commission Education Officer reviews and publishes localised version independently per language.
- **FR-M37-007** Languages: EN + SW baseline; additional targets configurable per content item from: Kikuyu, Luo, Kamba, Kalenjin, Luhya, Mijikenda, Meru, Embu, Somali, Turkana (11 languages total).
- **FR-M37-008** Format variants: a single content item can have linked variants for each format (video, audio, print, WhatsApp-card); all variants share the same approval status; variant-specific approval not required separately once master is approved.
- **FR-M37-009** Distribution channels: approved content automatically pushed to (a) M13 Public Verifier App content feed, (b) M40 Media Portal distribution queue, (c) county-coordinator inboxes (email + portal notification), (d) WhatsApp campaign asset store.
- **FR-M37-010** Archiving and expiry: content can be tagged with an expiry date (e.g. pre-election-only content expires on election day); expired content auto-archived; no longer served to voters but retained for audit.
- **FR-M37-011** Content analytics: aggregate consumption metrics (views, downloads) per content item; no individual tracking; county-level aggregation only; daily report to the Commission Voter Education Officer.
- **FR-M37-012** Search and discovery: voters accessing M13 can search educational content by topic, language, format; full-text search with language-aware stemming.
- **FR-M37-013** Physical distribution tracking: county coordinator marks materials as dispatched with quantity and date; system tracks physical distribution vs registered county population (coverage metric).

## 4. Non-Functional Requirements

- **NFR-M37-001** CMS authoring UI: p99 <= 800 ms under 100 concurrent editors.
- **NFR-M37-002** Content delivery to M13: p99 <= 1 second for content list; CDN-cached; >= 99.9% availability.
- **NFR-M37-003** WCAG 2.2 AA: all voter-facing content pages in M13 and public portal; CMS itself must be AA-compliant for officer use.
- **NFR-M37-004** Video streaming: adaptive bitrate (360p/480p/720p) for low-bandwidth environments; offline download option for M13 app.
- **NFR-M37-005** DPA 2019: no individual consumption data collected; aggregate-only analytics; no cookies beyond session on voter-facing pages; DPIA for any analytics enhancement.
- **NFR-M37-006** Retention: approved content retained for 7 years post-election; draft and rejected versions retained 2 years.
- **NFR-M37-007** Security: fact-check partner accounts are external and scoped read+annotate only; they cannot publish or access unpublished content outside their review queue.
- **NFR-M37-008** Scalability: up to 500 active content items across 11 languages; up to 1M content requests per day from M13 on election day.

- **NFR-37-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-37` (replace 37 with the module number); alert ruleset ID `iems-37-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Voter Education Officer, I create a short explainer video on how to mark a ballot paper; I submit it for fact-check review and receive annotated feedback within 5 business days.
2. As a Fact-Check Partner Reviewer, I annotate three inaccuracies in a draft article about voter registration; the author is notified and must address each annotation before resubmission.
3. As the Director of Voter Education, I see a queue of fact-check-passed content awaiting my approval; I approve a Swahili audio clip for immediate distribution to county radio partners.
4. As a Translator, I receive an EN master PDF about voter rights; I upload the Kikuyu translation; the Commission Education Officer reviews and publishes it.
5. As a County Education Coordinator in Kisumu County, I download the approved Luo-language print PDF pack and record dispatch of 5,000 copies.
6. As a voter using M13, I search for "how to vote" in Kalenjin and receive an approved video in my language.
7. As a WhatsApp Campaign Manager, I pull the latest approved infographic card about polling station hours and schedule it for broadcast.
8. As the Commission Director, I see a content-coverage report showing which counties have received physical materials and which are below 80% of target population coverage.
9. As a Commission Voter Education Officer, I set an expiry date of election day on a "register now" content item; after that date it is auto-archived and no longer served in M13.
10. As the Commission Digital DPO, I verify that the content analytics report contains no individual-level data — only aggregate views by county and format.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime, date

class ContentItem(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    title: str
    language: str                   # BCP-47 (e.g. "sw", "ki", "luo")
    format: Literal["ARTICLE","VIDEO","AUDIO","PRINT_PDF","WHATSAPP_CARD"]
    topic_category: Literal[
        "REGISTRATION","HOW_TO_VOTE","VOTER_RIGHTS","CANDIDATE_CONDUCT",
        "OBSERVER_RIGHTS","DISPUTE_RESOLUTION","POLLING_STATION","GENERAL"
    ]
    target_audience: list[str]
    election_cycle_tag: str
    author_id: str
    current_version_id: str
    status: Literal["DRAFT","IN_FACT_CHECK","FACT_CHECK_PASSED","IN_APPROVAL","APPROVED","PUBLISHED","ARCHIVED","EXPIRED"]
    approved_by: str | None
    approved_at: datetime | None
    expiry_date: date | None
    created_at: datetime
    updated_at: datetime

class ContentVersion(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    content_item_id: str
    version_number: int
    body_markdown: str | None
    asset_file_id: str | None       # blob reference
    parent_version_id: str | None
    created_by: str
    created_at: datetime

class FactCheckReview(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    content_item_id: str
    version_id: str
    reviewer_id: str
    annotations: list[dict]         # [{text, note, resolved}]
    decision: Literal["APPROVED","RETURNED"] | None
    completed_at: datetime | None

class DistributionEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    content_item_id: str
    channel: Literal["M13_APP","M40_MEDIA","COUNTY_EMAIL","WHATSAPP_STORE","CDN_PUSH"]
    county_code: str | None
    dispatched_at: datetime
    quantity: int | None            # for physical materials
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/cms/content | COMMISSION_EDITOR JWT | 50/hr | Create content item |
| PUT | /v1/cms/content/{id}/versions | COMMISSION_EDITOR JWT | 100/hr | Save new version |
| POST | /v1/cms/content/{id}/submit-review | COMMISSION_EDITOR JWT | 20/hr | Submit for fact-check |
| PATCH | /v1/cms/content/{id}/fact-check | REVIEWER JWT | 20/hr | Submit fact-check decision + annotations |
| POST | /v1/cms/content/{id}/approve | COMMISSION_DIRECTOR JWT | 20/hr | Approve and trigger distribution |
| GET | /v1/cms/content | COMMISSION_EDITOR JWT | 200/hr | List content items (filterable) |
| GET | /v1/cms/content/public | Public | 1000/min | Approved published items (M13 feed) |
| GET | /v1/cms/content/{id}/asset | CDN_SIGNED | 5000/min | Stream/download asset via CDN signed URL |
| POST | /v1/cms/distribution/{id}/physical | COUNTY_COORD JWT | 50/hr | Record physical dispatch |
| GET | /v1/cms/analytics | COMMISSION_DIRECTOR JWT | 20/hr | Aggregate content consumption report |

## 8. Integration Points

- **M13 Public Verifier App** — primary voter-facing consumption channel; content feed API
- **M40 Election Court IT Interface / Media Portal** — media partner distribution queue
- **M11 Public Portal** — supplementary publication of key approved materials
- **M28 AI Monitoring Stack** — optional: AI-assisted plain-language check before fact-check submission
- **WhatsApp Business API** (external) — push to campaign asset store
- **County education coordinator portal** — physical dispatch tracking
- **CDN (Cloudflare / Akamai class)** — asset delivery with signed URLs
- **Virus-scan service** — upload screening

## 9. UI/UX Key Screens

1. **Content Dashboard** — officer view: list of items by status (DRAFT / IN_REVIEW / APPROVED / PUBLISHED); filter by language, format, topic; quick-action buttons.
2. **Rich Text + Asset Editor** — split-pane: text editor left, asset upload right; metadata panel; version history sidebar; submit-for-review button.
3. **Fact-Check Review Screen** — reviewer sees rendered content; annotation tool (highlight text + add note); APPROVE / RETURN radio with overall comment; 5-day deadline countdown.
4. **Director Approval Queue** — cards showing content title, language, format, fact-check reviewer name, submission date; approve/return buttons; preview pane.
5. **Translation Task List** — translator view: EN master on left; translation input on right; progress indicator per language; submit button per language.
6. **County Distribution Dashboard** — Kenya map; county fill colour by coverage %; click county to see dispatch records; print pack download button.
7. **Voter Content Feed (M13 integration, WCAG 2.2 AA)** — topic grid with icons; language selector; format filter (video/audio/text/image); search bar; offline-download indicator.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Voter education mandate | Elections Act 2011 §44; the Commission Act 2011 §7(1)(h) |
| Multi-language obligation | Constitution Art 7(3)(b); National Cohesion Act 2008 §12 |
| Accuracy of public communications | Elections Act 2011 §64 (false information offence) |
| Digital content accessibility | Persons with Disabilities Act 2003 §21; WCAG 2.2 AA |
| Privacy of consumption data | DPA 2019 §26, §30; aggregate-only policy |
| Copyright in election materials | Copyright Act 2001 §26 (work-for-hire vests in the Commission) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Fact-check turnaround | >= 95% within 5 business days | Review completion timestamp | the Commission Director |
| Content available in all 11 languages | >= 80% of core catalogue | Language coverage report | the Commission Director |
| M13 content feed availability | >= 99.9% | CDN uptime monitor | Datacraft SRE |
| Physical distribution coverage | >= 90% of target counties by T-30 | Distribution dashboard | the Commission County Director |
| Zero unreviewed content published | 100% | Automated gate check on publish | Datacraft Technical |
| WCAG 2.2 AA pass on voter-facing content | 100% at audit | Independent accessibility audit | the Commission |

## 12. Test Strategy

- **Unit:** fact-check workflow state machine; expiry auto-archive trigger; distribution-channel push logic.
- **Integration:** content publish to M13 feed; CDN signed-URL generation; WhatsApp asset store push; physical dispatch recording.
- **E2E:** create draft video item; submit for fact-check; reviewer approves; director approves; verify M13 feed updated within 5 minutes.
- **Security:** reviewer attempts to publish directly (must fail); external reviewer attempts to access unapproved content outside queue (must fail); verify signed URLs expire after configured TTL.
- **Accessibility:** WCAG 2.2 AA scan on M13 content-feed integration and CMS authoring UI (axe-core + manual + screen-reader on 3 languages).
- **Load:** 1M content requests per day against CDN; verify p99 <= 1 second.
- **Chaos:** CDN origin failure — verify M13 falls back to cached content with staleness indicator.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| CMS core live | 2026-11-01 | Editor + fact-check + approval workflow |
| EN + SW content catalogue seeded | 2027-01-01 | Core 30 items in both languages |
| Translation pipeline active | 2027-02-01 | 9 additional languages in translation |
| M13 content feed integration | 2027-03-01 | Voter-facing feed live in app |
| Full 11-language catalogue | 2027-05-01 | T-100: all core items translated and approved |
| County physical dispatch complete | 2027-07-10 | T-31: all counties receive print packs |
| WhatsApp campaign active | 2027-07-15 | T-26: broadcast schedule begins |
| Election-day expiry of pre-election content | 2027-08-10 | Auto-archive on election day |

## 14. Dependencies

- M13 Public Verifier App — voter content consumption surface
- M40 Media Portal — media partner distribution
- M28 AI Monitoring Stack — optional plain-language assistance
- WhatsApp Business API — campaign asset channel (external approval lead time 6-8 weeks)
- CDN — asset delivery
- Fact-check partner CSOs — contracted externally by the Commission

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Fact-check partner fails to review within 5 days | Medium | Medium | Escalation to secondary reviewer at day 4; the Commission Director can override with documented justification |
| Translation quality inadequate for minority languages | Medium | High | Two-step: translator + community validator for each minority language; quality scorecard per language |
| WhatsApp Business API approval delayed | Medium | Medium | SMS broadcast and M13 carry full load; WhatsApp is supplementary |
| Expired content served via stale CDN cache | Low | High | Max-age headers set to 1 hour; expiry triggers CDN cache purge event |
| Voter-facing content contains factual error post-publication | Low | High | Immediate retract-and-republish workflow; M13 push notification of correction; version history preserved |

## 16. Cross-References

- `M13-public-verifier-app.md` — voter content feed
- `M11-public-portal.md` — supplementary publication
- `M28-ai-monitoring-stack.md` — plain-language check
- `M40-election-court-it-interface.md` — media portal
- `M38-poll-worker-training-lms.md` — training content shares the CMS asset store

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XVI.4 — voter education programme | FR-M37-001 through FR-M37-013 |
| §XVIII.5 — public communication | FR-M37-009, FR-M37-011 |
| Multi-language requirement | FR-M37-007, NFR-M37-003 |

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
