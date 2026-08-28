# M35 — CVR: Continuous Voter Register

**Module ID:** M35
**Name:** Continuous Voter Register Maintenance
**Runs on:** Central servers (Kenyan cloud) + IEBC back-office
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Maintain the voter register as a living record between election cycles rather than as a point-in-time snapshot. Ingest death notifications from CRS (Civil Registration Service), identity updates from NRB (National Registration Bureau), and KRA-PIN dormancy signals to remove ineligible entries; process family-notification short-code submissions for unreported deaths; synchronise updates with the ABIS biometric store (M03) and BVR back-end (M02). Distinct from M01 (initial registration) which handles first-time enrolment.

Strategy reference: §XXI.B.15. Elections Act 2011 §6A (continuous registration), §8A (register correction).

## 2. User Personas

- **IEBC Registration Officer** — reviews flagged records; approves or rejects proposed removals/corrections
- **NRB Data Officer** — submits ID-update batch files via secure API
- **CRS Data Officer** — submits death-notification batch files
- **KRA Data Officer** — submits PIN-dormancy flags
- **Citizen (family member)** — reports a registered voter's death via short-code *400#
- **ODPC Data Protection Officer** — audits processing basis and data-retention compliance
- **IAB Observer** — views aggregate churn metrics; does not access individual records
- **IEBC Commissioner** — signs off on quarterly deregistration batches
- **DPA Compliance Lead (Datacraft)** — oversees DPIA and lawful-basis documentation

## 3. Functional Requirements

- **FR-M35-001** CRS death-feed ingestion: receive daily batch (SFTP + PGP-encrypted, schema-validated) of new death certificates; match against voter register by National ID + biometric reference; generate proposed-removal records with confidence score.
- **FR-M35-002** NRB ID-update ingestion: receive weekly batch of name/ID-number corrections; apply corrections to voter records after officer review; update ABIS reference (M03) via M02 back-end.
- **FR-M35-003** KRA-PIN dormancy signal: receive quarterly flag of PINs with no activity for 7+ years; cross-reference with voter register; create a "dormancy review" record for officer inspection (not automatic removal — officer must confirm).
- **FR-M35-004** Family short-code intake: *400# USSD allows a family member to report a death by entering ID number of deceased; system creates a "family-notification" provisional record; requires CRS confirmation within 90 days before removal proceeds.
- **FR-M35-005** Officer triage queue: case officers see a prioritised queue of proposed removals with confidence scores; can approve, reject, or hold pending further evidence. All actions logged with actor + timestamp + basis code.
- **FR-M35-006** Biometric cross-check before removal: any proposed removal triggers an M03 ABIS soft-hold on the biometric template so no new registration can reuse it until the record is resolved.
- **FR-M35-007** Correction workflow: NRB-sourced corrections applied to name, ID number, date of birth; full before-and-after audit trail; voter notified by SMS/post where contact exists.
- **FR-M35-008** Quarterly gazette: IEBC Commissioner signs a gazette notice of removals; gazette notice PDF anchored in P05 Merkle chain; published on M11 public portal.
- **FR-M35-009** Register version snapshots: before any batch is committed, a signed snapshot of the register's root hash is stored; any snapshot can be reproduced for audit or court submission.
- **FR-M35-010** Voter self-check: citizen can query their own registration status via M13 Public Verifier App using National ID; system returns current status and last-updated date without exposing internal flags.
- **FR-M35-011** Bulk-correction audit: every batch import (CRS, NRB, KRA) produces a reconciliation report (records received, matched, unmatched, flagged, actioned) logged to the IAB dashboard (M15).
- **FR-M35-012** Multi-language notifications: SMS/USSD communications in EN + SW + Kikuyu + Luo + Kamba as detected from the voter's registered county.

## 4. Non-Functional Requirements

