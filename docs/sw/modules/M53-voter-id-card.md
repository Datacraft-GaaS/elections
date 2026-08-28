# M53 — Voter ID Card Production

**Module ID:** M53
**Name:** Voter ID Card Production
**Runs on:** Central servers (Kenyan cloud) + NRB integration + card manufacturing partner integration
**Owner:** Datacraft
**Status:** Draft v1 (CONDITIONAL — module is INACTIVE by default; activated only if the signed 2027 IEMS tender scope or IEBC written instruction explicitly includes physical voter card production; SCOPE.md v3 must carry an explicit ACTIVE/INACTIVE flag for M53 before FAT gate; absent such instruction the module is out of scope and no engineering effort is allocated)
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Manage the end-to-end lifecycle of per-voter physical voter ID cards, if included in the 2027 IEMS tender scope. Historical context: 2013 issued voter registration slips; 2017/2022 relied on the national ID (Kipande) as the sole voter identity document. A 2027 physical voter card (with embedded QR code and potentially RFID) would enable polling-station biometric verification against a card rather than purely against a central server. Module is designed as an optional layer above M01-M04; it is activated only if the tender includes physical card production.

## 2. User Personas

- **IEBC Registration Officer** — triggers card production batch per registration cycle; confirms delivery
- **NRB Data Officer** — provides base biographic data for card personalisation (NRB integration)
- **Card Manufacturing Partner** — external secure printing facility; receives personalisation data via encrypted channel
- **Voter** — receives card; uses it at polling station; requests replacement if lost
- **Polling Clerk** — scans card QR at polling station via M08 kit (if card mode enabled)
- **IEBC DPO** — oversees DPA 2019 DPIA for physical biometric card production; minimisation review
- **IEBC Logistics Officer** — tracks card distribution to constituencies and collection points

## 3. Functional Requirements

- **FR-M53-001** Voter eligibility check: before card issuance, verify voter is registered in M04 VPRS and NRB confirms national ID validity; flag duplicates.
- **FR-M53-002** Card personalisation data package: extract per-voter record (name, DOB, constituency, polling station, voter number, photo from BVR M02); apply DPA 2019 minimisation — exclude biometric templates from card data package; generate QR payload (voter ID hash, constituency, polling station, card serial).
- **FR-M53-003** RFID chip option: if tender includes RFID, generate contactless chip payload (same as QR; no biometric template on chip); chip content signed with IEBC HSM key; chip read at polling station verified against central record.
- **FR-M53-004** Batch production order: group personalisation packages by county; encrypt per-batch with manufacturer's public key; transmit via mutually-authenticated secure channel; receive batch receipt and estimated delivery date.
- **FR-M53-005** Production tracking: per-card status lifecycle (ORDERED, PRINTING, QUALITY_CHECK, DISPATCHED, DELIVERED, COLLECTED, ACTIVE, LOST_REPORTED, CANCELLED).
- **FR-M53-006** Distribution workflow: cards dispatched to constituency collection points; IEBC officer at collection point scans batch receipt; voter collects card with biometric verification (M01 BVR device at collection point); card status set to COLLECTED and ACTIVE.
- **FR-M53-007** Replacement workflow: voter reports lost/stolen card; IEBC officer verifies identity via biometric; previous card serial flagged as LOST in system (accepted no more at polling stations); new card issued with new serial.
- **FR-M53-008** Polling station integration: M08 kit reads QR or RFID; sends card serial to M06 BVI backend for verification (card active, voter eligible, not already voted); verification response in <= 2 s.
- **FR-M53-009** Fallback without card: if voter arrives without card (lost, not yet collected), polling clerk can authenticate via biometric only (M01 mode) subject to RO authorisation; all fallback events logged.
- **FR-M53-010** Post-election card management: after election, cards remain valid for next election cycle unless superseded; cancelled cards blacklisted.
- **FR-M53-011** Statistics dashboard: cards ordered, printed, delivered, collected, active, lost per county; coverage rate.

## 4. Non-Functional Requirements

- **NFR-M53-001** Card personalisation data package generation: full national batch (<= 22 million records) completed within 72 hours on commodity hardware.
- **NFR-M53-002** Polling-station card verification (QR/RFID to response) p99 <= 2 s including network round-trip to central server.
- **NFR-M53-003** DPA 2019: no biometric templates stored on card (chip or QR); data minimisation DPIA required before production; card data processed under s.30(1)(b) (legal obligation); retention until next election cycle or 7 years max.
- **NFR-M53-004** Card data package transmission to manufacturer: AES-256 encryption; mTLS channel; no intermediate storage in plaintext at any third party.
- **NFR-M53-005** WCAG 2.2 AA for voter-facing collection and replacement portal.
- **NFR-M53-006** Languages: EN + SW for all voter-facing communications.
- **NFR-M53-007** Availability of card verification endpoint at polling stations: >= 99.9% on polling day; offline fallback (card QR contains sufficient data for local verification if central server unreachable).
- **NFR-M53-008** Lost-card blacklist propagated to all polling station caches within 30 minutes of report.

