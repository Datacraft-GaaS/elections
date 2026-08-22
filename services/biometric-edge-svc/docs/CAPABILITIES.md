# biometric-edge-svc capabilities

Firmware and protocol hub for 65,000 new Android KIEMS kits and 14,100 legacy devices: four-tier hardware attestation, enrolment and identification apps, 7-day offline queue with crash-consistent journaling, diaspora embassy voting station protocol, and OTA manifest distribution.

**Tier**: T1-election-critical.
**SLO**: 99.99% availability (edge-api). Kit boot < 90s; enrolment throughput >= 12 enrolments/hour; scan + sign + transmit < 90s; 7-day offline queue; SMS fallback < 10 min.
**RTO**: 30 min (edge-api). **RPO**: 7 days of offline queue (kit-side); 15 min (server-side).

---

## Contents

1. Purpose and context
2. End-to-end pipelines
3. Domain aggregates and state stores
4. Algorithms and standards
5. Crypto and trust anchoring
6. Event surface
7. API surface
8. Integrations
9. Sharding, indexing, and scale
10. Degradation modes
11. Data lifecycle and retention
12. Observability
13. Testing pyramid
14. Follow-ups and known limitations

---

## 1. Purpose and context

### Modules owned

| Module | Name | RFP ref |
|--------|------|---------|
| M01 | bvr-device (kit-side biometric enrolment) | WP-3.8 |
| M05 | bvi-device (kit-side polling-day identification) | WP-3.8 |
| M24 | Diaspora voting station protocol | WP-8.1 |

### Kit apps

| App | Platform | Role |
|-----|----------|------|
| `apps/bvr-kit` | Flutter (Android) | Biometric enrolment: fingerprint, iris, face, NID scan, MRZ parse |
| `apps/bvi-kit` | Flutter (Android) | Polling-day voter identification: local 1:1 match against cached register extract |
| `apps/legacy-android` | Flutter shim (Android API 28+) | Feature parity on ~14,100 legacy devices |
| `diaspora` crate | Rust (server-side) | Embassy station protocol: DiasporaRoll, Form34AD, PouchCustody |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-12 months | Kit factory provisioning begins; Play Integrity attestation integration tested |
| T-6 months | bvr-kit deployed for biometric enrolment drives; OTA update pipeline live |
| T-3 months | bvi-kit deployed to all stations; offline queue stress tested |
| T-2 hours | Kit boots; attests; downloads config bundle; enters time-fence FSM (polling not open yet) |
| Election day T+0 | Time-fence FSM unlocks bvi-kit; identification session begins |
| Election day T+10 | Time-fence FSM closes identification session; kit enters post-poll state |
| T+7 days | Offline queue sync deadline; unsynced kits escalated to field support |
| M24 | Diaspora stations: T-30 days setup; T+0 Form34AD submission; T+3 days PouchCustody audit |

---

## 2. End-to-end pipelines

### 2.1 Kit boot and attestation

```
Kit powers on -> bvr-kit or bvi-kit Flutter app starts
  -> POST /v1/edge/kits/{serial}/attest (edge-api)
  -> kit-attest crate: parse attestation token by tier:
       Play Integrity (new Android 13+): verify verdict with Google Play Integrity API
       App Attest (iOS/future): verify via Apple App Attest API
       TPM quote (future hardware): verify TPM 2.0 quote over SPIRE-issued nonce
       legacy_software (API 28-32): accept with legacy_warning=TRUE, demote to MEETS_BASIC
  -> insert into kit_attestations (device_serial, attestation_kind, integrity_level, kit_tier,
     cert_thumbprint, legacy_warning, attested_at, expires_at)
  -> issue SPIRE SVID: spiffe://eel.local/kit/{serial}
  -> return attestation verdict + SVID to kit
  -> kit stores SVID in Android Keystore (hardware-backed on MEETS_STRONG tier)
```

### 2.2 OTA config and app update

```
Kit GET /v1/edge/kits/{serial}/config -> edge-api
  -> fetch kit_ota_manifests (latest active manifest for kit_tier)
  -> return {manifest_sig, download_url, version, enrolment_params, identification_params}
  -> kit verifies manifest_sig (Ed25519 over manifest JSON)
  -> if version > current: download APK from download_url, verify hash, install via PackageInstaller
```