- **NFR-M35-001** Batch ingestion processing: <= 4 hours for CRS daily batch of up to 2,000 records.
- **NFR-M35-002** Officer triage console: p99 <= 500 ms under 200 concurrent officers.
- **NFR-M35-003** Register snapshot creation: <= 60 seconds per snapshot for a register of 22M entries.
- **NFR-M35-004** Availability: >= 99.5% during non-election periods; >= 99.9% in T-90 to T+7 window.
- **NFR-M35-005** DPA 2019 lawful basis: s.30(1)(b) (legal obligation — Elections Act §6A/§8A); DPIA registered with ODPC; data-minimisation enforced (CRS death data retained only until removal confirmed, then purged); retention of audit trails 7 years.
- **NFR-M35-006** WCAG 2.2 AA: citizen self-check interface (M13 integration) and short-code confirmation screens.
- **NFR-M35-007** Security: batch files validated against PGP signature of sending agency before parsing; no raw CRS/NRB data stored beyond processing window.
- **NFR-M35-008** Scalability: handles concurrent batches from all three agencies without queue starvation.

- **NFR-35-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-35` (replace 35 with the module number); alert ruleset ID `iems-35-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As an IEBC Registration Officer, I see a queue of proposed removals ranked by confidence score so I can resolve high-confidence CRS matches first.
2. As a family member, I dial *400# and enter my parent's ID number to report their death; I receive an SMS acknowledging the provisional record.
3. As an NRB Data Officer, I upload a weekly ID-correction batch via the secure API and receive a reconciliation report within 4 hours.
4. As an IAB Observer, I view aggregate churn metrics (records added, corrected, removed this quarter) on the IAB dashboard without accessing any individual record.
5. As an IEBC Commissioner, I sign the quarterly gazette of deregistered voters using my HSM-backed credential; the gazette PDF is published on the public portal.
6. As a citizen, I check my registration status on the Public Verifier App and see it is active with a last-verified date of the previous quarter.
7. As a KRA Data Officer, I submit dormancy flags and receive confirmation that each flagged record has entered officer-review triage, not automatic removal.
8. As an ODPC DPO, I pull the processing log to verify that CRS death-notification raw data was purged within the defined retention window.
9. As an IEBC Registration Officer, I reject a CRS-proposed removal because the death certificate's ID number has a transcription error; I log the rejection basis and the record is restored.
10. As a petitioner, I request a signed register snapshot from election eve via the court IT interface (M40); the system produces a verifiable root hash and Merkle proof within 2 hours.

## 6. Data Model Additions

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import datetime, date

class VoterRegisterEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    voter_id: str                   # FK to core voter record
    event_type: Literal[
        "DEATH_PROPOSED","DEATH_CONFIRMED","ID_CORRECTION",
        "DORMANCY_FLAG","FAMILY_NOTIFICATION","REMOVAL_APPROVED",
        "REMOVAL_REJECTED","REINSTATEMENT"
    ]
    source: Literal["CRS","NRB","KRA","FAMILY_SHORTCODE","OFFICER_MANUAL"]
    confidence_score: float | None  # 0.0-1.0 for automated matches
    batch_id: str | None
    officer_id: str | None
    decision: Literal["APPROVED","REJECTED","PENDING","HELD"] | None
    decision_basis: str | None      # statutory code + free text
    created_at: datetime
    resolved_at: datetime | None

