# M01 — BVR Device Client (Biometric Voter Registration)

**Runs on:** KIEMS kit (and legacy Android with feature-matrix degradation)
**Owner:** Datacraft
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Capture biographic + biometric data for every citizen registering to vote in Kenya. Operate reliably in polling-centre or field environments with intermittent or absent connectivity. Guarantee that no captured record is lost, duplicated, or tampered with between capture and back-end acceptance.

## 2. Primary Personas

- Enrolment Clerk (uses the kit continuously in a registration window)
- Supervisor (approves transfers, oversees the team)
- Presiding Officer (post-registration reuse)

## 3. Functional Requirements

- **FR-M01-001** Capture the 22 mandated biographic fields, including name(s), DOB, gender (MALE / FEMALE / **INTERSEX**), ID type + ID number + expiry (2nd-gen ID / Maisha card / Passport), mobile, email, education level, physical-challenge type, assistance-required flag, county / constituency / ward / polling centre.
- **FR-M01-002** Capture 10 fingerprints with FBI PIV IQS or STQC-certified sensor at FAP 50/60; extract ISO/IEC 19794-2 minutiae; encode with FBI-certified WSQ at 500 PPI + JPEG-2000 (ISO/IEC 15444-1:2019) at ≤ 15:1 average compression.
- **FR-M01-003** Capture iris (both eyes) with ISO/IEC 19794-6 encoding.
- **FR-M01-004** Capture ICAO 9303 / ISO/IEC 19794-5 facial image with automated quality gate (pose, lighting, expression).
- **FR-M01-005** Capture signature on-screen with stylus.
- **FR-M01-006** Scan ID document (2nd-gen ID / Maisha card / Passport) with auto-alignment, double-feed detection, MRZ parse, encrypted-PDF output.
- **FR-M01-007** On-device dedup against a local ABIS-lite cache (last N enrolments in the polling centre) to catch same-day duplicates before submission.
- **FR-M01-008** Client-side validation: names alpha-only; DOB ≥ 18 at registration date; MRZ validity; passport expiry ≥ 6 months from date of registration.
- **FR-M01-009** Offline-first: full enrolment flow works with no network. Records are stored in an encrypted local queue with per-record digital signature and submitted on reconnect.
- **FR-M01-010** Role-based access on the device with biometric login (fingerprint) for Enrolment Clerks and Supervisors.
- **FR-M01-011** Voter transfer workflow: authenticated by biometric of Supervisor + originating voter; produces a transfer event submitted to back-end.
- **FR-M01-012** Configurable per-station capacity (default 700, configurable). QR-code-based station lock prevents cross-station capture.
- **FR-M01-013** Configurable enrolment window: kit refuses new enrolments outside gazetted window per Elections Act §6A.
- **FR-M01-014** Audit trail on device: every action logged with (who, what, timestamp) locally; syncs to back-end.
- **FR-M01-015** Screen-lock after N minutes inactivity; biometric re-authentication required to resume.
- **FR-M01-016** Language toggle (English / Kiswahili at launch; extensible).
- **FR-M01-017** Accessibility: tactile-overlay-compatible input, audio-guided workflow, high-contrast mode.
- **FR-M01-018** Distress finger (designated at PO briefing) triggers covert alert to back-end while displaying normal auth.
- **FR-M01-019** Real-time kit-health telemetry to back-end via low-bandwidth pings: battery, connectivity, storage, temperature.
- **FR-M01-020** Automatic firmware and application updates via signed OTA (see M06 back-end) with rollback capability.

## 4. Non-Functional Requirements

- **NFR-M01-001** Full enrolment cycle ≤ 3 minutes per voter under expected conditions.
- **NFR-M01-002** Battery ≥ 8 hours continuous operation with attached power bank at least +8 hours.
- **NFR-M01-003** Operating in 0–45 °C, humidity 15–95%.
- **NFR-M01-004** Zero data loss on power failure (encrypted local store with journal).
- **NFR-M01-005** Sync back-log of up to 5,000 records without user-visible degradation.
- **NFR-M01-006** UI response ≤ 200 ms for common actions.
- **NFR-M01-007** Confidentiality: no biometric templates or PII ever leave the device unencrypted or unsigned.