### 2.3 Biometric enrolment (M01, bvr-kit)

```
Applicant presents at enrolment station
  -> bvr-kit: capture fingerprint (10 fingers), iris (both eyes), face (frontal photo), NID scan, MRZ parse
  -> quality gate: NFIQ 2.2 score >= 60 per finger; ISO 29794-6 iris quality; ISO 29794-5 face quality
  -> duress finger check: if duress finger presented -> suppress enrolment, alert silently to edge-api
  -> store enrolment event in Isar (offline encrypted local DB, SQLCipher AES-256)
  -> sync when online: POST /v1/edge/kits/{serial}/enrolments (batch, SPIRE SVID auth)
  -> edge-api: ingest event -> Kafka eel-edge-enrolment topic -> bvr-svc + abis-svc
```

### 2.4 Polling-day identification (M05, bvi-kit)

```
Voter presents at polling station
  -> bvi-kit: enter voter ID -> lookup in local cached register extract (Isar)
  -> biometric scan (fingerprint or iris)
  -> local 1:1 match against cached biometric template (ONNX model in-app, planned)
  -> if duress finger: silent alert to edge-api; continue normal flow to avoid coercion detection
  -> record identification event in Isar offline queue
  -> sync when online: POST /v1/edge/kits/{serial}/identifications (batch, SPIRE SVID auth)
  -> edge-api: ingest event -> bvi-svc identification endpoint
```

### 2.5 7-day offline queue with crash-consistent journal

```
Kit offline: events stored in Isar encrypted DB (AES-256 via SQLCipher)
  -> HLC timestamp assigned at capture time (Hybrid Logical Clock)
  -> on crash: Isar ACID guarantees partial-write recovery
  -> on reconnect: sync all queued events with deduplication (event_id UUID7)
  -> if > 7 days queued: alert to field support; events accepted but flagged for review
```

### 2.6 Diaspora embassy voting (M24)

```
Embassy station staff: GET /v1/diaspora/stations/{station_id}/roll (diaspora crate)
  -> return DiasporaRoll { voter_list, station_config, session_keys }
  -> voting conducted locally (manual or biometric)
  -> POST /v1/diaspora/stations/{station_id}/form34ad (Form34AD submission)
  -> PouchCustody event logged for chain-of-custody audit
  -> relay to rts-svc for tally inclusion (planned)
```

---

## 3. Domain aggregates and state stores

Tables from `migrations/V1__kit_attestations.sql`, `V2__kit_heartbeats.sql`, `V3__kit_ota_manifests.sql`, `V4__kit_telemetry_outbox.sql`.

