# Mobile party agent (M30)

Module M30 (`mobile-party-agent`) owned by `field-mobile-svc`. A Flutter app for accredited political party agents at polling stations. Supports GPS-gated check-in (<=200 m), QR verification of Form 34A results, HSM-anchored countersignature on results, exactly-once dispute escalation, RASP protection, and a silent duress alert path (`eel.field.agent_incident.v1` DURESS). Screen-capture protection enforced on all sensitive views.

## Contents

1. What mobile party agent does (vs M34 observer app)
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

## 1. What mobile party agent does (vs M34 observer app)

- **M34** = accredited observers (domestic + international); incident reporting; read-only sampling verification; end-of-mission PDF.
- **M30** = political party agents with statutory countersignature rights on Form 34A; their signature is part of the results chain, not merely advisory.

M30 has narrower scope than M34: party agents have no access to cross-station aggregates and cannot file systemic incident signals. They act on a single station.

## 2. Data plane

Five aggregates (per `services/field-mobile-svc/crates/party-agent-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **AgentCheckIn** | check_in_id, agent_id, party_code, station_code, gps_lat, gps_lon, gps_accuracy_m, checked_in_at, biometric_ref_hash | D |
| **Form34AVerification** | verification_id, agent_id, station_code, qr_payload_hash, verified_at, signature (Ed25519 + biometric + PIN) | A |
| **DisputeEscalation** | dispute_id, agent_id, station_code, contest_id, reason_code, evidence_uri (encrypted AES-256-GCM), escalated_at, idempotency_key | A |
| **DuressAlert** | alert_id, agent_id, station_code, fired_at, resolved_at | A |
| **AccreditationRecord** | accreditation_id, agent_id, party_code, station_code, valid_from, valid_until, SpiceDB_role | D |

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **M08 signed Form 34A results** | Post-declaration push | Internal Kafka consumer | SVID mTLS | App shows pending until results received |
| **GPS / cell / Wi-Fi** | Check-in | Device location API | n/a | Falls back to cell; if accuracy > 200 m, check-in blocked with explanation |
| **HSM P06 countersignature** | Per Form 34A sign | `integrations/hsm/adapter` | SVID mTLS + PIN + biometric | Sign blocked if HSM unavailable; party agent notified |
| **Accreditation registry** (SpiceDB) | At login + hourly refresh | gRPC | JWT | Login blocked if accreditation expired; offline token valid 4 h |

## 4. Pipeline

Files under `services/field-mobile-svc/mobile/party-agent/` and `crates/party-agent-api/src/`:

```
Party agent opens app; authenticates (biometric + PIN)
      -> RASP check: root detection, debugger, emulator, hook detection
         (fail-closed: app exits if RASP detects compromise)
      -> accreditation verified via SpiceDB (offline token valid 4 h)
      -> GPS check-in:
            gps_accuracy_m <= 200 m required
            station_code confirmed against boundary-svc (M18) polygon
            AgentCheckIn persisted + eel.field.agent_checked_in.v1 emitted
      -> screen-capture protection enabled on all views with Form 34A data

Form 34A QR verification + countersignature:
      -> agent scans QR on posted results sheet
      -> libeel-verifier (Rust FFI) verifies Ed25519 sig on QR payload on-device
      -> if valid: display contest results
      -> agent reviews and countersigns:
            signature = Ed25519 keyed by HSM party-agent KEK
            requires biometric confirmation + PIN (dual-factor)
            Form34AVerification persisted
            eel.field.form34a_countersigned.v1 emitted
            anchored in P05 tlog within 60 s

Dispute escalation (exactly-once):
      -> agent selects dispute reason (code from SpiceDB-gated codelist)
      -> evidence captured (photo / audio, AES-256-GCM encrypted, UUID7 attachment_id)
      -> POST /agent/v1/disputes with Idempotency-Key = UUID7
         (server rejects duplicate within 24 h window)
      -> DisputeEscalation persisted
      -> eel.field.dispute_escalated.v1 emitted to ECC console (M23 integration)
      -> ECC acknowledges within SLA (P0 <= 2 min)

Duress alert (silent):
      -> triple-press volume button OR specific gesture
      -> DuressAlert created
      -> eel.field.agent_incident.v1 (type=DURESS) emitted silently
      -> no visual confirmation on device (prevents coercion detection)
      -> ECC and security coordinator notified immediately
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/party-agent.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /agent/v1/checkin` | Party agent (JWT + biometric) | GPS-gated station check-in |
| `POST /agent/v1/verify` | Party agent | QR Form 34A verification |
| `POST /agent/v1/sign` | Party agent (JWT + HSM + biometric + PIN) | Countersign Form 34A |
| `POST /agent/v1/disputes` | Party agent | Escalate dispute (exactly-once) |
| `GET /agent/v1/disputes/{id}` | Party agent | Dispute status |
| `POST /agent/v1/duress` | App (silent, no UI confirmation) | File duress alert |
| `GET /agent/v1/accreditation` | Party agent | Current accreditation status |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Flutter app | `services/field-mobile-svc/mobile/party-agent/` | Android 8+ / iOS 14+; RASP; screen-capture protection; EN + SW |
| ECC console (Phoenix LiveView) | `services/ecc-svc/ex/ecc_web/` | Receives duress alerts + dispute escalations (M23 integration) |

Languages: EN + SW. KSL video guide for deaf party agents linked from app help screen.

## 7. Integrity + anti-tamper

- **RASP (Runtime Application Self-Protection)** -- root, debugger, emulator, and hook detection at app start; fail-closed; RASP events logged and transmitted to ai-platform-svc M28 (E1-E2 internal model).
- **GPS <=200 m gating** -- check-in rejected if GPS accuracy > 200 m; cell-only check-in allowed at degraded-accuracy threshold with warning; spoof triangulation via cell + Wi-Fi delta checked (>1 km delta flagged by M33 pattern).
- **HSM-anchored countersignature** -- party-agent KEK stored in HSM (P06); signature requires both biometric confirmation and PIN; dual-factor prevents coercion-only signing.
- **Exactly-once dispute** -- Idempotency-Key (UUID7) enforced server-side with 24-hour dedup window; prevents accidental or adversarial duplicate escalations.
- **Duress silent alert** -- `eel.field.agent_incident.v1` type=DURESS emitted without any on-device UI confirmation; protects agent from visible tell.
- **Screen-capture protection** -- `FLAG_SECURE` (Android) + iOS screenshot notification suppressed on all Form 34A and countersignature views; prevents covert capture by adversary.
- **Offline token validity 4 h** -- accreditation token cached; if SpiceDB unreachable, agent can operate for 4 hours; after expiry, countersignature blocked until connectivity restored.

## 8. Kenya statute alignment

- **Elections Act s.54 (party agent rights)** -- Form 34A countersignature feature implements the statutory right; countersignature stored in P05 log alongside RO signature.
- **Elections Act s.72 (secrecy)** -- party agent cannot retrieve individual voter identification records; they see only declared totals and their own check-in history.
- **Whistleblower Protection Act 2022** -- duress alert path provides a protected disclosure mechanism; alert processed under whistleblower confidentiality provisions.
- **Elections (Technology) Regulations 2017** -- RASP and device security measures must be certified by KEBS before deployment.

## 9. Honest caveats + external prerequisites

- HSM countersignature depends on network connectivity at the moment of signing. If the station is offline, the party agent cannot countersign in real time; the offline token allows form review but not signing. The paper Form 34A countersignature remains the legal original; digital signature is a supplement.
- GPS accuracy <=200 m is not guaranteed in all station environments (thick-walled buildings, underground rooms). The 200 m threshold may need to be relaxed in specific stations; IEBC must enumerate exceptions before deployment.
- RASP false positives on certain OEM Android ROMs (Transsion, Infinix) have been observed in field testing; the RASP library must be tested on the top 10 Kenyan device models before go-live.
- Duress alert routing depends on ECC having 24/7 staffing during polling day. An unacknowledged duress alert within 10 minutes should escalate to the national security coordinator automatically; this escalation path requires integration with a communication system not yet specified.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/crates/party-agent-api/tests/` and `mobile/party-agent/integration_test/`:
- Unit: GPS accuracy check, RASP detection stubs, Idempotency-Key dedup, duress alert emission, screen-capture flag.
- Property (`proptest`): countersignature requires dual-factor; exactly-once dispute within window; no PII in push payloads.
- Contract (Pact): M08 Form 34A consumer; M23 ECC escalation producer; P05 tlog anchor.
- Device: RASP on emulator (expected block); GPS spoof detection; cold start <=3 s.
- Security: countersignature replay; dispute duplicate; RASP bypass; OWASP ZAP.

Runbooks under `docs/runbooks/`:
- `party-agent-duress.md` -- ECC alert receipt, security coordinator escalation, station security response.
- `hsm-sign-offline.md` -- offline period, paper fallback, post-connectivity sync.
- `dispute-escalation-sla.md` -- P0 2-min ECC acknowledgement, overflow handling.

Related: full spec at `docs/plans/modules/mobile-party-agent.md`; capability catalogue at `services/field-mobile-svc/docs/CAPABILITIES.md`.
