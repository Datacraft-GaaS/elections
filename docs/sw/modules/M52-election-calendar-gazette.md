# M52 — Election Calendar and Gazette Notice Management

**Module ID:** M52
**Name:** Election Calendar and Gazette Notice Management
**Runs on:** Central servers (Kenyan cloud) + the Commission back-office + Government Printer integration
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Track and enforce every statutory notice period required by the Constitution and Elections Act. Every the Commission action that requires a Kenya Gazette notice must flow through this module: notice preparation, statutory-period compliance checking, Government Printer submission, cross-notification to all downstream IEMS systems, and confirmation of publication. Also maintains the master election calendar visible to all IEMS modules. Constitution Art 88; Elections Act 2011 throughout.

## 2. User Personas

- **the Commission Calendar Officer** — creates and maintains statutory notice records; monitors pending obligations
- **the Commission Legal Officer** — reviews notice text for statutory compliance before submission
- **the Commission Commissioner (Chair)** — signs formal notice packages requiring Commissioner authority
- **Government Printer API** — receives notice packages for publication
- **RO / Constituency Officer** — reads calendar for their event; receives triggered alerts
- **Public** — reads published election calendar (anonymised view); checks gazette notice links
- **Downstream IEMS Modules** — subscribe to calendar events via event bus (voter register, CRMS, inventory, etc.)
- **the Commission DPO** — oversight for any personal data appearing in gazette notices

## 3. Functional Requirements

- **FR-M52-001** Election event master record: the Commission admin creates election event (general election, referendum, by-election); sets T-0 polling date; system auto-generates the full statutory notice calendar from a configurable rule set.
- **FR-M52-002** Statutory notice rule engine: for each notice type (voter register close, nomination period open/close, campaign period open/close, polling day gazette, result gazette, petition window close), system computes required publication date, statutory notice period (days before/after event), and earliest/latest permissible publication date. Rule set versioned and auditable.
- **FR-M52-003** Notice preparation workflow: for each due notice, system generates a draft notice document (EN + SW) using a statutory template; Calendar Officer reviews; Legal Officer approves; Commissioner signs (where required); system submits to Government Printer API.
- **FR-M52-004** Government Printer integration: submit signed notice package via API; receive publication confirmation with Gazette volume, issue number, and date; store confirmation and link in IEMS record.
- **FR-M52-005** Statutory compliance check: before each notice is submitted, system checks that: (a) publication date is within the statutory window, (b) prerequisite events are completed (e.g. nomination close before campaign open notice), (c) approvals are complete. Blocks submission if any check fails with a specific statutory violation citation.
- **FR-M52-006** Cross-notification bus: on gazette publication confirmation, emit events to subscribing IEMS modules (e.g. M04 VPRS for register close, M12 CRMS for nomination open, M22 for campaign period, M55 for staff activation).
- **FR-M52-007** Calendar dashboard: public-facing and internal; timeline view of all events and notice statuses; per-event deep link to gazette reference; countdown to next action.
- **FR-M52-008** Alert and escalation: for each pending notice, generate alerts at T-14 days, T-7 days, T-3 days, T-1 day before statutory deadline; escalate to Commissioner at T-1 day if not submitted.
- **FR-M52-009** By-election calendar: M50 creates a by-election event; M52 auto-generates the abbreviated statutory calendar for that single-constituency event.
- **FR-M52-010** Referendum notices: M49 triggers referendum gazette notices including question text, voting date, and county-threshold rules (where applicable).
- **FR-M52-011** Historical archive: all past elections' gazette notices retained and searchable by election year, notice type, and constituency.
- **FR-M52-012** Audit log: immutable log of every notice draft, approval, signature, submission, and Government Printer confirmation.

## 4. Non-Functional Requirements

- **NFR-M52-001** Notice calendar generation (from event creation to full calendar populated) <= 5 min.
- **NFR-M52-002** Government Printer API submission <= 30 s round-trip; retry on failure with exponential backoff for up to 24 h.
- **NFR-M52-003** Cross-notification bus event delivery to subscribers <= 60 s of gazette publication confirmation.
- **NFR-M52-004** Public calendar page p99 <= 1 s; available 24/7 (99.9% uptime).
- **NFR-M52-005** WCAG 2.2 AA for public calendar and the Commission internal screens.
- **NFR-M52-006** Languages: EN + SW for all notices and UI.
- **NFR-M52-007** DPA 2019 lawful basis: personal data in gazette notices (candidate names) processed under s.30(1)(b) (legal obligation); minimised to what statute requires; DPIA registered with ODPC for the combined election dataset.
- **NFR-M52-008** Statutory rule engine versioned: each rule change tracked with effective date; historical elections use the rule set applicable at the time.
- **NFR-M52-009** Audit log append-only; no soft-delete; retained 10 years post-election.

