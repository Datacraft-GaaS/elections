# field-mobile-svc capabilities and algorithms

Comprehensive capability catalogue for the field mobile service. Every capability lists
the algorithm, source reference (paper, standard, or statutory authority), and the crate
or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end request flows
3. Mobile BFF at 10M concurrent (M13, M29)
4. USSD, WhatsApp, and IVR flows (M13)
5. Binary integrity checks, clone detection, Play Integrity, and App Attest (M13, M29-M34)
6. Personnel check-in at 250k per 90 min with write-behind Kafka and Valkey (M33)
7. GPS, cell, and Wi-Fi spoof triangulation (M33, M34)
8. MDM-attested party-agent countersignature and duress alert (M30)
9. Offline verification and sync (M29, M31)
10. Observer mobile and audit trail (M34)
11. Operational excellence
12. Standards and interoperability
13. Validation harnesses
14. Data lifecycle and security
15. API surface
16. Summary

---

## 1. Modules and capabilities overview

| Module | Slug | Size | Primary crate | Notable risk |
|---|---|---|---|---|
| M13 | public-verifier-app | XL | crates/mobile-bff | WA quota; clone apps; 10M concurrent |
| M29 | mobile-citizen-verify | L | crates/mobile-bff | KES-10k Android device support; app store approval |
| M30 | mobile-party-agent | M | crates/mobile-bff | attestation false-fail; duress UX |
| M31 | mobile-poll-clerk | M | crates/mobile-bff | M06 companion API dependency |
| M32 | mobile-field-support | M | crates/support-api | remote-action safety |
| M33 | mobile-personnel-dispatch | XL | crates/dispatch-api | 250k concurrent; GPS spoof |
| M34 | mobile-observer | M | crates/mobile-bff | Auditor General mirror interface undefined |

T1 (M13 public verifier, M33 personnel dispatch 250k concurrent), T2 (M29-M32, M34).

Availability SLO: 99.99 % (M13), 99.9 % (M33). Personnel check-in throughput: 250,000 in 90-min window. M13: p99 < 500 ms at 10M concurrent (module inventory SLA).

---

## 2. End-to-end request flows

### 2.1 Voter verification (M13)

```
Citizen requests via Android/iOS/PWA/USSD/WhatsApp/IVR
  -> crates/mobile-bff/src/verify.rs: route by channel
  -> Valkey cache lookup (voter_id -> { registered, station_id, registration_hash })
  -> Cache hit: respond within 50 ms
  -> Cache miss: bvr-svc REST call -> populate Valkey (TTL 24 h) -> respond
  -> Proof request: bvi-svc Merkle inclusion proof -> verify locally -> return
  -> eel.field_mobile.verification_completed.v1 emitted (audit log)
```

### 2.2 Personnel check-in (M33)

```
Personnel app submits check-in (GPS coords, station_id, timestamp)
  -> crates/dispatch-api/src/checkin.rs: validate, assign attendance_id
  -> Write-behind: Kafka topic eel.field_mobile.checkin.v1 (at-least-once)
  -> Valkey HyperLogLog per station: approximate distinct personnel count (Flajolet et al. 2007)
  -> PostgreSQL batch write every 30 s via background worker
  -> 250k check-ins in 90 min at p99 < 500 ms
```

### 2.3 Party agent countersignature (M30)

```
Party agent app receives RTS tally event
  -> crates/mobile-bff/src/agent_countersign.rs
  -> Agent reviews tally; signs via MDM-attested HSM (Android Keystore + Play Integrity)
  -> AgentCountersignature { tally_hash, agent_id, station_id, timestamp, signature }
  -> eel.field_mobile.agent_countersignature.v1 emitted -> rts-svc ingests
```

---

## 3. Mobile BFF at 10M concurrent (M13, M29)

### 3.1 Architecture

Crate: `crates/mobile-bff/src/lib.rs`.

Rust axum 0.8 BFF aggregates three upstream services: bvr-svc (registration status), bvi-svc (inclusion proofs), transparency-svc (STH verification). Connection pool: 10,000 upstream connections per replica; 50 replicas minimum on election night = 500k upstream connections.

### 3.2 Verification cache

Valkey (Redis-compatible) cluster stores voter registration cache:

- Key: `vr:{sha256(voter_id)}` (SHA-256 of voter_id; plaintext voter_id never in cache key).
- Value: `{ registered: bool, station_id, registration_hash, roll_snapshot_id }`.
- TTL: 24 h from roll freeze; invalidated on `bvr.register.frozen` event.
- Cache hit rate target: > 95 % after 2 h warm-up on election morning.
- Valkey cluster: 3 primary shards, 3 replicas. Consistent hashing via rendezvous hash (Thaler and Ravishankar 1998 IEEE/ACM Transactions on Networking).