| Table | Purpose |
|-------|---------|
| `kit_attestations` | Last-seen attestation per kit. Columns: `id`, `device_serial`, `attestation_kind` (play_integrity/app_attest/tpm_quote/legacy_software), `integrity_level` (MEETS_STRONG/MEETS_DEVICE/MEETS_BASIC), `kit_tier`, `cert_thumbprint`, `legacy_warning`, `attested_at`, `expires_at`, `revoked`, `revoked_at`, `revoked_reason`, `created_at`, `updated_at`. Unique index on `(device_serial, attested_at DESC)`. Partial index on `expires_at WHERE revoked=FALSE`. |
| `kit_heartbeats` | Daily telemetry. Columns: `id`, `device_serial`, `station_code`, `recorded_at`, `battery_pct`, `gps_lat`, `gps_lon`, `gnss_fix_quality`, `network_rssi_dbm`, `uptime_s`, `enrolled_count`. Partitioned by RANGE(`recorded_at`); 30-day rolling retention. |
| `kit_ota_manifests` | Signed OTA update manifests. Columns: `id`, `kit_tier` (bvr-kit/bvi-kit/legacy-android), `version`, `download_url`, `manifest_sig`, `released_at`, `active`. Unique on `(kit_tier, version)`. Partial index on `(kit_tier, released_at DESC) WHERE active=TRUE`. |
| `kit_telemetry_outbox` (V4) | Telemetry outbox for Kafka relay. Planned columns: `id`, `device_serial`, `payload` (JSONB), `created_at`, `published_at`. Verify in Wave 2. |
| Isar (kit-side) | Encrypted local database on each kit (isar + SQLCipher). Stores: pending enrolment events, pending identification events, cached register extract, OTA manifest cache. Not server-side; crash-consistent via Isar ACID transactions. |

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| Play Integrity attestation | `crates/kit-attest` | Google Play Integrity API verdict parsing: MEETS_STRONG / MEETS_DEVICE / MEETS_BASIC | Google, "Play Integrity API", developer.android.com 2022 | GlobalPlatform TEE (Android keystore backed by TEE) |
| App Attest attestation | `crates/kit-attest` | Apple App Attest API (planned for future iOS support) | Apple, "App Attest", developer.apple.com 2020 | FIDO Biometric Component spec |
| TPM quote attestation | `crates/kit-attest` | TPM 2.0 quote over SPIRE-issued nonce; verified with TPM endorsement key | TCG, "TPM Library Part 1: Architecture", Trusted Computing Group 2019 | TCG TPM 2.0 |
| Software-tier attestation (legacy) | `crates/kit-attest` | Android safetynet-style heuristics; accepted only with `legacy_warning=TRUE`; demoted to MEETS_BASIC | Google, "SafetyNet Attestation API" (deprecated); Android Integrity API migration guide 2023 | none; legacy only |
| Kit certificate CA (SPIRE SVID issuance) | `crates/kit-attest` | SPIRE-issued SVID: `spiffe://eel.local/kit/{serial}`; Ed25519 key pair in Android Keystore; cert signed by SPIRE intermediate CA | SPIFFE Community, "SPIFFE Runtime Environment (SPIRE)", spiffe.io 2021; Bernstein et al. Ed25519 EUROCRYPT 2011 | ISO/IEC 19790 |
| Isar offline queue | `apps/bvr-kit`, `apps/bvi-kit` | Isar 3.x (Dart NoSQL) + SQLCipher AES-256-CBC; Isar ACID transactions for crash-consistent append | Isar Database, isar.dev 2023; SQLCipher, zetetic.net 2023; NIST SP 800-38A AES-CBC | FIPS 197 (AES) |
| HLC (Hybrid Logical Clock) | `apps/bvr-kit`, `apps/bvi-kit` | HLC = max(wall_clock, prev_hlc) + logical_counter; stored as int64 ns | Kulkarni et al., "Logical Physical Clocks", OPODIS 2014 | none binding |
| Time-fence FSM | `apps/bvi-kit` | State machine: BOOT -> ATTESTED -> CONFIG_READY -> POLL_OPEN -> IDENTIFICATION -> POLL_CLOSED; transitions gated on HLC time vs station schedule | FSM pattern; Kenya Elections Act 2011 §37 (polling hours) | Kenya Elections Act 2011 §37 |
| Duress finger signalling | `apps/bvr-kit`, `apps/bvi-kit` | Specific finger (pre-configured per-kit in enrolment_params) suppresses/alerts without operator feedback | Concept from duress codes in physical security; no published standard | none binding |
| OTA manifest signature | `crates/edge-api` | Ed25519 over BLAKE3(manifest JSON); kit verifies before installing | Bernstein et al. Ed25519 EUROCRYPT 2011; O'Connor et al. BLAKE3 2020 | ISO/IEC 19790 |
| RF ballot lot scan (M39) | `apps/bvr-kit` (planned) | NFC/RFID scan of ballot lot seal; hash appended to chain-of-custody event | ISO/IEC 15693 (NFC vicinity RFID) | ISO/IEC 15693 |

---

## 5. Crypto and trust anchoring

### Four-tier attestation hierarchy

| Tier | Hardware | Attestation type | SVID integrity level |
|------|----------|-----------------|---------------------|
| 1 | New Android (Pixel 6+, API 33+) with StrongBox | Play Integrity MEETS_STRONG | Hardware-backed; SVID key in StrongBox |
| 2 | New Android (API 30+) with TEE only | Play Integrity MEETS_DEVICE | TEE-backed; SVID key in TEE |
| 3 | Future: hardware TPM (planned) | TPM 2.0 quote | TPM-backed |
| 4 | Legacy Android (API 28-32) | Software (legacy_software) | Software-only; `legacy_warning=TRUE`; flagged in audit |

