# M34 — Mobile Observer (Accredited Independent and International Observer App)

**Module ID:** M34
**Module Name:** Mobile Observer — Accredited Observer App
**Type:** Mobile Application (Flutter, Android 10.0+ / iOS 15+)
**Distribution:** Private via IEBC MDM
**Users:** ~500 accredited observers (civil society, international bodies, academic, media)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in IEBC per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA06-observer.md`, sw/-module renumbered as M34.

---

## 1. Purpose

Support accredited observers with a structured mobile tool that mirrors the Public Portal's read views, provides observer-specific incident reporting per accreditation category, and lets observers verify results cryptographically at any polling station they visit. Observer findings feed the transparency layer and the post-election Auditor-General report.

**Relationship to M15 (IAB Dashboard):** M15 is for named Independent Assurance Board members with elevated privileges and governance-level result access. M34 is for field observer-mission personnel with read-only access to the same underlying data but no IAB role assignment. Distinct roles, distinct data-access tiers, distinct apps. Both should be kept separate.

**Relationship to M29 (Mobile Citizen Verify):** M34 shares the Merkle-inclusion-proof verification workflow with M29 but adds observer-specific layers: structured accreditation, station visit logs, countersignature of verification records, and Auditor-General escrow feed. The shared Rust FFI verifier component may be extracted as a common library.

---

## 2. Personas

- Domestic civil-society observer (ELOG, KEDOF, etc.)
- International observer (EU EOM, Carter Center, AU EOM, EAC observer mission)
- Academic observer (universities)
- Media accredited observer

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding and Accreditation

- **FR-M34-001** Enrolment via IEBC accreditation ID + biometric + device attestation.
- **FR-M34-002** MFA sign-in with FIDO2 or platform authenticator.
- **FR-M34-003** Accreditation category determines feature availability (some categories may not have countersignature rights).

### 3.2 Public Portal Mirror

- **FR-M34-004** Full read of Public Portal (M11) data: national + county + constituency + station results with signed Form 34A images.
- **FR-M34-005** Merkle-inclusion-proof retrieval + client-side verification.
- **FR-M34-006** Independent verification of published snapshots against transparency-log roots.
- **FR-M34-007** Bookmark stations of interest for follow-along notifications.
- **FR-M34-008** Bulk download of signed forms per constituency (rate-limited).

### 3.3 Station Visit Log

- **FR-M34-009** GPS check-in at a polling station observers visit.
- **FR-M34-010** Time-boxed observation session; sign off when leaving.
- **FR-M34-011** Signed observation record submitted to IEBC observer registry.

### 3.4 Observer Incident Reporting

- **FR-M34-012** Structured incident form: category (procedural, security, legal, operational, tampering, transparency, other), description, evidence upload, GPS.
- **FR-M34-013** Reports encrypted locally, signed, queued, submitted to IEBC observer coordination + war-room + Auditor-General mirror.
- **FR-M34-014** Anonymised aggregated observer-report dashboard visible on Public Portal (per IEBC-defined policy).
- **FR-M34-015** Observer chief may signal a systemic incident that requires immediate IEBC response.

### 3.5 Verification Sampling

- **FR-M34-016** Guided workflow to sample-verify polling stations per statistical protocol (n stations per constituency chosen randomly).
- **FR-M34-017** For each sampled station, observer completes structured verification: form-hash matches transmitted, agent countersignatures present, geo-fence intact, seals intact.
- **FR-M34-018** Observer countersignature attached to verification record.

### 3.6 Report Compilation

- **FR-M34-019** End-of-mission report auto-compiled from observer's activity: stations visited, verifications completed, incidents reported.
- **FR-M34-020** Exported to PDF for the observer's own mission-report handoff.
- **FR-M34-021** Feeds Auditor-General post-election audit input via M21 (RLA Audit Tool) observation feed.

### 3.7 Offline Behaviour

- **FR-M34-022** Public Portal snapshots cached; verifiable offline.
- **FR-M34-023** Station visit + incident reports queued; submitted on reconnect.
- **FR-M34-024** Chief-observer signal delivered via SMS/USSD fallback if network unavailable at critical moment.

---

## 4. Non-Functional Requirements

- **NFR-M34-001** Verification cycle per station <= 60 s.
- **NFR-M34-002** Battery cost <= 5% per hour foreground.
- **NFR-M34-003** Certificate pinning + device attestation mandatory.
- **NFR-M34-004** Multilingual: English + Kiswahili + international observer language pack (French, Portuguese) subject to demand.

---

## 5. Integrations

- IEBC accreditation database (observer registry)
- Public Portal (M11) for results and Form 34A data
- Public Verifier App (M13) shared Merkle-proof API
- RTS back-end (M10) for Merkle-inclusion-proof verification
- Transparency log service
- RLA Audit Tool (M21) — observer verification records feed into the RLA audit observation stream
- Auditor-General escrow mirror
- HSM via signed-event submission

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (hard-fail) |
| mTLS + certificate pinning | Yes |
| Biometric unlock | Required |
| Screen-capture protection | On incident-report screens |
| Root/jailbreak detection | Refuse |
| Local encrypted store | AES-256-GCM |
| Duress alerting | Optional |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Attestation fail | Refuse |
| GPS off | Verification session degraded; check-in requires manual justification |
| Verification hash mismatch | Automatic incident report generated |
| Chief-observer signal offline | SMS/USSD fallback path |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M34-01 | End-to-end verification of a polling station | UAT |
| A-M34-02 | Incident submission with signed attachments | UAT |
| A-M34-03 | Observer's end-of-mission PDF generated | Bench |
| A-M34-04 | Auditor-General mirror receives observer records | Audit trace |
| A-M34-05 | Multilingual pack loads for international observers | Bench |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M10 (RTS Backend) | Merkle-inclusion-proof retrieval for station verification |
| M11 (Public Portal) | Read-only results data source; same data the public sees |
| M13 (Public Verifier App) | Shared Merkle-proof API; common verifier library candidate |
| M15 (IAB Dashboard) | Sibling with elevated IAB access; M34 is observer-tier (read-only, no IAB role) |
| M21 (RLA Audit Tool) | Observer verification sampling feeds the RLA audit observation stream |
| M29 (Mobile Citizen Verify) | Sibling citizen app with subset verification workflow; shared Rust FFI verifier |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part IX (Public Transparency) | FR-M34-005, FR-M34-006 — cryptographic verification |
| Part XIV.3 (Observer Accreditation) | FR-M34-001 through FR-M34-003, FR-M34-009 through FR-M34-011 |
| Part XV.3 (Localisation) | NFR-M34-004 — international observer language packs |
| Part XVI.4 (Public-Facing Mobile) | Distribution and security requirements; POST_AWARD_PROGRAMME §3 (IAB) |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA06-observer.md`. References to "MA06" in the source file correspond to "M34" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