- **NFR-53-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-53` (replace 53 with the module number); alert ruleset ID `iems-53-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## 5. User Stories

1. As a voter, I receive a letter to my registered address and SMS notification that my voter card is ready for collection at my constituency office, with collection dates and biometric verification requirements.
2. As a voter, I present my national ID at the collection point, verify with fingerprint, and collect my voter card in under 3 minutes.
3. As a Polling Clerk, I scan the QR code on a voter's card with the kit camera; the system confirms "VOTER ELIGIBLE — NOT YET VOTED" in under 2 seconds.
4. As a voter who lost their card, I go to the IEBC constituency office, verify biometrically, and receive a replacement card within 5 working days; the old serial is immediately blacklisted.
5. As an IEBC Logistics Officer, I view a county-by-county dashboard showing collection rate (%) so I can target areas with low card uptake for additional mobile collection drives.
6. As a voter who arrives without their card, the Polling Clerk can authenticate me via biometric only after the Returning Officer authorises, ensuring I am not disenfranchised.
7. As the IEBC DPO, I review the card production data package manifest to confirm no biometric templates are included before approving batch transmission to the manufacturer.

## 6. Data Model

```python
from pydantic import BaseModel, ConfigDict
from typing import Literal
from datetime import date, datetime

class VoterCard(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                          # uuid7str() — internal
    serial: str                      # printed on card; QR payload
    voter_id: str                    # FK to M04 VPRS voter record
    constituency_code: str
    polling_station_code: str
    qr_payload_hash: str             # SHA-256 of signed QR content
    rfid_chip: bool
    rfid_payload_hash: str | None
    batch_id: str
    status: Literal["ORDERED","PRINTING","QUALITY_CHECK","DISPATCHED","DELIVERED","COLLECTED","ACTIVE","LOST_REPORTED","CANCELLED"]
    dispatched_at: datetime | None
    delivered_at: datetime | None
    collected_at: datetime | None
    collected_by_voter_biometric_confirmed: bool
    replacement_for_serial: str | None
    created_at: datetime

class CardProductionBatch(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    county_code: str
    card_count: int
    manufacturer_ref: str | None
    encrypted_package_hash: str
    transmission_channel: str        # e.g. "SFTP_MTLS_V2"
    transmitted_at: datetime | None
    manufacturer_receipt_at: datetime | None
    estimated_delivery: date | None
    status: Literal["PENDING","TRANSMITTED","RECEIVED","PRINTING","DISPATCHED","COMPLETE"]

class CardVerificationEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    card_serial: str
    polling_station_code: str
    kit_id: str
    verification_method: Literal["QR","RFID","BIOMETRIC_FALLBACK"]
    result: Literal["APPROVED","CARD_INVALID","ALREADY_VOTED","BLACKLISTED","FALLBACK_AUTHORISED"]
    response_ms: int
    timestamp: datetime
```

## 7. API (OpenAPI 3.1 excerpt)

```yaml
paths:
  /v1/voter-cards/batches:
    post:
      summary: Create card production batch for a county
      security: [{IEBCAdminJWT: []}]
    get:
      summary: List batches
      security: [{LogisticsOfficerJWT: []}]
  /v1/voter-cards/{serial}/status:
    get:
      summary: Get card status
      security: [{IEBCStaffJWT: []}]
  /v1/voter-cards/{serial}/collect:
    post:
      summary: Record voter collection with biometric confirmation
      security: [{RegistrationOfficerJWT: []}]
  /v1/voter-cards/{serial}/report-lost:
    post:
      summary: Report card lost; blacklist serial
      security: [{RegistrationOfficerJWT: []}]
  /v1/voter-cards/verify:
    post:
      summary: Polling station card verification (M08 kit calls this)
      security: [{KitMTLS: []}]
  /v1/voter-cards/stats:
    get:
      summary: Card production and collection statistics
      security: [{IEBCStaffJWT: []}]
```

## 8. Integrations

- **M02 BVR Back-End** — voter photo and registration data for card personalisation
- **M04 VPRS** — voter eligibility check before card issuance
- **M01 BVR Device / M08 RTS Device** — biometric verification at collection point and polling station
- **M06 BVI Back-End** — card verification endpoint called by M08 kit on polling day
- **NRB** (external) — national ID validation before card issuance
- **Card Manufacturing Partner** (external) — encrypted batch transmission
- **P06 HSM** — QR/RFID payload signing; card serial signing
- **P05 Merkle/Audit Chain** — card issuance and blacklist events anchored

