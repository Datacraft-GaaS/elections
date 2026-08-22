# ecc-svc capabilities and algorithms

Comprehensive capability catalogue for the Election Command Centre. Every capability lists
the algorithm, source reference (paper, standard, or statutory authority), and the crate
or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end data flows
3. Kit telemetry snapshot fusion (M16)
4. Alert deduplication, rate limiting, and severity ranking (M16)
5. S.44A dual-control state machine (M16)
6. IAB dashboard and cosign cross-check (M15)
7. Grievance channels and NLP triage (M23)
8. Real-time push via SSE (M15, M16, M23)
9. Dispatch loop (M16)
10. Multi-role portal access (M15, M16, M23)
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
| M15 | iab-dashboard | M | crates/ecc-api | hardware key logistics; cosign witness |
| M16 | election-command-centre | XL | crates/ecc-api + ex/ | RBAC complexity; NIS dispatch loop |
| M23 | grievance-channels | L | crates/ecc-api + py/ | WhatsApp Business API approval (6-8 wk) |

Tier: T2. Availability SLO: 99.9%. Alert delivery latency < 5 s. Grievance acknowledgement < 10 s.

RTO: 1 hour. RPO: 2 min.

---

## 2. End-to-end data flows

### 2.1 Kit telemetry ingestion (M16)

```
inventory-svc.kit.telemetry.received
  -> Kafka consumer (at-least-once idempotent)
  -> telemetry_fusion: merge kit_throughput + clerk_reports + cv_headcount + qr_scans
  -> KitTelemetrySnapshot upserted (46 k kits; p99 render < 2 s)
  -> SSE broadcast to ECC dashboard viewers
  -> anomaly check: delta vs rolling baseline -> ECCAlert if threshold exceeded
```

### 2.2 Grievance intake (M23)

```
WhatsApp Business API / USSD / SMS / web kiosk / web portal
  -> crates/ecc-api/src/grievance_ingest.rs: normalize channel, assign case_id
  -> Python NLP triage (py/nlp_triage.py): sentiment + category + urgency_score
  -> GrievanceCase persisted; acknowledgement event emitted within 10 s
  -> Assigned to officer; escalation to IAB if urgency_score > threshold
```

### 2.3 S.44A trigger path (M16)

```
queue-svc.s14a.recommended event
  -> S44AStateMachine: idle -> candidate_flagged -> dual_control_review -> { approved | rejected }
  -> Both Chair + Deputy Chair HSM signatures required for approval (dual-control)
  -> ecc.s44a.triggered.v1 emitted; ECC dashboard updated
```

---

## 3. Kit telemetry snapshot fusion (M16)

### 3.1 Fusion algorithm

Crate: `crates/ecc-api/src/telemetry_fusion.rs`.

Fuses four independent signal sources per station into one `KitTelemetrySnapshot`:

| Source | Signal | Weight |
|---|---|---|
| Kit throughput (BVI identification events) | voters_served_count, queue_length_estimate | 0.5 |
| Clerk reports (field-mobile.queue.reported) | clerk_queue_length, clerk_status | 0.3 |
| Computer vision head-count (queue-svc cv_estimates) | cv_head_count, cv_confidence | 0.1 |
| QR scan events (biometric-edge.qr.scanned) | qr_scan_count | 0.1 |

Fusion: weighted median across available sources (missing source weight redistributed proportionally). Confidence score = sum of weights of present sources. Snapshot stored every 30 s per station.

Scale: 46,000 kit snapshots; dashboard render p99 < 2 s via materialized read-model in `eel_ecc_kit_snapshot_latest` (upsert on station_id; no historical bloat in hot path).

### 3.2 Snapshot schema

`eel_ecc_kit_snapshots`: snapshot_id, station_id, captured_at, voters_served, queue_length, confidence_score, fusion_sources JSONB.
`eel_ecc_kit_snapshot_latest`: one row per station_id (upsert); used for dashboard rendering.

---

## 4. Alert deduplication, rate limiting, and severity ranking (M16)

### 4.1 Deduplication

Crate: `crates/ecc-api/src/alert_engine.rs`.

Fingerprint: `blake3(alert_type || source_service || station_id || 5-min-bucket)`. Duplicate fingerprints within the same 5-minute bucket are coalesced into one alert with occurrence_count increment. Prevents alert storms during mass kit failures.

### 4.2 Rate limiting

