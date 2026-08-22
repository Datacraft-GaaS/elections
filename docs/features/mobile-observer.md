# Mobile observer (M34)

Module M34 (`field-mobile-svc`) is the accredited-observer phone tool for ~500 civil-society, international, academic, and media observers. It mirrors Public Portal read views, logs GPS-verified station visits, guides structured incident reporting (with systemic-incident escalation), runs cryptographic result verification against the Merkle chain via the shared `libeel-verifier` Rust FFI, provides optional countersignature for eligible categories, and produces a signed end-of-mission PDF fed to the Auditor-General escrow mirror.

Contrast with M30 (Mobile Party Agent), which is restricted to political party agents with statutory Form 34A countersignature rights, and M29 (Mobile Citizen Verify), which is a public read-only tool with no incident or countersignature path.

## Contents

1. What the observer app does (vs related mobile modules)
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What the observer app does (vs related mobile modules)

- **M29** = public citizen verification of their own registration record; no incident reporting, no countersignature.
- **M30** = political party agents: Form 34A countersignature, dispute escalation; single-station scope.
- **M34** = accredited observers: GPS-verified multi-station visit logs, guided verification sampling (cryptographic seed), signed incident reports (systemic-incident flag available to OBSERVER_CHIEF role), category-gated countersignature, end-of-mission PDF, Auditor-General mirror within 24 h.

M34 shares the `libeel-verifier` Rust FFI binary with M29 and M30 for client-side Merkle proof validation. It is the only module producing `eel.field.observer_visit_logged.v1` and `eel.field.systemic_incident_flagged.v1`.

## 2. Data plane

