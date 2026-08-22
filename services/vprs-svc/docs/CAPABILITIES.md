# VPRS capabilities and algorithms

Comprehensive capability catalogue for the Voter Pre-Registration System (module M04). Every capability lists the algorithm, the source reference (paper, ISO standard, or statutory instrument), and the crate or file that implements it.

## Contents

1. Module scope
2. Application lifecycle state machine
3. IPRS integration and circuit breaker
4. Enumeration defences
5. Pre-registration PWA and QR handoff
6. Freeze-window compliance (T-60 / T-30)
7. Key algorithms and references
8. Integration points
9. Accessibility
10. Data protection and DPIA logging
11. Operational excellence
12. Standards and compliance
13. Validation harnesses
14. Data lifecycle and security
15. API surface
16. Events (CloudEvents 1.0 via outbox)

---

## 1. Module scope

| Module | Name | Tier | Crate / path |
|---|---|---|---|
| M04 | Voter Pre-Registration System | T2 | vprs-api, vprs-domain |

Availability SLO: 99.9%. Submission latency p95 < 1 s. RTO 60 min, RPO 15 min.

The service accepts online voter registration applications, validates identity against the IPRS national-ID system, tracks application state, and forwards approved applications to bvr-svc for enrolment. It also handles continuous-register change requests after T-60 freeze closes.

---

## 2. Application lifecycle state machine

Implemented in `crates/vprs-domain/src/aggregate.rs`.

```
Submitted
  -> IprsVerifying     (IPRS gRPC call dispatched)
  -> IprsVerified      (response matches)
  -> ManualReview      (IPRS unavailable or ambiguous match)
  -> Approved          (forward to bvr-svc)
  -> Rejected          (mismatch, duplicate, or freeze-period)
  -> Withdrawn         (applicant request before Approved)
```

Change requests follow a parallel FSM:

```
ChangeInitiated -> VerifyingCurrent -> ChangeApproved | ChangeRejected
```

State transitions are appended as immutable events to `application_timeline` (schema: `event_id`, `application_id`, `event_type`, `event_timestamp`, `actor_id`). `SELECT FOR UPDATE SKIP LOCKED` with a lease expiry guards at-most-once processing under concurrent Rust workers.

---

## 3. IPRS integration and circuit breaker

Crate: `crates/vprs-api/src/iprs.rs`. gRPC channel to IPRS national-ID service.

### 3.1 Cache-aside lookup

IPRS responses cached in `iprs_verification_cache` (`national_id`, `lookup_timestamp`, `cache_expiry`, `iprs_response_hash`) with a 24-hour TTL. On a cache hit the service skips the gRPC call and marks the application `IprsVerified` from cache.

### 3.2 Circuit breaker (fail-closed)

Pattern: modified Fowler-style circuit breaker (Martin Fowler, martinfowler.com/bliki/CircuitBreaker.html, 2014) with three states `Closed | Open | HalfOpen`:

- **Closed** (normal): every IPRS call forwarded.
- **Open** (IPRS unreachable or error rate > 30 % in a 60-second window): new applications transition to `ManualReview` rather than being rejected outright. This is fail-closed on identity verification: no application proceeds to `Approved` without a verified IPRS response or explicit manual override.
- **HalfOpen**: one probe call every 30 seconds; success closes the breaker.

Breaker state is scoped per gRPC channel (not global) so a single-node IPRS issue does not affect the entire fleet.

### 3.3 Fallback to staged data

During planned IPRS maintenance windows, a read-replica of pre-staged IPRS data (loaded nightly) is used as fallback. The `iprs_verified_at` timestamp carries a `source: staged` tag to flag downstream consumers.

---

## 4. Enumeration defences

Concern: an adversary probing `GET /v1/vprs/status/{national_id}` in bulk to extract registered/not-registered flags for the full population.

Three interlocking controls, implemented in `crates/vprs-api/src/middleware/`:

### 4.1 Constant-time response shape

All status responses (registered, not registered, pending) are padded to the same byte length and returned after the same minimum elapsed time (50 ms floor with `tokio::time::sleep_until`). This removes timing oracles that distinguish fast cache hits from slow IPRS misses.

### 4.2 Rate limiting

Token-bucket rate limiter (`vprs-api/src/middleware/rate_limit.rs`): 10 requests per minute per authenticated session, 60 per minute per /64 IPv6 subnet, 300 per minute per authenticated officer role. Exceeded buckets return 429 with `Retry-After`. Limits backed by Valkey atomic `INCR` + `EXPIRE`.

