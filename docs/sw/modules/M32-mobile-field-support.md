# M32 — Mobile Field Support (Datacraft Field Engineer App)

**Module ID:** M32
**Module Name:** Mobile Field Support — Datacraft Field Engineer App
**Type:** Mobile Application (Flutter, Android 10.0+ / iOS 15+)
**Distribution:** Private via Commission MDM (Datacraft device roster)
**Users:** Datacraft Field Support Engineers (~200 on election day)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in the Commission per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA04-field-support.md`, sw/-module renumbered as M32.

---

## 1. Purpose

Give every field engineer a mobile console for the assigned county / cluster of polling stations. Support-ticket queue, remote diagnostics against kits, KEBS / warranty claim initiation, escalation to Datacraft L2/L3 + Mechrevo OEM + the Commission ICT, on-call rotation, and end-of-day reporting.

**Relationship to M20 (Inventory Management):** M32 is the field-engineer mobile app that consumes the Inventory Management API (M20) to look up spares availability and locate the nearest spare-holding constituency store. M20 is the system of record; M32 is a consumer client. The two are distinct.

---

## 2. Personas

- Field Support Engineer (Datacraft L1)
- Regional Lead (multi-county oversight)
- Datacraft L2 / L3 (escalation target)

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding and Assignment

- **FR-M32-001** Enrolment via Datacraft staff ID + biometric + device attestation.
- **FR-M32-002** Cluster assignment: engineer bound to counties / constituencies for the election.
- **FR-M32-003** MFA sign-in.

### 3.2 Ticket Queue

- **FR-M32-004** Live ticket queue filtered by engineer's cluster, severity, age.
- **FR-M32-005** Sev 1 / 2 / 3 SLA timers visible (2h / 4h / 6h response; 1d / 2d / best-effort resolution).
- **FR-M32-006** Take / assign / re-assign ticket with dual notification.
- **FR-M32-007** In-app messaging with reporter (typically PO or the Commission County ICT).
- **FR-M32-008** Attach evidence: photo, short video, diagnostic log capture.

### 3.3 Kit Diagnostics

- **FR-M32-009** Look up kit by serial number / QR scan.
- **FR-M32-010** Live kit health telemetry: battery, connectivity, GPS fix, temperature, error counts, last-seen.
- **FR-M32-011** Remote diagnostic actions (subject to policy engine + kit consent): fetch logs, request reboot, request firmware attestation, trigger self-test.
- **FR-M32-012** All remote actions signed + audited; kit-side user visible confirmation for privileged actions.
- **FR-M32-013** Warranty status + spares availability check via Inventory Management (M20) and Warranty (O04) integration.

### 3.4 KEBS / Warranty Claim Initiation

- **FR-M32-014** File warranty claim: OEM identified via kit metadata; ticket sent to Mechrevo + OEM.
- **FR-M32-015** Track claim status.
- **FR-M32-016** Coordinate spare-part shipment / hot-swap kit dispatch via M20 IMS API.

### 3.5 Escalation

- **FR-M32-017** One-tap escalation to Datacraft L2 → L3 → Mechrevo → OEM → the Commission ICT per matrix.
- **FR-M32-018** Escalation carries ticket context + attachments.
- **FR-M32-019** Escalation logged permanently.

### 3.6 On-Call Rotation

- **FR-M32-020** On-call schedule per engineer visible.
- **FR-M32-021** Push notification on new ticket assignment or SLA breach approaching.
- **FR-M32-022** "Handoff" to next on-call at shift boundary; sign-off required.

### 3.7 Field Toolkit

- **FR-M32-023** Diagnostic runbook library indexed by symptom.
- **FR-M32-024** Contact card for PO / RO / county support desk / war-room hotline.
- **FR-M32-025** Kit reset / recovery step-by-step guidance.
- **FR-M32-026** Spares inventory in reach of engineer (mapped to nearest spare-holding constituency store via M20).

### 3.8 End-of-Day Report

- **FR-M32-027** Automatic end-of-shift report: tickets handled, SLA compliance, escalations, spares consumed, incidents.
- **FR-M32-028** Signed + submitted to Datacraft Ops + the Commission ICT.

### 3.9 Offline Behaviour

- **FR-M32-029** Runbook library fully offline.
- **FR-M32-030** Ticket queue read-only offline (last-sync).
- **FR-M32-031** Diagnostic actions queue if kit-gateway unreachable; retry on connectivity.
- **FR-M32-032** Incidents / reports queued locally, submitted on reconnect.

---

## 4. Non-Functional Requirements

- **NFR-M32-001** Ticket queue refresh <= 2 s over LTE.
- **NFR-M32-002** Battery cost <= 5% per hour foreground.
- **NFR-M32-003** Certificate pinning + device attestation mandatory.
- **NFR-M32-004** Every diagnostic action logged with actor + target + time + result.

---

## 5. Integrations

- Support / Ticketing (O02)
- Inventory Management (M20) for spares lookup and spare-part dispatch
- MDM (O01)
- Warranty (O04)
- BVI back-end (M06)
- Datacraft escalation matrix
- the Commission Election Command Centre (M16) for war-room dispatch visibility
- the Commission War-Room dashboard

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (hard-fail) |
| mTLS | Yes |
| Biometric unlock | Required |
| Screen-capture protection | On kit-diagnostic screens |
| Root/jailbreak detection | Refuse |
| Anti-tamper | Signature check on launch |
| PII in logs | None |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Kit unreachable | Queue diagnostic action; escalate after N min |
| Warranty ticket rejected by OEM | Auto-escalate to Mechrevo |
| SLA breach | Alert + escalation cascade |
| No network on engineer's phone | Runbook + escalation via SMS fallback (special short-code) |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M32-01 | End-to-end ticket cycle within SLA on real device | UAT |
| A-M32-02 | Remote diagnostic action signed + audited | Audit trace |
| A-M32-03 | Escalation matrix reachable and correct | Bench |
| A-M32-04 | Offline runbook search | Airplane mode test |
| A-M32-05 | End-of-day report signed + submitted | UAT |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M06 (BVI Backend) | Kit operational status and incident cross-reference |
| M16 (Election Command Centre) | Field dispatch visibility; ECC operators can see engineer location and ticket status |
| M20 (Inventory Management) | Spares availability, spare-part dispatch, kit self-check — M32 is a consumer of M20 IMS API |
| O01 (MDM) | Private app distribution and remote device management |
| O02 (Support / Ticketing) | Ticket lifecycle back-end |
| O04 (Warranty) | Warranty claim initiation and status tracking |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part XIV.3 (Field Operations) | FR-M32-004 through FR-M32-008 — ticket queue and SLA management |
| Part XV.1 (Resilience) | FR-M32-029 through FR-M32-032 — offline-first runbook and queued actions |
| Part XVI.4 (Internal Ops Mobile) | Full module scope; private MDM distribution |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA04-field-support.md`. References to "MA04" in the source file correspond to "M32" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
