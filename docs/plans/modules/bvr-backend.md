# M02 bvr-backend

| field | value |
|---|---|
| M-number | M02 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-3.5 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M02-bvr-backend.md |

## Purpose

Central authoritative store of the Kenya voter register. Ingests enrolment records from BVR device clients, orchestrates ABIS dedup and adjudication, publishes the register at each electoral scope, and provides back-office tools for Returning Officers.

## Surfaces and languages

| surface | technology | path under services/bvr-svc/ |
|---|---|---|
| Ingest API + gRPC | Rust (axum + tonic + utoipa) | crates/bvr-api/ |
| Domain logic | Rust | crates/bvr-domain/ |
| Projections / read models | Rust | crates/bvr-projections/ |
| Background workers (ABIS queue, export, IPRS) | Rust | crates/bvr-worker/ |
| RO back-office admin | Django on packages/eel-web-py | django/bvr-admin/ |
| ODPC subject-access workflow | Django | django/bvr-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Voter | voter_id (UUID7), biographic_fields (22), status, version, election_scope | A | T1-voter |
| BiometricTemplate | template_id, voter_id, modality, field-encrypted blob, key_id | A | T1-voter (field-encrypted) |
| IDDocumentScan | scan_id, voter_id, mrz_parsed, pdf_ref | A | MinIO |
| AuditEvent | event_id, actor, action, ts_hlc, signed_by | C | T1-audit |
| MerkleRoot | root_hash, seq, signed_by_hsm, escrow_confirmed | C | T1-audit |
| AdjudicationCase | case_id, voter_ids, modality, status, investigator_id | B | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/bvr/backend.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvr/v1/enrolments | internal gRPC | SVID mTLS | yes | Kit to backend ingest; signature verified |
| PATCH /bvr/v1/voters/{id}/status | 3 accredited | JWT+SpiceDB | yes | RO status change; biometric-auth gated |
| POST /bvr/v1/transfers | 3 accredited | JWT+SpiceDB | yes | Voter transfer; dual-auth |
| GET /bvr/v1/registers/{scope} | 4 bulk | JWT+SpiceDB | n/a | Scoped register generation; dual-control |
| GET /bvr/v1/voters/{id} | 3 accredited | JWT+SpiceDB | n/a | Single voter lookup; redacted fields enforced |
| POST /bvr/v1/sar | 3 accredited | JWT+SpiceDB | yes | ODPC subject access request |
| POST /bvr/v1/handoff/{token} | 2 live-per-user | JWT | yes | VPRS handoff token redemption |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.enrolment_accepted.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.enrolment_rejected.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_status_changed.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.register_frozen.v1 | emits | eel.bvr.voter.v1 | event-source | yes | exactly-once |
| eel.bvr.adjudication_opened.v1 | emits | eel.bvr.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.adjudication_resolved.v1 | emits | eel.bvr.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.merkle_root_published.v1 | emits | eel.bvr.audit.v1 | event-source | yes | exactly-once |
| eel.abis.match_result.v1 | consumes | eel.abis.match.v1 | operational | yes | at-least-once idempotent |
| eel.bve.enrolment_captured.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| ABIS (M03) | internal | integrations/abis/ | yes | circuit breaker; dedup queue accumulates; alert if backlog >1 h |
| IPRS | I01 | integrations/iprs/ | yes | fail-closed on new enrolments after 24 h down; reads from cache |
| HSM (P06) | I07 | integrations/hsm-pkcs11/ | yes (SoftHSM) | STH pauses; ingest continues; countersign queue drains on restore |
| Transparency log (P05) | internal | integrations/tlog/ | yes | Merkle root queued; published on restore |
| BI Power BI | I04 | integrations/bi-mart/ | yes | async only; degraded to stale snapshot |
| MDM (O01) | internal | integrations/mdm/ | yes | electoral-area propagation queued |

## ML and AI

none

## Dependencies