### 4.3 Step-up authentication

More than 3 status lookups within 5 minutes from an unauthenticated session triggers a TOTP or SMS OTP challenge before further lookups are served. Implemented via redirect to `ex/` LiveView challenge flow with a signed nonce.

---

## 5. Pre-registration PWA and QR handoff

### 5.1 Progressive web application

The citizen-facing portal is an Elixir Phoenix LiveView application (`ex/`) compiled as a PWA with a service worker for offline form caching. Citizens can fill in personal details offline; the form payload is encrypted in `localStorage` with a session-derived key and submitted on reconnection.

### 5.2 QR handoff to bvr-svc kits

On application approval, vprs-svc generates a time-limited QR code encoding `{ application_id, approved_at, expires_at, sig }` signed with an Ed25519 key (HSM-backed, role `vprs-handoff`). Field kits running bvr-svc scan the QR code, verify the signature and expiry, and retrieve the application record via `GET /v1/vprs/applications/{id}` to pre-populate the biometric capture form. The QR expires in 72 hours; after expiry the applicant must request a new handoff code.

### 5.3 USSD path (planned follow-up)

USSD submission (`*860#`) is listed as a follow-up in `README.md`; not implemented in the current codebase.

---

## 6. Freeze-window compliance (T-60 / T-30)

Kenya's Elections Act §6A and Elections Regulations impose two registration freeze dates:

- **T-60**: voter register closes for new applications 60 days before an election.
- **T-30**: continuous-register changes (transfers, corrections) freeze 30 days before.

### 6.1 OPA hard-close at transaction time

A Rego policy (`deploy/authz/vprs-freeze.rego`) is evaluated by Open Policy Agent at every write transaction. The policy reads `election_calendar` from a config feed and computes `days_to_election`. If `days_to_election <= 60` the policy rejects `POST /v1/vprs/applications`; if `days_to_election <= 30` it also rejects `POST /v1/vprs/applications/{id}/change-request`. Enforcement happens inside the axum middleware layer before any database write, eliminating a time-of-check/time-of-use race between application acceptance and OPA evaluation.

### 6.2 Race defence

To prevent a race where two concurrent submissions arrive at exactly T-60 boundary:

- OPA evaluation is wrapped in a serialisable database transaction.
- The freeze date is stored in `eel_election_calendar` (managed by boundary-svc) and read inside the transaction, not from a process-level cache.
- Any application whose `submitted_at` falls after the freeze date is rejected even if the OPA check passed during the pre-check phase, because the transaction re-evaluates the constraint at commit time.

---

## 7. Key algorithms and references

| Algorithm | Implementation path | Reference |
|---|---|---|
| Circuit breaker (fail-closed on identity verification) | vprs-api/src/iprs.rs | Martin Fowler, CircuitBreaker, martinfowler.com, 2014 |
| Token-bucket rate limiter | vprs-api/src/middleware/rate_limit.rs | Valkey INCR + EXPIRE; algorithm: token bucket (Tanenbaum, Computer Networks, 5th ed., 2011) |
| Constant-time response padding (enumeration defence) | vprs-api/src/middleware/enum_defence.rs | OWASP WSTG-ENUM-05 timing-oracle guidance |
| Ed25519 QR signature (HSM-backed) | vprs-api/src/qr.rs | Bernstein et al., "High-speed high-security signatures", J. Cryptographic Engineering, 2012 |
| OPA freeze-window Rego policy | deploy/authz/vprs-freeze.rego | Open Policy Agent v0.x Rego evaluation; Kenya Elections Act §6A |
| TOTP step-up authentication | ex/lib/vprs_web/live/challenge_live.ex | RFC 6238 (TOTP: Time-Based One-Time Password Algorithm, IETF, 2011) |

---

## 8. Integration points

| Direction | Service | Mechanism | Data |
|---|---|---|---|
| Upstream | IPRS (national ID system) | gRPC with mTLS | national_id, names, date_of_birth |
| Upstream | boundary-svc (election calendar) | REST / config feed | freeze dates, election metadata |
| Downstream | bvr-svc (voter enrolment) | Kafka `vprs.application.approved` + QR code | application_id, demographic pre-fill |
| Downstream | audit logging | Kafka `vprs.application.*` outbox | all lifecycle events |
| Downstream | admin portal | REST (read) | application status |
| Consumed | bvr-svc | Kafka `bvr.voter.enrolled` | enrolment confirmation cross-check |

