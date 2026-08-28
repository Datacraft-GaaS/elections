# M31 — Mobile Poll Clerk Companion (Deputy PO / Kiosk App)

**Module ID:** M31
**Module Name:** Mobile Poll Clerk Companion — Deputy PO / Kiosk App
**Type:** Mobile Application (Flutter, Android 10.0+ / iOS 15+)
**Distribution:** Private via IEBC MDM
**Users:** Deputy Presiding Officers + Polling Clerks (~92K on polling day)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in IEBC per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA03-poll-clerk.md`, sw/-module renumbered as M31.

---

## 1. Purpose

Give the Deputy Presiding Officer and polling clerks a companion tool that offloads work from the KIEMS device: reference materials, incident logging, voter-lookup backup when kit is single-user busy, structured hand-off queue, mirrored kit dashboards, and paper-backup workflow guidance. The kit remains authoritative for voter identification; this app makes the team around the kit faster and less error-prone.

Also serves as kiosk-operator app at IEBC information kiosks during the register-inspection period.

**Relationship to M05 (BVI Device):** M31 is a companion app for the poll clerk, running alongside the KIEMS device. M05 is the primary device-side BVI software that runs on the ruggedised KIEMS kit. M31 reads the BVI back-end (M06) in read-only mode; it cannot authenticate voters (that stays on the kit). The two are distinct and complementary.

---

## 2. Personas

- Deputy PO
- Polling Clerk
- IEBC Kiosk Operator (during register inspection)

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding

- **FR-M31-001** Enrolment via IEBC staff ID + biometric + device attestation.
- **FR-M31-002** Assignment: bound to specific polling station per election.
- **FR-M31-003** MFA sign-in: biometric + PIN + device attestation.

### 3.2 Reference Materials

- **FR-M31-004** Polling manual (English + Kiswahili) with searchable index.
- **FR-M31-005** Statutory forms library (Form 34A/B/C templates) for paper-backup workflow.
- **FR-M31-006** Elections Act key clauses (§55 emergency close, §6A cut-off) for on-the-spot reference.
- **FR-M31-007** Contact card: PO, RO, CRO, county support desk, war-room hotline, emergency numbers.

### 3.3 Kit Companion Dashboard

- **FR-M31-008** Live view mirroring the KIEMS BVI on-device dashboard: election, opening/closing, turnout %, voters authenticated, kit battery, connectivity, GPS fix.
- **FR-M31-009** Read-only from BVI back-end (M06); no writes.

### 3.4 Voter Lookup Fallback

- **FR-M31-010** In case KIEMS temporarily unavailable, Deputy PO can look up voter register entry by ID (with justification + biometric + audit entry).
- **FR-M31-011** Rate-limited; every lookup logged permanently with PO + Deputy PO biometric attestation.
- **FR-M31-012** No authentication of voters — that stays on the kit. Only informational lookup.

### 3.5 Structured Hand-off Queue

- **FR-M31-013** Queue management: register voter presence at station; call next per capacity; manage priority (elderly, pregnant, disabled).
- **FR-M31-014** Voter takes a ticket; DPO sees position; PO calls forward.
- **FR-M31-015** Metrics: average wait time, throughput per hour.

### 3.6 Incident Logging

- **FR-M31-016** Structured incident form (attempted double-vote, biometric fail, kit issue, physical incident, voter dispute, other) with description, evidence upload, location.
- **FR-M31-017** Escalation options: to PO, to RO, to county support desk, to war room.
- **FR-M31-018** Incidents signed by DPO biometric + queued + sent to BVI back-end (M06).

### 3.7 Paper-Backup Workflow

- **FR-M31-019** Step-by-step guidance for when the KIEMS is unavailable: paper voter authentication, paper Form 34A completion, subsequent digital reconciliation.
- **FR-M31-020** Templates + instructions embedded; no external network required.

### 3.8 Kiosk Operator Mode

- **FR-M31-021** During register-inspection period, app switches to kiosk mode: BVV register lookup for citizens with staff-assisted OTP retrieval.
- **FR-M31-022** Objection filing on behalf of citizen with citizen consent + witness signature.

### 3.9 End-of-Day

- **FR-M31-023** End-of-day report generated automatically: opening/closing times, throughput, incidents, spoiled ballots (paper log), agent presence.
- **FR-M31-024** Signed by PO + DPO biometric; submitted to BVI back-end.

### 3.10 Offline Behaviour

- **FR-M31-025** Reference materials fully offline.
- **FR-M31-026** Incident logs queued; submitted on reconnect.
- **FR-M31-027** Register-lookup fallback requires network; queued if offline with informational status.

---

## 4. Non-Functional Requirements

- **NFR-M31-001** Reference materials searchable in <= 1 s on-device.
- **NFR-M31-002** Battery cost <= 5% per hour foreground.
- **NFR-M31-003** All incident submissions succeed within 24h regardless of election-day network conditions.
- **NFR-M31-004** Certificate pinning + device attestation mandatory.

---

## 5. Integrations

- BVI back-end (M06) for dashboard + incidents + end-of-day report
- BVR back-end (M02) for register lookup fallback
- BVV back-end (M07) for kiosk mode
- Inventory Management (M20) — kit self-check on app launch confirms assigned kit is on inventory
- IEBC staff directory + accreditation
- HSM via signed-event submission

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (hard-fail) |
| mTLS | Yes |
| Biometric unlock | Required |
| Duress code | Yes |
| Root/jailbreak detection | Refuse |
| Screen-capture protection | On voter-lookup + incident screens |
| Local encrypted store | AES-256-GCM |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Kit fails | Paper-backup workflow guides; incidents queued |
| Voter lookup rate limit | Explain; suggest RO involvement |
| Attestation fail | Refuse |
| DPO absence | PO alone can still authorise via kit; app degrades |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M31-01 | End-of-day report generated + signed + submitted | UAT |
| A-M31-02 | Paper-backup workflow followed by real clerk in trial | Field trial |
| A-M31-03 | Voter-lookup fallback fully audited | Audit trace |
| A-M31-04 | Kiosk mode passes ODPC | ODPC review |
| A-M31-05 | Reference materials searchable offline | Airplane mode test |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M02 (BVR Backend) | Register lookup fallback data source |
| M05 (BVI Device) | Primary KIEMS-side BVI client; M31 is the companion (read-only mirror) |
| M06 (BVI Backend) | Dashboard data, incident submission, end-of-day report |
| M07 (BVV) | Kiosk mode register lookups and objection filing |
| M08 (RTS Device) | Kit status visible in companion dashboard via M06 |
| M20 (Inventory) | Kit self-check on launch; confirms assigned kit is in inventory system |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part XIV.3 (Operational Efficiency) | FR-M31-013 through FR-M31-015 — queue management and throughput |
| Part XV.1 (Resilience) | FR-M31-019, FR-M31-020 — paper-backup workflow |
| Part XVI.4 (Staff-Facing Mobile) | Full module scope; distribution via MDM |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA03-poll-clerk.md`. References to "MA03" in the source file correspond to "M31" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