Per source-service: max 100 new alert fingerprints per 5 min per source. Excess alerts dropped and a single `rate_limit_exceeded` meta-alert raised. Token bucket via in-process Valkey-backed counter.

### 4.3 Severity ranking

| Severity | Criteria | SSE priority |
|---|---|---|
| CRITICAL | S.44A candidate; mass kit failure (> 5 % of stations); HSM unavailable | immediate push |
| HIGH | Single kit offline > 15 min; disenfranchisement flag from queue-svc | push within 5 s |
| MEDIUM | Turnout deviation > 2 sigma; grievance backlog > 100 | push within 30 s |
| INFO | Routine telemetry; model drift alerts from ai-platform-svc | batched per 60 s |

Severity determined by `AlertSeverityPolicy` in `crates/ecc-api/src/alert_policy.rs`.

---

## 5. S.44A dual-control state machine (M16)

### 5.1 State machine

Crate: `crates/ecc-api/src/s44a.rs`.

```
Idle
  -> CandidateFlagged (trigger: queue-svc.s14a.recommended or manual operator action)
  -> DualControlPending (both Chair and Deputy Chair must sign)
  -> { Approved (both HSM signatures present + verified) | Rejected (either party rejects) }
  -> Reverted (post-election; audit record retained)
```

Transitions use `SELECT FOR UPDATE` on `eel_s44a_events` to prevent concurrent state corruption. Each transition appended as immutable event row with HSM signature from the acting principal.

### 5.2 HSM dual-control

Signing: PKCS#11 HSM (key role `s44a_authority`). Both Chair (slot 0) and Deputy Chair (slot 1) Ed25519 keys must sign the same `S44ADecision` struct: `{ election_id, station_id, trigger_time, decision, justification }`. Verification: `crates/ecc-api/src/s44a_verify.rs` checks both signatures against registered public keys in `eel_authority_keys`.

---

## 6. IAB dashboard and cosign cross-check (M15)

### 6.1 IAB dashboard

Component: `ex/iab_live/` (Elixir Phoenix LiveView). Role: `iab_member` (SPIRE SVID required). Displays:
- Escalation queue: unresolved grievances escalated from M23.
- Kit telemetry summary: aggregated snapshot fusion status across all stations.
- S.44A event log: all state transitions with HSM signature status.
- cosign witness panel: per-publication cosign verification status vs Rekor STH.

### 6.2 cosign witness cross-check

Crate: `crates/ecc-api/src/iab_witness.rs`.

For each tarball published by portal-svc, IAB dashboard independently re-verifies cosign signature against Rekor STH. Algorithm:

```
1. Fetch tarball SHA-256 digest from eel_portal_snapshot_index
2. Query Rekor /api/v1/index?hash=sha256:<digest>
3. Verify inclusion proof (RFC 6962)
4. Verify cosign Ed25519 signature against registered public key
5. Persist IABWitnessResult: { snapshot_id, verified_at, rekor_log_id, outcome }
```

| Algorithm | Reference |
|---|---|
| RFC 6962 inclusion proof verification | Laurie, Langley, Kasper RFC 6962 2013 |
| Ed25519 signature verification | Bernstein et al. IACR ePrint 2011 |
| SHA-256 digest | FIPS 180-4, NIST 2015 |

### 6.3 IAB escalation workflow

`eel_iab_escalations`: escalation_id, grievance_id, assigned_iab_member, override_decision (APPROVE/REJECT/REFER), decision_timestamp, hsm_signature. HSM signature on each IAB decision for non-repudiation.

---

## 7. Grievance channels and NLP triage (M23)

### 7.1 Intake channels

| Channel | Integration | Crate / component |
|---|---|---|
| WhatsApp Business API | Meta Cloud API webhook | py/wa_grievance_bot.py |
| USSD | Africa's Talking USSD API | py/ussd_grievance.py |
| SMS | Africa's Talking SMS API | py/sms_grievance.py |
| Web kiosk | Astro 5 SSR form | ex/grievance_kiosk/ |
| Web portal | REST POST /v1/ecc/grievances | crates/ecc-api |

### 7.2 NLP triage pipeline

Python worker: `py/nlp_triage.py`.

- Multilingual classification: mBERT (Devlin et al. NAACL 2019) fine-tuned on Kenyan electoral grievance corpus (11 languages).
- Categories: kit_failure, intimidation, queue_closure, ballot_irregularity, impersonation, other.
- Urgency score: 0.0-1.0 regression head; score > 0.8 triggers immediate IAB escalation.
- Sentiment: binary positive/negative auxiliary head for officer prioritization.
- Inference latency p95 < 200 ms (batch size 1; ONNX Runtime quantized int8 model).
- PII detection: spaCy NER removes phone numbers and names before storing triage output.