---

## 9. Accessibility

The LiveView portal (`ex/`) targets WCAG 2.2 Level AA (W3C Recommendation, October 2023):

- All form controls have programmatically associated `<label>` elements.
- Focus order matches DOM order; no keyboard traps.
- Contrast ratio >= 4.5:1 for body text, >= 3:1 for large text and UI components.
- Touch targets >= 24x24 CSS pixels (WCAG 2.5.8 Minimum Target Size).
- Time-limited QR sessions are extendable without re-entering data (WCAG 2.2.1 Timing Adjustable).
- Error messages identify the field and describe the correction required (WCAG 3.3.1/3.3.3).

Automated coverage: Axe-core integration tests run in CI against the LiveView portal on every pull request. Manual WCAG audit is required at each major release.

---

## 10. Data protection and DPIA logging

### 10.1 Kenya Data Protection Act 2019

Personal data collected (names, national_id, date_of_birth, phone) is processed under Act §30 (lawful basis: public task), §34 (data minimisation), and §41 (security of processing). Voter data is never transferred cross-border without explicit Act §71 authorisation.

### 10.2 ISO/IEC 27701 PIMS

The service implements Privacy Information Management System controls per ISO/IEC 27701:2019:

- `PII_CONTROLLER` role assigned to the registrar; `PII_PROCESSOR` role to the technical operator.
- Data-processing records (`eel_processing_activities`) document purpose, retention period, and legal basis per Annex A.7.
- DPIA log scrubber: before application rejection notices are sent to applicants, a scrubber (`vprs-domain/src/scrubber.rs`) strips all fields classified `PII_RESTRICTED` (national_id, biometric template refs) and replaces them with `[REDACTED]`. Scrubber is triggered via a pre-publish hook on the `vprs.application.rejected` CloudEvent.

### 10.3 Right to erasure (Act §38)

An applicant may request erasure before approval. The `DELETE /v1/vprs/applications/{id}` endpoint soft-deletes the application record (sets `erased_at`, zeros PII fields in-place) and publishes a `vprs.application.erased` event. Hard delete of S3/MinIO blobs is scheduled within 30 days per the retention policy.

---

## 11. Operational excellence

| Capability | Path | Mechanism |
|---|---|---|
| HPA | deploy/base/ | CPU 70%, memory 80%; min 3, max 20 replicas |
| Blue-green deploy | deploy/service.yaml | primary + mirror sites |
| Rolling deploy | deploy/service.yaml | 5 edge sites (Nairobi, Mombasa, Kisumu, Eldoret, Garissa) |
| PDB | deploy/base/ | minAvailable 1 (always 2+ running) |
| Application queue backpressure | vprs-api/src/queue.rs | Kafka consumer with backpressure; alert at queue depth > 5 k |
| IPRS cache miss alert | Prometheus | vprs_iprs_cache_miss_rate > 30 % fires warning |
| Submission latency SLO | Prometheus | vprs_application_submission_latency_seconds p95 > 1 s fires warning |
| Chaos: IPRS unavailable | loadtest/ | Kill IPRS; verify cached data fallback and ManualReview path |
| Chaos: PostgreSQL failover | loadtest/ | Kill primary; observe failover to DR within RPO |
| Chaos: LiveView disconnect | loadtest/ | Simulate network partition; verify graceful reconnect |

Key metrics: `vprs_application_submission_latency_seconds`, `vprs_iprs_lookup_latency_seconds`, `vprs_application_approval_rate`, `vprs_iprs_cache_hit_rate`, `vprs_application_queue_depth`, `vprs_liveview_concurrent_users`.

---

## 12. Standards and compliance

| Standard / statute | Coverage |
|---|---|
| WCAG 2.2 Level AA (W3C, October 2023) | LiveView portal accessibility |
| Kenya Data Protection Act 2019 (Act No. 24 of 2019) | Lawful basis, data minimisation, security, erasure |
| ISO/IEC 27701:2019 Privacy Information Management | PIMS controller/processor roles, processing records, DPIA |
| Kenya Elections Act Cap 7 §6A, §35 | Voter registration eligibility and procedures |
| Kenya Elections Regulations (2012, as amended) | Freeze dates T-60 (applications), T-30 (changes) |
| RFC 6238 (IETF TOTP, 2011) | Step-up authentication OTP |
| CloudEvents 1.0 (CNCF, 2022) | Event envelope for all outbox messages |

---

## 13. Validation harnesses

