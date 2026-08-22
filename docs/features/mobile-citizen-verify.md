# Mobile citizen verify (M29)

Module M29 (`eel-mobile-citizen-verify`) owned by `field-mobile-svc`. Lets any Kenyan voter verify their registration status, polling station assignment, and candidate list from a Flutter app (Android 8+, iOS 14+) or a PWA, with a Rust FFI verifier for offline-capable cryptographic proof checking, an Isar + SQLCipher local cache, and a 24-hour post-launch target of at least 10% of registered voters verifying.

## Contents

1. What M29 does (vs adjacent modules)
2. Data plane (schema)
3. Inputs
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M29 does (vs adjacent modules)

- **M01 BVR Enrolment** and **M35 CVR** maintain the voter register. M29 is a read-only consumer: it verifies status but cannot change it.
- **M04 VPRS** is the Voter Portal Registration System that handles online registration amendments. M29 surfaces the deep link to M04 when a voter's record needs updating.
- **M23 Grievance Channels** is the escalation path when a voter finds an error they cannot fix via M04. M29 shows the grievance deep link.
- **M26 Anomaly + Fraud Detection** consumes verification-rate metrics from M29 as a Layer 2 signal: an unexpected surge or collapse in verification rates at a geographic cluster can indicate ballot-stuffing preparation or voter suppression.

M29 does not write to the voter register. It does not re-identify voters; the national ID hash is used as a lookup key, never stored alongside a verified profile. Verification results are ephemeral at rest (Valkey TTL 24 hours).

## 2. Data plane

Three aggregates (per `services/field-mobile-svc/crates/citizen-verify-api/src/models.rs` and the M29 spec):

| Aggregate | Purpose | Data class |
|---|---|---|
| **VerificationResult** | Ephemeral per-session result: session_id (random UUID7), status (REGISTERED/NOT_FOUND/DATA_MISMATCH/TRANSFER), polling_station_code, polling_station_name, ward, constituency, county, candidate_list_ref, proof_type (MERKLE/SIGNED_BLOB), proof_bytes, issued_at, ttl_s. | D |
| **VerificationBookmark** | Valkey KV: key = sha256(national_id_hash || device_id), value = {polling_station, candidate_list_hash, verified_at, ttl_seconds=86400}. Not persisted to PostgreSQL. | D |
| **CitizenReport** | Station-level issue reports from voters: report_id, station_code, issue_type (QUEUE_LENGTH/ACCESSIBILITY/STAFF_MISCONDUCT/INTIMIDATION/EQUIPMENT_FAULT/OTHER), description_hash, photos_count, submitted_at, routed_to_grievance_id. | C |

No voter-level data persists beyond the Valkey TTL. PostgreSQL stores only `CitizenReport` (class C; description hash only, no raw PII) and aggregate verification metrics.

## 3. Inputs

| Source | Trigger | Auth | Notes |
|---|---|---|---|
| Voter national ID + date of birth (on-device only) | Voter initiates lookup | Device-local; hash derived on device before any network call | National ID and DOB never leave the device in plaintext |
| BVR voter register (read path via M01/M35 API) | Per verification request | JWT PKCE (anonymous citizen, no sign-in) | Rate-limited 3/device/15 min; SHA-256(national_id) used as lookup key |
| Merkle proof (offline pack or real-time) | Per verification request | Offline: locally bundled pre-signed pack; online: signed blob from P05 | Offline pack updated at T-7 and T-1 OTA push |
| Station-level aggregate data (for verification-rate monitoring) | Every 10 min | Internal from Valkey aggregate | No voter PII in aggregate |
| KSL video guides | At first app launch + guide screen | Bundled in app OTA pack | All 11 Kenyan Sign Language videos bundled; no external CDN at runtime |

## 4. Pipeline

