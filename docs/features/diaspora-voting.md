# Diaspora voting (M24)

Module M24 (`diaspora-voting`) owned by `biometric-edge-svc`. Extends the biometric verification pipeline to Kenyan embassies and high commissions abroad. An `EmbassyStation` Flutter app runs biometric matching offline; the diaspora roll is frozen by Chair HSM before dispatch; Form34A variants carry dual timestamps (local + EAT); physical ballot custody travels via `PouchCustody` RFID chain; delayed result batches are transmitted AES-256-encrypted over mTLS back to Nairobi.

## Contents

1. What diaspora voting does (vs related modules)
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

## 1. What diaspora voting does (vs related modules)

- **M01/M02 BVR** = domestic biometric enrolment; M24 reads the diaspora-eligible subset but does not write to the BVR aggregate.
- **M24** = end-to-end management of overseas polling: roll freeze, station configuration, biometric offline match, Form34A variant with dual-clock timestamps, physical ballot pouch custody chain, and encrypted result batch transmission.
- **M09 Results tabulation** = receives the encrypted result batch from M24 after transmission and includes diaspora totals in the certified result.

M24 cannot extend the diaspora roll after freeze or accept new biometric enrolments. Any voter not on the frozen roll is turned away; no override exists without Chair HSM + IAB commissioner cosignature.

## 2. Data plane

Six aggregates under `services/biometric-edge-svc/crates/diaspora-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **DiasporaRoll** | Frozen subset of BVR for overseas voters: roll_hash, freeze_ts, chair_sig (HSM), station_assignments. | A |
| **EmbassyStation** | Overseas polling station config: station_id, country, embassy_ref, roll_slice_hash, kit_serial, status. | B |
| **DiasporaBiometricEvent** | Offline match result: voter_id_hash, station_id, match_score, outcome (VERIFIED/REJECTED/FALLBACK), ts_local, ts_eat, sig (kit Ed25519). | A |
| **Form34ADiaspora** | Tallied result for a station: station_id, candidate_tallies (JSON), ts_local, ts_eat, presiding_sig, agent_sigs (array), pouch_id. | A |
| **PouchCustody** | RFID-tracked physical ballot pouch: pouch_id, seal_id, rfid_tag, events (custody chain), current_status. | A |
| **ResultBatch** | Encrypted delayed transmission: batch_id, station_ids, aes256_ciphertext, mTLS_session_ref, received_at, ack_sig. | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| BVR diaspora-eligible subset (M02) | Pre-election freeze | Signed binary export | Chair HSM signature | Roll not dispatched if signature fails; block on Chair |
| Embassy station kit (EmbassyStation Flutter app) | Polling day | Offline Isar+SQLCipher sync | Kit Ed25519 cert | No connectivity required; events queued locally |
| RFID seal scanner (pouch custody) | At each custody transfer | BLE/USB serial | Device cert | Buffer locally; flush on reconnect |
| Delayed result batch (satellite / embassy LAN) | Post-poll transmission | AES-256+mTLS | Kit cert + IEBC relay cert | Retry up to 7 days; batch marked PENDING until received |
| M09 tabulation service | Per batch received | Internal event | SVID mTLS | M09 waits for diaspora batches; configurable timeout per country |

## 4. Pipeline

```
pre-election phase:
  DiasporaRoll sliced from BVR (diaspora_eligible flag)
  -> Chair HSM signs roll_hash
  -> roll slices encrypted per embassy station (station KEK)
  -> station kits provisioned: roll slice + biometric templates loaded offline
  -> EmbassyStation created per kit; PouchCustody seals assigned

polling day (embassy, offline):
  voter presents at EmbassyStation Flutter app
  -> offline biometric match (on-device Isar+SQLCipher, no network required)
        match_score >= threshold (0.85 configurable):
          DiasporaBiometricEvent(VERIFIED, ts_local, ts_eat)
        match_score < threshold:
          DiasporaBiometricEvent(FALLBACK) -> paper fallback logged
        not on roll:
          DiasporaBiometricEvent(REJECTED, reason=NOT_ON_ROLL)
  -> ballot issued; pouch RFID custody tracked
  -> presiding officer closes station; Form34ADiaspora tallied
       dual timestamp: ts_local (embassy timezone) + ts_eat (Africa/Nairobi)
       presiding_sig + party agent sigs collected on-device
  -> pouch sealed; PouchCustody updated

transmission phase:
  ResultBatch assembled (AES-256, key derived from station KEK + IEBC relay cert)
  -> transmitted over mTLS (satellite or embassy LAN)
  -> M09 receives batch; decrypts; includes in tabulation
  -> eel.diaspora.batch_received.v1 emitted; P05 Merkle anchor within 60 s