Five aggregates (under `services/field-mobile-svc/crates/field-mobile-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **ObserverAccreditation** | Category and rights: observer_id, category (CIVIL_SOCIETY/INTERNATIONAL/ACADEMIC/MEDIA), issuing_body, accreditation_code, valid_from, valid_to, countersign_rights bool. | D |
| **StationVisitLog** | Signed station visit: observer_id, station_code, check_in_at, check_out_at, gps_lat, gps_lon, observation_note, signed_at, Ed25519 signature. | A |
| **ObserverIncidentReport** | Structured incident: observer_id, station_code, category (PROCEDURAL/SECURITY/LEGAL/OPERATIONAL/TAMPERING/TRANSPARENCY/OTHER), description, evidence_blob_ids, gps_lat, gps_lon, is_systemic_flag, submitted_at, Ed25519 signature. | B |
| **ObserverVerificationRecord** | Merkle verification result: observer_id, station_code, form_id, form_hash, proof_verified bool, countersign_at, countersignature, audit_trail_id. | A |
| **EndOfMissionReport** | Compiled mission summary: observer_id, mission_id, stations_visited, verifications_completed, incidents_reported, generated_at, pdf_blob_id (MinIO). | B |

Local store: Isar + SQLCipher (AES-256-GCM), key in device secure enclave.

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **Accreditation service** (SpiceDB) | On launch and on reconnect | `integrations/spicedb-client/` | SVID mTLS | Incident and countersign features blocked if accreditation absent/expired; stale-since timestamp displayed |
| **RTS back-end (M10)** | On verification request or periodic sync | `packages/openapi-contracts/field/observer.yaml` | JWT + SpiceDB `OBSERVER` role | Cached Form 34A images + Merkle root used for offline verification |
| **Public Portal (M11)** | Snapshot on launch | Internal API | JWT | Snapshot cached in Isar; stale-since timestamp displayed |
| **P05 transparency log** | On Merkle root refresh | `integrations/tlog-client/` | SVID mTLS | Stale root displayed with stale-since timestamp; offline verification runs against cached root |
| **USSD gateway (I08)** | On systemic incident via OBSERVER_CHIEF with no network | `integrations/ussd-gateway/adapter` | Gateway secret | IVR ack within 2 min; SMS delivery receipt |

## 4. Pipeline

```
observer launches app
  -> accreditation check via SpiceDB (category, countersign_rights, valid_to)
  -> Public Portal snapshot cached in Isar
  -> Merkle root cached from P05

  [Station visit]
  -> GPS check-in; distance validated against station polygon
  -> check-in timestamp signed with observer biometric Ed25519 key
  -> StationVisitLog submitted (UUID7 Idempotency-Key); queued if offline
  -> eel.field.observer_visit_logged.v1

  [Guided verification sampling]
  -> n stations per constituency selected by SHANGRLA-compatible cryptographic seed
  -> structured checklist per sampled station:
       form_hash matches transmitted value (Boolean)
       party-agent countersignatures present (Boolean)
       geofence polygon intact (Boolean)
       physical seals intact (Boolean + notes field)
  -> libeel-verifier (Rust FFI, shared with M29 + M30): validate Merkle inclusion proof offline
  -> ObserverVerificationRecord created; countersignature appended if rights permit
  -> eel.field.observer_verification.v1

  [Incident reporting]
  -> ObserverIncidentReport AES-256-GCM encrypted locally in Isar + SQLCipher
  -> signed by observer biometric; queued with UUID7 idempotency key
  -> verification hash mismatch auto-generates TAMPERING category incident;
     station marked "Unverifiable" locally
  -> submitted to observer coordination + war-room Kafka topic + AG mirror on reconnect
  -> eel.field.observer_incident.v1

  [Systemic incident -- OBSERVER_CHIEF role only]
  -> is_systemic_flag set; escalated to OBSERVER_CHIEF in app
  -> if no connectivity: USSD fallback via I08 short-code (reserved pre-election)
  -> IVR ack within 2 min
  -> eel.field.systemic_incident_flagged.v1 (exactly-once via eel-outbox-rs)

  [Check-out + end-of-mission]
  -> observer signs off at station with biometric
  -> StationVisitLog.check_out_at recorded

  [End-of-mission PDF]
  -> EndOfMissionReport auto-compiled from local records
  -> exported to PDF via server-side render
  -> PDF hash anchored in ObserverVerificationRecord
  -> AG escrow mirror: all records with signed provenance within 24 h
     via integrations/ag-mirror/ adapter; delivery confirmed by audit-trace query
```

**Offline verification**: Merkle proof validation executable against cached root; stale-since timestamp displayed. Full verification cycle target: <= 60 s on LTE.

## 5. API surface

Contract: `packages/openapi-contracts/field/observer.yaml`. All writes enforce `Idempotency-Key` header.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/observer/visits` | `OBSERVER` (JWT + SpiceDB) | Log GPS-verified station visit |
| `PATCH /v1/observer/visits/{id}/checkout` | `OBSERVER` | Record check-out time |
| `GET /v1/observer/forms/{id}` | `OBSERVER` | Fetch Form 34A + Merkle proof |
| `POST /v1/observer/verifications` | `OBSERVER` | Submit verification record |
| `POST /v1/observer/incidents` | `OBSERVER` | File incident report |
| `POST /v1/observer/systemic-incidents` | `OBSERVER_CHIEF` | File systemic incident (exactly-once) |
| `POST /v1/observer/countersignatures` | `OBSERVER` (SpiceDB countersign_rights check) | Submit countersignature where rights permit |
| `GET /v1/observer/end-of-mission` | `OBSERVER` | Download signed end-of-mission PDF |

SpiceDB enforces `countersign_rights` at the API layer. A `403` is returned for countersignature attempts by observers without the right; all attempts are logged.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Observer Flutter app (Android 10+/iOS 15+) | `services/field-mobile-svc/apps/observer/` | All observer-facing workflows |
| Observer coordination dashboard (Phoenix LiveView) | `services/field-mobile-svc/ex/dispatch_web/` | IEBC observer coordinators; incident feed, coverage map |
| Auditor-General escrow (external stub) | `integrations/ag-mirror/` | AG receives observer records within 24 h |

Distribution via private IEBC MDM. Languages: EN + SW. Accreditation category badge displayed in app header.

## 7. Integrity + anti-tamper

- **Merkle proof on device**: `libeel-verifier` Rust FFI crate (shared with M29 and M30) validates Merkle inclusion proofs client-side against cached root. No server trust required for proof validity. Hash mismatch auto-escalates as TAMPERING incident.
- **GPS-verified visits**: station visit check-in rejected if GPS places observer outside station polygon. Signed with observer biometric Ed25519 key.
- **Countersignature rights via SpiceDB**: `countersign_rights` flag is evaluated at the API layer for every countersignature request. Attempts without the right return 403 and are permanently logged. Feature availability is enforced at runtime from the accreditation record; the app renders the countersignature UI only when SpiceDB grants the right.
- **Systemic incident exactly-once**: `eel.field.systemic_incident_flagged.v1` carried by `eel-outbox-rs` transactional outbox with UUID7 idempotency key. Delivered exactly-once to war-room and observer coordination Kafka topics.
- **AG mirror provenance**: all observer records submitted to the AG escrow mirror include signed provenance (observer_id, submission_at, Ed25519 signature). Delivery confirmed by audit-trace query via `integrations/ag-mirror/`. Target: within 24 h of original submission.
- **Local store encryption**: AES-256-GCM Isar + SQLCipher; key in device secure enclave. Offline queue survives phone reboot.

## 8. Kenya statute alignment

- **Elections Act s.6A (public inspection)**: observers are entitled to inspect the voter register and results records. M34 provides this access via the Public Portal mirror and Form 34A verification. The app surfaces no voter-level PII beyond what is publicly visible.
- **Evidence Act Cap 80, s.93 (electronic records)**: countersignatures produced under the `countersign_rights` path are signed by an HSM-backed Ed25519 key (P06) and anchored in P05. Paired with the Merkle inclusion proof, they satisfy the admissibility requirements for electronically signed records.
- **Kenya Data Protection Act 2019 s.26 (minimisation)**: observer ID is never combined with GPS coordinates in persistent log records. Station visit logs store GPS at polygon-centroid precision (not exact device coordinates). Log-scanner CI gate enforces this at build time.
- **Whistleblower Protection Act 2022**: systemic incident reports filed via M34 are treated as protected disclosures. Observer identity is separated from the incident record in the war-room view by default; IEBC Privacy Officer approval required to join the fields.

## 9. Honest caveats + external prerequisites

- The AG escrow mirror endpoint (`integrations/ag-mirror/`) is an external stub requiring the Auditor-General's office to provision the endpoint and credential before the election. The 24-hour delivery SLA cannot be met until the live endpoint is available.
- The USSD systemic-incident path requires a pre-election short-code reservation with Safaricom / Africa's Talking via I08. If the short-code is not reserved, OBSERVER_CHIEF escalation falls back to voice call only; no digital audit trail.
- Verification sampling using a cryptographic seed assumes observers follow the guided protocol. An observer who selects stations arbitrarily instead of following the seed-derived sample produces a non-probabilistic sample, reducing the statistical validity of their verification report. The app guides but cannot enforce.
- The countersignature rights flag is set at accreditation time by the IEBC accreditation officer and cannot be changed by the observer. If an observer category is granted rights incorrectly, correction requires re-issuing the accreditation record before election day.
- End-of-mission PDF generation is server-side. If field-mobile-svc is unreachable at end of mission, the observer receives the PDF once connectivity is restored. The local records are the source of truth; the PDF is a rendered view.

## 10. Test coverage + operational runbooks

Tests under `services/field-mobile-svc/crates/field-mobile-api/tests/`:
- `observer_visit_tests.rs` -- GPS polygon boundary, signed visit log, offline queue drain.
- `verification_sampling_tests.rs` -- cryptographic seed determinism, checklist submission, hash-mismatch auto-escalation.
- `incident_observer_tests.rs` -- systemic incident exactly-once semantics, USSD fallback path.
- `countersignature_rights_tests.rs` -- SpiceDB enforcement, 403 on rights-absent attempt, attempt logging.
- `ag_mirror_tests.rs` -- 24-hour delivery SLA, signed provenance, audit-trace confirmation.

Integration tests use `testcontainers` for Postgres + Kafka + AG mirror stub. Chaos scenarios: `chaos/scenarios/eel-observer-*.yaml` (P05 stale root, AG mirror unavailable, USSD gateway timeout).

Runbooks under `docs/runbooks/`:
- `field-mobile-svc.md` -- deploy, rollback, MDM re-provisioning.
- `observer-accreditation-ops.md` -- countersign rights correction; re-issue procedure.
- `ag-mirror-delivery.md` -- monitoring delivery lag; manual re-drain procedure if mirror falls behind.
- `systemic-incident-response.md` -- war-room triage of OBSERVER_CHIEF systemic flags; escalation to RO and legal team.