| Model | Reference |
|---|---|
| mBERT multilingual | Devlin, Chang, Lee, Toutanova NAACL 2019 |
| ONNX Runtime int8 quantization | Bhandare et al. IEEE 2019 |
| spaCy NER | Honnibal, Montani (Explosion AI 2017) |

### 7.3 Whistleblower protection

WhistleblowerProfile encrypted with per-profile DEK (AES-256-GCM). DEK wrapped under grievance KEK. Identity disclosed only to designated IAB officer via HSM-authorized decryption. Statutory basis: Whistleblower Protection Act 1998.

---

## 8. Real-time push via SSE (M15, M16, M23)

Component: `ex/ecc_live/SseController` (Elixir Phoenix).

- Topic `eel:ecc_alerts`: alert_type, severity, station_id, timestamp. No PII.
- Topic `eel:grievance_updates`: case_id, status, assigned_officer_role. No grievance text.
- Topic `eel:kit_telemetry`: station_id, voters_served, queue_length, confidence_score.
- Connection: standard `text/event-stream` (W3C Server-Sent Events; WHATWG HTML Living Standard).
- Backpressure: per-connection send buffer 1000 events; slow consumers disconnected and must reconnect.
- Authentication: Phoenix token (signed, 1 h TTL) required per connection for M15/M16; M23 public grievance status uses anonymous connection.

---

## 9. Dispatch loop (M16)

### 9.1 Personnel dispatch

Crate: `crates/ecc-api/src/dispatch.rs`.

Operator selects dispatch command (relocate, reinforce, technical-support, medical). Command signed by ECC operator identity (SPIRE SVID). Delivered to field-mobile-svc via `eel.ecc.dispatch.command.v1` CloudEvent. Acknowledgement tracked in `eel_dispatch_commands`: command_id, personnel_id, command_type, destination, ack_timestamp, ack_by.

### 9.2 NIS (National Integration System) loop

Async loop polling `eel_ecc_alerts` for CRITICAL severity. Auto-routes to ECC duty officer push notification (Firebase Cloud Messaging) within 30 s of alert creation. Manual fallback: operator console for radio dispatch.

### 9.3 Dispatch acknowledgement tracking

Unacknowledged dispatch commands escalate:
- T+5 min: repeat push notification.
- T+15 min: ECC supervisor alerted; backup personnel identified from reserve pool.
- T+30 min: command auto-cancelled; `dispatch_timeout` event logged; incident opened in M23 grievance channel.

`eel_dispatch_commands.status` state machine: `Issued -> Acknowledged -> InProgress -> Completed | Cancelled | TimedOut`. Each state transition timestamped and persisted as append-only row in `eel_dispatch_command_events`.

### 9.4 Duty roster and resource map

`eel_duty_roster`: personnel_id, station_id, role, shift_start, shift_end, is_reserve. Resource map rendered in ECC dashboard LiveView as PostGIS-backed geographic overlay from boundary-svc constituency polygons. Dispatch commands cross-referenced against duty roster to confirm target personnel are on-shift before sending.

---

## 10. Multi-role portal access (M15, M16, M23)

| Role | Module | Access | Auth |
|---|---|---|---|
| ECC operator | M16 | Full dashboard + dispatch | SPIRE SVID + TOTP |
| IAB member | M15 | Escalation queue + witness panel | SPIRE SVID + hardware key |
| Party agent (read) | M16 | Kit status per registered stations | SPIRE SVID (MDM-enrolled) |
| Observer (read) | M15, M16 | Aggregate dashboard; no PII | SPIRE SVID (MDM-enrolled) |
| Media / journalist | M16 | Aggregate live counts; no kit detail | API key + rate limit |
| Grievance officer | M23 | Assigned case queue; no IAB escalations | SPIRE SVID + TOTP |
| Public (anonymous) | M23 | File grievance; check own case status | Case token (UUID7) |

RBAC enforced via SpiceDB shape: `eel_spicedb_schema.zed` defines role-resource-action triples. Every API request verified against SpiceDB `check` before handler executes.

---

## 11. Operational excellence