```
voter installs Flutter app (Android 8+ / iOS 14+ / PWA)
   -> app downloads OTA offline pack (T-7 + T-1): Merkle proof snapshot,
      polling station lookup table, candidate list (cosign-signed by IEBC)
   -> offline pack verified by Rust FFI libeel-verifier
      (Android JNI, iOS XCFramework, WASM for PWA)
      heap constraint: <=15 MB on KES-10k device profile
      Isar + SQLCipher local cache for station lookup

voter initiates verification
   -> device derives sha256(national_id || date_of_birth) on-device
   -> if online:
      POST /v1/citizen/verify with {id_hash}
      -> BVR read path returns VerificationResult + Merkle proof
      -> libeel-verifier verifies Merkle inclusion (offline-capable path)
      -> VerificationBookmark stored in Valkey (TTL=86400 s)
      -> result displayed: polling station + ward + constituency + county
         + candidate list link
   -> if offline:
      -> id_hash looked up in local Isar SQLCipher cache
         (cache populated from offline pack at T-7 and T-1)
      -> if found: VerificationResult constructed from cache + local proof
      -> if not found: "not in offline pack; try when connected" message
         with helpful guidance on network connectivity

if status = NOT_FOUND or DATA_MISMATCH:
   -> app shows M04 VPRS deep link for online registration amendment
   -> app shows M23 grievance deep link for assisted resolution
   -> advisory: electoral office contact (displayed, not auto-dialled)

if station has KSL guide not yet viewed:
   -> KSL video guide plays (bundled, no network required)

voter can submit CitizenReport
   -> issue type + optional description (hashed before transmission)
   -> optional photo(s): stripped of EXIF metadata on-device before transmission
   -> POST /v1/citizen/reports -> CitizenReport created
   -> auto-routed to M23 grievance system with station context

verification-rate monitoring
   -> every 10 min: aggregate successful verifications by ward/constituency/county
   -> if rate collapses or spikes unexpectedly: eel.citizen.verification_rate_anomaly.v1
      consumed by M26 Layer 2

events emitted via packages/eel-outbox-rs:
   eel.citizen.verification_complete.v1 (aggregate; no voter PII)
   eel.citizen.report_submitted.v1
   eel.citizen.offline_pack_updated.v1
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/citizen-verify.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/citizen/verify` | Anonymous citizen (PKCE, no sign-in) | Verification lookup by id_hash; 3/device/15 min |
| `GET /v1/citizen/bookmark` | Anonymous citizen | Retrieve cached VerificationBookmark from Valkey |
| `GET /v1/citizen/station/{code}` | Anonymous citizen | Polling station detail (address, accessibility info, directions) |
| `GET /v1/citizen/candidates/{list_ref}` | Anonymous citizen | Candidate list by ref; cosign-verified |
| `POST /v1/citizen/reports` | Anonymous citizen | Submit CitizenReport |
| `GET /v1/citizen/offline-pack` | Anonymous (CDN pre-signed URL) | Download latest offline pack (cosign-signed) |
| `GET /v1/citizen/offline-pack/manifest` | Anonymous | Offline pack manifest: version, hash, coverage stats |
| `GET /v1/citizen/metrics/verification-rate` | `ECC_ANALYST` (JWT+SpiceDB) | Verification-rate aggregate by geo level |

All endpoints rate-limited. No endpoint returns voter-level PII. The `verify` endpoint returns data keyed to the id_hash; it does not return the name or national ID from its own response.

## 6. Human surfaces

| Surface | Path | Persona |
|---|---|---|
| Flutter citizen app (Android/iOS) | `services/field-mobile-svc/mobile/citizen-verify-flutter/` | Citizens |
| PWA (Astro + WASM verifier) | `services/field-mobile-svc/astro/citizen-verify-pwa/` | Citizens (low-spec devices, no app store) |
| ECC Verification-Rate Monitor (Phoenix LiveView) | `services/field-mobile-svc/ex/ecc_citizen_web/` | ECC analysts |
| Station Detail Widget (embedded in apps) | Shared component in Flutter + PWA | Citizens |

Accessibility:
- KSL video guides bundled in app for all 11 Kenyan official languages; no external CDN required.
- Screen-reader labels on all interactive elements (Flutter Semantics, WCAG 2.2 AA for PWA).
- High-contrast mode toggle.
- USSD fallback (`*400#` short-code for feature phones) via M16 USSD Integration; M29 generates the same polling-station answer for that channel.
- Designed for KES-10k device profile: heap <=15 MB for the verifier, APK download <=20 MB.

## 7. Integrity + anti-tamper

- **Rust FFI libeel-verifier**: Merkle proof verification runs entirely on-device in Rust (libeel-verifier.so for Android JNI, libeel-verifier.xcframework for iOS, WASM for PWA). Even if the network response is tampered with, the proof fails verification locally and the app shows an error.
- **Cosign-signed offline pack**: OTA packs are cosign-signed by IEBC at T-7 and T-1. The verifier refuses to load a pack whose cosign signature fails against the embedded IEBC public key.
- **SQLCipher + Isar encryption**: local Isar cache is encrypted via SQLCipher (key derived from device keystore, not user-visible). An attacker with physical device access cannot read the cached voter lookup table without the device keystore.
- **EXIF strip on citizen photos**: CitizenReport photos are stripped of EXIF metadata on-device before transmission, removing GPS coordinates and device identifiers.
- **Valkey TTL**: VerificationBookmarks expire after 24 hours. No persistent voter profile is built in the system.
- **Rate limiting**: 3 verification requests per device per 15-minute window prevents brute-force enumeration of national IDs.
- **Aggregate-only monitoring**: verification-rate events carry only ward/constituency/county aggregates and timestamps; no voter identifiers.

