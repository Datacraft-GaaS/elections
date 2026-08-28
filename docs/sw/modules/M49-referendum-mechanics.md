# M49 — Referendum Mechanics

**Module ID:** M49
**Name:** Referendum Mechanics
**Runs on:** Central servers (Kenyan cloud) + IEBC back-office + kit fleet (mode-switched)
**Owner:** Datacraft
**Status:** Draft v1
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Provide a first-class referendum mode across the IEMS platform. Covers constitutional referendum (Art 255-257 pathway), popular-initiative referendum, and BBI-style county-approval referenda. Distinct from general-election workflow: single YES/NO question, simplified kit mode, simplified tally, county-threshold compliance checking, and dedicated publication. Strategy §XXI.B.18.

## 2. User Personas

- **IEBC Commissioner (Chair)** — activates referendum mode; signs national result declaration
- **Returning Officer (county/constituency)** — tallies Forms 37A-C; signs county tally
- **Polling Clerk** — uses kit in referendum mode (single question, reduced ballot face)
- **Public Voter** — casts YES/NO vote; verifies at public portal
- **Party/Petition Agent** — monitors referendum tally in real time
- **Attorney General** — receives formal notice of referendum result for gazette purposes
- **Supreme Court Registrar** — receives referendum certification if petition is anticipated
- **IEBC DPO** — oversees DPA 2019 compliance for voter data processed during referendum

## 3. Functional Requirements

- **FR-M49-001** Referendum configuration: IEBC admin creates referendum record — question text (EN + SW), constitutional pathway (Art 255, Art 256, or Art 257), voting date, participating counties (all 47 for constitutional; subset for devolved), threshold rules.
- **FR-M49-002** Kit mode-switch: kit firmware receives referendum config via secure OTA push; switches ballot face to single YES/NO; kit retains all M08 biometric verification and tamper-evidence.
- **FR-M49-003** Forms: generate referendum-specific Form 37A (polling station tally), 37B (constituency), 37C (county), 37D (national). Forms auto-populate question text and county-threshold requirements.
- **FR-M49-004** County-threshold engine: for Art 255/257 constitutional referendum, enforce the 24-of-47 county majority rule (each county must individually return a YES majority for the question to pass); real-time county-by-county tracker.
- **FR-M49-005** Referendum-specific tally pipeline: parallel to but separate from M10 general-election pipeline; avoids cross-contamination of result streams.
- **FR-M49-006** Referendum dashboard: live YES/NO national bar, per-county breakdown, county-threshold counter (e.g. "YES leads in 19 / 47 counties"), turnout, time-to-close estimate.
- **FR-M49-007** Automatic national result computation: once all Forms 37D are confirmed, system computes national YES total, NO total, county-majority count; surfaces pass/fail determination with statutory basis.
- **FR-M49-008** Result publication: signed snapshot to M11 Public Portal with referendum-specific template; includes county-level breakdown, county-threshold outcome per county.
- **FR-M49-009** Kit recall + mode-reset: after referendum, kits receive OTA reset to general-election mode; prior referendum data archived and locked.
- **FR-M49-010** Petition window tracking: system tracks the 7-day petition window post-declaration; alerts IEBC legal team at T-48 h and T-24 h.
- **FR-M49-011** Multi-question support (future): data model supports up to 5 simultaneous referendum questions (e.g. constitutional amendments presented as separate questions on same day) — activation configurable.

## 4. Non-Functional Requirements

- **NFR-M49-001** Kit mode-switch OTA propagated to all deployed kits within 4 hours of referendum config publication.
- **NFR-M49-002** Tally pipeline p99 form-ingest-to-confirmation <= 30 s.
- **NFR-M49-003** County-threshold engine updated within 60 s of each county Form 37C confirmation.
- **NFR-M49-004** Dashboard available to 500,000 concurrent public viewers; p99 page load <= 2 s via CDN.
- **NFR-M49-005** WCAG 2.2 AA for all public-facing referendum screens.
- **NFR-M49-006** Languages: EN + SW; question text field supports Unicode for potential future language variants.
- **NFR-M49-007** DPA 2019 lawful basis: voter biometric data processed under s.30(1)(b) (legal obligation — Elections Act); retained per IEBC data-retention schedule; referendum participation record anonymised in published tally.
- **NFR-M49-008** RTO <= 30 min; RPO <= 5 min; referendum-specific data stored in isolated schema partition.
- **NFR-M49-009** TLA+ formal specification for county-threshold state machine to eliminate off-by-one errors in Art 255 computation.

