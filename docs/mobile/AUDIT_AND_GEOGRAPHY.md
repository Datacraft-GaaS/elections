# Audit + Geography — Universal 5-Axis Standard

Every action on every mobile app logs an `AuditEntry` to encrypted Isar with the 5 axes: **where / when / who / why / how**. Outbox replay handles offline.

## 5 Axes Reference

| Axis | Field | Source | Examples |
|---|---|---|---|
| **where** | `whereGps` | `LocationService.capture()` | `"-1.286,36.817,8.0"` (GPS); `"station:KE-NBI-001,polygon_valid"` (station-bound) |
| **when** | `whenUtc` | `DateTime.now().toUtc().toIso8601String()` | `2026-09-05T03:52:27.000Z` |
| **who** | `actorId` + `actorRole` | `Session` (OIDC token) | `("clerk-12345", "CLERK")`, `("observer-789", "OBSERVER_FIELD")` |
| **why** | `reason` | Justification / dispute / form | `"Voter record lookup for station KE-NBI-001, registration dispute"`, `"Fingerprint quality below threshold, retry"`, `"Kiems-down paper backup"` |
| **how** | `howMethod` + `howProofHash` + `howSignature` | Method / SHA-256 / Ed25519 | `("fingerprint_fap60_plus_iris_antiris", "sha256:9a8b...", "ed25519:...")` |

## Architecture

```
Action → AuditService.log(actorId, role, action, target, reason, where, when, how)
  → AuditEntry.create() [UUID7 per entry]
  → _persist(entry)        [encrypted Isar SQLCipher, audit_entries table]
  → _outboxIfOffline(...)  [queued for replay when online]
  → On reconnect:
    - replay() with 5-axis verification
    - HTTP POST /v1/audit/replay
    - BFF returns 201 with audit_id for each accepted entry
```

## AuditEntry model (canonical)

```json
{
  "auditId": "0184d3b0-7a2e-7c9f-9b6d-...",
  "actorId": "clerk-12345",
  "actorRole": "CLERK",
  "action": "voter_lookup_attested",
  "target": "voter:KE-NBI-001:12345",
  "reason": "Voter record lookup per FR-010, DPO attestation captured",
  "whereGps": "-1.286,36.817,8.0",
  "whenUtc": "2026-09-05T03:52:27.000Z",
  "howMethod": "biometric_attestation_fingerprint",
  "howProofHash": "sha256:9a8b...",
  "howSignature": "ed25519:0xABC...",
  "appName": "poll-clerk",
  "isOffline": false,
  "sessionId": "sess:0184d3a0-..."
}
```

## Per-app audit surface

| App | Examples of audited actions |
|---|---|
| `poll-clerk` | check_in, lookup_attested (DPO biometric), queue_call_next, incident_create, eod_signed (dual biometric PO+DPO), kiosk_objection_filing |
| `observer` | visit_check_in, checklist_submit, verify_proof (Merkle FFI), countersign_signed, mission_pdf_export |
| `party-agent` | check_in, form_34a_verify, countersign_sealed, dispute_create, duress_silent_alert (exactly-once) |
| `citizen-verify` | otp_request, otp_verify, register_lookup, results_browse, incident_anonymous |
| `field-support` | shift_report_signed, ticket_create, warranty_claim, diagnostics_run |
| `personnel` | check_in, check_out, status_ping, incident_create |
| `rts-kit` | rfid_scan, at_command (AT+CMGS), sms_sent, transmit_signed |
| `bvr-kit` | enroll_dual_biometric (FAP60 + AntIris), match_1_1 |
| `bvi-kit` | verify_dual_biometric (AntIris + FAP60), match_1_1 |
| `diaspora-kit` | station_sync, pouch_verify, form34a_section_sign |
| `lms-offline` | module_download, assessment_submit (UUID7 idempotent) |
| `accred-scanner` | badge_scan, crl_lookup, status_display |
| `legacy-android` | deep_link_open, mdm_install_guidance |
| `kit-l1` | anomaly_detect (audit-grade) |

## Geography

`LocationService.capture()` is called on every action that has physical location. When a station polygon is available, point-in-polygon validation is recorded. When offline, GPS captured is timestamped and replayed on reconnect.

Apps marked as **geography-aware** (8 of 14):
- `observer` (GPS check-in polygon)
- `party-agent` (Haversine ≤ 200m)
- `personnel` (worker check-in)
- `citizen-verify` (station finder)
- `bvi-kit` (`gnss_guard.dart` + ISO metadata)
- `diaspora-kit` (station coordinates + `tz_display.dart`)
- `accred-scanner` (zone metadata)
- `bvr-kit` (now geo-aware via dual biometric enrollment)

## Dual Biometric (BVR + BVI)

As of M3 update, **BOTH** BVR enrollment and BVI verification require **fingerprint + iris** (2-factor). The `DualBiometricCapture` class (`services/biometric-edge-svc/apps/bvr-kit/lib/src/core/drivers/dual_biometric/`) captures FAP60 fingerprint, then AntIris iris, validates both, computes combined SHA-256 proof hash, signs with Ed25519, and emits a single audit entry with method `fingerprint_fap60_plus_iris_antiris`. Either biometric failing → entire enrollment/verification fails (no partial credit).

## Compliance

- **DPA 2019**: every PII access logged with actor + reason (FR-010/011/012)
- **Elections Act**: every vote-affecting action requires dual biometric (BVR + BVI)
- **Audit retention**: 5 years (encrypted Isar, replicated to BFF)

---

## Local Dedup + Operator-Controlled Upload

Biometric submissions are **never uploaded automatically**. The flow is offline-first with explicit operator action:

### Local Dedup (before submission)

`BiometricSubmissionOutbox.enqueue()` runs **device-local dedup** BEFORE writing to the encrypted outbox:

- **Fingerprint match**: SHA-256 of `fpTemplate` checked against local dedup index
- **Iris match**: SHA-256 of `irisTemplate` checked against index
- **Both match**: strong duplicate (entire same person + same biometric)
- **Either match**: weak duplicate (same biometric on this device)
- **Match found**: submission stored as `status=deduplicated` with `existing_submission_id` for audit; NOT queued for upload

This prevents the same device from creating multiple enrollments for the same person — even if the operator retakes the capture 10 times, only the first goes to the server.

### Operator-Controlled Upload

`BiometricSubmissionOutbox.uploadAll()` is called **only** when the operator taps the "Upload now" button in the `BiometricOutboxPanel` UI:

- Drains outbox in chronological order
- Each submission: `POST /v1/bvr/enrol` with both templates + audit metadata
- Server may return `status=duplicate` (already in central registry) → marked `deduplicated` with `server_duplicate:voter_id`
- Failed attempts retry up to 5 times
- Status: `pendingUpload → uploading → uploaded | failed | deduplicated`
- Cleanup: uploaded + deduplicated entries removed after 30 days

### Why this design

- **No silent uploads** — operator sees exactly what will leave the device
- **No accidental duplicate enrollments** — local dedup runs first
- **Offline-first** — works on a kit that's never been online for hours; queue persists across reboots (encrypted secure storage)
- **Audit trail** — every dedup rejection, every upload attempt, every server response logged with audit entry
- **Manual control** — kit operator decides when to upload, how much cellular data to spend, and can review each submission before send
