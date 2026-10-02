# M30 — Mobile Party Agent (Agent App)

**Module ID:** M30
**Module Name:** Mobile Party Agent — Agent App
**Type:** Mobile Application (Flutter, Android 10.0+ / iOS 15+)
**Distribution:** Private via Commission MDM (Google Play Managed + Apple Business Manager)
**Users:** Accredited party agents (~2K per party concurrent peak; national total 20K+ across all parties)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in the Commission per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA02-agent.md`, sw/-module renumbered as M30.

---

## 1. Purpose

Give every accredited party agent a phone-in-hand tool that (a) authenticates them into their assigned polling station, (b) lets them independently verify that the Form 34A transmitted from the KIEMS matches the physical form in front of them, (c) countersigns that verification cryptographically, (d) retrieves and validates constituency + county rollups, and (e) reports incidents in a structured, signed way that reaches the Commission war room.

The Agent app is the citizen-verifiability layer's institutional twin: agents witness in-person and cryptographically attest — building on the same signed-events chain the public sees in M29.

No direct overlap with M01-M28. New capability that adds agent-layer countersignature to the existing RTS transmission chain.

---

## 2. Personas

- Party Agent (accredited per polling station)
- Party Chief Agent (constituency-level; roll-up view)
- Independent Auditor (accredited)

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding and Accreditation

- **FR-M30-001** Agent enrols via Commission accreditation ID + party-issued token + biometric enrolment on device.
- **FR-M30-002** Assignment: agent bound to a specific polling station (or constituency scope for chief agent) per election.
- **FR-M30-003** Device attestation required; refuses on rooted / jailbroken / attestation-fail devices.
- **FR-M30-004** MFA sign-in: biometric + short PIN + device attestation.

### 3.2 Polling-Day Check-In

- **FR-M30-005** Agent checks in at assigned polling station with biometric + GPS-verified location.
- **FR-M30-006** Presiding Officer receives notification of agent presence via BVI back-end.
- **FR-M30-007** Party badge display for identification to PO.

### 3.3 Live Station Dashboard

- **FR-M30-008** Real-time view (subject to configurable delay): voters authenticated, turnout %, opening/closing status, kit status.
- **FR-M30-009** No PII; aggregate counters only.

### 3.4 Form 34A Verification

- **FR-M30-010** When PO scans a Form 34A on the KIEMS RTS device, the transmission includes a QR code visible to agents present.
- **FR-M30-011** Agent uses app to scan the QR from the physical form OR from the kit screen.
- **FR-M30-012** App fetches signed Form 34A record from RTS back-end; displays digital signature chain (PO signature, kit signature, back-end countersignature).
- **FR-M30-013** Agent compares displayed values to physical form in front of them.
- **FR-M30-014** Agent presses "Verify" — app fetches Merkle-inclusion proof, validates client-side, confirms transmission integrity.
- **FR-M30-015** Agent presses "Countersign" — biometric + PIN authorises HSM-backed party-agent countersignature attached to the form's audit trail.
- **FR-M30-016** Agent presses "Dispute" — flags form as disputed with mandatory reason; escalates to national tally + war room.

### 3.5 Constituency and County Rollup

- **FR-M30-017** Chief Agent view: aggregated Form 34B / 34C as they publish; per-position counts; per-station contribution.
- **FR-M30-018** "Verify" button on rollups: fetches Merkle root, validates aggregated hashes against contributing forms.
- **FR-M30-019** "Dispute" available at rollup level too.

### 3.6 Incident Reporting

- **FR-M30-020** Structured incident form: type (obstruction, tampering, voter intimidation, kit malfunction, illegal campaigning, other), description, evidence upload (photo, short video, audio).
- **FR-M30-021** Location auto-captured with consent per-report.
- **FR-M30-022** Reports encrypted locally, queued, signed by agent, sent to the Commission war room.
- **FR-M30-023** Report status tracked; the Commission responder can request more info via in-app messaging.

### 3.7 Notifications

- **FR-M30-024** Push notifications for: assigned-station events (opening, closing, form submission), chief-agent rollup publication, the Commission broadcast messages.
- **FR-M30-025** No PII in payloads.

### 3.8 Offline Behaviour

- **FR-M30-026** All verification flows work offline against cached signed records if the record has been fetched before.
- **FR-M30-027** Countersignatures + incident reports queued locally, encrypted, signed, submitted on reconnect with idempotency keys.

---

## 4. Non-Functional Requirements

- **NFR-M30-001** Full form verification cycle (scan QR + fetch + verify + countersign) <= 30 s.
- **NFR-M30-002** Signature verification via Rust FFI validator.
- **NFR-M30-003** Battery cost <= 5% per hour foreground use (polling-day-long deployment).
- **NFR-M30-004** Local queue survives phone reboot.
- **NFR-M30-005** Certificate pinning + device attestation mandatory.

---

## 5. Integrations

- the Commission accreditation database (agent registry)
- RTS back-end (M10) for form retrieval + countersignature submission
- BVI back-end (M06) for check-in notification
- Public Portal (M11) for rollup snapshots (read-only, same feed as M15 IAB Dashboard but without IAB elevated privileges)
- Transparency log service
- HSM via signed-event submission
- the Commission war-room incident-response channel

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (hard-fail) |
| mTLS + certificate pinning | Yes |
| Biometric unlock | Required |
| Duress code | Yes (silent alert) |
| Root/jailbreak detection | Refuse |
| Hooking detection (Frida, Xposed) | Refuse |
| Screen-capture protection | On all screens |
| App-switcher preview | Hidden |
| Local encrypted store | AES-256-GCM in secure enclave |
| PII in logs | None |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Attestation fail | Refuse; user briefed |
| No network | Cached forms verify offline; countersign queued |
| Signature mismatch on Form 34A | Automatic dispute flag; alert war room |
| GPS off / spoofed | Alert; feature limited |
| Duress code | Silent alert to war room; visible flow proceeds |
| Kit not present (wrong station) | Refuse check-in; instruct agent to right station |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M30-01 | End-to-end verify + countersign flow on real device | UAT |
| A-M30-02 | Attestation fail refuses correctly | Rooted-device test |
| A-M30-03 | Countersignature reaches audit ledger with Merkle proof | Audit trace |
| A-M30-04 | Duress code triggers silent war-room alert | Red team |
| A-M30-05 | Incident report end-to-end with attachments | UAT |
| A-M30-06 | Offline verification works for cached records | Bench |
| A-M30-07 | MDM force-update policy applied | Bench |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M06 (BVI Backend) | Check-in notification to Presiding Officer |
| M08 (RTS Device) | Source of Form 34A QR code at kit side |
| M09 (RTS RO) | Constituency-level rollup data (Form 34B) |
| M10 (RTS Backend) | Form retrieval and countersignature submission |
| M11 (Public Portal) | Rollup snapshots (read-only) |
| M15 (IAB Dashboard) | Sibling module for named IAB members with elevated privileges; M30 is party-agent view with countersignature rights but no IAB role access |
| M23 (Grievance) | Incident reports from agents escalate through Grievance module |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part IX (Public Transparency) | FR-M30-014, FR-M30-015 — cryptographic countersignature on Form 34A |
| Part XIV.3 (Multi-Party Verification) | FR-M30-010 through FR-M30-016 — agent-layer verification |
| Part XVI.4 (Public-Facing Mobile) | Distribution and security hardening requirements |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA02-agent.md`. References to "MA02" in the source file correspond to "M30" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