SPIRE issues SVIDs to kits that pass attestation. The SVID is used to authenticate all subsequent API calls (bvr-kit enrolments to edge-api; bvi-kit identifications to bvi-svc). SVID private keys are stored in Android Keystore hardware-backed key store on Tier 1-2 kits.

OTA manifests are signed by the edge-api signing key (Ed25519). Kits verify the manifest signature before installing any update, preventing supply-chain attacks.

Kit attestations are persisted in `kit_attestations` with `cert_thumbprint` for revocation cross-reference. Revoking a kit (via bvi-svc `kit_certs.revoked=true`) triggers a CRL push to the kit on next heartbeat.

---

## 6. Event surface

### Emitted (via Kafka, edge-api relay)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.biometric-edge.enrolment.submitted.v1` | eel-edge-enrolment | `device_serial`, `voter_id`, `modalities` (fingerprint/iris/face), `ts_hlc`, `station_code` |
| `eel.biometric-edge.identification.submitted.v1` | eel-edge-identification | `device_serial`, `voter_id`, `station_code`, `outcome`, `ts_hlc` |
| `eel.biometric-edge.kit.attested.v1` | eel-edge-kit | `device_serial`, `attestation_kind`, `integrity_level`, `attested_at` |
| `eel.biometric-edge.diaspora.roll.submitted.v1` | eel-edge-diaspora | `station_id`, `country`, `voter_count`, `ts_hlc` |

All CloudEvents 1.0. Source: `spiffe://eel.local/svc/biometric-edge-svc`.

### Consumed

| Event type | Topic | Action |
|------------|-------|--------|
| `eel.bvr.register.frozen.v1` | eel-bvr-events | Trigger register extract push to kit cache |
| `eel.bvi.kit.cert.revoked.v1` | eel-bvi-kit-mgmt | Push updated CRL to kit on next heartbeat |

---

## 7. API surface

### REST (edge-api crate, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/edge/kits/{serial}/attest` | Kit submits attestation token; receives SPIRE SVID |
| GET | `/v1/edge/kits/{serial}/config` | Fetch app config (register extract, OTA manifest, enrolment/identification params) |
| POST | `/v1/edge/kits/{serial}/enrolments` | Kit submits batch enrolment events (SPIRE SVID auth) |
| POST | `/v1/edge/kits/{serial}/identifications` | Kit submits batch identification events (SPIRE SVID auth) |
| POST | `/v1/edge/kits/{serial}/sync` | Kit requests offline queue sync; supplies queued events from 7-day local cache |
| POST | `/v1/edge/kits/{serial}/heartbeat` | Kit telemetry: battery, GPS, RSSI, uptime (inserted into kit_heartbeats) |
| GET | `/v1/diaspora/stations/{station_id}/roll` | Fetch DiasporaRoll config for embassy station |
| POST | `/v1/diaspora/stations/{station_id}/form34ad` | Embassy staff submit Form34AD closure results |
| GET | `/healthz` | Liveness |
| GET | `/readyz` | Readiness (checks DB, SPIRE, Kafka) |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### Django kit management console (django/)

Web-based kit registry management: legacy device registration, OTA manifest promotion, attestation review, bulk revocation. Not externally reachable; internal admin only.

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| Google Play Integrity API | HTTP client in `crates/kit-attest` | REST; verify `POST https://playintegrity.googleapis.com/v1/{packageName}:decodeIntegrityToken` |
| SPIRE (SVID issuance) | `crates/kit-attest` workload API | SPIFFE Workload API gRPC |
| bvr-svc (enrolment relay) | Kafka producer in edge-api | Topic `eel-edge-enrolment` -> bvr-svc consumer |
| bvi-svc (identification relay) | Kafka producer in edge-api | Topic `eel-edge-identification` -> bvi-svc consumer |
| bvi-svc (CRL push) | Kafka consumer `eel-bvi-kit-mgmt` in edge-api | Consume revocation events; push CRL to kit on next heartbeat |
| MinIO (register extract storage) | S3-compatible client | `GET` extract bundles pre-staged by bvi-svc |
| abis-svc (biometric dedup) | Kafka consumer relay | enrolment event triggers abis-svc dedup job |

---

## 9. Sharding, indexing, and scale