### 3.3 Autoscaling

HPA: CPU 60 %, memory 75 %; min 50, max 500 replicas (election night). PDB: minAvailable 50. CDN in front for static assets; BFF handles only dynamic verification queries.

---

## 4. USSD, WhatsApp, and IVR flows (M13)

### 4.1 USSD

Python worker: `py/field_mobile/ussd_server.py`. Africa's Talking USSD API (I08).

Menu tree:
```
*123#
  1. Voter registration status
     -> Enter ID number -> lookup -> respond
  2. Polling station
     -> Enter ID number -> lookup station -> respond
  3. Queue status
     -> Enter county code -> queue-svc API -> respond
```

Session state managed in Valkey (session_id -> menu_state, TTL 3 min per USSD session timeout). USSD session events logged in `eel_ussd_sessions` (V6 migration in field-mobile migrations).

### 4.2 WhatsApp Business

Python worker: `py/field_mobile/wa_bot.py`. Meta Cloud API webhook (I11).

Commands: `CHECK ID <national_id>`, `STATION ID <national_id>`, `GRIEVANCE <text>`. Rate limit: 100,000 messages per 24 h per WABA phone number. Overflow: 429 response with retry_after; user informed to try USSD. Message delivery tracked in `eel_wa_message_log`.

### 4.3 IVR

Python worker: `py/field_mobile/ivr_hook.py`. Safaricom SMPP / Africa's Talking IVR.

11-language voice menu matching M68 explainer language set. DTMF input: 1-9 for national_id digits. TTS responses via NLLB + VITS audio cache (same assets as M68; served from MinIO). IVR sessions logged in `eel_ivr_sessions`.

---

## 5. Binary integrity checks, clone detection, Play Integrity, and App Attest (M13, M29-M34)

### 5.1 Play Integrity (Android)

Crate: `crates/mobile-bff/src/attestation.rs`.

Every role app (M30-M34) and every sensitive M13 action submits a Google Play Integrity token. Server-side verification:
1. `POST https://playintegrity.googleapis.com/v1/{packageName}:decodeIntegrityToken` with token.
2. Verify `deviceIntegrity.deviceRecognitionVerdict` contains `MEETS_STRONG_INTEGRITY`.
3. Verify `appIntegrity.apkCertificateSha256Digest` matches registered APK signing certificate.
4. Verify `requestDetails.nonce` matches server-issued nonce (prevents replay).
5. Nonce stored in `eel_integrity_nonces` (UUID7, 5-min TTL, single-use).

### 5.2 App Attest (iOS)

Server-side verification of Apple App Attest assertion:
1. Verify assertion `clientDataHash` = SHA-256(server_challenge || client_data).
2. Verify attestation statement signature against Apple App Attest root CA.
3. Verify `aaguid` matches Apple App Attest AAGUID (`appattestdevelop` in dev, `appattest` in prod).
4. Counter: verify assertion counter > previously stored counter (prevents replay).

### 5.3 Clone detection

Heuristics applied per device_id across verification requests:
- Same device_id submitting queries for > 500 distinct voter IDs per hour: flagged as clone/enumeration attempt.
- Device clock skew > 300 s from server time: flagged (GPS spoof indicator).
- App binary hash mismatch vs registered build_id: flagged as sideloaded or modified APK.
- Flagged devices quarantined in `eel_device_quarantine` for 24 h; events logged.

| Algorithm | Reference |
|---|---|
| Play Integrity API | Google Play Integrity API Docs 2023 |
| Apple App Attest | Apple Developer Documentation; WebAuthn FIDO2 attestation flow |
| SHA-256 nonce | FIPS 180-4, NIST 2015 |

---

## 6. Personnel check-in at 250k per 90 min with write-behind Kafka and Valkey (M33)

### 6.1 Write-behind architecture

Crate: `crates/dispatch-api/src/checkin.rs`.

High-throughput check-in path (250k in 90 min = ~2,800 check-ins/s peak):

1. Validate check-in payload (station_id, personnel_id, timestamp, gps_coords, play_integrity_token).
2. Play Integrity verification (async; 50 ms SLA; cached for 15 min per device_id).
3. Publish to Kafka topic `eel.field_mobile.checkin.v1` (at-least-once; < 10 ms Kafka ack).
4. Respond 202 Accepted immediately (write-behind; PostgreSQL write deferred).
5. Valkey HyperLogLog increment for station-level distinct personnel count (p/64 error < 1.6 %).
6. Background worker: batch PostgreSQL INSERT every 30 s from Kafka consumer.