```

## 5. API surface

Contract: `packages/openapi-contracts/biometric-edge/diaspora.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /diaspora/v1/roll/freeze` | IEBC Chair (JWT + HSM) | Freeze and sign diaspora roll |
| `GET /diaspora/v1/roll/{id}` | Embassy kit / IAB | Roll metadata and hash |
| `POST /diaspora/v1/stations` | IEBC officer (JWT + SpiceDB) | Register embassy station |
| `GET /diaspora/v1/stations/{id}/sync` | Embassy kit (device cert) | Download roll slice for offline use |
| `POST /diaspora/v1/batches` | IEBC relay (mTLS) | Submit encrypted result batch |
| `GET /diaspora/v1/batches/{id}` | M09 / IAB | Batch status; decrypted totals after verification |
| `GET /diaspora/v1/pouch/{id}/history` | IAB / AG | Full PouchCustody chain |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| EmbassyStation Flutter app (Android/iOS) | `services/biometric-edge-svc/flutter/embassy-station/` | Biometric verification; voter lookup; tally entry; Form34A signing; offline-first |
| Pre-election kit provisioning (Django) | `services/biometric-edge-svc/django/diaspora-admin/` | Roll freeze approval; kit provisioning; station configuration |
| Diaspora monitoring dashboard (Phoenix LiveView) | `services/biometric-edge-svc/ex/diaspora_web/` | Real-time: station status by country, biometric event counts, batch transmission progress, pouch custody |
| M09 batch intake (internal) | `services/tabulation-svc/` | Decrypts and ingests diaspora result batches |

Languages: EN + SW on the Flutter app and Django back-office. WCAG 2.2 AA on all web surfaces; the Flutter app follows Material You accessibility guidelines.

## 7. Integrity + anti-tamper

- **Chair HSM roll freeze** -- the roll hash is signed by the IEBC Chair HSM key before any kit is provisioned; a kit that receives an unsigned or mismatched roll slice refuses to start the polling session.
- **Dual-clock Form34A** -- ts_local and ts_eat are both recorded from the device clock and a reference NTP timestamp fetched at station open; any Form34A with ts_eat outside the statutory polling window is flagged but not rejected (presiding officer must annotate).
- **Offline biometric -- no exfiltration pathway** -- the roll slice and biometric templates are stored in SQLCipher encrypted storage on the kit; no API endpoint exists to extract templates; the kit is wiped and the KEK revoked after the result batch is transmitted and acknowledged.
- **PouchCustody RFID chain** -- every pouch transfer is signed by the transferring and receiving officers; the chain is synced to the central custody store via the transmission batch.
- **AES-256 + mTLS batch** -- result batch ciphertext uses AES-256-GCM with a key derived from the station KEK and the IEBC relay certificate; a compromised relay cert does not expose station KEKs.
- **Exactly-once batch ingestion** -- `eel.diaspora.batch_received.v1` uses idempotency key `batch_id`; a second receipt of the same batch returns the cached ack without double-counting.

## 8. Kenya statute alignment

- **Elections Act ss.47-49 (overseas voting)** -- M24 implements the statutory process: diaspora roll, embassy stations, Form34A variant, and result transmission.
- **Elections (Technology) Regulations 2017** -- biometric kit certification for embassy stations; Form34A dual-signature; signed batch transmission logged.
- **DPA 2019 s.26 (minimisation)** -- biometric templates stored only on the embassy kit during polling; wiped after batch acknowledgment; no long-term biometric retention overseas.
- **Vienna Convention on Consular Relations** -- embassy IT equipment is on diplomatic premises; physical security and export controls are embassy-managed; M24 cannot enforce these but documents the dependency.
- **Elections (General) Regulations 2012 r.53** -- Form34A signed by presiding officer and party agents; M24 captures digital signatures on-device and syncs with the physical form scan.

## 9. Honest caveats + external prerequisites

- Satellite bandwidth for result batch transmission varies by location; the 7-day retry window is a design concession to poor connectivity; M09 tabulation does not certify the result until all expected batches are received or the IEBC Board formally waives a specific station.
- Embassy kit physical security is the responsibility of the hosting mission; the software cannot compensate for physical compromise of the kit or its KEK.
- The AES-256 KEK derivation scheme requires that the IEBC relay certificate is not compromised; a compromised relay cert must be reported within 1 h and the affected station's batch retransmitted over an alternative channel.
- Local time zone handling for countries that observe daylight saving time requires manual confirmation of the UTC offset before polling day; the Flutter app reads the device clock and cannot correct for an incorrect device timezone setting.
- The diaspora roll freeze date must be confirmed with the BVR M02 team at least 30 days before polling day; late enrolments cannot be added after freeze.

## 10. Test coverage + operational runbooks

Coverage under `services/biometric-edge-svc/crates/diaspora-api/tests/` and the Flutter integration tests: unit tests for roll freeze signer, biometric threshold classifier, Form34A dual-clock serialiser, AES-256 batch encryptor, PouchCustody RFID event processor; property-based tests asserting Form34A tally totals sum to issued ballot count; contract tests (Pact) for M09 batch ingestion and M02 roll slice export; integration tests with SoftHSM, Isar+SQLCipher test instances, mTLS relay stub, and P05 tlog stub; e2e Flutter integration test covering offline verification, Form34A signing, batch assembly, and mTLS transmission; chaos test where mTLS relay is unavailable for 72 h (verifies retry queue).

Runbooks under `docs/runbooks/`:

- `diaspora-transmission-failure.md` (create): satellite down; embassy LAN failure; batch retry procedure; alternative transmission channel.
- `embassy-kit-compromise.md` (create): KEK revocation; batch retransmission; M09 notification.

Dashboards: `platform/observability/biometric-edge-svc/grafana-diaspora.json` (station status by country, batch transmission lag, biometric event rate, pouch custody gap count). SLO: 100% roll freeze before kit dispatch; batch received within 7 days for all stations.

Chaos scenarios under `chaos/scenarios/eel-diaspora-*.yaml`: embassy satellite link brownout during batch transmission; RFID pouch-custody gap injection; dual-clock skew > 30 s; HSM roll-freeze attempted after deadline.

Performance targets: biometric match < 3 s on embassy kit at p99; batch transmission completion < 24 h from station close for all embassies; Form34A-D public publish < 30 min of digital transmission.

Dual-timestamp display: all embassy kit UIs show both local time (station timezone) and EAT simultaneously; clerks must confirm EAT timestamp before finalising each Form34A-D record to prevent timezone-related ordering disputes.

Related: `docs/plans/modules/diaspora-voting.md`; `services/biometric-edge-svc/docs/CAPABILITIES.md`; edge-case matrix EC-DIA-* in `docs/plans/08-edge-case-matrix.md`.