## 9. UI/UX Key Screens

1. **Card Production Dashboard** — batch status by county; cards ordered/printed/dispatched/collected; coverage heatmap.
2. **Voter Collection Portal** — IEBC officer view; scan voter national ID; biometric prompt; card handover confirmation with signature capture.
3. **Lost Card Replacement Form** — voter details, biometric prompt, reason for replacement; immediate blacklist of old serial; replacement order triggered.
4. **Polling Station Verification Screen** (M08 kit) — large YES/NO card eligibility result; fallback authorisation flow if card absent.
5. **Card Statistics Dashboard** — per-county collection rate; percentage of registered voters with active card; low-uptake alert list.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Voter identification at polling | Elections Act 2011 §62; BVI Regulations |
| Biometric data minimisation | DPA 2019 s.25 (data minimisation principle) |
| Lawful basis for card data | DPA 2019 s.30(1)(b) (legal obligation) |
| DPIA requirement | DPA 2019 s.31; ODPC DPIA Guidelines |
| NRB data sharing | National Registration Act Cap 107; NRB-IEBC MoU |
| Card security standards | ISO/IEC 7816 (chip); ISO/IEC 14443 (RFID) |
| Card data retention | IEBC data-retention schedule; 7 years max |

## 11. Success Metrics + SLA

| Metric | Target | Measurement |
|--------|--------|-------------|
| Card collection rate before polling day | >= 90% of registered voters | Collection dashboard |
| Card verification response time at polling station | p99 <= 2 s | Kit telemetry |
| Lost-card blacklist propagation | <= 30 min to all kit caches | Sync log |
| Biometric-fallback rate on polling day | < 2% of verifications | Polling day log |
| Batch transmission encryption confirmed | 100% of batches | HSM audit log |

## 12. Test Strategy

- **Unit:** QR payload generation and signing; RFID payload; card status lifecycle transitions; blacklist propagation.
- **Integration:** End-to-end from VPRS eligibility check through batch production to polling-station verification.
- **Security:** Confirm no biometric template in card data package; attempt to verify blacklisted card; mTLS channel verification with manufacturer mock.
- **Performance:** 22M record personalisation batch generation within 72 h on reference hardware.
- **Offline:** Kit offline verification using QR-embedded data when central server unreachable.
- **Accessibility:** WCAG 2.2 AA on voter collection portal.

## 13. Rollout (T-0 = 2027-08-10)

| Milestone | Date | Description |
|-----------|------|-------------|
| Tender scope confirmed (card or no card) | 2026-10-01 | Module activated or deferred |
| NRB MoU and integration | 2026-12-01 | Data sharing agreement |
| DPIA registered with ODPC | 2027-01-15 | IEBC DPO sign-off |
| Card manufacturer contracted | 2027-02-01 | Procurement via PPADA |
| First county batch transmission | 2027-04-01 | Pilot county |
| National batch complete | 2027-06-01 | T-70 before polling day |
| 90% collection target | T-7 (2027-08-03) | |

## 14. Dependencies

- M02 BVR Back-End (voter data source)
- M04 VPRS (eligibility check)
- M06 BVI Back-End (polling-day verification)
- M08 RTS Device (kit QR/RFID scanner)
- NRB (external MoU)
- Card Manufacturing Partner (external procurement)
- P06 HSM

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tender excludes physical card (module unused) | Medium | Low | Module designed as optional overlay; no impact on core IEMS if deactivated |
| Low card collection rate disenfranchises voters | Medium | High | Biometric-fallback mode preserves access; mobile collection drives; SMS reminders |
| Card manufacturer security breach (personalisation data leaked) | Low | Critical | Data package contains no biometric templates; QR payload is voter ID hash not raw data; minimisation DPIA pre-approved |
| NRB MoU delays card production timeline | Medium | High | IEBC to engage NRB at contract award; backup: use M02 BVR data only without NRB cross-check |
| RFID card clone attack | Low | High | RFID payload HSM-signed; polling station verifies signature against central record; cloned card fails signature check |

## 16. Cross-References

- `M01-bvr-device.md` — biometric verification at collection point
- `M02-bvr-backend.md` — voter photo and registration source
- `M04-vprs.md` — voter eligibility check
- `M06-bvi-backend.md` — polling-day card verification endpoint
- `M08-rts-device.md` — QR/RFID scanner at polling station

## 17. Strategy Traceability

- Elections Act 2011 §62 — voter identification at polling station
- DPA 2019 s.25, s.30(1)(b), s.31 — data minimisation, lawful basis, DPIA
- NRB-IEBC MoU — national ID data sharing
- ISO/IEC 7816, ISO/IEC 14443 — card and RFID standards

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
