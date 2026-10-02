# Mobile citizen verify (M29)

Module M29 (`mobile-citizen-verify`) owned by `field-mobile-svc`. A Flutter Android/iOS/PWA app for citizens to verify their voter registration status, locate their polling station, and verify official communications. Runs fully offline on a KES-10 000 device profile using an Isar+SQLCipher local store and a shared Rust FFI `libeel-verifier` for Merkle inclusion proof verification. Push and SMS fallback ensure delivery on low-connectivity devices. KSL video accessibility for Deaf users.

## Contents

1. What mobile citizen verify does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What mobile citizen verify does (vs related modules)

- **M02 BVR backend** = authoritative voter register; M29 provides read-only citizen self-service against a cached subset.
- **M29** = citizen-facing verification: registration status, polling station, and deep-link to VPRS. No data is written to any register by M29.
- **M30 Mobile party agent** = party-facing polling station tool; M29 is citizen-facing only and has no access to party or result data.

M29 never displays raw national ID numbers, biometric references, or any data class higher than D-class. Status checks use a hashed voter reference.

## 2. Data plane

Four local aggregates in the Flutter Isar+SQLCipher store (per `services/field-mobile-svc/flutter/citizen-verify/`):

| Aggregate | Purpose | Sync cadence |
|---|---|---|
| **VoterStatusCache** | Cached status: voter_ref_hash, station_id, status (REGISTERED/TRANSFERRED/DEREGISTERED), cached_at. | Daily push; on-demand pull |
| **StationLocator** | Polling station metadata: station_id, name, lat, lon, boundary_geojson, accessibility_features. | Pre-election bulk; delta push |
| **MerkleVerifyResult** | Local FFI verification result: leaf_hash, root_hash, proof_valid (bool), verified_at. | On-demand |
| **NotificationCache** | Push + SMS notification log: message_id, body, channel, received_at, read (bool). | Real-time push |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Voter status push (Firebase / Africa's Talking) | Daily + on status change | Push notification + JSON payload | Firebase project cert + JWT | SMS fallback if push not delivered within 5 min |
| Station locator bulk (pre-election) | Once pre-election | Signed JSON bundle | HTTPS + app cert pinning | Cached bundle serves offline; stale warning after 30 days |
| Merkle inclusion proof (P05 tlog) | On-demand | REST + JSON proof | HTTPS (public) | Offline: cached proof from last verification shown with staleness badge |
| VPRS deep-link target | On-demand | Deep link | App cert | VPRS unavailable: fallback to cached station info |

## 4. Pipeline

```
citizen opens citizen-verify app
  -> device checks VoterStatusCache (Isar+SQLCipher):
       if cached_at < 24 h: show cached status with freshness badge
       if stale: trigger pull from field-mobile-svc REST endpoint
  -> status pull:
       POST /citizen/v1/status with voter_ref_hash (SHA-256 of national_id)
       response: status, station_id, station_name
       VoterStatusCache updated; push subscription refreshed
  -> station locator:
       GPS location -> nearest stations from StationLocator cache
       map view (offline tiles bundled)
  -> Merkle verification (user-initiated):
       libeel-verifier FFI called with leaf_hash
       proof fetched from P05 tlog (or cached)
       MerkleVerifyResult stored; result displayed to user
  -> VPRS deep-link:
       station_id passed to VPRS app (if installed) or VPRS PWA
  -> push notification received:
       NotificationCache updated; badge shown; tappable to detail view
  -> SMS fallback:
       Africa's Talking MO/MT SMS via field-mobile-svc adapter
       SMS content parsed; NotificationCache updated
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/citizen.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /citizen/v1/status` | Citizen app (HTTPS, rate-limited 30/hr) | Status check by voter_ref_hash |
| `GET /citizen/v1/stations` | Citizen app (HTTPS) | Station list for a bounding box |
| `GET /citizen/v1/stations/{id}` | Citizen app | Station detail with accessibility features |
| `GET /citizen/v1/notifications/{voter_ref_hash}` | Citizen app | Notification history (last 30 days) |

No authentication beyond HTTPS rate-limiting; voter_ref_hash is a one-way hash; no raw ID accepted.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Flutter Android/iOS/PWA | `services/field-mobile-svc/flutter/citizen-verify/` | Registration status; station finder; Merkle verifier; push inbox |
| KSL video guides (embedded) | Within Flutter app | Video in Kenyan Sign Language covering registration check, station finder, polling day instructions |

Material You accessibility: minimum touch target 48dp; dynamic font scaling supported; high-contrast mode. Languages: EN + SW; KSL video for key flows. WCAG 2.2 AA on PWA surfaces.

## 7. Integrity + anti-tamper

- **Rust FFI `libeel-verifier`** -- the Merkle inclusion proof verifier is a shared Rust library (`packages/libeel-verifier/`) compiled to native code and linked into the Flutter app; it cannot be overridden by app updates without a new signed binary; the library verifies the P05 tlog root against a list of trusted witness public keys bundled in the app.
- **App cert pinning** -- the app pins the field-mobile-svc TLS certificate; a MITM presenting a different cert is rejected.
- **Voter ref hash** -- the API never receives the raw national ID; only SHA-256(national_id + app_secret) is transmitted; the server cannot reverse this to identify the voter.
- **SMS content integrity** -- SMS messages from the Commission sender ID are signed with an HMAC in the message body; the app verifies the HMAC before displaying the message; spoofed messages without the HMAC are shown with a `UNVERIFIED` badge.
- **KES-10k device profile** -- all app features tested on a 2 GB RAM / 32 GB storage Android Go device; app bundle size capped at 50 MB; offline tile bundle uses vector tiles (Mapbox Protobuf format) to minimise storage.

## 8. Kenya statute alignment

- **Elections Act s.44 (right to verify registration)** -- citizen self-check via voter_ref_hash satisfies the statutory right without exposing raw personal data.
- **DPA 2019 s.26 (minimisation)** -- only voter_ref_hash transmitted; no raw ID, name, or address collected by the app.
- **Communications Authority Digital Literacy guidelines** -- offline-first design and SMS fallback comply with the CA's rural connectivity guidance.
- **WCAG 2.2 AA (national accessibility policy)** -- PWA surfaces and Flutter app both meet AA criteria; KSL video satisfies CRPD Article 30 cultural participation for Deaf users.

## 9. Honest caveats + external prerequisites

- Push notification delivery depends on Firebase project cert and Google Play Services on the device; Africa's Talking SMS is the mandatory fallback but SMS delivery in rural areas is not guaranteed.
- The libeel-verifier library requires a network connection to fetch the inclusion proof unless the proof was cached during a previous online session; fully offline verification is limited to the cached proof with a staleness badge.
- VPRS deep-link requires the VPRS app to be installed or a PWA to be loaded; the app cannot guarantee VPRS availability.
- KSL video content must be produced and quality-reviewed by a certified Kenyan Sign Language interpreter before app release; this is a content production prerequisite.
- The SHA-256(national_id + app_secret) scheme requires the app_secret to be embedded in the app binary; this provides de-identification against the server but not against a reverse-engineering attacker; the app_secret is rotated each election cycle.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/flutter/citizen-verify/test/` and `services/field-mobile-svc/crates/citizen-api/tests/`: unit tests for voter_ref_hash generator, station locator, push notification parser, SMS HMAC verifier, libeel-verifier FFI binding; widget tests for all Flutter screens; integration tests with Africa's Talking SMS stub, Firebase push stub, P05 tlog stub; e2e Flutter integration test covering status check, station finder, Merkle verification, and push inbox; performance test on KES-10k device profile (2 GB RAM emulator).

Runbooks under `docs/runbooks/`:

- `citizen-verify-degradation.md` (create): push delivery failure; SMS gateway down; P05 tlog unreachable; VPRS deep-link broken.

Dashboards: `platform/observability/field-mobile-svc/grafana-citizen.json` (status check rate, push delivery rate, SMS fallback rate, Merkle verification rate). SLO: 99.5% status check success; push-or-SMS delivery within 10 min of status change.

Chaos scenarios under `chaos/scenarios/eel-citizen-verify-*.yaml`: CDN edge node failure (origin shield test); offline pack OTA push failure at T-1; libeel-verifier NDK crash on specific OEM (WASM fallback); push gateway brownout during declaration day.

Performance targets: CDN-served GET /verify p99 < 200 ms under 5 million concurrent users; offline Isar lookup < 100 ms on KES-10k device; libeel-verifier QR sig verify < 500 ms on device.

SLOs: 99.5% verification success during polling day; push-or-SMS delivery within 10 min of result availability; offline pack coverage >= 95% of registered voters at T-1.

Offline pack update protocol: T-7 OTA push covers the full voter register snapshot; T-1 OTA push covers only delta (new registrations and corrections since T-7); pack integrity verified by libeel-verifier on device before installation; corrupted pack triggers rollback to T-7 version.

KSL video accessibility: 11 KSL guide videos (registration lookup, station finding, results QR, grievance link) bundled in the app binary; no external CDN required at runtime; total video bundle <= 50 MB compressed to fit within app size budget for Google Play / App Store.

Verification rate monitoring: M26 consumes aggregated verification-rate data (no voter PII) as a Layer 2 anomaly signal; unexpected collapse (voter suppression pattern) or spike (stuffing preparation) at geographic cluster fires `eel.citizen.verification_rate_anomaly.v1`.

CitizenReport privacy: description text is truncated to 500 characters before hashing; photos have EXIF metadata stripped on-device before upload; neither the description nor the original photo is stored by the Commission; only the hash and issue type are persisted.

App size budget: Flutter app target <= 25 MB installed (excluding KSL video bundle which is OTA-downloaded at first launch); PWA core <= 2 MB cached; libeel-verifier NDK binary <= 1.5 MB per ABI.

Rate limiting: GET /v1/citizen/verify is rate-limited at 5 requests per device per 10 minutes to prevent enumeration of the voter register; the limit is relaxed on polling day to 20 per device per 10 minutes to accommodate voters who need to recheck their station assignment.

CitizenReport M23 routing: submitted reports auto-route to M23 with station_code, issue_type, and description_hash; the M23 grievance record links back to the report_id; the observer or citizen can track the grievance status via M23's reference number without re-entering data.

Related: `docs/plans/modules/mobile-citizen-verify.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