| Harness | Location | What it tests |
|---|---|---|
| OpenAPI contract tests | tests/ (*_contract.rs) | Every REST endpoint matches openapi-contracts/vprs/ schema |
| IPRS circuit breaker property tests | crates/vprs-domain/tests/ | Breaker state transitions under randomised error rates (proptest) |
| Freeze-window boundary tests | crates/vprs-domain/tests/ | T-60 and T-30 race conditions at second-level granularity |
| WCAG axe-core integration | ex/test/ | Automated accessibility scan on LiveView pages |
| Enumeration defence timing tests | crates/vprs-api/tests/ | Response-time variance < 5 ms across registered/not-registered branches |
| Load test: 500 VUs | loadtest/application-submit.js | p95 < 1 s, no 5xx under sustained load |
| gRPC contract tests | tests/ | SubmitApplication, GetApplicationStatus, VerifyWithIPRS RPCs |

---

## 14. Data lifecycle and security

### 14.1 Encryption at rest

All PII fields (`national_id`, `first_names`, `surname`, `date_of_birth`) are encrypted with per-row DEKs. DEKs are wrapped under a cycle KEK managed by the platform key-management service. IPRS response hashes in `iprs_verification_cache` are SHA-256 of the cleartext response; the response itself is not persisted.

### 14.2 Retention

| Table | Retention |
|---|---|
| applications (approved) | Election cycle + 15 years (KNADS Act) |
| applications (rejected) | 6 months post-rejection |
| application_timeline | Same as parent application |
| iprs_verification_cache | 24 hours (cache_expiry column) |
| change_requests | Election cycle + 7 years |

### 14.3 Audit immutability

`application_timeline` is append-only (UPDATE and DELETE denied by a PG trigger). Every event row carries `actor_id` and `event_timestamp`. Compliance with ADR-0016 (retention policy) enforced by monthly partition pruning jobs.

### 14.4 TLS and transport

All REST and gRPC endpoints require TLS 1.3. IPRS gRPC channel uses mTLS with SPIRE-issued SVIDs (`spiffe://eel.local/vprs-svc`).

---

## 15. API surface

### 15.1 REST (vprs-api, axum 0.8, port 8080)

- `POST /v1/vprs/applications`: submit new registration application. Requires `Idempotency-Key` header.
- `GET /v1/vprs/applications/{id}`: retrieve application status and timeline.
- `POST /v1/vprs/applications/{id}/updates`: submit application update.
- `POST /v1/vprs/applications/{id}/change-request`: initiate continuous-register change.
- `GET /v1/vprs/status/{national_id}`: check registration status (rate-limited, constant-time response).
- `DELETE /v1/vprs/applications/{id}`: soft-delete (erasure request).
- `GET /healthz` + `/readyz` + `/openapi.json`.

Every write endpoint enforces `Idempotency-Key` (backed by `eel_idempotency_keys`). Responses are cosign-signed.

### 15.2 gRPC (proto/vprs.proto, port 8443)

- `SubmitApplication(application_data) -> ApplicationId`
- `GetApplicationStatus(application_id) -> ApplicationStatus`
- `VerifyWithIPRS(national_id, names) -> IPRSResult`

### 15.3 LiveView portal (ex/, port 4000)

Phoenix LiveView PWA. Routes: `/apply` (new application), `/status` (application tracker), `/change` (update request), `/challenge` (step-up OTP). WebSocket channel for real-time state updates.

---

## 16. Events (CloudEvents 1.0 via outbox)

### 16.1 Emitted

| Event type | Trigger |
|---|---|
| `vprs.application.submitted` | New application accepted |
| `vprs.application.iprs-verified` | IPRS verification succeeded |
| `vprs.application.approved` | Application approved, forwarded to bvr-svc |
| `vprs.application.rejected` | Application rejected (with reason code) |
| `vprs.application.updated` | Applicant updated details |
| `vprs.application.erased` | Soft-delete on erasure request |
| `vprs.change-request.initiated` | Continuous-register change started |

### 16.2 Consumed

| Event type | Source | Purpose |
|---|---|---|
| `bvr.voter.enrolled` | bvr-svc | Confirm enrolment; close application lifecycle |

---

## Summary

Total footprint: 2 Rust crates (vprs-api, vprs-domain), 1 Elixir LiveView portal (ex/), PostgreSQL schema (4 tables), Kafka outbox, Kustomize overlays for 7 sites. Core correctness guarantees: IPRS fail-closed circuit breaker, OPA-enforced freeze-window hard-close at transaction time, constant-time enumeration defence, WCAG 2.2 AA portal, DPIA-compliant log scrubber on rejection events.