### 6.2 HyperLogLog for concurrent count approximation

Valkey `PFADD eel:checkin:{station_id}:{election_id} {personnel_id}`. Error bound epsilon = 1.04 / sqrt(m) where m = 16384 (Valkey default HLL precision). Error < 0.8 % for counts < 250k. Exact count reconciled from PostgreSQL at end of polling day.

| Algorithm | Reference |
|---|---|
| HyperLogLog cardinality estimation | Flajolet, Fusy, Gandouet, Meunier AOFA 2007 |
| Rendezvous (highest-random-weight) hashing | Thaler and Ravishankar 1998 IEEE/ACM Transactions on Networking |
| Kafka write-behind pattern | Kleppmann "Designing Data-Intensive Applications" 2017 |

---

## 7. GPS, cell, and Wi-Fi spoof triangulation (M33, M34)

### 7.1 Multi-source location verification

Crate: `crates/dispatch-api/src/location.rs`.

Personnel check-in and observer visit log include location_bundle:
- `{ gps_coords, cell_tower_ids[], wifi_bssid_hashes[], timestamp }`.

Server-side spoof detection:

1. GPS vs cell triangulation: compute expected location from cell tower CIDs via OpenCelliD database (Creative Commons BY license). Maximum expected error 500 m (urban) / 2 km (rural). If GPS reports location > 3x expected_error from cell estimate: spoof_flag = POSSIBLE.
2. GPS vs Wi-Fi: compare GPS against Google Maps Geolocation API (or fallback OpenWifi.su) for BSSID list. Divergence > 300 m: spoof_flag = POSSIBLE.
3. GPS velocity check: consecutive check-ins with distance / time_delta > 200 km/h (faster than plausible road travel): spoof_flag = LIKELY.
4. LIKELY spoof: event quarantined; ECC alerted. POSSIBLE spoof: flagged for manual review.

| Algorithm | Reference |
|---|---|
| Cell tower triangulation | Caffery and Stuber IEEE Communications Magazine 1998 |
| OpenCelliD database | Mozilla Location Services / OpenCelliD (CC BY) |
| Velocity spoof detection | Narain et al. "Inferring user routes and locations using zero-permission mobile sensors" IEEE S&P 2016 |

---

## 8. MDM-attested party-agent countersignature and duress alert (M30)

### 8.1 MDM attestation

All M30-M34 apps enrolled in MDM (Mobile Device Management). Device attestation record in `eel_mdm_devices`: device_id, mdm_enrollment_id, spire_svid, enrolled_at, party_id (M30 only).

### 8.2 Countersignature

Crate: `crates/mobile-bff/src/agent_countersign.rs`.

Party agent reviews Form34 tally presented via M30 app. Signs via Android Keystore (hardware-backed key). Signature: Ed25519 over `SHA-256({ tally_hash, agent_id, station_id, review_timestamp })`. Countersignature forwarded to rts-svc as `AgentCountersignature` entity. rts-svc verifies against registered agent public key in `eel_agent_keys`.

### 8.3 Duress silent alert

If agent enters 6-digit duress PIN (configured at MDM enrollment; distinct from normal PIN) instead of normal PIN to approve tally:
1. Countersignature submitted normally (no external indication of duress to observer).
2. Silent alert: `eel.field_mobile.duress_alert.v1` emitted to ecc-svc (CRITICAL severity).
3. ECC ops contacted; security response initiated.

Duress PIN stored as PBKDF2-HMAC-SHA256 hash (iterations: 600,000 per NIST SP 800-63B 2023) in `eel_agent_duress_pins`. Normal PIN stored separately with same derivation.

| Algorithm | Reference |
|---|---|
| Ed25519 party-agent signing | Bernstein et al. IACR ePrint 2011 |
| PBKDF2-HMAC-SHA256 (600k iterations) | NIST SP 800-63B 2023; RFC 8018 |
| Android Keystore hardware-backed key | Android Keystore system documentation |
| Play Integrity MDM attestation | Google Play Integrity API; Android Enterprise |

---

## 9. Offline verification and sync (M29, M31)

### 9.1 Offline cache (M29)

Flutter app `apps/citizen-verify/` downloads encrypted voter register cache shard on Wi-Fi before election day. Cache format: sorted list of `{ voter_id_hash, station_id, registration_hash }` (no plaintext voter_id; SHA-256 hash used for lookup). Lookup: binary search O(log n) on sorted hash list.

Roll snapshot signed by bvr-svc HSM; signature verified in-app before cache accepted. Cache TTL: 24 h from roll freeze. Stale cache renders stale label in UI; full re-download prompted.