## 5. Integrations

- **BVR Back-End (M02)** for record submission.
- **Local ABIS-lite** for same-day dedup.
- **MDM (O01)** for configuration and updates.
- **Geo-fence (P01)** for polygon check.
- **Kit certificate (P04 / PKI)** for signing and mTLS.
- **Kit inventory (P03)** for lifecycle events.
- **Distress channel** (out-of-band SMS/USSD path).

## 6. Data touched

- `Voter` (create / update)
- `BiometricTemplate` (create)
- `IDDocumentScan` (create)
- `Signature` (create)
- `AuditEvent` (create)
- `KitCustodyEvent` (create at hand-off)
- `KitAttestation` (send on boot)

## 7. UX Flow (registration)

1. Clerk logs into kit with fingerprint.
2. Clerk scans citizen's ID document.
3. MRZ parses; biographic fields pre-populated for confirmation.
4. Clerk captures missing fields; validates.
5. Kit captures 10 fingerprints, iris, facial, signature.
6. On-device ABIS-lite check against last N.
7. Clerk confirms; kit signs record; queues for submission.
8. On reconnect (or immediately if online), record posts to BVR back-end.
9. Confirmation printed / displayed with voter's registration reference.

## 8. Failure Modes & Recovery

| Failure | Detection | Response |
|---|---|---|
| Network loss | Sync monitor | Queue continues; UI shows "OFFLINE" banner; visible pending count |
| Local storage near-full | Disk monitor | UI warns clerk; back-end pull priority |
| Sensor failure | Self-test | Kit refuses to enrol; escalates to war-room |
| Same-day duplicate detected | Local ABIS-lite | Kit refuses submission; instructs clerk to verify |
| Distress finger detected | Local match | Silent alert to war-room; clerk unaware |
| Time drift detected | NTP + roughtime | Kit blocks operation until re-synced |
| Off-fence detected | GNSS check | Kit blocks and alerts |
| Firmware attestation mismatch | Boot | Kit refuses to run; quarantined centrally |
| Kit lost / stolen | Central revocation | Kit certificate revoked ≤ 60 s; kit becomes paperweight |

## 9. Standards Compliance

- ISO/IEC 19794-2 (fingerprint minutiae)
- ISO/IEC 19794-5 + ICAO 9303 (facial)
- ISO/IEC 19794-6 (iris)
- ANSI/NIST-ITL 1-2011 Part 1 + ITL 2-2008 XML
- FBI IAFIS IQS + EBTS IAFIS-DOC-01078-9.0
- WSQ (FBI IC-0110) + JPEG-2000 (ISO/IEC 15444-1:2019)
- WCAG 2.1 AA (screen accessibility)
- ISO/IEC 27001:2022 (device security controls)
- Kenya DPA 2019

## 10. Acceptance Criteria (module-specific)

| # | Criterion | Method |
|---|---|---|
| A-M01-01 | End-to-end enrolment flow completes in ≤ 3 min | Field trial |
| A-M01-02 | Offline for 4 hours, 100 records enrolled, all sync on reconnect | Bench |
| A-M01-03 | Fingerprint template quality NFIQ ≥ 60 for ≥ 95% of captures | Bench |
| A-M01-04 | Distress finger triggers alert without visible cue | Red-team exercise |
| A-M01-05 | Geo-fence violation blocks enrolment | Field trial |
| A-M01-06 | Firmware attestation deviation quarantines kit centrally | Chaos exercise |
| A-M01-07 | All 22 biographic fields present; INTERSEX gender selectable | Manual test |
| A-M01-08 | Accessibility: screen-reader traversal complete flow | NVDA / TalkBack |
| A-M01-09 | Legacy Android compatibility: kit runs on lowest-tier Commission-supplied device with documented feature degradation | Bench on real the Commission device |

## 11. Cross-References

- `M02-bvr-backend.md` — receiving end
- `M03-abis.md` — 1:N matching
- `P01-geofence.md` — geo-fence layer (implicit)
- `P04-iam-pki.md` — kit cert
- `../SECURITY.md` §3.12 — device-level security
- `../THREAT_MODEL.md` §4 — physical device threats