- Modules: M01 (device client, ingest source), M03 (ABIS dedup), M04 (VPRS handoff), M07 (BVV register reads)
- Platform: P04 (IAM/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Ingest and signature verification
FR-M02-001: ingest enrolment records from BVR device kits via SVID mTLS; verify per-record Ed25519 signature and kit-certificate binding on every submission.
FR-M02-002: verify KitAttestation (TPM quote + firmware digest) on every ingest batch; quarantine kit centrally within 60 s on attestation mismatch.
FR-M02-003: enforce idempotency per eel_idempotency_key: duplicate same key+body returns cached 200; same key different body returns 422 idempotency-key-reuse.
FR-M02-004: store voter record in T1-voter with versioned append; each mutation appends a new version row; no in-place delete.
FR-M02-005: field-encrypt biometric template blobs with per-field KEK via eel-crypto-rs; eel_key_id stored alongside each template.
FR-M02-006: store ID document scans in MinIO with encrypted reference and SHA-256 hash; PDF never stored unencrypted.
FR-M02-007: enqueue ABIS 1:N dedup request on every accepted enrolment; track eel_request_id and queue backlog depth.
FR-M02-008: route ABIS outcomes: match above threshold opens AdjudicationCase; no-match accepts record; below-quality triggers re-enrolment request to kit.
FR-M02-009: IPRS validation on every ingest: verify ID number, name, and DOB; cache last-known status; fail-closed on new enrolments after 24 h IPRS downtime.
FR-M02-010: IPRS returning two persons for one ID: create AdjudicationCase; block enrolment until resolved; no auto-merge.

Adjudication
FR-M02-011: adjudication queue with minutiae overlay and side-by-side comparison; case SLA timer visible (first response <= 4 h, resolution <= 24 h).
FR-M02-012: investigator biometric-auth approval required to resolve any AdjudicationCase.
FR-M02-013: dual-control on adjudication resolution: two officers approve; both re-validated at commit via SpiceDB.
FR-M02-014: AdjudicationCase immutable after resolution; new disputes open a linked successor case.

RO back-office
FR-M02-015: voter status lifecycle: ACTIVE, DECEASED, DELETED, TRANSFERRED, SUSPENDED; each transition requires biometric auth and mandatory reason code.
FR-M02-016: voter transfer: dual-auth (RO + originating voter fingerprint); polling-station reassigned; signed eel.bvr.voter_status_changed.v1 emitted.
FR-M02-017: single voter lookup with SpiceDB field-level access control; redacted fields enforced per requester role.
FR-M02-018: voter enumeration protection: constant-time response shape; request caps; step-up auth on bulk lookups; redacted error bodies.

Register generation and export
FR-M02-019: gazetted cut-off enforcement: register freeze atomic across all in-flight writes for the election scope; OPA hard-close at transaction time.
FR-M02-020: register snapshot published to escrow mirror on freeze; eel_escrow_lag alert fires if mirror is > 5 min behind primary.
FR-M02-021: register generation at any scope (station, ward, constituency, county, national) in PDF + CSV.
FR-M02-022: 30M record register generation completes in <= 30 min; streaming with checkpoint-resumable progress indicator.
FR-M02-023: voter records streamed and sorted deterministically per polling station for register generation; sort reproducible across runs.
FR-M02-024: bulk export: PDF + CSV; dual-control approval required for > 1 000 rows; DLP rate-limit; mandatory audit entry per export.
FR-M02-025: bulk export produces reproducible PDF matching gazette register format; output file hash logged in T1-audit.
FR-M02-026: QR/barcode generated per voter record for BVI-day lookup; QR payload signed with Ed25519.
FR-M02-027: electoral-area mapping propagated to kits via MDM channel; unacknowledged kits listed in ECC dashboard.

Audit and integrity
FR-M02-028: every state change signed by originator, countersigned by HSM, Merkle-logged, and escrow-mirrored; audit entry mandatory.
FR-M02-029: HSM unavailable: STH pauses; ingest continues; countersign queue drains on restore; no gap in Merkle sequence.
FR-M02-030: KEK rotation: online re-wrap with new key_id; old templates readable until re-wrap complete; full audit of re-wrap progress.
FR-M02-031: Merkle root published to transparency log (P05) on every register snapshot and on configurable batch interval.
FR-M02-032: Debezium outbox CDC slot lag alert fires at 60 s; WAL bloat guard active; disk guard for outbox retention.
FR-M02-033: Kafka ISR shrink: outbox producer blocks; accumulates in PostgreSQL outbox; no loss; page alert fired.
FR-M02-034: split-view attack on transparency log: witness divergence detected; consistency check fails closed; alert paged.

Monitoring and operations
FR-M02-035: per-kit telemetry dashboard: ingest rate, ABIS queue depth, adjudication SLA, escrow lag, SNMP alerts on disk/memory/CPU.
FR-M02-036: ABIS backlog alert fires if dedup queue depth exceeds 30 min of sustained capacity.
FR-M02-037: availability target 99.99% election-critical; SLO burn-rate alert on 30-day rolling window.
FR-M02-038: ingest throughput >= 1 000 records/sec sustained; horizontal scale-out via eel-resilience-rs backpressure.
FR-M02-039: configurable max voters per polling station (default 700); capacity alert when station approaches limit.

Data governance
FR-M02-040: ODPC SAR workflow: retrieve, rectify, erase, port per Kenya DPA 2019; SAR fulfilled with erasure proof within 30 days.
FR-M02-041: retention scheduler enforcing 5-year minimum retention on voter data; purge job logged to T1-audit.
FR-M02-042: soft-delete tombstone honoured in all search, export, and projection paths; purge removes tombstone from all indexes.
FR-M02-043: ODPC erasure proof stored in T1-audit; erasure acknowledged in SAR response document.
FR-M02-044: BI mart async publication to Power BI; stale snapshot served on connector down; no blocking of ingest path.

Security
FR-M02-045: all inputs parameterised; sqlx compile-time query checks enforced; no dynamic SQL construction.
FR-M02-046: SpiceDB down: fail-closed on writes; class-C reads served from cache; 503 problem+json returned.
FR-M02-047: JWT expiry mid-request: in-flight request completes; next request 401 with silent refresh; JWKS refetched on kid miss.
FR-M02-048: realm key rotation with stale JWKS: refetched on kid miss from Keycloak; no persistent caching of 401 responses.
FR-M02-049: rate-limit evasion: keyed on subject + device + /64 prefix; priority shedding on overload.
FR-M02-050: secrets and PII never emitted to logs or traces; telemetry pipeline refuses eel_pii_tier_a and eel_pii_tier_b fields.
FR-M02-051: SSRF via URL fields: allowlist enforced; private IP ranges blocked; DNS rebinding guard active.
FR-M02-052: XXE and deserialisation: DTD disabled; strict Pydantic/serde models; no pickle; billion-laughs test in CI.
FR-M02-053: cache poisoning of register snapshots: cache key includes signature version; SDK verifies Merkle proof before serving.
FR-M02-054: TLS 1.3 minimum; PQ-hybrid negotiated where peer supports; TLS 1.2 refused.

Performance and scale
FR-M02-055: hot Kafka partition: stream-id spread across partitions; lag alert; HPA triggered on consumer lag.
FR-M02-056: consumer lag > 60 s: backpressure applied; paged via Alertmanager; no OOM on ingest path.
FR-M02-057: Valkey eviction does not lose idempotency keys: PG-durable idempotency store; Valkey is cache only.
FR-M02-058: sequencer stall: writes unaffected; cadence alert fired; drain rate >= peak ingest rate on recovery.
FR-M02-059: thundering herd on register snapshot: SDK jitter applied; ETag 304 served; rate-limit on STH endpoint.

Connectivity and time
FR-M02-060: primary/mirror site partition: Patroni fencing; no split-brain; MirrorMaker lag alert; RPO <= 5 min.
FR-M02-061: HLC monotonic: backend clock jump backward detected; HLC never regresses; skew alert fired within 60 s.
FR-M02-062: leap-second event tolerated in timestamp parser; 23:59:60 format not rejected.
FR-M02-063: event time < causation time: accepted; audit anomaly logged; projection order independent of wall-clock order.
FR-M02-064: register-freeze race at T-60 and T-30: OPA hard-close at transaction time; all-or-nothing; TLA+ invariant verified.

Data integrity and schema
FR-M02-065: out-of-order eel.bve.enrolment_captured.v1 events: buffered until gap filled or routed to DLQ; partition processing continues.
FR-M02-066: replayed signed event: rejected on eel_sequence counter mismatch; eel_tamper event emitted.
FR-M02-067: poison message (votes > registered): DLQ with reason code; partition processing continues unaffected.
FR-M02-068: schema v1 and v2 coexist in stream: both accepted; dual-emit; Karapace compat check passes.
FR-M02-069: oversized event (> 1 MB): outbox rejects; payload offloaded to MinIO blob + hash reference.
FR-M02-070: legacy kit pre-CloudEvents payload: shim upgrades envelope; original signature preserved; Pact golden field corpus verified.

Compliance and DR
FR-M02-071: OWASP ASVS Level 3 controls implemented; no unresolved P1 pen-test findings before any production deployment.
FR-M02-072: 11-language support in RO back-office UI; no truncation at 150% pseudo-locale; ICU plurals; font fallback.
FR-M02-073: RTO <= 4 h RPO <= 15 min for election-critical data; PITR restore drill run and logged to transparency log before each election.
FR-M02-074: path traversal and zip-slip sanitised in all file-upload and export paths; fuzz corpus in CI.
FR-M02-075: Debezium tiered-storage lag: <= 10 MB/s throttle; snapshot-then-tail replay for cold consumer.
FR-M02-076: disk full or bucket quota: bounded queues applied; 507 returned; alert fired; no data corruption.

## Acceptance criteria

AC-M02-01: ingest 1 000 records/sec sustained 10 min without loss; zero duplicates in voter table confirmed by post-run audit (load test).
AC-M02-02: adjudication loop closes end-to-end within 24 h SLA; SLA compliance visible in case dashboard across full UAT run (UAT).
AC-M02-03: IPRS failure fails-closed on new enrolments; cached reads served; alert fires within 30 s of blackhole injection (chaos test).
AC-M02-04: 30M record register generation completes in <= 30 min; output PDF hash reproducible across two successive runs (bench).
AC-M02-05: bulk export > 1 000 rows blocked without dual approval; DLP audit entry present in T1-audit for every blocked attempt (policy test).
AC-M02-06: every voter state change has originator Ed25519 signature, HSM countersignature, and Merkle inclusion proof in T1-audit (audit trace).
AC-M02-07: ODPC SAR fulfilled with erasure proof in T1-audit within 30 days; all data classes confirmed absent after purge (workflow test).
AC-M02-08: gazetted cut-off freezes all writes atomically; register snapshot published to escrow within 5 min; no partial writes under concurrent load (time-based test + TLA+ verification).
AC-M02-09: 99.99% availability sustained over 30-day SLO window; burn-rate alert fires at 5x error budget consumption (SLO measurement).
AC-M02-10: register freeze atomic under concurrent load; no partial writes; TLA+ invariant verified and chaos test confirms (formal + chaos test).
AC-M02-11: ABIS 1:N match p95 <= 5 s, p99 <= 8 s at 30M records; 10 000 enrolments/hr throughput sustained (load test via M03 integration).
AC-M02-12: RTO <= 4 h RPO <= 15 min demonstrated by PITR restore drill; restore event logged to transparency log (DR drill).
AC-M02-13: ASVS L3 scan passes with no unresolved P1 findings; Semgrep eel_pii_tier_a and eel_pii_tier_b clean (security audit).
AC-M02-14: HLC never regresses under TimeChaos -10 min injection; skew alert fires within 60 s (chaos test).
AC-M02-15: SpiceDB down fails closed on writes; class-C reads served; 503 problem+json returned (chaos test).

## Applicable edge cases

connectivity
EC-CON-04: primary/mirror site partition; Patroni fencing prevents split-brain; MirrorMaker lag alert; RPO <= 5 min maintained.
EC-CON-11: Kafka ISR shrinks; outbox producer blocks; PostgreSQL outbox accumulates; no loss; page alert fired.

time
EC-TIM-03: backend clock jumps backward; HLC detects regression and never decrements; skew alert fired within 60 s.
EC-TIM-05: leap second (23:59:60) in ingest timestamp; parser tolerates non-standard second; record accepted.
EC-TIM-06: event time < causation time on reordered stream; record accepted; skew anomaly logged to T1-audit; projection order unaffected.

identity
EC-IDN-02: JWT expires mid-request; in-flight request completes; next request returns 401 with silent refresh.
EC-IDN-03: SPIFFE SVID rotates during ingest batch; old SVID valid to its expiry; eel_producer_key_id per event stamped.
EC-IDN-09: dual-control approver revoked between approval steps; both principals re-validated at commit; transaction aborted if revoked.
EC-IDN-10: SpiceDB down; fail-closed on writes; class-C reads served from local cache; 503 problem+json returned.
EC-IDN-11: realm key rotated with stale JWKS cache; JWKS refetched on kid miss from Keycloak; no persistent 401 caching.

data
EC-DAT-01: duplicate ingest same eel_idempotency_key and body; cached 200 returned; voter table unchanged.
EC-DAT-02: same eel_idempotency_key different body; 422 idempotency-key-reuse returned; no record written.
EC-DAT-03: out-of-order events in Kafka stream; buffered until gap filled or routed to DLQ after timeout.
EC-DAT-04: replayed signed event; rejected on eel_sequence counter mismatch; eel_tamper event emitted.
EC-DAT-05: poison message with votes > registered; DLQ with reason code; partition processing continues.
EC-DAT-06: schema v1 and v2 coexist; both accepted; dual-emit; Karapace compatibility check passes.
EC-DAT-07: partial batch failure on 30M ingest; per-row report generated; valid rows committed; failed rows in DLQ.
EC-DAT-08: 22-30M row register generation; streaming with bounded memory; checkpoint-resumable after interrupt.
EC-DAT-09: Unicode names in voter records; NFC normalisation applied; ICU collation used; grapheme-safe splitting.
EC-DAT-10: zero-result register scope; valid empty PDF generated; no div-by-zero; "0 records" rendered.
EC-DAT-11: totals exceed registered count; block with reason code; RO override creates DisputeCase; TLA+ invariant enforced.
EC-DAT-12: Debezium slot lag or WAL bloat; alert at 60 s; outbox retained; disk guard prevents overflow.
EC-DAT-13: soft-deleted voter in search or export; tombstone honoured; purge removes record from all paths.
EC-DAT-14: oversized event (> 1 MB); outbox rejects inline payload; offloaded to MinIO blob with hash reference.
EC-DAT-15: IPRS returns two persons for one ID; AdjudicationCase opened; no auto-merge; enrolment blocked.
EC-DAT-16: legacy kit pre-CloudEvents payload; shim upgrades to eel.bvr.enrolment_captured.v1 envelope; original signature preserved.

security
EC-SEC-01: signature mismatch on ingest record; record never applied; eel_tamper event emitted; page alert within 30 s.
EC-SEC-02: chassis tamper event received from kit; quarantine stream activated; ECC alerted; custody event generated.
EC-SEC-03: HSM unavailable; STH pauses; ingest continues; countersign queue drains on restore; no gap in Merkle log.
EC-SEC-04: KEK rotation mid-cycle; online re-wrap; old templates readable until done; audit of re-wrap progress complete.
EC-SEC-05: injection on every input field; parameterised storage only; sqlx compile-time; ZAP and Semgrep clean.
EC-SEC-06: SSRF via URL fields in back-office; allowlist enforced; private ranges blocked; DNS rebinding guard active.
EC-SEC-07: path traversal or zip-slip in file upload; sanitised before write; fuzz corpus in CI.
EC-SEC-08: XXE or deserialisation in SAML or PDF parsing; DTD disabled; strict serde models; billion-laughs in CI.
EC-SEC-09: rate-limit evasion; keyed on subject + device + /64; headers inspected; priority shedding on overload.
EC-SEC-10: voter enumeration attempt; constant-time response shape; caps enforced; step-up auth required; redacted bodies.
EC-SEC-11: timing side-channel on secret comparison; constant-time compare used throughout eel-crypto-rs.
EC-SEC-12: cache poisoning of register snapshot; cache key includes sig version; SDK verifies Merkle proof before serving.
EC-SEC-13: split-view on transparency log; witness divergence detected; consistency check fails closed.
EC-SEC-14: stolen kit in honey mode; sandbox served; attacker biometric captured; back-end records forensic event.
EC-SEC-15: LiveView or session hijack attempt; CSRF token + session binding + re-auth on step-up enforced.
EC-SEC-16: secrets or PII in logs or traces; telemetry refuses eel_pii_tier_a and eel_pii_tier_b fields; CI regex enforced.
EC-SEC-17: PQ-hybrid negotiation failure; classical TLS 1.3 only; TLS 1.2 refused on all internal and external endpoints.

operations
EC-OPS-08: disk full or bucket quota hit; bounded queues applied; 507 returned; alert fired; no data corruption or silent drop.

scale
EC-SCL-01: 5M portal visitors/hr; >= 95% edge-cached; origin p99 <= 500 ms under k6+Gatling load across 5 regions.
EC-SCL-02: thundering herd on STH endpoint; SDK jitter applied; ETag 304 served; rate-limit prevents origin overload.
EC-SCL-03: hot Kafka partition on ingest topic; stream-id spread across partitions; lag alert; HPA triggered.
EC-SCL-04: consumer lag > 60 s; backpressure applied; Alertmanager page fired; no OOM on ingest workers.
EC-SCL-05: sequencer stall; writes unaffected; cadence alert fired; drain rate >= peak ingest rate on recovery.
EC-SCL-06: tiering lag or cold replay; <= 10 MB/s throttle applied; snapshot-then-tail replay for cold consumer.
EC-SCL-07: 250k check-ins, 60k RTS events, 18k BVI events concurrently; SLAs met at 1x; <= 10% p99 regression at 5x.
EC-SCL-08: Valkey eviction loses idempotency key from cache; PG-durable store serves as authoritative source; Valkey is cache only.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ingest signature verifier, register-freeze FSM, IPRS circuit-breaker, retention scheduler, DLP block | nextest | M02-UNIT- |
| property | queue never loses a record under arbitrary partition; freeze is atomic; HLC monotonic | proptest | M02-PROP- |
| contract | M01 ingest API; M03 ABIS callback; M07 register-read; VPRS handoff | Pact | M02-PACT- |
| integration | IPRS double; ABIS double; HSM SoftHSM; Merkle log testcontainer | nextest + testcontainers | M02-INT- |
| e2e | full enrolment-to-register-generation flow; freeze + snapshot | nextest | M02-E2E- |
| fuzz | ingest payload; MRZ; Unicode fields; bulk export boundary | cargo-fuzz | M02-FUZZ- |
| formal | register-freeze atomicity TLA+; Merkle append-only kani | TLC + kani | M02-FORMAL- |
| load | 1 000 records/sec 10 min; 30M generation; bulk export | k6 + pg bench | M02-LOAD- |
| chaos | primary kill; IPRS blackhole; HSM kill; Kafka ISR shrink | Chaos Mesh | M02-CHAOS- |
| security | Semgrep; ASVS L3; ZAP; STRIDE threat-model mitigations | Semgrep + ZAP + OWASP | M02-SEC- |

## Runbooks and dashboards

- docs/runbooks/bvr-svc-degradation.md (create): offline IPRS; ABIS backlog; escrow lag; bulk-export DLP fire.
- docs/runbooks/bvr-svc-register-freeze.md (create): cut-off procedure; escrow confirmation; rollback if freeze fails.
- platform/observability/bvr-svc/grafana-ingest.json: ingest rate, ABIS queue depth, adjudication SLA, escrow lag.
- platform/observability/bvr-svc/slo-alerts.yaml: 99.99% burn rate; ingest throughput; IPRS circuit state.

## Open questions

- IPRS variability: IPRS SLA is not contractually defined; circuit-breaker thresholds need agreement with NRB before G3.
- 30M scale: confidential-compute workers for biometric matching (M03) need infrastructure allocation confirmed.
- Escrow custodian identity: the sw/ spec references "escrow mirror" without naming a custodian; needs IEBC decision before G2.