- **NFR-49-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-49` (replace 49 with the module number); alert ruleset ID `iems-49-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a Polling Clerk, I see only the YES/NO ballot face when my kit is in referendum mode, reducing my chance of presenting a wrong ballot.
2. As a Returning Officer, I confirm Form 37A tallies per polling station and the system automatically aggregates to 37B without re-entry.
3. As a party agent, I watch a live dashboard showing county-by-county YES/NO leads so I can track the 24-county threshold in real time.
4. As the IEBC Chair, I receive an automated determination ("Question PASSES: YES majority in 31 counties") with statutory basis cited, before I sign the national declaration.
5. As a member of the public, I can view my polling station's Form 37A image with Merkle proof on the results portal, the same as for general elections.
6. As the Attorney General, I receive a formal gazette-ready notice from M52 triggered automatically when the IEBC Chair signs the referendum result.
7. As an IEBC legal officer, I receive a calendar alert 48 h before the petition window closes so I can confirm whether any petition has been lodged.
8. As a kit technician, I confirm via the management console that all kits received the referendum OTA before polls open.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime
from decimal import Decimal

class Referendum(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str()
    title_en: str
    title_sw: str
    question_en: str
    question_sw: str
    pathway: Literal["ART255", "ART256", "ART257", "POPULAR_INITIATIVE"]
    voting_date: date
    county_threshold: int            # 24 for constitutional; 0 if not applicable
    status: Literal["DRAFT","CONFIGURED","ACTIVE","CLOSED","DECLARED","PETITIONED"]
    declared_at: datetime | None
    declaration_signed_by: str | None  # Commissioner ID
    created_at: datetime

class ReferendumCountyResult(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    referendum_id: str
    county_code: str
    yes_votes: int
    no_votes: int
    rejected_votes: int
    turnout: int
    form_37c_signed_by: str          # RO ID
    form_37c_signed_at: datetime
    county_yes_majority: bool
    status: Literal["PENDING","CONFIRMED","DISPUTED"]

class ReferendumNationalResult(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    referendum_id: str
    total_yes: int
    total_no: int
    total_rejected: int
    counties_yes_majority: int
    threshold_met: bool
    statutory_basis: str             # e.g. "Constitution Art 255(2)"
    result: Literal["PASSED","FAILED","PENDING"]
    form_37d_hash: str
    merkle_root: str
    published_at: datetime | None
    gazette_notice_id: str | None    # FK to M52 GazetteNotice
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/referenda:
    post:
      summary: Create referendum record
      security: [{IEBCAdminJWT: []}]
  /v1/referenda/{id}/kit-config:
    post:
      summary: Push OTA kit mode-switch config
      security: [{IEBCAdminJWT: []}]
  /v1/referenda/{id}/county-results:
    get:
      summary: List county tally results
      security: [{PublicRead: []}]
  /v1/referenda/{id}/national-result:
    get:
      summary: Get national computation
      security: [{PublicRead: []}]
  /v1/referenda/{id}/national-result/declare:
    post:
      summary: Commissioner signs national result declaration
      security: [{CommissionerHSM: []}]
  /v1/referenda/{id}/petition-window:
    get:
      summary: Petition window status and deadline
      security: [{IEBCLegalJWT: []}]
```

## 8. Integrations

- **M08 RTS Device** — kit OTA referendum mode-switch; receives single-question ballot config
- **M10 RTS Back-End** — referendum-specific tally pipeline (isolated schema partition)
- **M11 Public Portal** — referendum result publication; county dashboard
- **M52 Election Calendar + Gazette** — petition window calendar; gazette notice trigger
- **M51 Result Declaration** — hooks into RO declaration workflow for Forms 37A-D
- **P05 Merkle/Audit Chain** — Form 37D hash anchoring; publication snapshot
- **P06 HSM** — Commissioner digital signature on national result

## 9. UI/UX Key Screens

1. **Referendum Config Console** — IEBC admin: question text bilingual editor, pathway selector, county-threshold rule, OTA push status per kit batch.
2. **Live County Threshold Tracker** — 47-county grid; each cell shows YES/NO lead badge; counter "X of 24 counties reached YES majority".
3. **Referendum Tally Dashboard** — national YES/NO bar charts; turnout; time of last form confirmation; auto-refresh SSE feed.
4. **Public Results Page (referendum variant)** — simplified single-question layout; county breakdown; Merkle proof per polling station; links to Form 37A images.
5. **Kit Mode Status Console** — per-region grid showing OTA receipt confirmation before polls open.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Constitutional referendum thresholds | Constitution Art 255(2), Art 257(10) |
| Referendum procedure | Elections Act 2011 §51-§62; Referendum Regulations 2017 |
| Gazette notice of referendum result | Government Proceedings Act; Elections Act §86 |
| Petition window | Constitution Art 87; Elections Act §75 |
| Voter data processing lawful basis | DPA 2019 s.30(1)(b) |
| Kit tamper evidence | Elections (Technology) Regulations 2017 reg.8 |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Kit OTA propagation before polls open | 100% of deployed kits | Kit management console |
| County result confirmation lag | <= 2 h after county close | Tally pipeline log |
| National computation accuracy | Zero off-by-one on county threshold | Audit replay |
| Public dashboard uptime on result day | >= 99.9% | Uptime monitor |
| Petition window alert delivery | 100% at T-48 h and T-24 h | Alert system log |

## 12. Test Strategy

- **Unit:** County-threshold engine (boundary at 24; tie-break rules); Form 37D aggregation arithmetic.
- **Integration:** Kit OTA mode-switch end-to-end; tally pipeline isolation (no general-election cross-contamination).
- **E2E:** Full simulated referendum from config push through county tally to national declaration and M52 gazette notice trigger.
- **Formal:** TLA+ model of county-threshold state machine; run TLC model checker.
- **Accessibility:** WCAG 2.2 AA axe-core scan on public dashboard and kit ballot face screenshot.
- **Chaos:** Kit OTA partial failure; tally pipeline node loss during county aggregation.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| Referendum config module UAT | 2027-01-15 | IEBC admin acceptance |
| Kit OTA referendum firmware | 2027-02-01 | OTA infrastructure integration test |
| County-threshold engine validation | 2027-03-01 | IEBC legal sign-off on Art 255 computation |
| Dry-run mock referendum | 2027-06-01 | End-to-end simulation with 47 counties |
| Production-ready | T-30 (2027-07-11) | Standby; activated only if referendum is called |

## 14. Dependencies

- M08 RTS Device (kit OTA infrastructure)
- M10 RTS Back-End (isolated referendum tally partition)
- M11 Public Portal (referendum publication template)
- M51 Result Declaration (RO declaration workflow)
- M52 Election Calendar + Gazette (gazette notice trigger)
- P05 Merkle/Audit Chain
- P06 HSM

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| OTA failure leaves kits in wrong mode before polls | Low | Critical | Pre-poll kit audit required; kits refuse to open polls until mode confirmed |
| County-threshold computation off-by-one | Low | Critical | TLA+ formal verification; independent parallel computation service; RO manual confirmation |
| Referendum called with < 30 days notice | Medium | High | All referendum modules kept in standby-ready state from T-90; config-to-live < 24 h |
| Tally pipeline cross-contamination with general election | Low | High | Isolated schema partition; separate ingest queue; mutual exclusion enforced at application layer |
| Legal challenge on county-majority determination | Medium | High | Published statutory basis per county result; Merkle-anchored; Supreme Court IT Registrar pre-registration |

## 16. Cross-References

- `M08-rts-device.md` — kit OTA and mode-switch
- `M10-rts-backend.md` — tally pipeline pattern (referendum uses isolated variant)
- `M11-public-portal.md` — publication destination
- `M51-result-declaration-certification.md` — RO declaration for Forms 37A-D
- `M52-election-calendar-gazette.md` — gazette notice and petition window calendar

## 17. Strategy Traceability

- Strategy §XXI.B.18 — Referendum Mechanics module
- Constitution Art 255, Art 256, Art 257 — constitutional amendment referendum pathways
- Elections Act 2011 §51-§62 — referendum conduct
- Referendum Regulations 2017 — form requirements

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
