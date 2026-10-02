# Mobile party agent (M30)

Module M30 (`mobile-party-agent`) owned by `field-mobile-svc`. A Flutter Android/iOS app for accredited party agents at polling stations. Supports GPS-verified check-in, QR-scan Form 34A countersignature, exactly-once dispute recording, duress silent alert, and chief agent rollup view across the party's assigned stations. HSM-anchored countersignatures feed M09 tabulation.

## Contents

1. What mobile party agent does (vs related modules)
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

## 1. What mobile party agent does (vs related modules)

- **M12 CRMS** = accredits party agents; M30 reads the agent credential from CRMS at check-in to verify entitlement.
- **M30** = election-day party agent tooling: check-in at station, countersign Form 34A, record disputes, and send duress alerts.
- **M09 Results tabulation** = receives HSM-anchored party agent countersignatures as part of the Form 34A certification chain.

M30 does not tally votes or modify the voter roll. It is a read-heavy, write-sparse app that must function in offline-first mode throughout polling day.

## 2. Data plane

Five local aggregates in the Flutter Isar+SQLCipher store:

| Aggregate | Purpose | Sync cadence |
|---|---|---|
| **AgentCheckIn** | GPS-verified arrival: agent_id, station_id, gps_lat, gps_lon, gps_accuracy_m, ts_hlc, agent_sig. | Immediate sync on connectivity |
| **Form34AScan** | QR-scanned Form 34A image and extracted totals: scan_id, station_id, form_image_hash, totals_json, countersig (HSM-anchored), ts_hlc. | Immediate sync |
| **DisputeRecord** | Exactly-once dispute event: dispute_id (UUID7), station_id, category, description, evidence_hashes (array), agent_sig, ts_hlc. | Immediate sync; duplicate dispute_id rejected |
| **DuressAlert** | Silent alert: alert_id, station_id, agent_id, ts_hlc. Content: none (minimised). | Immediate send via SMS fallback if no data |
| **StationRollup** | Chief agent aggregate: station_id, checkin_status, form34a_status, dispute_count, ts. | Push from field-mobile-svc on any station event |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| CRMS credential (M12) | At check-in | REST | JWT (agent device cert) | Check-in blocked if CRMS unreachable and no cached credential; agent retries |
| GPS (device) | At check-in and every 30 min | OS location API | N/A (device) | Low accuracy (<100 m) blocks check-in; fallback to station QR code check-in |
| Form 34A QR code | At station close | Camera scan | N/A | Manual entry fallback logged as MANUAL_ENTRY |
| Chief agent push | Real-time | Push + Kafka | JWT (chief agent device cert) | Queued offline; synced on connectivity |

## 4. Pipeline

```
agent check-in:
  agent opens M30 app; scans station QR code
  -> CRMS credential verified (or cached)
  -> GPS reading: accuracy must be < 100 m AND station geofence match
       (PostGIS point-in-polygon via field-mobile-svc REST)
  -> AgentCheckIn persisted; agent_sig applied
  -> eel.party.agent_checkin.v1 emitted

Form 34A countersignature:
  presiding officer presents Form 34A QR code
  -> agent scans; totals extracted and displayed for review
  -> agent confirms or rejects totals
  -> if confirmed: HSM-anchored countersig (field-mobile-svc HSM signs on behalf of agent,
                   agent approves via PIN; final sig includes agent public key)
  -> Form34AScan persisted; eel.party.form34a_countersig.v1 emitted
  -> M09 receives countersig event as part of Form 34A certification chain

dispute recording:
  agent taps "Record dispute" -> category selection -> description -> evidence photo
  -> DisputeRecord(dispute_id = UUID7) persisted
  -> exactly-once: duplicate dispute_id on sync returns cached ack
  -> eel.party.dispute.v1 emitted

duress alert:
  agent holds power button for 5 s (configurable)
  -> DuressAlert sent via: (1) app API (2) SMS to the Commission Security Coordinator
  -> alert logged; no visible UI change on device (silent)

chief agent rollup:
  chief agent sees live StationRollup for all party-assigned stations
  -> check-in count, Form 34A status, dispute count
  -> drill-down per station
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/party-agent.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /party/v1/checkins` | Agent device (cert) | Submit GPS check-in |
| `POST /party/v1/form34a` | Agent device (cert) | Submit Form 34A countersignature |
| `POST /party/v1/disputes` | Agent device (cert) | Record dispute (exactly-once) |
| `POST /party/v1/duress` | Agent device (cert) | Send duress alert |
| `GET /party/v1/stations/{agent_id}/rollup` | Chief agent (JWT) | Station rollup for all assigned stations |
| `GET /party/v1/disputes/{station_id}` | Chief agent / IAB (JWT) | Dispute list for a station |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Party agent Flutter app (Android/iOS) | `services/field-mobile-svc/flutter/party-agent/` | Check-in; Form 34A scan; dispute; duress |
| Chief agent rollup screen (within Flutter app) | Same app, chief_agent role | Aggregated station view |