| Capability | Mechanism |
|---|---|
| Grievance backlog | Prometheus alert when outstanding grievances > 100; auto-page duty officer |
| Alert dedup storm | Rate-limit meta-alert surfaced on ECC dashboard with dedup fingerprint count |
| ECC dashboard failover | Read-only cached snapshot from Valkey on Postgres unavailability; stale label shown |
| NLP model reload | Zero-downtime reload: new model loaded alongside old; health check passes before traffic shifted |
| k6 load test | `loadtest/ecc_load.js` targets 100 VUs / 600 s; alert streaming and grievance intake |
| HPA | CPU 65 %, memory 75 %; min 2, max 20 replicas |
| PDB | minAvailable 1 (always 2+ running) |
| IAB witness lag | Prometheus alert when unverified cosign proofs > 10; transparency-svc dependency paged |
| S.44A candidate backlog | Alert when S.44A candidates pending dual-control > 5 for > 10 min; escalate to Principal ECC officer |
| Grievance channel parity | WhatsApp, USSD, and SMS intake health checked every 60 s; automatic rerouting if one channel unavailable |

### 11.1 Kit telemetry materialized view refresh

`eel_ecc_kit_snapshot_latest` is refreshed by upsert on each incoming `KitTelemetrySnapshot` event (not a scheduled REFRESH MATERIALIZED VIEW). No lock contention: `INSERT INTO eel_ecc_kit_snapshot_latest (...) ON CONFLICT (station_id) DO UPDATE SET ...`. This gives read-consistent views of current kit status to dashboard queries at dashboard render time without a full table scan (station_id primary key indexed).

Backup aggregate: `eel_ecc_county_summary` (one row per county) maintained by a Kafka consumer computing county-level voters_served sum and kit_offline_count. This powers county-level summary tiles in the ECC overview dashboard without joining 46k rows per page load.

---

## 12. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| STIX 2.1 | Security incident evidence package format for M23 (optional; used when incident referred to NCIC/DCI) | crates/ecc-api/src/stix.rs |
| CloudEvents 1.0 | All event envelopes consumed and emitted | crates/ecc-api |
| PKCS#11 | HSM interface for S.44A dual-control and IAB decision signing | crates/ecc-api/src/hsm.rs |
| OpenAPI 3.1 | API contract at packages/openapi-contracts/ecc/ | crates/ecc-api |
| W3C Server-Sent Events | Real-time push to ECC dashboard viewers | ex/ecc_live/ |
| RFC 6962 | Inclusion proof verification in IAB cosign cross-check | crates/ecc-api/src/iab_witness.rs |

---

## 13. Validation harnesses

| Harness | Scope | Command |
|---|---|---|
| Alert dedup unit tests | Fingerprint collision, rate limit | `cargo test alert_engine:: --lib` |
| S.44A state machine tests | Transition coverage, dual-control | `cargo test s44a:: --lib` |
| NLP triage accuracy | Category F1 per class; urgency RMSE | `cd py && uv run pytest tests/test_triage.py` |
| IAB witness integration | cosign verify against mock Rekor | `cargo nextest run --test iab_witness` |
| Grievance intake contract | OpenAPI compliance per channel | `cargo test --test grievance_contract` |
| k6 alert streaming load | 10x alert volume; p99 < 5 s | `cd loadtest && k6 run ecc_load.js --vus 100 --duration 600s` |

---

## 14. Data lifecycle and security

### 14.1 Encryption

Grievance text: encrypted at rest with per-case DEK (AES-256-GCM). WhistleblowerProfile: per-profile DEK wrapped under grievance KEK. Kit telemetry snapshots: no PII; stored in plaintext. STIX 2.1 evidence packages: per-package DEK (AES-256-GCM), DEK wrapped under ecc-svc security KEK; packages accessible only to NCIC/DCI roles.

### 14.2 Audit

Every IAB decision HSM-signed and appended to `eel_iab_escalations` (append-only trigger; no UPDATE or DELETE). S.44A transitions: immutable event rows in `eel_s44a_events` with HSM signatures.

### 14.3 Retention

- `eel_ecc_alerts`: 90-day rolling partition; critical alerts 3 years.
- `eel_grievance_cases`: 7 years post-election (Whistleblower Protection Act; Elections Act §39).
- `eel_iab_escalations`: permanent (election archive).
- `eel_s44a_events`: permanent.
- `eel_dispatch_commands`: 3 years.
- `eel_ecc_kit_snapshot_latest`: no historical retention (upsert; one row per station).
- `eel_ecc_kit_snapshots`: 90-day rolling partition; critical-window snapshots (T-2 h to T+2 h on poll day) retained 3 years.
- `eel_stix_packages`: 7 years (legal referral records).

