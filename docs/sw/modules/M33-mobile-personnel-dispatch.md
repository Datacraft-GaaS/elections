# M33 — Mobile Personnel Dispatch (the Commission Dispatch App)

**Module ID:** M33
**Module Name:** Mobile Personnel Dispatch — the Commission Dispatch App
**Type:** Mobile Application (Flutter, Android 8.0+ / iOS 14+ / PWA fallback)
**Distribution:** Private via Commission MDM + signed APK for casual-worker devices
**Users:** ~250,000 polling-day personnel (POs, DPOs, Clerks, RO staff, security, cleaners, drivers)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in the Commission per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA05-personnel-dispatch.md`, sw/-module renumbered as M33.

---

## 1. Purpose

Dispatch, attend, and track every polling-day worker across ~46,000 polling stations. Deliver assignment, GPS-verify check-in, on-site status pings, incident reporting, GPS-verified check-out, and payroll trigger. Successor to Datacraft's MeGuard pattern proven for large field-workforce management.

**Relationship to M16 (Election Command Centre) and M20 (Inventory Management):** M33 is the dispatch-side app that workers use on their own phones. ECC operators in M16 see the M33 attendance feed in real time and can reassign workers. M20 provides the inventory reference that M33 uses to validate assigned kit against the station being checked in to.

---

## 2. Personas

- Polling-day worker (PO, DPO, Clerk, RO staff, security, driver)
- the Commission HR dispatcher
- the Commission County ICT (oversight per county)
- the Commission Finance (payroll trigger)

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding

- **FR-M33-001** Enrolment via Commission HR staff ID + biometric enrolment on device.
- **FR-M33-002** MFA sign-in: biometric preferred; PIN fallback.
- **FR-M33-003** Consent per Kenya DPA for location + attendance capture.
- **FR-M33-004** Device attestation (soft-fail warn on citizen-tier devices; hard-fail for supervisor roles).

### 3.2 Assignment Delivery

- **FR-M33-005** Push notification when worker assigned to a station for an election.
- **FR-M33-006** Assignment detail: station name + code + GPS + report time + role + supervisor contact + reporting docs.
- **FR-M33-007** Assignment acceptance / decline with reason if declined.
- **FR-M33-008** Substitution flow: RO can reassign via HR back-office; app updates in real time.

### 3.3 Check-In

- **FR-M33-009** On arrival at assigned station, worker checks in with biometric + GPS.
- **FR-M33-010** GPS distance validated against station polygon; check-in fails if > threshold from station.
- **FR-M33-011** Check-in timestamp signed + submitted.
- **FR-M33-012** Supervisor notified.

### 3.4 On-Site Status

- **FR-M33-013** Periodic status pings (every 15 min) confirm presence.
- **FR-M33-014** Explicit "status" screen: On-station, Break, Absent (with reason).
- **FR-M33-015** Supervisor view of team status live.

### 3.5 Incident Reporting

- **FR-M33-016** Quick-report form: type (medical, security, kit-issue, dispute, other), description, evidence.
- **FR-M33-017** Escalation to supervisor + county lead + war-room per severity.

### 3.6 Check-Out

- **FR-M33-018** End-of-shift check-out with biometric + GPS.
- **FR-M33-019** Check-out triggers payroll milestone (integration with the Commission Finance).
- **FR-M33-020** Summary card: hours worked, incidents reported, next assignment (if any).

### 3.7 Notifications

- **FR-M33-021** Assignment received, reassignment, supervisor message, incident escalation, payroll processed.
- **FR-M33-022** No PII in payloads; content fetched authenticated on receipt.

### 3.8 Offline Behaviour

- **FR-M33-023** Assignment cached locally, viewable offline.
- **FR-M33-024** Check-in / check-out captured locally, signed, queued, submitted on reconnect with idempotency keys.
- **FR-M33-025** Timestamp integrity: use signed local time source; server-side verification.

### 3.9 Accessibility and Localisation

- **FR-M33-026** English + Kiswahili.
- **FR-M33-027** WCAG 2.2 AA (upgraded from WCAG 2.1 per Teaming Annex A).
- **FR-M33-028** Voice-guided workflow for low-literacy users.

---

## 4. Non-Functional Requirements

- **NFR-M33-001** Check-in flow <= 30 s.
- **NFR-M33-002** Battery cost <= 3% per hour foreground.
- **NFR-M33-003** Handles 250K concurrent check-ins over 90-minute window.
- **NFR-M33-004** Runs on Android 8+ mid-tier devices used by casual workers.
- **NFR-M33-005** Certificate pinning + device attestation (soft) mandatory.

---

## 5. Integrations

- the Commission HR / staff directory
- Personnel Dispatch back-end (O05, extended from MeGuard)
- the Commission Finance for payroll trigger
- BVI back-end (M06) for cross-reference to polling-station status
- Election Command Centre (M16) for real-time attendance feed and reassignment
- Inventory Management (M20) for kit validation at check-in
- Location services (GPS)
- SMS Gateway (I08) for OTP fallback

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (soft-fail warn) |
| mTLS | Yes |
| Biometric unlock | Required |
| Screen-capture protection | On assignment + PII screens |
| Local encrypted store | AES-256-GCM |
| Anti-spoof GPS | IMU cross-check where available |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Worker off-station | Check-in refused; instructed to move; supervisor alerted |
| GPS unavailable | Fallback to network-triangulated location + supervisor override |
| Duplicate check-in | Idempotency prevents; alert on multiple devices for same worker |
| Payroll trigger fails | Queue; retry; alert Finance |
| Assignment collision (two workers assigned same station role) | HR back-office resolves |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M33-01 | End-to-end assign → check-in → check-out flow | UAT |
| A-M33-02 | 250K concurrent check-ins over 90 min | Load test on back-end |
| A-M33-03 | Offline check-in submits correctly on reconnect | Airplane mode + reconnect |
| A-M33-04 | Payroll trigger fires on check-out | Integration test |
| A-M33-05 | GPS-off-station check-in refused | Field trial |
| A-M33-06 | WCAG 2.2 audit pass | Independent audit |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M06 (BVI Backend) | Polling-station status cross-reference at check-in |
| M16 (Election Command Centre) | ECC operators see real-time attendance feed from M33; ECC dispatches reassignments |
| M20 (Inventory Management) | Kit validation at check-in; confirms assigned station kit is in system |
| O05 (Personnel Dispatch Backend) | System of record for assignments, check-in, check-out, payroll trigger |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part XIV.3 (Field Operations) | FR-M33-009 through FR-M33-015 — GPS check-in and on-site monitoring |
| Part XV.1 (Resilience) | FR-M33-023 through FR-M33-025 — offline-first attendance capture |
| Part XV.3 (Localisation) | FR-M33-026, FR-M33-028 — voice-guided workflow |
| Part XVI.4 (Public-Facing Mobile) | Distribution, accessibility, and wide device-base requirements |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA05-personnel-dispatch.md`. References to "MA05" in the source file correspond to "M33" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
