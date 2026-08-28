# M40 — Election Court IT Interface

**Module ID:** M40
**Name:** Election Court IT Interface and Evidence Package System
**Runs on:** Central servers (Kenyan cloud) + Supreme Court IT Registrar integration
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Provide the Supreme Court IT Registrar, High Court election judges, and registered petitioners with pre-emptive, authenticated access to IEBC's evidentiary record. Assemble and publish the fifteen-item evidentiary bundle specified in Strategy §XII.4 within 24 hours of election day close. Maintain a pre-registered technical-witness portal. Answer the Raila v IEBC [2017] eKLR para 272 "declined access" finding by making access architecturally guaranteed rather than discretionary.

Strategy reference: §XII.4 (evidentiary bundle items), §XII (election petition management). Constitution Art 140, Elections Act 2011 §75-§85.

## 2. User Personas

- **Supreme Court IT Registrar** — receives automated evidence bundle; manages court-side verification
- **Election Petitioner (or Petitioner's Counsel)** — authenticated portal user; requests specific evidence items
- **IEBC Technical Witness** — pre-registers availability; receives witness portal access for court-appearance support
- **IEBC Legal Director** — oversees evidence bundle publication; signs off on completeness
- **IEBC Chief Technical Officer** — technical witness lead; certifies system-generated data accuracy
- **High Court / Election Court Judge** — receives bundle via Supreme Court IT Registrar; may request supplementary items
- **IAB Observer** — views evidence bundle index (public-facing summary); confirms items were published on schedule
- **Datacraft CISO** — ensures no unauthorised access to unpublished evidence items

## 3. Functional Requirements

- **FR-M40-001** Pre-emptive bundle publication: within 24 hours of election-day results-transmission close (T+1), the system publishes a signed evidentiary bundle containing all fifteen Strategy §XII.4 items. Publication is automatic and does not require a petition to trigger it.
- **FR-M40-002** The fifteen §XII.4 bundle items are:
  1. Certified electronic voter register (signed snapshot from M35)
  2. Biometric verification logs per station (from M06 BVI back-end)
  3. Scanned Form 34A, 34B, 34C images with cryptographic provenance (from M10/M11)
  4. Transparency-log Merkle roots + inclusion proofs for all result forms
  5. Ballot paper chain-of-custody bundle per constituency (from M39)
  6. KIEMS kit audit logs per station (from M08)
  7. Anomaly detection event log — sanitised (from M26)
  8. Voter turnout time-series per station (from M06)
  9. Network connectivity log (uptime per station, aggregated)
  10. Certification log for all poll workers who served (from M38)
  11. Grievance and election offences case log — sanitised (from M23)
  12. Campaign finance compliance status at election day (from M22)
  13. Boundary delimitation dataset (from M18)
  14. Technical witness register + credential hashes (M40 internal)
  15. System audit trail: all IEBC officer actions on result records from T-7 to T+1
- **FR-M40-003** Bundle integrity: each item is individually HSM-signed; a bundle manifest (JSON) is signed over the SHA-256 of each item; manifest hash anchored in P05 Merkle chain.
- **FR-M40-004** Petitioner portal: registered petitioners (authenticated via Kenya Judiciary eCourt system or IEBC-issued portal credential) can browse the bundle index; request specific items; download signed packages; track delivery status.
- **FR-M40-005** 24-hour log-production commitment: any specific log file (e.g. KIEMS audit log for station X) requested by the Supreme Court IT Registrar or a registered petitioner is produced and delivered within 24 hours of the authenticated request.
- **FR-M40-006** Supplementary request workflow: Court Registrar submits a structured supplementary request (item type, scope, date range); IEBC Legal Director reviews and approves; system assembles and signs the supplementary package; delivery tracked.
- **FR-M40-007** Technical-witness pre-registration: IEBC technical witnesses register via portal (name, credential type, expected availability, areas of testimony); system assigns portal credentials and generates a signed witness statement template for each registered area.
- **FR-M40-008** Court-appearance support portal: during petition hearing, technical witness can access a read-only evidence console showing all items they may be questioned on; console logs witness access events for court records.
- **FR-M40-009** Public bundle index: the bundle index (item names, publication timestamps, item hashes) is published on M11 public portal without requiring a login. Item downloads require petitioner authentication. IAB observers can verify hash publication independently.
- **FR-M40-010** Declined-access prevention: every bundle item is available to a registered petitioner by right, not by IEBC discretion. If an item is not yet produced, the system shows a scheduled-delivery timestamp. No item can be withheld without a formal IEBC Legal Director override logged to the audit trail with reason.
- **FR-M40-011** WCAG 2.2 AA: petitioner portal and public bundle index are fully accessible.
- **FR-M40-012** Multi-language: EN + SW on all portal surfaces; bundle manifest in EN.

## 4. Non-Functional Requirements

- **NFR-M40-001** Bundle publication: all fifteen items published within 24 hours of T+1 results close.
- **NFR-M40-002** Specific log production on request: <= 24 hours from authenticated request to delivery.
- **NFR-M40-003** Petitioner portal: p99 <= 1 second under 1,000 concurrent authenticated users.
- **NFR-M40-004** Bundle availability: >= 99.99% from T+1 to T+90 (beyond typical petition filing window of 28 days under Elections Act §75; retained through appeal).
- **NFR-M40-005** HSM signing: all items and bundle manifest signed with IEBC Legal Director HSM credential; signing completes within 60 seconds per item.
- **NFR-M40-006** DPA 2019: bundle items containing personal data (biometric logs, voter register) access-controlled to authenticated petitioners; no PII in public bundle index; DPA §30(1)(d) (legal proceedings) as lawful basis.
- **NFR-M40-007** Audit trail: every petitioner download, every supplementary request, every technical-witness access event logged with actor, item, timestamp, IP; retained for 7 years.
- **NFR-M40-008** Security: petitioner portal uses Kenya Judiciary eCourt OAuth or IEBC-issued certificate; all bundle items served over TLS 1.3; bundle items signed and verifiable independently.

- **NFR-40-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-40` (replace 40 with the module number); alert ruleset ID `iems-40-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As the Supreme Court IT Registrar, I receive a notification at T+1+18h that all fifteen bundle items are published; I download the bundle manifest and verify each item hash independently.
2. As a petitioner's counsel, I log into the portal and download the biometric verification log for Kibra constituency within 2 hours of filing my petition.
3. As an IEBC Technical Witness, I pre-register my availability and areas of testimony (KIEMS kit operation, BVI biometric matching); I receive a signed witness statement template for each area.
4. As a High Court election judge, I receive via the Supreme Court IT Registrar the full ballot chain-of-custody bundle for Mathare constituency; every scan event has a Merkle proof.
5. As a petitioner, I see in the portal that KIEMS audit logs for my target station are scheduled for T+1+20h; I receive a delivery notification when they are available.
6. As an IAB Observer, I check the public bundle index on M11 and verify that the Form 34A image hashes match what I independently captured during results transmission.
7. As an IEBC Technical Witness, during the hearing I open the evidence console for the specific station under examination; my access is logged and timestamped for court records.
8. As a petitioner, I submit a supplementary request for the network connectivity log for all stations in Nairobi County; I receive the signed package within 24 hours.
9. As the IEBC Legal Director, I confirm the bundle completeness by signing the manifest; I cannot mark the bundle complete if any of the fifteen items has not been attached.
10. As a Datacraft CISO, I audit the access log for the petitioner portal and confirm that no item was accessed before the authenticated petitioner's registration was verified.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime

class BundleItem(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    bundle_id: str
    item_number: int                # 1-15 per §XII.4
    item_name: str
    source_module: str              # e.g. "M35","M06","M39"
    scope: str                      # e.g. "NATIONAL","CONSTITUENCY:NBI-010"
    file_id: str                    # encrypted blob reference
    item_hash: str                  # SHA-256 of item file
    signed_by: str                  # officer_id
    signed_at: datetime | None
    scheduled_at: datetime          # expected availability
    published_at: datetime | None
    merkle_node_id: str | None

class EvidenceBundle(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    election_id: str
    items: list[str]                # BundleItem IDs
    manifest_hash: str              # SHA-256 of manifest JSON
    manifest_merkle_node_id: str | None
    legal_director_signed_at: datetime | None
    published_at: datetime | None
    publication_deadline: datetime  # T+1 + 24 hours

class PetitionerAccess(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    petitioner_id: str
    auth_method: Literal["JUDICIARY_ECOURT_OAUTH","IEBC_PORTAL_CERT"]
    registered_at: datetime
    petition_reference: str | None
    items_requested: list[str]      # BundleItem IDs
    items_delivered: list[str]

class SupplementaryRequest(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    requestor_id: str
    requestor_type: Literal["COURT_REGISTRAR","PETITIONER","HIGH_COURT_JUDGE"]
    item_type: str
    scope: str
    date_range_start: str | None
    date_range_end: str | None
    status: Literal["RECEIVED","APPROVED","ASSEMBLING","DELIVERED","DECLINED"]
    legal_director_approved_at: datetime | None
    delivered_at: datetime | None
    decline_reason: str | None

class TechnicalWitness(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    officer_id: str
    credential_type: str
    testimony_areas: list[str]
    availability_start: datetime
    availability_end: datetime
    statement_template_ids: list[str]
    registered_at: datetime
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/court/bundles | IEBC_LEGAL JWT | 2/election | Create evidence bundle record |
| POST | /v1/court/bundles/{id}/items | SOURCE_MODULE mTLS | 100/hr | Attach signed bundle item from source module |
| POST | /v1/court/bundles/{id}/sign | IEBC_LEGAL HSM | 1/election | Legal Director sign and publish bundle |
| GET | /v1/court/bundles/{id}/index | Public | 500/min | Bundle index (item names, hashes, timestamps; no files) |
| GET | /v1/court/bundles/{id}/items/{item_id} | PETITIONER JWT | 50/hr | Download signed bundle item file |
| POST | /v1/court/petitioners | IEBC_LEGAL JWT | 100/hr | Register petitioner with portal access |
| POST | /v1/court/supplementary | COURT_REGISTRAR JWT | 20/day | Submit supplementary evidence request |
| PATCH | /v1/court/supplementary/{id} | IEBC_LEGAL JWT | 20/day | Approve / assemble / deliver supplementary |
| POST | /v1/court/witnesses | IEBC_TECHNICAL JWT | 50/hr | Pre-register technical witness |
| GET | /v1/court/witnesses/{id}/console | TECHNICAL_WITNESS JWT | 50/hr | Read-only evidence console (hearing support) |
| GET | /v1/court/access-log | IEBC_LEGAL JWT | 20/hr | Petitioner download audit log |

All endpoints: TLS 1.3; every response signed with IEBC public key.

## 8. Integration Points

- **M35 CVR** — item 1: signed voter register snapshot
- **M06 BVI Back-End** — items 2, 8: biometric logs, turnout time-series
- **M10 RTS Back-End / M11 Public Portal** — items 3, 4: Form 34 images and Merkle proofs
- **M39 Ballot Printing Chain** — item 5: chain-of-custody bundle
- **M08 RTS Device** — item 6: KIEMS kit audit logs
- **M26 Anomaly Detection** — item 7: sanitised anomaly event log
- **M38 Poll Worker Training LMS** — item 10: worker certification log
- **M23 Grievance Channels** — item 11: sanitised grievance log
- **M22 Campaign Finance** — item 12: compliance status
- **M18 Boundary Transparency** — item 13: delimitation dataset
- **P05 Merkle Chain** — bundle manifest anchoring
- **P06 HSM** — Legal Director signing
- **Kenya Judiciary eCourt** — petitioner OAuth (external, MoU)
- **M37 Voter Education CMS** — media portal (separate surface sharing M40 infrastructure)

## 9. UI/UX Key Screens

1. **Bundle Publication Dashboard (IEBC Legal)** — fifteen-item checklist with traffic-light status (not started / scheduled / published); deadline countdown; Legal Director sign button (enabled only when all 15 items are attached).
2. **Public Bundle Index (M11 integration, WCAG 2.2 AA)** — table: item name, publication timestamp, SHA-256 hash; download icon (greyed for unauthenticated users); IAB verification note.
3. **Petitioner Portal (WCAG 2.2 AA)** — authenticated; left-nav: bundle items, supplementary requests, delivery tracker; item detail: metadata, download button, verification guide.
4. **Supplementary Request Form** — item type picker, scope (national / county / constituency / station), date range, justification; submit to IEBC Legal for approval.
5. **Technical Witness Pre-Registration** — areas of testimony checklist; availability calendar; generate statement template button.
6. **Evidence Console (Hearing Support)** — read-only display of all items relevant to the witness's registered testimony areas; search by station code; access events auto-logged.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Election petition process | Constitution Art 140; Elections Act 2011 §75-§85 |
| 28-day petition filing window | Elections Act 2011 §75(2) |
| Evidence disclosure obligation | Civil Procedure Act Cap 21; Evidence Act Cap 80 |
| Access to information (petitioner) | Access to Information Act 2016 §4 |
| Raila v IEBC [2017] eKLR para 272 compliance | Judicial precedent — declined access finding addressed by FR-M40-010 |
| Personal data in legal proceedings | DPA 2019 §30(1)(d) (legal proceedings lawful basis) |
| Electronic evidence admissibility | Kenya Information and Communications Act 2013 §83F |
| Technical witness obligations | Evidence Act Cap 80 (expert witness) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| All 15 bundle items published | 100% within 24 hours of T+1 close | Bundle publication timestamp | IEBC Legal Director |
| Specific log production on request | 100% within 24 hours | Supplementary delivery timestamp | IEBC CTO |
| Petitioner portal availability (T+1 to T+90) | >= 99.99% | Uptime monitor | Datacraft SRE |
| Bundle manifest anchored in Merkle chain | 100% | Merkle reconciliation | Datacraft Technical |
| Zero unauthorised item access | 0 incidents | Access log audit | Datacraft CISO |
| Technical witness pre-registration complete | 100% by T-7 | Witness registry | IEBC Legal Director |

## 12. Test Strategy

- **Unit:** bundle-item hash computation; manifest signing logic; declined-access prevention gate; 24-hour deadline calculator.
- **Integration:** each source module (M06, M10, M26, M38, M39 etc.) pushes its item to the bundle; verify item count reaches 15; Legal Director sign triggers Merkle anchor.
- **E2E:** simulate T+1 event; verify automatic bundle assembly and publication; register test petitioner; download item; verify signature against IEBC public key.
- **Security:** unauthenticated user attempts to download item file (must fail); petitioner attempts to download item outside their registered scope (must fail); verify Legal Director cannot mark complete with any of the 15 items missing.
- **Accessibility:** WCAG 2.2 AA on public bundle index and petitioner portal (axe-core + screen reader + manual).
- **Load:** 1,000 concurrent authenticated petitioner downloads during 28-day petition window.
- **Chaos:** P05 Merkle chain temporarily unavailable — verify bundle publication proceeds; Merkle anchor retried with exponential back-off; alert raised.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Kenya Judiciary eCourt OAuth MoU | 2027-01-15 | External; IEBC-facilitated |
| Bundle assembly pipeline live | 2027-03-01 | All source-module integrations tested |
| Technical-witness portal UAT | 2027-04-01 | IEBC CTO + 5 technical witnesses |
| Petitioner portal UAT | 2027-05-01 | Simulated petition with mock bundle |
| Dry-run with by-election evidence bundle | 2027-02-28 | End-to-end with real system outputs |
| Pre-election technical-witness registration | 2027-07-31 | T-10: all witnesses pre-registered |
| Election day: bundle assembly begins | 2027-08-10 | T+0: automated pipeline starts |
| Bundle published | 2027-08-11 | T+1 + 24 hours deadline |
| Petition window close | 2027-09-07 | T+28: evidence retained through appeal |

## 14. Dependencies

- All fifteen source modules (M06, M08, M10, M11, M18, M22, M23, M26, M35, M38, M39, M40 internal)
- P05 Merkle Chain — manifest anchoring
- P06 HSM — Legal Director signing
- Kenya Judiciary eCourt OAuth — external petitioner authentication
- M11 Public Portal — public bundle index

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| One source module fails to deliver its item within 24 hours | Medium | High | Per-item scheduled-delivery timestamps published; automated retry with alert; IEBC Legal Director may publish partial bundle and annotate missing item with scheduled-delivery timestamp |
| Kenya Judiciary eCourt OAuth not ready | Medium | High | IEBC-issued portal certificate as fallback authentication; Judiciary integration live by T-30 or fallback activates |
| Volume of supplementary requests overwhelms IEBC Legal team | Medium | Medium | Auto-assemble standard supplementary items (known item types); Legal Director approves at batch level |
| Bundle manifest challenged as tampered in court | Low | Critical | Merkle proof + HSM signature + public-index hash provide triple verification; IAB independently publishes item hashes at T+1 |
| Technical witness unavailable during hearing | Low | High | Minimum two witnesses registered per testimony area; schedule conflicts resolved at pre-registration stage |

## 16. Cross-References

- `M06-bvi-backend.md`, `M08-rts-device.md`, `M10-rts-backend.md`, `M11-public-portal.md` — result evidence items
- `M18-boundary-transparency.md` — delimitation dataset
- `M22-campaign-finance-transparency.md` — compliance status item
- `M23-grievance-channels.md` — grievance log item
- `M26-anomaly-fraud-detection.md` — anomaly log item
- `M35-cvr-continuous-voter-register.md` — voter register snapshot
- `M38-poll-worker-training-lms.md` — certification log item
- `M39-ballot-printing-chain.md` — chain-of-custody bundle item

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XII.4 — fifteen evidentiary bundle items | FR-M40-002 (items 1-15) |
| §XII — election petition management | FR-M40-004, FR-M40-006, FR-M40-010 |
| Raila v IEBC [2017] eKLR para 272 | FR-M40-010; NFR-M40-004; design rationale |
| Constitution Art 140 | Compliance Mapping |
| Elections Act §75-§85 | Compliance Mapping; FR-M40-001, FR-M40-005 |

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