- **NFR-52-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `commission-iems-52` (replace 52 with the module number); alert ruleset ID `iems-52-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Commission Calendar Officer, I create the 2027 general election event and the system immediately shows me a full statutory notice calendar with 47 required actions and their deadline dates.
2. As a Commission Legal Officer, I receive a draft gazette notice for review 14 days before its statutory deadline, giving me time to revise without risk of missing the window.
3. As the Commission Chair, I sign the nomination-period gazette notice via my HSM-backed signature in the back-office app; the system submits it to the Government Printer automatically.
4. As the M04 VPRS module, I receive a cross-notification event when the voter register close gazette notice is published, so I can freeze the register at the correct statutory date.
5. As a member of the public, I view the election calendar on the Commission portal and see all upcoming statutory dates with links to the relevant Kenya Gazette issue.
6. As a Commission Calendar Officer managing a by-election, I see a separate mini-calendar for the constituency event alongside the general election calendar.
7. As the Commission Commissioner, I receive an escalation alert when a gazette notice is T-1 day from its statutory deadline and has not been submitted, with one-click sign-and-submit.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime

class ElectionEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    event_type: Literal["GENERAL_ELECTION","BY_ELECTION","REFERENDUM","PARTY_PRIMARY"]
    name_en: str
    name_sw: str
    polling_date: date               # T-0
    scope: Literal["NATIONAL","COUNTY","CONSTITUENCY","WARD"]
    scope_code: str | None           # constituency code for by-election
    status: Literal["DRAFT","GAZETTED","ACTIVE","CLOSED"]
    statutory_rule_version: str
    created_by: str
    created_at: datetime

class GazetteNotice(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    election_event_id: str
    notice_type: str                 # e.g. "REGISTER_CLOSE", "NOMINATION_OPEN", "POLLING_DAY"
    statutory_basis: str             # e.g. "Elections Act s.11(3)"
    earliest_publication: date
    latest_publication: date
    draft_text_en: str
    draft_text_sw: str
    legal_officer_approved_by: str | None
    legal_officer_approved_at: datetime | None
    commissioner_signed_by: str | None
    commissioner_signed_at: datetime | None
    submitted_to_gp_at: datetime | None
    gazette_volume: str | None
    gazette_issue: str | None
    gazette_date: date | None
    publication_confirmed_at: datetime | None
    status: Literal["DRAFT","LEGAL_REVIEW","APPROVED","SIGNED","SUBMITTED","PUBLISHED","OVERDUE"]
    cross_notifications_sent: bool

class StatutoryNoticeRule(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    version: str
    notice_type: str
    event_type: str
    days_before_t0: int | None       # negative = before T-0; positive = after T-0
    days_after_trigger_event: int | None
    trigger_event_type: str | None
    minimum_notice_days: int
    statutory_basis: str
    requires_commissioner_signature: bool
    effective_from: date
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/calendar/events:
    post:
      summary: Create election event; auto-generate notice calendar
      security: [{CommissionAdminJWT: []}]
    get:
      summary: List election events
      security: [{PublicRead: []}]
  /v1/calendar/events/{id}/notices:
    get:
      summary: Get all gazette notices for an election event
      security: [{CommissionStaffJWT: []}]
  /v1/calendar/notices/{id}/legal-approve:
    post:
      summary: Legal Officer approves draft notice
      security: [{LegalOfficerJWT: []}]
  /v1/calendar/notices/{id}/sign:
    post:
      summary: Commissioner signs notice via HSM
      security: [{CommissionerHSM: []}]
  /v1/calendar/notices/{id}/submit:
    post:
      summary: Submit to Government Printer
      security: [{CalendarOfficerJWT: []}]
  /v1/calendar/public:
    get:
      summary: Public election calendar (anonymised)
      security: [{PublicRead: []}]
  /v1/calendar/events/subscribe:
    post:
      summary: Subscribe to cross-notification events (IEMS modules only)
      security: [{ModuleAPIKey: []}]
```

## 8. Integrations