### 9.2 Poll clerk offline mode (M31)

Flutter app `apps/poll-clerk/` caches station-specific voter subset (voters registered at that station only; provided as encrypted shard from bvr-svc). Offline incident reports queued locally; sync on connectivity recovery. End-of-day (EOD) report submitted via M31 app: `{ station_id, clerk_id, voters_served_count, incidents JSONB, eodr_timestamp }`. EOD report HSM-signed by clerk device (Android Keystore).

---

## 10. Observer mobile and audit trail (M34)

### 10.1 Observer visit log

Crate: `crates/mobile-bff/src/observer.rs`.

MDM-enrolled observer app (apps/observer/) records station visits:
`ObserverVisitLog { visit_id (UUID7), observer_id, station_id, arrived_at, departed_at, observation_notes_hash, location_verified }`.

Notes: hash only stored server-side (SHA-256 of AES-256-GCM encrypted notes); plaintext retained on device and submitted to governance-svc M21 RLA audit tool on demand.

### 10.2 Verification cross-check

Observer app can independently re-verify Form34 tally inclusion proof (same WASM verifier as M11 portal-svc). Verification result logged in `eel_observer_verifications`: observer_id, station_id, leaf_hash, verified_at, outcome.

### 10.3 Auditor General mirror

AG mirror interface not yet defined (noted as risk in module inventory). Observer data exported in JSON format from `GET /v1/observer/visits` (SPIRE SVID required) pending AG interface specification.

---

## 11. Operational excellence

| Capability | Mechanism |
|---|---|
| 10M concurrent M13 | HPA min 50, max 500 replicas; Valkey cache hit > 95 %; CDN for static assets |
| Verification cache warm-up | Pre-warm on bvr.register.frozen event; cache heat tracker in Prometheus |
| USSD channel saturation | Queue depth alert; overflow to SMS fallback |
| WhatsApp quota | 100k msg/24h per WABA; quota monitor with daily reset alert; USSD fallback |
| 250k check-ins (M33) | Write-behind Kafka; Valkey HLL; batch PostgreSQL; HPA min 10, max 100 |
| GPS spoof alert | LIKELY spoof: immediate ECC CRITICAL alert; POSSIBLE: manual review queue |
| Duress alert | CRITICAL severity to ECC within 5 s of detection |
| Offline sync recovery | App retries on reconnect with exponential backoff (max 5 min) |
| k6 load test | `loadtest/verify.js` targets 10k VUs / 600 s; `personnel-checkin.js` 5k VUs / 90 min |

---

## 12. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| OWASP MASVS-L2+R | Mobile app security: data storage, cryptography, network, platform, resilience (M13, M30-M34) | apps/, crates/mobile-bff |
| NIST SP 800-63B 2023 | Duress PIN derivation (PBKDF2, 600k iterations); authentication assurance | crates/mobile-bff/src/agent_countersign.rs |
| Android Keystore | Hardware-backed key storage for MDM-enrolled apps | apps/ (Flutter secure_storage + key_attestation) |
| Apple App Attest | iOS hardware attestation for M13 on iOS | crates/mobile-bff/src/attestation.rs |
| RFC 6962 | Inclusion proof format for in-app tally verification (M34) | apps/observer/ (WASM verifier) |
| CloudEvents 1.0 | All consumed and emitted events | crates/mobile-bff, crates/dispatch-api |
| OpenAPI 3.1 | API contract at packages/openapi-contracts/field-mobile/ | crates/mobile-bff |

---

## 13. Validation harnesses

| Harness | Scope | Command |
|---|---|---|
| Verification cache correctness | Cache hit, miss, invalidation | `cargo test verify::cache --lib` |
| Play Integrity mock verification | Token validation, nonce replay | `cargo test attestation::play_integrity --lib` |
| App Attest mock verification | Assertion counter, AAGUID | `cargo test attestation::app_attest --lib` |
| Clone detection tests | Enumeration threshold, clock skew | `cargo test clone_detection:: --lib` |
| Duress PIN detection | PBKDF2 hash comparison | `cargo test agent_countersign::duress --lib` |
| GPS spoof detection tests | Velocity, cell divergence | `cargo test location::spoof --lib` |
| HyperLogLog accuracy | Error bound at 250k cardinality | `cargo test checkin::hll_accuracy --lib` |
| Flutter app tests | citizen-verify unit tests | `cd apps/citizen-verify && flutter test` |
| k6 M13 load (10M concurrent) | p99 < 500 ms | `cd loadtest && k6 run verify.js --vus 10000 --duration 600s` |
| k6 M33 dispatch peak | 250k check-ins / 90 min | `cd loadtest && k6 run personnel-checkin.js` |
| Integration tests | Full pipeline with mock upstreams | `cargo nextest run` |