---

## Appendix A: Deployment topology

`deploy/service.yaml` declares the full site matrix:

| Site | Strategy | Replicas |
|---|---|---|
| primary (Anniversary Towers) | blue-green | 3 |
| mirror (Bomas) | blue-green | 3 |
| edge-nairobi | rolling | 2 |
| edge-mombasa | rolling | 2 |
| edge-kisumu | rolling | 2 |
| edge-eldoret | rolling | 2 |
| edge-garissa | rolling | 2 |

Blue-green at primary and mirror ensures zero-downtime deploy with instant rollback: the inactive slot is kept warm, traffic shifted at the load balancer only after the health check passes. Rolling at edge sites keeps at least one replica available throughout the update.

ArgoCD ApplicationSet (`deploy/applicationset.yaml`) drives Kustomize overlay selection per site and environment. The image is `harbor.eel.internal/eel/vprs-svc`.

---

## Appendix B: Degradation contract

| Mode | Behaviour |
|---|---|
| Normal | Online application submission with live IPRS verification |
| Degraded | IPRS from 24-hour cache; application approval delayed pending manual check |
| Read-only | Application status tracking only; submission disabled; LiveView serves cached data |

Degradation is declared via the `eel_service_mode` config entry consumed by the axum middleware. Mode transitions are logged in `eel_service_mode_log` with `actor_id`, `from_mode`, `to_mode`, `reason`, `transitioned_at`.

---

## Appendix C: Application processing flow detail

The Rust event aggregator (`src/`) drives the following flow for each approved application on the `vprs.application.approved` outbox topic:

```
1. Kafka consumer picks up vprs.application.approved event
2. Aggregator fetches application record (SELECT FOR UPDATE)
3. Generates QR handoff payload: { application_id, approved_at, expires_at }
4. Signs payload with HSM Ed25519 key (role: vprs-handoff)
5. Stores QR code reference in applications.qr_ref
6. Publishes downstream notification (bvr-svc ingest queue)
7. Updates application_timeline: QR_GENERATED event
```

Expiry is 72 hours from `approved_at`. At expiry the `applications.qr_expired_at` field is set by the nightly sweep job. Applicants can request a fresh QR via `POST /v1/vprs/applications/{id}/refresh-qr`; this invalidates the prior QR by rotating the `qr_ref` reference. The prior QR's `expires_at` is set to `now()` to prevent concurrent use of both codes.

---

## Appendix D: Security threat model summary

| Threat | Control |
|---|---|
| Mass enumeration of registration status | Constant-time response + token bucket + step-up OTP |
| Replay of approved QR handoff codes | 72-hour expiry + Ed25519 signature over `{application_id, approved_at, expires_at}` |
| Freeze-window bypass (race condition) | OPA hard-close inside serialisable PG transaction |
| IPRS impersonation | mTLS with SPIRE SVID; server certificate pinned per environment |
| Audit log tampering | Append-only `application_timeline` with PG trigger blocking UPDATE/DELETE |
| Cross-border PII transfer | Act §71 authz required; no default cross-border replication |
| Session fixation on LiveView | Phoenix LiveView generates a new session token on every page load; prior token immediately invalidated |
| Stale QR code after data correction | QR `qr_ref` rotated on every `refresh-qr` call; prior QR `expires_at` set to now() |

---

## Appendix E: Key Prometheus metrics reference

| Metric | Type | Description |
|---|---|---|
| `vprs_application_submission_latency_seconds` | Histogram | End-to-end latency from HTTP receipt to DB write |
| `vprs_iprs_lookup_latency_seconds` | Histogram | gRPC round-trip to IPRS per lookup |
| `vprs_application_approval_rate` | Gauge | Fraction of submitted applications reaching Approved state |
| `vprs_iprs_cache_hit_rate` | Gauge | Cache hits as fraction of total IPRS lookups |
| `vprs_application_queue_depth` | Gauge | Applications in IprsVerifying state awaiting IPRS response |
| `vprs_liveview_concurrent_users` | Gauge | Active WebSocket sessions on the LiveView portal |
| `vprs_circuit_breaker_state` | Gauge (0/1/2) | 0=Closed, 1=Open, 2=HalfOpen per gRPC channel |
| `vprs_rate_limit_rejections_total` | Counter | Requests rejected by token-bucket limiter per route |