- **M49 Referendum Mechanics** — referendum gazette notices (question text, voting date)
- **M50 By-Election Operations** — by-election mini-calendar
- **M51 Result Declaration** — result gazette notice trigger after RO declaration
- **M04 VPRS** — subscriber: register close event
- **M12 CRMS** — subscriber: nomination open/close events
- **M22 Campaign Finance** — subscriber: campaign period open/close
- **M55 HR + Payroll** — subscriber: staff activation events
- **M20 Inventory Management** — subscriber: logistics activation events
- **Government Printer API** (external) — gazette notice submission and confirmation
- **P06 HSM** — Commissioner signature on notices requiring it

## 9. UI/UX Key Screens

1. **Election Calendar Timeline** — Gantt-style view of all statutory events; colour-coded by status (grey: upcoming, amber: pending action, red: overdue, green: published); exportable as PDF.
2. **Notice Preparation Workspace** — bilingual draft editor with statutory template; checklist of prerequisites; legal review + Commissioner sign-off workflow; submit button (disabled until all checks pass).
3. **Government Printer Submission Tracker** — per-notice submission status; retry button; Gazette reference once published.
4. **Public Calendar Page** — simplified chronological list of election events; gazette reference links; no internal workflow details.
5. **Alert Dashboard** — the Commission Calendar Officer view; countdown widgets per pending notice; escalation history.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| the Commission mandate for gazette notices | Constitution Art 88(3)(b) |
| Voter register close notice | Elections Act §5(2) |
| Nomination period notice | Elections Act §22(1) |
| Polling day notice | Elections Act §11(3) |
| Result gazette notice | Elections Act §86 |
| Referendum question gazette | Referendum Regulations 2017 reg.5 |
| Gazette as official legal record | Interpretation and General Provisions Act Cap 2 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Notice calendar generation time | <= 5 min after event creation | System log |
| Notices published within statutory window | 100% | Compliance dashboard |
| Government Printer submission lead time | >= minimum_notice_days before deadline | GP submission log |
| Cross-notification delivery | <= 60 s of gazette confirmation | Event bus log |
| Overdue notice rate | 0 | Compliance dashboard |

## 12. Test Strategy

- **Unit:** Statutory notice rule engine (date computation for each notice type; prerequisite checks; boundary conditions for statutory windows).
- **Integration:** Full notice workflow from creation through GP submission and cross-notification; M04, M12, M22, M55 subscription receipts.
- **E2E:** Simulate general election from event creation through all 47 required gazette notices.
- **Regression:** Any rule engine change must pass a suite of historical election date computations.
- **Accessibility:** WCAG 2.2 AA on public calendar page and the Commission notice workspace.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| Rule engine built + legal review | 2026-12-01 | the Commission Legal + Elections Act expert sign-off on rule set |
| Government Printer API MoU + integration | 2027-01-15 | Technical integration test |
| General election calendar created | 2027-02-01 | First notices begin flowing |
| First gazette notice published via system | 2027-03-01 | Voter register close notice |
| All modules subscribed to event bus | 2027-04-01 | Cross-notification integration test |

## 14. Dependencies

- Government Printer API (external MoU; the Commission to negotiate)
- P06 HSM (Commissioner signatures)
- M49, M50, M51 (upstream notice triggers)
- M04, M12, M22, M55, M20 (downstream subscribers)

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Government Printer API not available | High (new integration) | High | Manual upload fallback: system packages PDF for email to GP; human tracking of publication |
| Statutory rule engine misconfigured for new legislation | Medium | High | Rule set versioned; legal expert review required before activation; regression test suite |
| Notice submitted outside statutory window | Low | Critical | System blocks submission if outside window with specific statutory cite; Commissioner override requires reason logged |
| GP fails to publish within legal timeframe | Medium | High | System tracks publication confirmation; escalation to Commissioner if GP not confirmed within 48 h |
| Cross-notification delivery failure to downstream module | Medium | Medium | Retry queue with DLQ; alert to IEMS ops team; manual re-trigger available |

## 16. Cross-References

- `M04-vprs.md` — voter register close subscriber
- `M12-crms.md` — nomination period subscriber
- `M22-campaign-finance-transparency.md` — campaign period subscriber
- `M49-referendum-mechanics.md` — referendum notices
- `M50-by-election-operations.md` — by-election calendar
- `M51-result-declaration-certification.md` — result gazette trigger
- `M55-hr-payroll-temporary-staff.md` — staff activation subscriber

## 17. Strategy Traceability

- Constitution Art 88(3)(b) — the Commission mandate includes electoral calendar management
- Elections Act 2011 §5, §11, §22, §86 — specific notice requirements
- Referendum Regulations 2017 reg.5 — referendum question gazette
- Interpretation and General Provisions Act Cap 2 — gazette as legal record

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