---

## 14. Data lifecycle and security

### 14.1 Encryption

Offline voter cache: AES-256-GCM, key derived from roll_snapshot_id + device_id (prevents cross-device sharing). Observer notes: AES-256-GCM on device; only SHA-256 hash stored server-side. Personnel GPS coords: stored as PostGIS POINT; access restricted to ECC ops and audit roles.

### 14.2 Audit

`eel_verification_logs` (V1 migration): append-only, voter_id_hash only (no plaintext voter_id). `eel_personnel_attendance` (V1): append-only. `eel_agent_countersignatures`: append-only; HSM signature on each row. Duress alerts: `eel_duress_events`, permanent retention.

### 14.3 Retention

- `eel_verification_logs`: 7 years post-election (Elections Act §39; right to audit).
- `eel_personnel_attendance`: 3 years (payroll audit; M55 dependency).
- `eel_grievance_records`: 7 years.
- `eel_ussd_sessions` (V6): 90 days (operational).
- `eel_device_quarantine`: 90 days.
- `eel_duress_events`: permanent.
- `eel_observer_verifications`: permanent.

### 14.4 USSD and WhatsApp privacy

USSD session identifiers are telco-assigned MSISDNs. Hashed with SHA-256 before storage: `eel_ussd_sessions.msisdn_hash`. Plaintext MSISDN not stored. Same pattern for WhatsApp sender_id.

---

## 15. API surface

### 15.1 REST (`crates/mobile-bff`, axum 0.8, port 8080)

- `GET /v1/verify/{voter_id}`: voter registration status (Valkey-backed; p99 < 500 ms).
- `POST /v1/verify`: batch verification with offline fallback.
- `GET /v1/verify/proof/{leaf_hash}`: Merkle inclusion proof from bvi-svc.
- `GET /v1/results/{station_id}`: polling station results (read-only after lock).
- `GET /v1/observer/visits`: observer visit log (SPIRE SVID required).
- `POST /v1/observer/visits`: submit observer visit log entry.
- `GET /v1/grievances`: list grievances filed by this device.
- `POST /v1/grievances`: file field grievance.
- `GET /healthz` + `/readyz` + `/openapi.json`.

### 15.2 REST (`crates/dispatch-api`, port 8081)

- `POST /v1/personnel/check-in`: dispatch personnel check-in (write-behind Kafka).
- `GET /v1/personnel/assignments/{personnel_id}`: dispatch assignments.
- `GET /v1/personnel/count/{station_id}`: HLL approximate distinct personnel count.

### 15.3 REST (`crates/support-api`, port 8082)

- `POST /v1/support/tickets`: field support ticket submission (M32).
- `GET /v1/support/tickets/{id}`: ticket status.

### 15.4 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.bvr.register_frozen.v1` (cache invalidation + offline cache rebuild)
- `eel.rts.tally_updated.v1` (M30 party agent tally review)
- `eel.transparency.sth_published.v1` (STH for proof verification)

Emitted (6 migrations V1-V6 in field-mobile-svc/migrations):
- `eel.field_mobile.verification_completed.v1`
- `eel.field_mobile.checkin.v1` (M33, write-behind)
- `eel.field_mobile.agent_countersignature.v1`
- `eel.field_mobile.grievance_filed.v1`
- `eel.field_mobile.duress_alert.v1`
- `eel.field_mobile.observer_visit.v1`

---

## 16. Summary

Total footprint: 3 Rust crates (mobile-bff, dispatch-api, support-api) + 6 Flutter apps (citizen-verify, party-agent, poll-clerk, field-support, personnel, observer) + 3 Python workers (ussd_server, wa_bot, ivr_hook) + 6 PostgreSQL migrations (V1-V6). Covers M13 (public verifier at 10M concurrent p99 < 500 ms with Valkey write-through cache and USSD/WhatsApp/IVR channels), M29 (encrypted offline cache with in-app binary search), M30 (MDM-attested party-agent countersignature with duress silent alert via PBKDF2-distinguishable PIN), M31 (poll-clerk offline with EOD report HSM signing), M32 (field support ticket management), M33 (250k check-ins per 90 min via write-behind Kafka + HyperLogLog + GPS spoof triangulation), and M34 (observer visit log with WASM inclusion proof verification). Security baseline: OWASP MASVS-L2+R across all apps. Key production inputs remaining: WhatsApp Business API WABA approval; Africa's Talking USSD short code; Apple App Attest production AAGUID registration; Google Play Integrity API project credentials; OpenCelliD database licence.