class RegisterSnapshot(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    snapshot_at: datetime
    total_registered: int
    root_hash: str                  # SHA-256 of sorted voter ID list
    merkle_root: str
    gazette_notice_id: str | None
    signed_by: str                  # IEBC Commissioner officer_id
    signed_at: datetime | None

class BatchIngestionReport(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    source: Literal["CRS","NRB","KRA"]
    received_at: datetime
    record_count: int
    matched: int
    unmatched: int
    flagged: int
    actioned: int
    errors: int
    report_url: str
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/cvr/batches/crs | CRS_SYSTEM mTLS | 5/day | Ingest CRS death-notification batch |
| POST | /v1/cvr/batches/nrb | NRB_SYSTEM mTLS | 5/day | Ingest NRB ID-correction batch |
| POST | /v1/cvr/batches/kra | KRA_SYSTEM mTLS | 5/quarter | Ingest KRA dormancy flags |
| GET | /v1/cvr/events | IEBC_OFFICER JWT | 200/hr | List triage queue (proposed removals) |
| PATCH | /v1/cvr/events/{id} | IEBC_OFFICER JWT | 100/hr | Approve / reject / hold event |
| GET | /v1/cvr/snapshots | IEBC_ADMIN JWT | 20/hr | List register snapshots |
| POST | /v1/cvr/snapshots | IEBC_COMMISSIONER HSM | 2/day | Create and sign new snapshot |
| GET | /v1/cvr/snapshots/{id}/proof | COURT_REGISTRAR JWT | 10/hr | Return Merkle proof for snapshot |
| POST | /v1/cvr/family-notifications | USSD_GATEWAY secret | 1000/hr | Intake *400# family death report |
| GET | /v1/cvr/status/{national_id_hash} | PUBLIC (rate-limited) | 30/hr | Voter self-check (status + last updated; no internal flags) |

## 8. Integration Points

- **M02 BVR Back-End** — voter record corrections applied here
- **M03 ABIS** — biometric soft-hold on proposed removals
- **M13 Public Verifier App** — citizen self-check surface
- **M15 IAB Dashboard** — batch reconciliation and aggregate churn metrics
- **M36 Voter Deregistration Appeal** — escalation path for rejected removals disputed by family
- **M40 Election Court IT Interface** — signed snapshots for petitioners
- **P05 Merkle Chain** — gazette notice and snapshot anchoring
- **P06 HSM** — Commissioner signing
- **CRS (Civil Registration Service)** — external, MoU, SFTP batch daily
- **NRB (National Registration Bureau)** — external, MoU, SFTP batch weekly
- **KRA** — external, MoU, SFTP batch quarterly
- **Africa's Talking / Safaricom** — *400# USSD gateway

## 9. UI/UX Key Screens

1. **Officer Triage Queue** — table ranked by confidence score; columns: voter ID masked, proposed event type, source, age of event; approve/reject/hold buttons with required basis note.
2. **Record Detail** — before-and-after view of proposed change; source document link (redacted except for officer); ABIS biometric status indicator; decision history.
3. **Batch Ingestion Dashboard** — per-agency tiles showing last batch received, record counts, error count, next expected batch; drill-through to reconciliation report.
4. **Gazette Preparation** — commissioner view: summary of approved removals this quarter; total count; HSM sign button; preview gazette PDF.
5. **Citizen Self-Check (M13 integration)** — single field: National ID; returns status badge (Active/Pending Review/Removed) + last-updated date; WCAG 2.2 AA; EN + SW + four Kenyan languages.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Continuous registration and correction | Elections Act 2011 §6A, §8A |
| Right to be registered as voter | Constitution Art 38(3)(b) |
| Data protection — lawful processing basis | DPA 2019 §30(1)(b) |
| DPIA and ODPC notification | DPA 2019 §31 |
| Data minimisation and retention limits | DPA 2019 §26 |
| Gazette of removals | Elections Act 2011 §15 |
| Death registration integration authority | Births and Deaths Registration Act Cap 149 |
| National ID data sharing | Registration of Persons Act Cap 107 (NRB MoU) |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| CRS batch processed within SLA | 100% within 4 hours | Ingestion timestamp delta | Datacraft SRE |
| Officer triage queue backlog (non-election) | <= 500 pending events at any time | Queue depth monitor | IEBC Registration Director |
| Register snapshot time | <= 60 seconds | Snapshot creation log | Datacraft Technical |
| False-positive proposed removals | <= 0.1% of CRS matches | Post-quarter audit | IEBC + CRS |
| Citizen self-check availability | >= 99.5% | Uptime monitor | Datacraft SRE |
| Gazette notices anchored in Merkle chain | 100% | Merkle reconciliation | Datacraft Technical |

## 12. Test Strategy

- **Unit:** CRS matching algorithm; confidence-score calculator; ID-correction deduplication; dormancy-flag cross-reference.
- **Integration:** SFTP batch ingestion end-to-end; ABIS soft-hold trigger; M02 voter-record update; Merkle anchor.
- **E2E:** simulate CRS batch with known test records; verify proposed-removal creation, officer approval, ABIS hold, gazette inclusion, and Merkle proof.
- **Security:** attempt to ingest unsigned batch (must reject); attempt to access triage queue without IEBC_OFFICER role; verify PII purge after processing window.
- **Accessibility:** WCAG 2.2 AA audit on citizen self-check screen via axe-core + manual.
- **Load:** 2,000 CRS records processed within 4-hour SLA under normal officer queue load.
- **Chaos:** NRB batch arrives 24 hours late — verify queue does not stall; CRS SFTP unreachable — verify retry with alerting.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| CRS/NRB/KRA MoUs signed | 2026-12-01 | IEBC-facilitated; Datacraft delivers technical spec |
| Batch ingestion pipeline live | 2027-01-15 | Dry-run with synthetic data |
| Officer triage console UAT | 2027-02-01 | 20 IEBC Registration Officers |
| *400# USSD live | 2027-03-01 | Family short-code; Safaricom integration |
| First live batch run | 2027-04-01 | CRS quarterly batch processed for real |
| Pre-election register freeze | 2027-07-01 | T-40: snapshot signed and gazetted |
| Post-election analytics | 2027-09-10 | T+31: churn report published |

## 14. Dependencies

- M02 BVR Back-End — voter record storage
- M03 ABIS — biometric reference management
- M36 Voter Deregistration Appeal — downstream appeal workflow
- M40 Election Court IT Interface — snapshot delivery to petitioners
- P05 Merkle Chain — gazette and snapshot anchoring
- P06 HSM — Commissioner signing
- CRS, NRB, KRA MoUs — external; IEBC to negotiate post-award
- *400# USSD gateway — Safaricom/Africa's Talking

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CRS death records contain transcription errors matching live voters | High | High | Confidence-score threshold; officer review mandatory before removal; affected voter SMS notification |
| CRS/NRB MoU delayed past go-live | Medium | High | Build ingestion pipeline against synthetic data; activate MoU as post-award IEBC priority |
| *400# short-code abused to mass-flag voters | Medium | High | Rate-limit per MSISDN (5/month); CRS confirmation required before any removal proceeds |
| KRA dormancy flag sweeps recently deceased voters already in CRS queue | Low | Medium | Deduplication layer: if voter already has pending CRS event, KRA flag is subordinated |
| Register snapshot latency exceeds 60 seconds at 22M entries | Low | Medium | Pre-computed partial Merkle tree updated incrementally; full recompute only on gazette |

## 16. Cross-References

- `M01-bvr-device.md` — initial biometric registration
- `M02-bvr-backend.md` — voter record store
- `M03-abis.md` — biometric template management
- `M13-public-verifier-app.md` — citizen self-check surface
- `M15-iab-dashboard.md` — aggregate churn metrics
- `M36-voter-deregistration-appeal.md` — appeal workflow for disputed removals
- `M40-election-court-it-interface.md` — signed snapshot for court use

## 17. Traceability to Strategy Paper

| Strategy Reference | Module Coverage |
|--------------------|----------------|
| §XXI.B.15 — inter-cycle register maintenance | FR-M35-001 through FR-M35-011 |
| §XXI.B.15 — CRS integration for deaths | FR-M35-001, FR-M35-004 |
| §XXI.B.15 — NRB integration for ID updates | FR-M35-002, FR-M35-007 |
| Elections Act §6A/§8A — continuous registration | FR-M35-005, FR-M35-008, NFR-M35-005 |

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