### 14.4 Key management

DEK generation: `OsRng` (ring crate); 256-bit random key per case. KEK: stored in OpenBao (HashiCorp Vault-compatible) at `62.169.25.77` (per SERVICE_CATALOGUE.md). DEK wrapped under KEK using AES-256-KW (RFC 3394). KEK rotation: new KEK version generated annually; old DEKs re-wrapped under new KEK version by batch job with progress tracked in `eel_dek_rewrap_jobs`.

---

## 15. API surface

### 15.1 REST (`crates/ecc-api`, axum 0.8, port 8080)

- `GET /v1/ecc/alerts`: real-time alert list with severity and temporal filters.
- `GET /v1/ecc/alerts/{id}`: single alert with evidence JSON.
- `GET /v1/ecc/kit/snapshots`: all station kit snapshots (paginated).
- `GET /v1/ecc/kit/snapshots/{station_id}`: latest snapshot for one station.
- `POST /v1/ecc/grievances`: submit grievance (all channels normalised here).
- `GET /v1/ecc/grievances/{id}`: case status and workflow.
- `GET /v1/ecc/iab/escalations`: IAB escalation queue.
- `POST /v1/ecc/iab/escalations/{id}/override`: IAB override decision (HSM-signed).
- `GET /v1/ecc/s44a/events`: S.44A event log.
- `POST /v1/ecc/dispatch/{personnel_id}/command`: dispatch instruction.
- `GET /healthz` + `/readyz` + `/openapi.json`.

Idempotency-Key required on all POST endpoints. Rate limits per /64 subnet and per role. SPIRE SVID verified on all authenticated routes.

### 15.2 gRPC (`crates/ecc-api/src/grpc`, port 8443)

`proto/ecc.proto`: StreamAlerts, SubmitGrievance, GetGrievanceWorkflow, OverrideDecision RPCs.

### 15.3 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.rts.tally_updated.v1`, `eel.bvi.turnout_updated.v1`
- `eel.queue.disenfranchisement_flagged.v1`, `eel.queue.s14a_recommended.v1`
- `eel.ai.alert_raised.v1`, `eel.inventory.kit_telemetry_received.v1`

Emitted:
- `eel.ecc.alert_raised.v1`, `eel.ecc.s44a_triggered.v1`
- `eel.ecc.grievance_filed.v1`, `eel.ecc.iab_escalated.v1`
- `eel.ecc.dispatch_command.v1`

---

## 16. Summary

Total footprint: 1 Rust crate (ecc-api) + 1 Elixir LiveView app (ecc_live + iab_live) + 3 Python NLP/bot workers + PostgreSQL (migrations not yet seeded; schema in data_dictionary.yaml). Covers M15 (IAB dashboard with cosign witness cross-check and HSM-signed decisions), M16 (election command centre with 46k-kit telemetry fusion at p99 < 2 s, alert dedup, S.44A dual-control, and multi-role portals), and M23 (grievance channels across 5 intake paths with multilingual NLP triage and whistleblower protection). Key production inputs remaining: WhatsApp Business API account approval (6-8 week lead time); HSM key provisioning for S.44A authority slots.

### 16.1 Inter-service dependencies

| Upstream service | Data consumed | Failure mode |
|---|---|---|
| rts-svc | `eel.rts.tally_updated.v1` (M16 tally display) | Tally tiles frozen at last update; stale label shown |
| bvi-svc | `eel.bvi.turnout_updated.v1` (M16 turnout rollup) | Turnout tiles frozen; 60 s delay label retained |
| queue-svc | `eel.queue.s14a_recommended.v1` (M16 S.44A trigger) | S.44A state machine not triggered; manual override path available via admin API |
| ai-platform-svc | `eel.ai.alert_raised.v1` (M16 anomaly alerts) | Anomaly alerts not shown; ECC notified of ai-platform-svc outage separately |
| inventory-svc | `eel.inventory.kit_telemetry_received.v1` (M16 kit map) | Kit map freezes; stale snapshot from Valkey served for up to 5 min |
| field-mobile-svc | `eel.field_mobile.duress_alert.v1` (M16 CRITICAL) | Duress alert not received; security gap; field-mobile-svc outage triggers CRITICAL ECC page |
| portal-svc | cosign witness cross-check source (M15) | Witness panel shows unverified status; IAB manually reviews via transparency-svc directly |