## 8. Kenya statute alignment

- **Elections Act s.6A** (public access to the register): M29 is the primary citizen-facing implementation of the voter's right to inspect their own registration status without attending an IEBC office.
- **DPA 2019 s.26** (data minimisation): national ID and DOB are hashed on-device; only the hash traverses the network. Verification results are ephemeral (Valkey TTL 24 h, no PostgreSQL persistence of the result).
- **DPA 2019 s.29** (right of access): M29 gives citizens direct read access to the data IEBC holds about their registration. The M04 VPRS deep link gives them the amendment path.
- **Kenya ICT Policy 2020** (digital inclusion): offline-capable design, KES-10k device support, PWA for low-end devices, and USSD fallback (M16) together meet the policy's requirement that digital government services remain accessible across the digital-divide.

## 9. Honest caveats + external prerequisites

- **Offline coverage**: the T-7 offline pack covers approximately 90% of registered voters based on baseline register size. The remaining 10% (late registrations, transfer approvals in the 7-day window) are online-only lookups. Citizens in this group who are offline on election day will see "not in offline pack; connect to verify" rather than a definitive status.
- **KES-10k device heap constraint**: the <=15 MB heap limit is validated against a Samsung Galaxy A03s (2 GB RAM, Android 11) profile. Devices with less RAM may see performance degradation. The Flutter app's memory profiler results for that profile are published in `services/field-mobile-svc/mobile/citizen-verify-flutter/docs/PERF_PROFILE.md`.
- **App store distribution**: Google Play and Apple App Store accounts must be provisioned by IEBC. Timeline risk: App Store review can take 3-7 days. Plan submission no later than T-14 to allow for review cycles.
- **10% verification target**: the target that at least 10% of registered voters verify within 24 hours of launch is a success metric, not a technical threshold. It requires awareness campaigns and USSD complementarity (M16); the app alone is unlikely to achieve it without coordinated outreach.
- **KSL video quality**: KSL guides are produced by the IEBC Comms team. If they are not delivered by T-14, the guides default to text-only accessible descriptions. The bundling mechanism is ready; the content is an IEBC deliverable.

## 10. Test coverage + operational runbooks

| Test type | Test-ID prefix | Key assertion |
|---|---|---|
| Unit | M29-UNIT- | Rust FFI proof verification; id_hash derivation; SQLCipher key derivation; rate-limit enforcement |
| Property | M29-PROP- | Merkle proof round-trip; offline pack cosign verify; EXIF strip completeness |
| Contract | M29-PACT- | BVR read API; M23 grievance handoff; M26 verification-rate event consumer |
| Integration | M29-INT- | Online verify flow end-to-end; offline pack load + local lookup; CitizenReport submission |
| E2E | M29-E2E- | Flutter app: Android (API 27) + iOS 14 + PWA on KES-10k profile; KSL guide load; M04 deep link |
| Load | M29-LOAD- | 1 million concurrent anonymous verifications; Valkey bookmark TTL under load |
| Chaos | M29-CHAOS- | BVR service offline: offline pack serves 90% of lookups; app shows graceful degradation message |

Performance target: verification response <=2 s on LTE; offline lookup <=500 ms on KES-10k device.

Runbooks:
- `docs/runbooks/field-mobile-svc-degradation.md`: BVR read path offline; offline pack fallback; CDN outage.
- `docs/runbooks/offline-pack-rotation.md`: T-7 and T-1 OTA pack generation, cosign signing, CDN push, and verification.
- `docs/runbooks/citizen-report-surge.md`: handling a surge in CitizenReport submissions that could saturate M23 grievance intake.

Related documents: full spec at `docs/plans/modules/mobile-citizen-verify.md`; M04 VPRS at `docs/features/vprs.md`; M16 USSD at `docs/features/ussd-integration.md`; M23 Grievance at `docs/features/grievance-channels.md`; edge-case matrix rows tagged EC-MOB-* in `docs/plans/08-edge-case-matrix.md`.