- `kit_attestations`: unique index on `(device_serial, attested_at DESC)`; partial index on `expires_at WHERE revoked=FALSE`. 65,000 kits = 65,000 rows (plus 14,100 legacy) = ~80k rows; fits in PG cache trivially.
- `kit_heartbeats`: partitioned by day (RANGE on `recorded_at`); 30-day rolling retention. At 65k kits heartbeating every 5 min = 13k heartbeats/min; daily partition = 18.7M rows; manageable with daily partition drops.
- `kit_ota_manifests`: at most 3 tiers x dozens of versions = < 100 rows; no sharding needed.
- Scale target: 65,000 concurrent kit attestations at poll-open (T-2 hours); edge-api HPA scales to handle burst; kit attestation is idempotent (kit retries on timeout).
- Load test: `loadtest/` virtual kit simulator at 65,000 concurrent devices.
- Offline queue: 7 days of events on kit; typical station throughput 300-500 identifications/day; 7-day queue holds 3,500 events per kit; Isar easily handles this on-device.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | Attestation, config, enrolment, identification, heartbeat at full capacity | docs/runbooks/biometric-edge-svc.md |
| Degraded (Play Integrity API unavailable) | Google API down | Legacy-software tier used as fallback for new kits (with legacy_warning=TRUE); alert fired | docs/runbooks/biometric-edge-svc.md#attestation-fallback |
| Degraded (SPIRE unavailable) | SPIRE agent unreachable | Cached SVID used until expiry (typically 24h); new attestations blocked; alert fired | docs/runbooks/biometric-edge-svc.md#spire-unavailable |
| Offline (kit disconnected) | Network unavailable | bvr-kit/bvi-kit continue locally; events queued in Isar; sync on reconnect; 7-day window | docs/runbooks/biometric-edge-svc.md#offline-mode |
| Duress (duress finger presented) | Pre-configured finger scanned | Alert sent to edge-api; enrolment/identification continues normally (avoid operator coercion detection) | docs/runbooks/biometric-edge-svc.md#duress |
| Time-fence blocked | HLC before poll-open or after poll-close | bvi-kit refuses to start identification session; displays countdown; operator cannot override | docs/runbooks/biometric-edge-svc.md#time-fence |
| Legacy kit (API 28-32) | `legacy_warning=TRUE` | MEETS_BASIC only; additional field supervisor verification required; flagged in audit | docs/runbooks/biometric-edge-svc.md#legacy-kit |

---

## 11. Data lifecycle and retention

| Store | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `kit_attestations` | Class B: operational | 10 years |
| `kit_heartbeats` | Class C: ephemeral | 30 days (rolling partition drops) |
| `kit_ota_manifests` | Class B | 10 years (historical versions retained) |
| `kit_telemetry_outbox` | Class C | 30 days |
| Isar offline queue (kit-side) | Class A (election records) | 7 days on device; sync to server before expiry |
| Diaspora Form34AD | Class A | Permanent (election result record) |

Enrolment and identification events relayed from edge-api to bvr-svc and bvi-svc; authoritative retention is in those services (Class A, permanent). edge-svc does not retain event payload blobs after successful relay.

---

## 12. Observability

### Dashboards (Grafana, d/biometric-edge)

- Kit fleet health: attestation success rate by tier, SVID expiry countdown, heartbeat coverage.
- Enrolment throughput: enrolments/hour by county, quality gate pass rate.
- Offline queue depth: kits with queued events > 24h, > 72h, > 7d.
- OTA rollout progress: percentage of fleet on latest manifest version per tier.
- Duress events: count by station (masked; visible only to security team).

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `edge_attestation_failure_rate_high` | Attestation failures > 5% of attempts | warning |
| `edge_svid_expiry_imminent` | SVID expiry < 1 hour for > 100 kits | warning |
| `edge_offline_queue_stale` | Kit offline queue > 72 hours | warning |
| `edge_duress_event_detected` | Any duress finger event | critical (security team only) |
| `edge_legacy_kit_proportion_high` | > 20% of active kits on legacy tier | warning |

### Key metrics