Material You accessibility; EN + SW. WCAG 2.2 AA on all screens. Offline-first: all actions available without connectivity; synced on reconnect.

## 7. Integrity + anti-tamper

- **GPS geofence check-in** -- check-in is accepted only if the GPS reading places the agent within the station geofence (PostGIS polygon from M18 boundary-svc); spoof-resistant because the geofence polygon is not transmitted to the app before check-in.
- **HSM-anchored countersignature** -- the Form 34A countersig is computed by the field-mobile-svc HSM using the agent's registered public key; the agent's PIN authorises the signing operation but the private key never leaves the HSM; M09 verifies the sig against the CRMS-registered agent public key.
- **Exactly-once dispute** -- `DisputeRecord.dispute_id` is a UUID7 generated on the device at the moment the agent taps "Record dispute"; the server rejects any second event with the same dispute_id; network retries are idempotent.
- **Silent duress** -- the duress code path has no visible UI feedback to prevent coercion detection; the SMS fallback uses a one-time pre-loaded message that does not require connectivity.
- **Offline Isar+SQLCipher** -- all local data is encrypted at rest using SQLCipher with a key derived from the agent's device cert and PIN; a wiped device has no recoverable data.

## 8. Kenya statute alignment

- **Elections Act s.62 (party agent rights)** -- M30 implements the statutory right to be present and to countersign Form 34A; the countersig is part of the certification chain for M09.
- **Elections (General) Regulations 2012 r.68** -- Form 34A agent signature required; M30 digital countersig satisfies this when presented alongside the physical Form 34A.
- **DPA 2019 s.26 (minimisation)** -- duress alert contains no descriptive content; check-in GPS stored per-agent; no voter data accessible in M30.
- **Elections Act s.78 (dispute right)** -- dispute recording provides the statutory mechanism for agents to register objections; disputes are timestamped and exactly-once for auditability.

## 9. Honest caveats + external prerequisites

- GPS accuracy in urban high-rise environments or rural areas with poor satellite visibility may fall below the 100 m threshold; the QR code station check-in fallback is less secure (no location verification) and is logged as FALLBACK_QR.
- The HSM-anchored countersig requires the field-mobile-svc HSM to be reachable at Form 34A time; if the HSM is unreachable, the countersig is queued and applied when connectivity is restored; M09 does not accept an unsigned Form 34A.
- The duress SMS fallback requires a pre-loaded message on the device SIM; if the SIM is removed, the SMS path fails silently; the app logs a `DURESS_SMS_FAIL` event on reconnect.
- Party agent accreditation data from CRMS is cached for 24 hours; an agent accreditation revoked after cache refresh may still be able to check in; the CRMS real-time revocation feed must be confirmed as operational before election day.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/flutter/party-agent/test/` and `services/field-mobile-svc/crates/party-api/tests/`: unit tests for GPS geofence, Form 34A QR parser, exactly-once dispute guard, duress alert, chief agent rollup aggregator; widget tests for all screens; integration tests with CRMS stub, M18 boundary stub, SoftHSM countersig, M09 countersig consumer; e2e Flutter integration test covering full polling day flow (check-in, Form 34A, dispute, duress); chaos test for HSM unavailable during countersig.

Runbooks under `docs/runbooks/`:

- `party-agent-degradation.md` (create): GPS accuracy failure; HSM unavailable; CRMS credential cache miss; duress SMS fail.

Dashboards: `platform/observability/field-mobile-svc/grafana-party.json` (check-in rate, countersig rate, dispute count, duress alert count). SLO: check-in sync <30 s; countersig sync <60 s; duress alert delivery <10 s.

Chaos scenarios under `chaos/scenarios/eel-party-agent-*.yaml`: GPS accuracy degradation (urban canyon); HSM unavailable during countersig window; CRMS credential cache miss; duress SMS delivery failure.

Performance targets: check-in sync < 30 s on 3G; Form34A countersig sync < 60 s; duress alert delivery (app + SMS) < 10 s; StationRollup push < 5 s from station event.

SLOs: 100% of check-in events reach server before polls close; 0 duplicate dispute records per station; duress alert unacknowledged > 10 min triggers auto-escalation.

Chief agent rollup security: the StationRollup contains only aggregate counts (check-in status, dispute count, Form34A status), not individual voter data; a chief agent cannot drill into a specific voter record through M30.

CRMS credential offline window: the cached CRMS credential allows operation for up to 4 hours without connectivity; beyond 4 hours the agent cannot countersign new Form34A records until connectivity is restored and CRMS re-validates; paper-countersignature on the physical Form34A remains valid regardless.

Related: `docs/plans/modules/mobile-party-agent.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