- `eel_edge_kit_attestations_total` - Counter by attestation_kind and integrity_level.
- `eel_edge_enrolments_total` - Counter by county and quality outcome.
- `eel_edge_offline_queue_depth` - Gauge: kits with unsynced events > N hours.
- `eel_edge_ota_version_by_tier` - Gauge: fleet version distribution per tier.
- `eel_edge_heartbeat_age_seconds` - Gauge: time since last heartbeat per kit.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit (Rust) | `cargo nextest run --lib` | Attestation verdict parsing, HLC clock, time-fence FSM transitions, OTA manifest signature verify |
| Unit (Flutter) | `flutter test` (apps/bvr-kit, apps/bvi-kit) | Offline queue CRUD, quality gate thresholds, duress finger detection |
| Property | proptest | HLC monotonicity: any sequence of events produces non-decreasing HLC; attestation: any MEETS_STRONG verdict issues a valid SVID |
| Contract | OpenAPI conformance | REST response shapes; CloudEvent envelope schema |
| Integration (Rust) | `cargo nextest run --test '*' -- --test-threads=1` | PG + Kafka testcontainers; attestation pipeline; OTA manifest distribution; offline sync |
| Load | `loadtest/` virtual kit simulator | 65,000 concurrent kit attestations; 300 enrolments/station/hour; p99 < 90s per cycle |
| Chaos | planned (Wave 3) | Play Integrity API unavailable; SPIRE agent kill; Kafka partition loss; kit crash during enrolment |
| Security | planned (Wave 3) | Replay attack on attestation token; tampered OTA manifest; duress finger suppression timing |

---

## 14. Follow-ups and known limitations

- `kit_telemetry_outbox` (V4) migration filename confirmed on disk; content not fully read. Columns are planned. Verify in Wave 2.
- In-app 1:1 biometric matching (ONNX model in bvi-kit) is referenced in the architecture but no model weights or ONNX file were found in `apps/bvi-kit` on disk. This is the core identification algorithm; treat as a critical external input. Confirm delivery timeline with ABIS vendor.
- Isar + SQLCipher integration in Flutter apps is referenced in README and per-service specifics but no Dart pubspec or source files were read. Verify actual isar and sqlcipher_flutter_libs dependencies in `apps/bvr-kit/pubspec.yaml` and `apps/bvi-kit/pubspec.yaml` before production.
- RF ballot lot scan (M39) is listed as a planned capability for bvr-kit. No NFC/RFID code was found on disk. Marked deferred to Wave 2; depends on ballot lot hardware specification from inventory-svc (M39).
- App Attest (iOS) is planned for future diaspora use; no iOS target configured in Flutter apps. Deferred.
- Diaspora Form34AD relay to rts-svc for tally inclusion is marked "planned" in README. The diaspora crate exists but relay logic was not verified. Confirm before M24 is considered complete.
- Legacy Android (API 28-32) devices accept only software-tier attestation. This is a known security trade-off accepted by the Commission for the 14,100 legacy devices. Each legacy kit requires additional field supervisor signature on identification session start (implementation not verified on disk).

---

## Appendix A: Attestation tier comparison

| Property | Tier 1 (Play Integrity MEETS_STRONG) | Tier 2 (Play Integrity MEETS_DEVICE) | Tier 3 (TPM, planned) | Tier 4 (legacy software) |
|----------|--------------------------------------|---------------------------------------|----------------------|--------------------------|
| Key storage | Hardware-backed StrongBox (ARM TrustZone + SE) | TEE (ARM TrustZone) | TPM 2.0 chip | Android Keystore (software) |
| Extraction resistance | Very high | High | Very high | Low |
| SVID storage | StrongBox | TEE | TPM key hierarchy | Keystore (software) |
| Audit flag | None | None | None | `legacy_warning=TRUE` |
| Required actions | None additional | None additional | TPM certificate validation | Field supervisor countersignature |
| Vote session authorization | Full | Full | Full | Restricted (supervisor must authorize) |

The `integrity_level` enum in `kit_attestations` maps directly to this tier: `MEETS_STRONG` = Tier 1, `MEETS_DEVICE` = Tier 2, `MEETS_BASIC` = Tier 4 (software). Tier 3 (TPM) maps to `tpm_quote` attestation_kind with `MEETS_STRONG` integrity_level.

---

## Appendix B: 7-day offline queue design

The offline queue is the primary resilience mechanism for kits in areas with unreliable connectivity. Design constraints:

- **Durability**: Events in the Isar database survive app crashes and device restarts. Isar uses memory-mapped files with write-ahead log semantics on Android.
- **Capacity**: At 500 identifications/day per station, a 7-day queue holds 3,500 events. Each event is approximately 300 bytes (event_id 16B + voter_id 16B + outcome 1B + modality 1B + ts_hlc 8B + kit_sig 64B + envelope ~200B). Total queue: approximately 1 MB per kit per 7 days.
- **Deduplication**: Each event has a UUID7 `event_id`. On sync, the server's UNIQUE constraint on `event_id` rejects duplicates silently (200 OK returned to kit to prevent unnecessary re-sends).
- **Ordering**: HLC timestamps preserve causal ordering across sync batches, even when batches arrive out of wall-clock order.
- **Sync protocol**: On reconnect, kit sends all unsynchronized events in a single batch POST to `/v1/edge/kits/{serial}/sync`. The batch is processed transactionally (all-or-nothing Kafka produce + PG insert). If any event fails server-side validation, the kit receives a per-event status array and retries only the failed events.
- **Expiry policy**: Events older than 7 days are flagged (not deleted) on the kit and sent with a `queue_age_days` field. The server accepts them but flags them for field supervisor review.

---

## Appendix C: OTA update security model

The OTA (Over-The-Air) firmware update pipeline has the following security properties:

1. **Manifest authenticity**: Each `kit_ota_manifests` row contains a `manifest_sig` (Ed25519 over `BLAKE3(version || download_url || kit_tier || released_at)`). The Ed25519 signing key is held in the edge-api HSM (or software key in dev/staging).
2. **APK integrity**: `download_url` references an object in MinIO. The object name includes the SHA-256 hash of the APK (`ota/bvr-kit/1.0.1/<sha256>.apk`). Kit verifies the download matches the hash in the manifest before installation.
3. **Anti-downgrade**: Kit rejects manifests with `version <= current_version`. The `kit_tier` field must match the installed app tier.
4. **Rollout**: `kit_ota_manifests.active=true` controls which version is served. The Django admin UI allows staged rollout: set `active=true` for a new version while keeping the previous version active; kits receive the newer version and the older one becomes inactive after a grace period.
5. **Emergency rollback**: Setting `active=false` on a bad version and `active=true` on the previous version causes all kits to receive the rollback on next config fetch. This is the primary rollback mechanism; no flag day required.

---

## Appendix D: Diaspora voting protocol (M24)

The diaspora voting protocol supports Kenyan citizens voting at embassy stations in foreign countries. Key design elements:

- **DiasporaRoll**: A pre-approved voter roster for each embassy station, derived from the voter register and cross-checked with the diaspora registration list. Distributed as a signed bundle (same structure as kit config bundles) at T-30 days.
- **Form34AD**: Embassy staff submit results on Form34AD (diaspora analog of Form 34A). Submitted via POST to `/v1/diaspora/stations/{station_id}/form34ad` with SPIRE SVID authentication. The `diaspora` crate validates, signs, and relays to rts-svc for tally inclusion.
- **PouchCustody**: Physical ballot papers are sealed in a diplomatic pouch and couriered to Nairobi. The `PouchCustody` event records: pouch seal number, seal timestamp, diplomatic courier ID, expected arrival date, actual arrival date. This creates a physical chain-of-custody audit trail.
- **Timezone handling**: Embassy stations span multiple timezones (London UTC+1, Washington UTC-5, etc.). All timestamps stored in UTC; the `diaspora_stations.timezone` field enables local-time display in station UI.
- **Relay to rts-svc (planned)**: Form34AD relay to rts-svc tally engine is not yet implemented. Current state: Form34AD is accepted and persisted in the diaspora crate's local storage; tally inclusion requires Wave 2 integration work.


---

## Appendix E: kit-attest crate responsibilities

The `crates/kit-attest` crate is the security perimeter for all kit identity operations. It is the only crate with direct knowledge of Play Integrity API credentials and SPIRE agent connection parameters. Its responsibilities are strictly bounded: verify attestation token, issue SVID, record result. It does not process business events, voter data, or result forms. This minimal-surface design limits the blast radius of a compromise in this crate to kit identity only, not voter or result data.
