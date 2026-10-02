# API Catalog — the Commission IEMS (All 69 Modules)

**Companion to:** `PRD.md`, `ARCHITECTURE.md`, `DATA-MODEL.md`.
**Version:** 2.0 — Full 69-module canonical reference.
**Status:** Pre-award bid specification; full OpenAPI 3.1 YAML delivered at `/api/` in source repository.

---

## 1. API Design Principles

The Commission IEMS API surface is **spec-first OpenAPI 3.1**. Every endpoint's OpenAPI YAML is the contract; implementation must conform to the spec, not the other way around. Semantic versioning (`MAJOR.MINOR.PATCH`) governs all schemas; breaking changes always increment MAJOR and land on a new URI version.

**Base conventions:**

- **URI versioning:** `/{service}/v{N}/{resource}`. The version prefix is mandatory and never omitted.
- **JSON encoding:** `Content-Type: application/json` for REST; `application/cloudevents+json` for event payloads.
- **Error model:** RFC 7807 Problem Details (`type`, `title`, `status`, `detail`, `instance`). All 4xx/5xx responses return a Problem Details body.
- **Pagination:** cursor-based using `after` / `before` query parameters; `limit` capped at 200. Response envelope: `{ data: [...], meta: { next_cursor, prev_cursor, total_count } }`.
- **Filtering + sorting:** `filter[field]=value` for equality; `sort=-created_at` (minus prefix for descending). Complex filters use a structured `q` parameter parsed server-side.
- **Idempotency:** all POST and PATCH requests that mutate state require `Idempotency-Key: <UUID7>` header. The server stores results for 24 hours and returns the cached response on duplicate keys with `Idempotency-Replayed: true`.
- **Correlation IDs:** `X-Request-Id` header mandatory on all requests. If absent, the gateway generates one. Propagated to every downstream service call and included in all log lines, traces, and error responses.
- **Rate-limit headers:** `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` returned on every response. Exceedance returns `429 Too Many Requests` with `Retry-After` header.
- **Deprecation lifecycle:** `Deprecation: <date>` and `Sunset: <date>` headers on deprecated endpoints from announcement date. Minimum 12 months from `Deprecation` to `Sunset`.
- **Timestamps:** RFC 3339 UTC in all request/response bodies (`2027-08-10T07:00:00Z`); displayed EAT in UIs.
- **IDs:** UUID7 strings everywhere — ordered, K-sortable, no sequential enumeration risk.
- **Monetary values:** integer cents (KES) in all request/response bodies, never floats.

---

## 2. Authentication and Authorization

### 2.1 Human principals

All human-facing API calls use OAuth 2.1 / OIDC via Keycloak. Token format: short-lived JWT (15-minute access token, 8-hour refresh). Token audience is the specific service (`aud: bvr-svc`, `aud: crms-svc`, etc.) — tokens are not cross-service.

MFA is mandatory for all roles above `PUBLIC`. Admin operations additionally require a dual-control countersignature from a second authorized user before the action is committed. The dual-control model is enforced in OPA policy, not in application code, making it auditable and testable independently of the service.

Role claims are embedded in the JWT via a custom `commission_roles` claim that lists `{role, scope_type, scope_id}` tuples. The OPA policy evaluator validates every request against the claim set.

### 2.2 Service-to-service

All back-end service-to-service calls use mTLS with SPIFFE/SPIRE-issued SVIDs (X.509 short-lived certificates, 1-hour TTL). No shared API keys or static secrets pass between services. The Linkerd service mesh enforces that only services presenting valid SVIDs from the same SPIRE trust domain can establish connections.

### 2.3 Kit-to-back-end

KIEMS kits authenticate with per-kit X.509 client certificates (issued by the internal PKI, bound to kit TPM). Kit certificates are short-lived (7 days, rotated automatically by the kit MDM). Certificate revocation propagates to all edge nodes within 60 seconds via CRL + OCSP.

Every kit API call additionally includes:
- `X-Kit-Serial`: kit serial number.
- `X-Kit-Attestation-Token`: latest TPM-bound attestation token (refreshed at kit boot).
- `X-Kit-Signature`: ECDSA-P384 signature over the request body + timestamp (prevents replay).

### 2.4 Public endpoints

Public read endpoints (results portal, verifier app, transparency log) require no authentication. They are served from CDN-cached signed snapshots; origin is rate-limited but not publicly accessible for direct unauthenticated writes.

### 2.5 HSM-anchored signing

Critical state changes (voter register gazette, result publication, declaration) require an HSM-produced signature from the authorised officer. These calls carry an `X-HSM-Signature` header containing the signature and the key ID used.

---

## 3. API Gateway Design

The API gateway is Envoy, deployed as both an edge proxy (north-south) and a sidecar (east-west via Linkerd). Key gateway concerns:

| Concern | Implementation |
|---|---|
| TLS termination | TLS 1.3 at edge; mTLS inside mesh |
| Rate limiting | Per-user + per-IP + per-endpoint token buckets in Redis; separate buckets for kit, public, and admin traffic |
| DDoS protection | Upstream scrubbing (Cloudflare / Akamai) for public-facing surface; Envoy circuit breaker for back-end protection |
| Request signing verification | Gateway verifies `X-Kit-Signature` on all kit-originated calls before routing |
| WAF | L7 WAF rules on public endpoints (SQLi, XSS, path traversal) |
| Correlation ID injection | Added if absent; propagated to all upstream headers |
| Observability | OpenTelemetry span created per request at gateway; trace context propagated |
| JWT validation | Gateway validates signature, expiry, audience; role claim extracted and passed downstream as `X-Verified-Claims` header |

Rate limit tiers:

| Surface | Limit | Notes |
|---|---|---|
| Public portal read | Unlimited (CDN-served) | Origin protected by upstream scrubbing |
| VPRS public API | 60 req/min/IP + 10 req/min/session | Captcha on write |
| BVV public lookup | 30 req/min/IP | Prevent bulk register scraping |
| Verifier app public | 100 req/s/IP | CDN-fronted |
| Kit mTLS APIs | 10 req/s/kit | Per-kit budget |
| Admin APIs | 100 req/min/user | Higher for bulk ops with dual-control |
| Webhook inbound | 500 req/s/provider | Per-provider allowlist |

---

## 4. Endpoint Catalogue by Module

### Standard health / observability endpoints (all modules)

Every deployable service exposes these at `/{service}/`:

| Path | Method | Auth | Notes |
|---|---|---|---|
| `/health` | GET | None | Liveness probe; returns `{ status: "ok" }` |
| `/ready` | GET | None | Readiness probe; checks DB + dependency connectivity |
| `/metrics` | GET | Internal | Prometheus format; not publicly routable |
| `/version` | GET | None | `{ version, build_sha, deployed_at }` |
| `/audit` | GET | Admin + MFA | Last 100 admin actions on this service |

---

### M01 — BVR Device Client

Device-side only. M01 is a Flutter app on the KIEMS kit; it does not expose server-side endpoints. It consumes M02 for submission and MDM for configuration.

**Kit → server calls (outbound):**
- `POST /bvr/v1/enrolments` (M02) — submit enrolment record.
- `GET /mdm/v1/kits/{kit_id}/config` (MDM) — fetch config on boot.
- `POST /pki/v1/attestation` (PKI) — submit boot attestation.
- `POST /bvr/v1/voters/{voter_id}/transfer` (M02) — transfer workflow.

---

### M02 — BVR Backend

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/bvr/v1/enrolments` | Kit mTLS | 10/s/kit | Submit enrolment + biometrics; triggers ABIS |
| GET | `/bvr/v1/enrolments/{id}` | RO+MFA | 200/hr/user | Retrieve enrolment record |
| GET | `/bvr/v1/voters/{voter_id}` | RO+MFA / Auditor | 200/hr/user | PII redacted per role |
| PATCH | `/bvr/v1/voters/{voter_id}/status` | RO+MFA+dual | 20/hr | `ACTIVE`, `DECEASED`, `DELETED`, `SUSPENDED`, `TRANSFERRED` |
| POST | `/bvr/v1/voters/{voter_id}/transfer` | Kit mTLS + biometric | 10/s/kit | Transfer between stations |
| POST | `/bvr/v1/register/generate` | RO+MFA | 10/hr | Generate register at scope (station/ward/const/county/national) |
| POST | `/bvr/v1/register/gazette` | Admin+MFA+dual+HSM | 2/day | Freeze register at gazetted cut-off |
| POST | `/bvr/v1/register/exports` | Admin+MFA+dual | 5/day | CSV / PDF bulk export |

**Kafka events produced:** `voter.enrolled`, `voter.updated`, `voter.transferred`, `register.gazetted`.

**Idempotency:** all POST calls require `Idempotency-Key`. Enrolment deduplication is by `voter_id` (UUID7 stable across retries for same capture session).

**SLA:** enrolment ingestion P50 ≤ 200 ms, P99 ≤ 2 s. ABIS trigger is async (Kafka); result returned via webhook or polling.

---

### M03 — ABIS Integrator

ABIS is invoked by M02 via Kafka; it does not expose a public REST surface. Internal gRPC service:

| gRPC method | Purpose |
|---|---|
| `SubmitTemplate` | Ingest biometric template from M02; queue for 1:N search |
| `GetMatchResult` | Poll match result by submission ID |
| `GetAdjudicationQueue` | List pending investigator cases |
| `SubmitAdjudicationDecision` | Human investigator resolves duplicate flag |
| `GetSLAStatus` | Match queue depth + p99 latency |

Adjudication workbench is a web UI backed by these gRPC calls, not REST-exposed.

---

### M04 — VPRS (Voter Pre-Registration)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/vprs/v1/applications` | Public+reCAPTCHA | 60/min/IP | Create pre-registration application |
| POST | `/vprs/v1/applications/{id}/verify-iprs` | Public+session | 10/min/session | Trigger IPRS identity check |
| POST | `/vprs/v1/applications/{id}/otp` | Public+session | 3/min/session | Request SMS OTP |
| POST | `/vprs/v1/applications/{id}/otp/verify` | Public+session | 5/min/session | Confirm OTP |
| POST | `/vprs/v1/applications/{id}/submit` | Public+session | 3/min/session | Finalise application |
| GET | `/vprs/v1/applications/{id}/qr` | Public+session | 20/min/session | KIEMS device pick-up QR |
| GET | `/vprs/v1/applications/{id}` | Public(owner via OTP) / Admin | 30/min | Read application status |
| GET | `/vprs/v1/applications/handoff/{token}` | Kit mTLS | 10/s/kit | KIEMS clerk pulls pre-registered form |
| GET | `/vprs/v1/admin/applications` | Admin+MFA | 100/hr | List + search applications |
| GET | `/vprs/v1/admin/analytics` | Admin+MFA | 20/hr | Registration analytics |

**Events produced:** `vprs.application_submitted`, `vprs.application_converted`.

---

### M05 — BVI Device Client

Device-side only. Consumes M06 for session management and identification logging.

**Kit → server calls (outbound):**
- `POST /bvi/v1/sessions/{station_id}/open` (M06)
- `POST /bvi/v1/identifications` (M06)
- `POST /bvi/v1/sessions/{station_id}/close` (M06)
- `GET /bvi/v1/kits/{kit_id}/config-bundle` (M06)

---

### M06 — BVI Backend

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/bvi/v1/sessions/{station_id}/open` | Kit mTLS+PO biometric | 1/session | Open polling session; HSM-countersigned |
| POST | `/bvi/v1/sessions/{station_id}/close` | Kit mTLS+PO biometric+dual | 1/session | Normal close |
| POST | `/bvi/v1/sessions/{station_id}/close/emergency` | Kit mTLS+PO biometric | 1/session | Emergency close per Elections Act §55 |
| POST | `/bvi/v1/identifications` | Kit mTLS | 10/s/kit | Log voter identification (voter_id + outcome; no biometric in transit) |
| GET | `/bvi/v1/sessions/{station_id}/status` | Various roles+MFA | 100/hr | Live station status |
| POST | `/bvi/v1/kits/{kit_id}/wipe` | Admin+MFA+dual | 2/hr | Remote wipe rogue kit |
| GET | `/bvi/v1/kits/{kit_id}/config-bundle` | Kit mTLS | 10/s/kit | Station-specific config + register extract |
| GET | `/bvi/v1/turnout/{scope_type}/{scope_id}` | Various roles | 1000/hr | Live turnout by area |

**Kafka events:** `session.opened`, `session.closed`, `voter.authenticated`, `voter.distress`, `kit.wiped`.

**SLA:** identification ingestion P99 ≤ 100 ms. Config-bundle fetch P99 ≤ 500 ms.

---

### M07 — BVV (Biometric Voter Verification Portal)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/bvv/v1/register/lookup` | Public+rate-limit+captcha | 30/min/IP | Voter looks up own register entry |
| POST | `/bvv/v1/register/objection` | Public+OTP | 5/day/voter | File objection on register entry |
| GET | `/bvv/v1/register/objection/{id}` | Public(owner) / Admin | 30/min | Track objection status |
| POST | `/bvv/v1/kiosk/lookup` | Kiosk mTLS | 10/s/kiosk | In-person lookup at the Commission kiosk |

---

### M08 — RTS Device Client

Device-side only. Consumes M10 for result form submission.

**Kit → server calls (outbound):**
- `POST /rts/v1/form34a` (M10) — submit scanned Form 34A.
- `POST /rts/v1/sms-fallback/outbound` — SMS channel fallback submission.

---

### M09 — RTS Returning Officer Tablet

Constituency tally officer interface. Consumes M10 for Form 34B generation and signing. No server-side endpoints; tablet is a thin client.

---

### M10 — RTS Backend

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/rts/v1/form34a` | Kit mTLS+PO signature | 10/s/kit | Submit scanned Form 34A + digital signature |
| GET | `/rts/v1/form34a/{id}` | RO / Auditor / Public (after publish) | unlimited | Form + Merkle inclusion proof |
| POST | `/rts/v1/form34a/{id}/agent-countersign` | Party Agent+MFA | 20/hr | Party agent countersignature |
| POST | `/rts/v1/form34a/{id}/human-confirm` | RO+MFA | 20/hr | Confirm ICR extraction |
| POST | `/rts/v1/form34b` | RO+MFA+dual | 10/hr | Generate constituency form |
| POST | `/rts/v1/form34b/{id}/sign` | RO+MFA+biometric+HSM | 5/hr | Sign Form 34B |
| POST | `/rts/v1/form34c` | CRO+MFA+dual | 5/hr | Generate county / national form |
| POST | `/rts/v1/publish/snapshot` | the Commission Commissioner+MFA+M-of-N+HSM | 5/election | Publish results snapshot |
| GET | `/rts/v1/parallel-tally/status` | Admin / Auditor | 100/hr | Independent parallel-tally comparison |
| GET | `/rts/v1/kits/{kit_id}/retry-queue` | Support+MFA | 50/hr | Inspect kit's pending transmissions |
| POST | `/rts/v1/sms-fallback/inbound` | Telco webhook+HMAC | 500/s | Receive SMS-fallback form submission |

**Kafka events:** `form34a.submitted`, `form34a.verified`, `form34a.published`, `form34b.generated`, `form34c.generated`, `snapshot.published`.

**Async / sync:** Form 34A submission is synchronous for ingestion acknowledgement; ICR extraction and Merkle proof generation are async (Kafka). The kit polls `GET /rts/v1/form34a/{id}` for final status.

---

### M11 — Public Results Portal

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/portal/v1/elections` | Public (CDN) | unlimited | List elections |
| GET | `/portal/v1/elections/{id}/results` | Public (CDN) | unlimited | National results |
| GET | `/portal/v1/elections/{id}/counties/{county_id}` | Public (CDN) | unlimited | County results |
| GET | `/portal/v1/elections/{id}/constituencies/{const_id}` | Public (CDN) | unlimited | Constituency results |
| GET | `/portal/v1/elections/{id}/stations/{station_id}` | Public (CDN) | unlimited | Station result + Form 34A + Merkle proof |
| GET | `/portal/v1/elections/{id}/download/{form_type}` | Public+rate-limit | 5/min/IP | Bulk download signed forms |
| GET | `/portal/v1/transparency-log/root/{index}` | Public | unlimited | Merkle root at index |
| GET | `/portal/v1/transparency-log/proof/{event_id}` | Public | 50/min/IP | Inclusion proof for event |

All responses signed by the Commission HSM. CDN-cached signed snapshots; origin is not directly accessible for public reads.

---

### M12 — CRMS (Candidate Registration)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/crms/v1/party/nominations` | Party Official+MFA | 20/hr | Submit candidate nomination |
| GET | `/crms/v1/party/nominations/{id}` | Party Official+MFA | 200/hr | View nomination |
| PATCH | `/crms/v1/party/nominations/{id}` | Party Official+MFA | 20/hr | Update while status permits |
| POST | `/crms/v1/party/nominations/{id}/documents` | Party Official+MFA | 50/hr | Upload nomination documents |
| GET | `/crms/v1/ro/clearance-queue` | RO+MFA | 200/hr | RO clearance queue |
| POST | `/crms/v1/ro/nominations/{id}/clear` | RO+MFA+dual | 20/hr | Approve nomination |
| POST | `/crms/v1/ro/nominations/{id}/reject` | RO+MFA+reason | 20/hr | Reject with reason |
| GET | `/crms/v1/nominations/{id}/ballot-proof` | Admin | 10/hr | Generate ballot proof PDF |
| POST | `/crms/v1/gazette/close-nominations` | Admin+M-of-N+HSM | 1/election | Gazette candidate list |

**Events:** `candidate.submitted`, `candidate.cleared`, `candidate.gazetted`.

---

### M13 — Public Verifier App

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/stations/{code}/result` | Public | 100/s/IP | Tally JSON + hash + Merkle path |
| GET | `/api/v1/stations/{code}/form34a.pdf` | Public | 20/s/IP | Signed PDF |
| GET | `/api/v1/stations/{code}/queue` | Public | 60/s/IP | Queue status from M19 |
| POST | `/api/v1/participation/verify` | Public | 10/s/IP | `{token_hash, station_code}` — verify participation token |
| GET | `/api/v1/merkle/root` | Public | 1000/s | Current published root + timestamp |
| GET | `/api/v1/merkle/path/{form_hash}` | Public | 50/s/IP | Merkle inclusion proof |
| POST | `/api/v1/grievances` | Public | 5/s/IP | Submit grievance; returns ticket_id (routed to M23) |
| GET | `/api/v1/grievances/{ticket_id}` | Ticket token | 20/s | Grievance status |
| GET | `/api/v1/datasets/{constituency_code}` | Public | 5/s/IP | CSV + JSON bulk download |
| GET | `/api/v1/integrity/hash` | Public | 1000/s | Published app binary hash |
| POST | `/webhooks/whatsapp` | HMAC-SHA256 | 500/s | WhatsApp Business API inbound |
| POST | `/webhooks/ussd` | API key | 200/s | Aggregator USSD callback |
| GET | `/api/v1/stations/{code}/result/stream` | Public | 100/s/IP | Server-Sent Events live tally stream |

---

### M14 — Reproducibility Bundle

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/reproducibility/v1/bundles` | Public | 100/min | List published bundles |
| GET | `/reproducibility/v1/bundles/{id}` | Public | 200/min | Bundle manifest + download links |
| GET | `/reproducibility/v1/bundles/{id}/download` | Public | 10/min/IP | Download signed bundle archive |
| POST | `/reproducibility/v1/bundles` | Admin+M-of-N+HSM | 5/election | Publish new bundle |
| GET | `/reproducibility/v1/participation-tokens/{station_code}` | Public (post-publication) | 50/min/IP | Station token count + hash |
| GET | `/reproducibility/v1/verify/{form_hash}` | Public | 100/min/IP | Verify form hash against published bundle |

**Events produced:** `bundle.published` (triggers escrow mirror sync, IPFS pinning, GitHub push, Gazette export).

---

### M15 — IAB Dashboard

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/iab/v1/feed` | IAB Member+MFA | 200/hr | Hash-verified signed event feed |
| GET | `/iab/v1/alerts` | IAB Member+MFA | 200/hr | Active alerts from M26, M16 |
| POST | `/iab/v1/escalations` | IAB Member+MFA | 20/hr | File escalation |
| GET | `/iab/v1/escalations/{id}` | IAB Member / Admin | 100/hr | Track escalation + the Commission response |
| POST | `/iab/v1/complaints` | IAB Member+MFA | 20/hr | File formal complaint |
| GET | `/iab/v1/reports/live` | IAB Member+MFA | 100/hr | Live election dashboard (read-only mirror) |

IAB feed is a signed read-only mirror of AuditEvent stream; no write path from IAB into operational systems.

---

### M16 — Election Command Centre

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/ecc/v1/dashboard` | the Commission Commissioner / ICT Ops | 100/min | National situational view |
| GET | `/ecc/v1/kits/telemetry` | War-room | 100/min | Fleet telemetry summary |
| GET | `/ecc/v1/alerts` | War-room | 100/min | Active P0–P3 alerts |
| POST | `/ecc/v1/alerts/{id}/acknowledge` | War-room | 200/min | Acknowledge alert |
| POST | `/ecc/v1/alerts/{id}/resolve` | War-room | 200/min | Resolve alert |
| GET | `/ecc/v1/stream` | War-room | 10/user | SSE stream of real-time events from all modules |
| POST | `/ecc/v1/s44a/recommend` | the Commission Commissioner+MFA | 5/hr | Recommend polling extension per Elections Act §44A |
| GET | `/ecc/v1/counties/{county_code}/status` | War-room | 200/min | County-level drill-down |

ECC SSE stream aggregates events from M06, M19, M26, M10, and all other modules that emit on `security.incidents`.

---

### M17 — Speech Integrity

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/speech/v1/flags` | Analyst+MFA | 200/hr | List content flags |
| POST | `/speech/v1/flags` | System (auto-detected) / Analyst | 100/hr | Create flag |
| PATCH | `/speech/v1/flags/{id}` | Analyst+MFA | 100/hr | Set analyst decision |
| POST | `/speech/v1/flags/{id}/referral` | Senior Analyst+MFA | 20/hr | Refer to NCIC / DCI |
| GET | `/speech/v1/incidents` | Analyst+MFA | 200/hr | Safety incidents |
| POST | `/speech/v1/incidents` | Analyst+MFA | 50/hr | Create safety incident |
| POST | `/speech/v1/webhooks/platform` | Platform HMAC | 500/s | Inbound platform content reports |
| GET | `/speech/v1/safety-tab` | Public (M13 integration) | 1000/s | Current safety situation for citizen display |

---

### M18 — Boundary Transparency

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/boundary/v1/boundaries` | Public | 200/min | List all boundaries (filterable by type) |
| GET | `/boundary/v1/boundaries/{code}` | Public | 200/min | Boundary detail + GeoJSON + Merkle proof |
| GET | `/boundary/v1/boundaries/{code}/changes` | Public | 100/min | History of boundary changes |
| PUT | `/boundary/v1/boundaries/{code}` | Admin+MFA+dual+HSM | 5/day | Update boundary (versioned; gazette required) |
| GET | `/boundary/v1/stations/{code}/polygon` | Kit / Admin | 1000/s | Fetch geo-fence polygon for kit |
| POST | `/boundary/v1/submissions` | Public | 10/day/IP | Submit boundary objection (public consultation) |
| GET | `/boundary/v1/audit` | Auditor+MFA | 100/hr | Boundary change audit log |

**Events produced:** `boundary.updated` (consumed by all map-consuming modules including M01, M05, M18, M44).

---

### M19 — Queue Transparency

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/queue/v1/snapshots` | Kit mTLS | 10/s/kit | Submit queue snapshot (every 15 min) |
| GET | `/queue/v1/stations/{code}/current` | Public | 1000/s | Current queue status |
| GET | `/queue/v1/stations/{code}/history` | Admin | 200/hr | Historical snapshots |
| POST | `/queue/v1/attendance` | Kit mTLS | 1/15min/kit | Attendance snapshot |
| GET | `/queue/v1/alerts` | War-room (M16) | 200/min | Stations with RED / CRITICAL queue status |

**Kafka events:** `queue.snapshot`, `queue.alert`.

---

### M20 — Inventory Management

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/inventory/v1/kits` | Ops+MFA | 200/hr | List kits (filter by status, county) |
| GET | `/inventory/v1/kits/{kit_id}` | Ops+MFA | 200/hr | Kit detail |
| POST | `/inventory/v1/kits/{kit_id}/custody` | Ops+MFA+biometric | 20/hr | Custody hand-off event |
| POST | `/inventory/v1/kits/{kit_id}/seals` | Ops+MFA+biometric | 20/hr | Apply tamper seal |
| GET | `/inventory/v1/kits/{kit_id}/health` | Ops / War-room | 200/min | Live health telemetry |
| GET | `/inventory/v1/seals/{seal_id}/verify` | Public / Ops | 1000/s | QR-verifiable seal lookup |
| GET | `/inventory/v1/assets` | Ops+MFA | 200/hr | Asset register |
| POST | `/inventory/v1/assets/{uid}/custody` | Ops+MFA | 50/hr | Asset custody event |
| GET | `/inventory/v1/facilities` | Ops+MFA | 100/hr | Facility list |
| GET | `/inventory/v1/reports/readiness` | Admin+MFA | 20/hr | Election-readiness report |
| GET | `/inventory/v1/spares` | FieldEngineer+JWT | 200/hr | Spares availability by SKU and nearest facility; used by M32 Mobile Field Support to locate replacement parts |
| GET | `/inventory/v1/spares/{sku}` | FieldEngineer+JWT | 200/hr | Quantity on hand per facility for one SKU |

**Kafka events:** `kit.custody`, `asset.custody`.

---

### M21 — RLA Audit Tool

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/rla/v1/runs` | the Commission Commissioner+MFA+dual | 2/election | Create audit run |
| POST | `/rla/v1/runs/{id}/draw-seed` | Multi-party ceremony (public event) | 1/run | Draw seed from 5 independent sources |
| GET | `/rla/v1/runs/{id}/sample` | Auditor+MFA | 100/hr | Sampled ballot list |
| POST | `/rla/v1/runs/{id}/ballot-submissions` | Auditor+MFA | 200/hr | Submit physical count for sampled ballot |
| GET | `/rla/v1/runs/{id}/status` | Public | 1000/hr | Audit run status + progress |
| GET | `/rla/v1/runs/{id}/manifest/verify` | Public | 200/hr | Verify audit manifest hash |
| GET | `/rla/v1/entropy-sources` | Public | 200/hr | Published entropy sources for current seed draw |
| POST | `/rla/v1/runs/{id}/conclusion` | the Commission Commissioner+MFA+dual | 1/run | Sign and publish audit conclusion |
| GET | `/rla/v1/bug-bounty` | Public | 100/hr | Public bug bounty submissions |
| POST | `/rla/v1/bug-bounty` | Public | 5/day/IP | Submit bug bounty finding |
| GET | `/rla/v1/runs/{id}/sampled-ballots/{ballot_id}` | Auditor+MFA | 100/hr | Individual sampled ballot detail |

---

### M22 — Campaign Finance Transparency

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/finance/v1/disclosures` | Candidate Portal+MFA | 20/hr | Submit financial disclosure |
| GET | `/finance/v1/disclosures/{id}` | Party Official / Auditor / Public (post-publish) | 200/hr | Disclosure detail |
| PATCH | `/finance/v1/disclosures/{id}` | Candidate Portal+MFA | 20/hr | Update disclosure (before deadline) |
| GET | `/finance/v1/public/heat-map` | Public | 1000/hr | Aggregate spend heat map (no individual data) |
| POST | `/finance/v1/tips` | Public+OTP | 10/day/IP | Submit bribery tip (encrypted) |
| POST | `/finance/v1/evidence-packages` | Senior Analyst+MFA+dual | 5/hr | Generate ODPP referral package |
| GET | `/finance/v1/evidence-packages/{id}` | Auditor / ODPP | 20/hr | Download evidence package |
| GET | `/finance/v1/aggregate/{constituency_code}` | Public | 200/hr | Constituency cash movement (z-score, no PII) |
| GET | `/finance/v1/reports/compliance` | Auditor+MFA | 20/hr | Ceiling compliance report |
| POST | `/finance/v1/webhooks/cbk` | CBK HMAC | 100/s | Inbound CBK financial intelligence |
| POST | `/finance/v1/webhooks/frc` | FRC HMAC | 100/s | Inbound FRC data feed |
| GET | `/finance/v1/candidates/{id}/disclosure-status` | RO+MFA | 200/hr | Clearance check |

---

### M23 — Grievance Channels

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/grievances/v1/cases` | Public / Kit kiosk | 10/min/IP | File grievance |
| GET | `/grievances/v1/cases/{id}` | Public (via ticket token) / Admin | 200/hr | Track case |
| PATCH | `/grievances/v1/cases/{id}` | Case Officer+MFA | 100/hr | Update case status |
| POST | `/grievances/v1/cases/{id}/escalate` | Case Officer+MFA | 20/hr | Escalate to senior / DCI |
| GET | `/grievances/v1/analytics` | Admin+MFA | 20/hr | Grievance analytics |
| POST | `/grievances/v1/intake/whatsapp` | WhatsApp HMAC | 500/s | WhatsApp intake webhook |
| POST | `/grievances/v1/intake/ussd` | Telco API key | 500/s | USSD intake webhook |
| POST | `/grievances/v1/intake/sms` | Telco HMAC | 500/s | SMS intake webhook |
| POST | `/grievances/v1/cases/{id}/referral` | Senior Officer+MFA | 10/hr | Refer to ODPP / KNPS |
| GET | `/grievances/v1/cases/{id}/evidence` | Case Officer+MFA | 50/hr | Evidence files (encrypted) |
| GET | `/grievances/v1/live-count` | Public | 1000/s | Anonymised live grievance count by category |

---

### M24 — Diaspora Voting

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/diaspora/v1/embassies` | Public | 200/min | List embassy stations + opening hours |
| POST | `/diaspora/v1/embassies/{code}/roll` | Embassy Officer+mTLS | 5/day | Upload diaspora voter roll |
| GET | `/diaspora/v1/embassies/{code}/roll` | Embassy Officer+mTLS | 100/hr | Download diaspora voter roll |
| POST | `/diaspora/v1/forms/34ad` | Embassy Kit mTLS | 10/s/kit | Submit diaspora Form 34AD |
| GET | `/diaspora/v1/forms/34ad/{id}` | Embassy Officer / Auditor | 100/hr | Form + Merkle proof |
| POST | `/diaspora/v1/pouches/{id}/custody` | Diplomatic Officer+biometric | 20/hr | Ballot pouch custody event |
| GET | `/diaspora/v1/pouches/{id}/chain` | Auditor | 100/hr | Full custody chain for pouch |
| POST | `/diaspora/v1/pouches/{id}/receive` | the Commission HQ Officer+biometric | 10/hr | Receive pouch at the Commission HQ |
| GET | `/diaspora/v1/status/{voter_id_hash}` | Public+OTP | 30/hr | Voter participation status check |
| POST | `/diaspora/v1/webhooks/pouch-alert` | Logistics Provider HMAC | 100/s | Physical logistics tracking webhook |
| GET | `/diaspora/v1/results/34ad` | Public (post-publish) | 200/hr | Published diaspora results |

---

### M25 — Special Voter Workflows

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/special/v1/facilities` | Public | 200/hr | List special facilities (hospitals, prisons, IDP) |
| POST | `/special/v1/facilities` | Admin+MFA+dual | 10/hr | Register special facility |
| GET | `/special/v1/facilities/{id}` | Public | 200/hr | Facility detail + assigned officer |
| POST | `/special/v1/events` | Special Facility Officer+mTLS | 10/s | Log special polling event |
| GET | `/special/v1/events/{id}` | RO+MFA / Auditor | 100/hr | Event detail |
| POST | `/special/v1/idp/reassignments` | RO+MFA+dual | 20/hr | IDP voter station reassignment |
| GET | `/special/v1/forms/34` | RO+MFA | 50/hr | Special facility Form 34 variants |
| POST | `/special/v1/forms/34` | Special Kit mTLS | 5/s/kit | Submit special facility form |
| GET | `/special/v1/reports/coverage` | Admin+MFA | 10/hr | Coverage report |
| POST | `/special/v1/webhooks/facility-status` | Facility Officer | 100/s | Facility status webhook |
| GET | `/special/v1/accessible-design` | Public | unlimited | Accessibility resources for special voters |

---

### M26 — Anomaly and Fraud Detection

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/detection/v1/alerts` | Analyst+MFA | 200/hr | Active anomaly alerts |
| GET | `/detection/v1/alerts/{id}` | Analyst+MFA | 200/hr | Alert detail + contributing signals |
| PATCH | `/detection/v1/alerts/{id}` | Analyst+MFA | 100/hr | Set resolution |
| GET | `/detection/v1/model-cards` | Public | 200/hr | Published AI model cards for detection models |
| GET | `/detection/v1/cryptographic` | Public+Auditor | 100/hr | Cryptographic anomaly check results |
| GET | `/detection/v1/cross-source/{constituency_code}` | Analyst+MFA | 50/hr | Cross-source comparison (M10 vs M09 vs M13) |
| POST | `/detection/v1/telemetry` | Kit mTLS (via M06) | 10/s/kit | Ingest telemetry packet |
| GET | `/detection/v1/thresholds` | Admin+MFA | 20/hr | Current detection thresholds |
| PATCH | `/detection/v1/thresholds` | Admin+MFA+dual | 5/hr | Update thresholds (dual-commissioner) |
| POST | `/detection/v1/override` | the Commission Commissioner+MFA+dual | 2/hr | Human override of detection suppression |
| GET | `/detection/v1/distributions` | Analyst+MFA | 100/hr | Statistical distribution analysis results |

---

### M27 — Intent Preservation

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/intent/v1/pre-mark-events` | Kit mTLS | 10/s/kit | Submit pre-mark validation event |
| GET | `/intent/v1/rejection-rates` | RO+MFA | 200/hr | Rejection rate by station + contest |
| GET | `/intent/v1/rejection-rates/{station_code}` | RO+MFA / Public (post-poll) | 100/hr | Station-level rejection analysis |
| POST | `/intent/v1/spoilt-events` | Kit mTLS | 5/s/kit | Log spoilt ballot event |
| GET | `/intent/v1/adjudication` | PO+MFA | 50/hr | Pending adjudication queue |
| GET | `/intent/v1/adjudication/{id}` | PO+MFA | 100/hr | Adjudication case detail |
| GET | `/intent/v1/adjudication/{id}/photo` | PO+MFA | 20/hr | Spoilt ballot photo (encrypted) |
| POST | `/intent/v1/adjudication/{id}/decision` | PO+MFA+Agent countersign | 50/hr | Record adjudication decision |
| GET | `/intent/v1/alerts` | War-room | 200/hr | Stations with rejection rate anomalies |
| GET | `/intent/v1/reports/station/{code}` | RO+MFA | 50/hr | Station intent-preservation report |

---

### M28 — AI Monitoring Stack

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/ai/v1/models` | Public | 200/hr | Published model registry |
| POST | `/ai/v1/models` | ML Engineer+MFA+dual | 10/hr | Register new AI model |
| GET | `/ai/v1/models/{id}` | Public | 200/hr | Model card detail |
| GET | `/ai/v1/models/{id}/decisions` | Auditor+MFA | 100/hr | Decision log for model |
| GET | `/ai/v1/disinformation/flags` | Analyst+MFA | 200/hr | Disinformation detection flags |
| PATCH | `/ai/v1/disinformation/flags/{id}` | Analyst+MFA | 100/hr | Review flag |
| GET | `/ai/v1/pii-stripping` | Admin+MFA | 50/hr | PII stripping audit log |
| POST | `/ai/v1/pii-stripping/verify` | Auditor+MFA | 20/hr | Verify dataset is PII-clean |
| GET | `/ai/v1/bias-reports` | Public | 100/hr | Published bias evaluation reports |
| POST | `/ai/v1/bias-reports` | ML Engineer+MFA | 10/hr | Submit bias evaluation |
| GET | `/ai/v1/performance` | Admin+MFA | 100/hr | Real-time model performance metrics |
| GET | `/ai/v1/adversarial-tests` | Auditor+MFA | 50/hr | Adversarial robustness test results |
| POST | `/ai/v1/adversarial-tests` | ML Engineer+MFA+dual | 5/hr | Submit adversarial test |
| GET | `/ai/v1/model-cards/public` | Public | unlimited | Public model card publication page |

---

### M29-M34 — Mobile Apps (Citizen, Party Agent, Poll Clerk, Field Support, Personnel Dispatch, Observer)

Mobile apps are clients; they expose no server-side endpoints. Each app calls the following back-end APIs:

| App | Primary back-ends consumed |
|---|---|
| M29 Mobile Citizen Verify | M04 (VPRS), M11 (Portal), M13 (Verifier), M19 (Queue), M23 (Grievance) |
| M30 Mobile Party Agent | M10 (RTS), M06 (BVI status), M23 (Grievance), M22 (Finance disclosure) |
| M31 Mobile Poll Clerk | M06 (BVI config-bundle), M10 (form submission), M19 (queue) |
| M32 Mobile Field Support | M20 (Inventory), M16 (ECC alerts), O02 (Support tickets) |
| M33 Mobile Personnel Dispatch | M20 (Inventory), O05 (Dispatch), M55 (HR/Payroll) |
| M34 Mobile Observer | M43 (Accreditation), M10 (Results), M23 (Grievance), M15 (IAB) |

Mobile apps authenticate via Keycloak OIDC with PKCE on public clients. Local SQLite state is encrypted with per-app keystore key; no PII cached beyond session timeout.

---

### M35 — CVR (Continuous Voter Register)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/cvr/v1/batches/crs` | CRS System mTLS | 5/day | Ingest CRS death-notification batch |
| POST | `/cvr/v1/batches/nrb` | NRB System mTLS | 5/day | Ingest NRB ID-correction batch |
| POST | `/cvr/v1/batches/kra` | KRA System mTLS | 5/quarter | Ingest KRA dormancy flags |
| GET | `/cvr/v1/events` | the Commission Officer+MFA | 200/hr | Triage queue (proposed removals) |
| PATCH | `/cvr/v1/events/{id}` | the Commission Officer+MFA | 100/hr | Approve / reject / hold event |
| GET | `/cvr/v1/snapshots` | the Commission Admin+MFA | 20/hr | List register snapshots |
| POST | `/cvr/v1/snapshots` | the Commission Commissioner+HSM | 2/day | Create and sign new snapshot |
| GET | `/cvr/v1/snapshots/{id}/proof` | Court Registrar+JWT | 10/hr | Merkle proof for snapshot |
| POST | `/cvr/v1/family-notifications` | USSD Gateway secret | 1000/hr | `*400#` family death report |
| GET | `/cvr/v1/status/{national_id_hash}` | Public+rate-limit | 30/hr | Voter self-check (status + last updated; no flags) |

**Events produced:** `voter.register_event`, `register.snapshot_signed`.

---

### M36 — Voter Deregistration Appeal

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/deregistration/v1/cases` | the Commission Officer+MFA | 200/hr | Deregistration case queue |
| GET | `/deregistration/v1/cases/{id}` | the Commission Officer / Affected Voter+OTP | 100/hr | Case detail |
| PATCH | `/deregistration/v1/cases/{id}` | the Commission Officer+MFA | 100/hr | Update status |
| POST | `/deregistration/v1/cases/{id}/appeal` | Public+OTP | 3/case | File appeal |
| GET | `/deregistration/v1/cases/{id}/appeal` | the Commission Officer / Voter+OTP | 100/hr | Appeal status |
| POST | `/deregistration/v1/cases/{id}/decision` | Senior Officer+MFA+dual | 20/hr | Record adjudication decision |
| GET | `/deregistration/v1/reports/{id}` | Admin+MFA | 20/hr | Quarterly deregistration report |
| POST | `/deregistration/v1/reports` | Admin+MFA | 4/year | Generate quarterly report |
| GET | `/deregistration/v1/notices/{voter_id_hash}` | Public+OTP | 10/hr | Check if notice was dispatched |
| POST | `/deregistration/v1/reinstate/{id}` | Commissioner+MFA+dual | 5/hr | Reinstate voter |
| GET | `/deregistration/v1/analytics` | Admin+MFA | 20/hr | Deregistration analytics |

---

### M37 — Voter Education CMS

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/cms/v1/content` | Admin+MFA | 200/hr | List all content items |
| POST | `/cms/v1/content` | Content Editor+MFA | 50/hr | Create content item |
| PATCH | `/cms/v1/content/{id}` | Content Editor+MFA | 50/hr | Update content |
| POST | `/cms/v1/content/{id}/approve` | Senior Editor+MFA | 20/hr | Approve for publication |
| GET | `/cms/v1/content/public` | Public | unlimited | Published content (CDN-cached) |
| POST | `/cms/v1/distribution/{id}/whatsapp` | Admin+MFA | 50/hr | Distribute via WhatsApp broadcast |
| POST | `/cms/v1/distribution/{id}/sms` | Admin+MFA | 50/hr | Distribute via SMS blast |
| POST | `/cms/v1/distribution/{id}/physical` | Admin+MFA | 20/hr | Dispatch for physical printing |
| GET | `/cms/v1/fact-checks` | Public | 200/hr | Fact-check registry |
| POST | `/cms/v1/fact-checks` | Analyst+MFA | 20/hr | Submit fact-check review |

---

### M38 — Poll Worker Training LMS

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/lms/v1/modules` | All authenticated | 200/hr | Available training modules |
| POST | `/lms/v1/modules` | Admin+MFA | 20/hr | Create module |
| POST | `/lms/v1/enrollments` | Staff+MFA | 50/hr | Enroll in training |
| GET | `/lms/v1/enrollments` | Staff+MFA | 200/hr | My enrollments |
| POST | `/lms/v1/enrollments/{id}/progress` | Staff+MFA | 100/hr | Update progress |
| POST | `/lms/v1/enrollments/{id}/complete` | Staff+MFA | 10/hr | Submit final assessment |
| GET | `/lms/v1/certificates/verify/{qr_token}` | Public | 1000/hr | Verify training certificate QR |
| GET | `/lms/v1/records/{user_id}` | Admin / User | 100/hr | Training records |
| POST | `/lms/v1/sync/polling-stations` | Admin+MFA | 5/day | Sync unqualified officers list to M06 |
| GET | `/lms/v1/reports/readiness` | Admin+MFA | 20/hr | Training readiness by county |
| GET | `/lms/v1/case-studies` | Enrolled+MFA | 200/hr | Real adjudication case studies from M27 |

---

### M39 — Ballot Printing Chain

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/ballot-chain/v1/orders` | the Commission Commissioner+MFA+M-of-N+HSM | 1/election/position | Create print order |
| GET | `/ballot-chain/v1/orders/{id}` | Auditor+MFA | 100/hr | Print order detail |
| POST | `/ballot-chain/v1/lot-data` | Printer Vendor mTLS | 10/hr | Submit printed lot manifest |
| GET | `/ballot-chain/v1/lot-data/{id}` | Auditor | 100/hr | Lot manifest + hash |
| POST | `/ballot-chain/v1/events` | Ops+biometric | 50/hr | Custody event along chain |
| GET | `/ballot-chain/v1/events/{id}` | Auditor | 100/hr | Custody event detail |
| GET | `/ballot-chain/v1/reconciliation/{station_code}` | RO+MFA | 50/hr | Station reconciliation |
| POST | `/ballot-chain/v1/reconciliation/{station_code}` | RO+MFA | 10/hr | Submit reconciliation report |
| GET | `/ballot-chain/v1/audit` | Auditor+MFA | 100/hr | Full chain audit |

---

### M40 — Election Court IT Interface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/court/v1/bundles` | the Commission Legal+MFA+dual | 5/petition | Create evidence bundle for petition |
| GET | `/court/v1/bundles/{id}` | Court Registrar+JWT / Petitioner+token | 50/hr | Bundle manifest |
| GET | `/court/v1/bundles/{id}/index` | Court Registrar+JWT | 50/hr | Indexed evidence list |
| GET | `/court/v1/bundles/{id}/items/{item_id}` | Court Registrar+JWT | 20/hr | Download evidence item |
| PATCH | `/court/v1/supplementary/{id}` | the Commission Legal+MFA | 20/hr | Update supplementary request |
| GET | `/court/v1/petitions/{id}/status` | Petitioner+token | 50/hr | Petition status |
| POST | `/court/v1/witnesses` | the Commission Legal+MFA | 10/hr | Register technical witness |
| GET | `/court/v1/witnesses/{id}` | Court Registrar+JWT | 20/hr | Witness detail |
| POST | `/court/v1/access-log` | System | automatic | Log every access to evidence bundle |
| POST | `/court/v1/webhooks/judiciary` | Judiciary CTS HMAC | 100/s | Court case status updates |
| GET | `/court/v1/access-log/{bundle_id}` | Auditor+MFA | 50/hr | Access log for bundle |

---

### M41 — Post-Election Review

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/review/v1/findings` | Submitter+MFA | 50/hr | Submit lessons-learning finding |
| GET | `/review/v1/findings` | Admin+MFA | 200/hr | List findings |
| PATCH | `/review/v1/findings/{id}` | Admin+MFA | 100/hr | Update finding status |
| POST | `/review/v1/findings/{id}/recommendations` | Admin+MFA | 20/hr | Add recommendation |
| GET | `/review/v1/recommendations` | Public | 200/hr | Published recommendations |
| PATCH | `/review/v1/recommendations/{id}` | Admin+MFA | 100/hr | Update recommendation |
| POST | `/review/v1/referrals` | Admin+MFA+dual | 10/hr | Refer to Parliament |
| GET | `/review/v1/public` | Public | unlimited | Public lessons-learned publication |
| GET | `/review/v1/reports/{election_cycle}` | Public | 200/hr | Full post-election report |
| POST | `/review/v1/reports/{election_cycle}/publish` | Commissioner+MFA+dual | 1/report | Publish final report |
| GET | `/review/v1/long-drip-alerts` | Admin+MFA | 100/hr | Long-drip systemic alerts |
| POST | `/review/v1/long-drip-alerts` | System | automatic | Create long-drip alert from pattern |
| GET | `/review/v1/analytics` | Admin+MFA | 50/hr | Finding analytics by category |

---

### M42 — Media Accreditation Portal

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/api/v1/accreditations` | Media Organisation Portal | 50/hr | Submit media accreditation application |
| GET | `/api/v1/accreditations/{id}` | Applicant+token / Admin | 200/hr | Application status |
| PATCH | `/api/v1/accreditations/{id}` | Vetting Officer+MFA | 100/hr | Update status |
| POST | `/api/v1/accreditations/{id}/badge` | Admin+MFA+dual | 20/hr | Issue badge |
| POST | `/api/v1/accreditations/{id}/revoke` | Admin+MFA+dual | 10/hr | Revoke accreditation |
| GET | `/api/v1/crl` | Public | 1000/s | Certificate revocation list for badges |

---

### M43 — Observer Accreditation Portal

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/api/v1/observers` | Observer Organisation Portal | 50/hr | Submit accreditation application |
| GET | `/api/v1/observers/{id}` | Applicant / Admin | 200/hr | Application status |
| PATCH | `/api/v1/observers/{id}` | Vetting Officer+MFA | 100/hr | Update status |
| POST | `/api/v1/observers/{id}/deploy` | Admin+MFA | 20/hr | Assign to deployment |
| POST | `/api/v1/observers/{id}/revoke` | Admin+MFA+dual | 10/hr | Revoke credential |
| GET | `/api/v1/crl` | Public | 1000/s | Observer CRL |
| POST | `/observers/v1/visit-logs` | Observer+JWT | 200/hr | Submit station visit log from M34 Mobile Observer app; GPS-timestamped; idempotent on `id` field |
| GET | `/observers/v1/visit-logs` | Admin+MFA | 500/hr | List visit logs (filter by observer_id, station_code, date) |
| GET | `/observers/v1/visit-logs/{id}` | Observer+JWT / Admin+MFA | 200/hr | Single visit log detail |

**Kafka event produced:** `observer.visit` (consumed by M15 IAB Dashboard).

---

### M44 — Turnout Forecasting Analytics

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/forecasts/latest` | Admin+MFA | 200/hr | Latest turnout forecast |
| GET | `/api/v1/forecasts/{id}` | Admin+MFA | 200/hr | Specific forecast release |
| GET | `/api/v1/forecasts/{id}/constituencies` | Admin+MFA | 200/hr | Per-constituency forecasts |
| GET | `/api/v1/turnout/alerts` | War-room | 1000/hr | Turnout variance alerts |
| GET | `/api/v1/datasets/post-election` | Public (post-election) | 100/hr | Historical turnout dataset |

---

### M45 — Party Registration Management

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/api/v1/parties` | Party applicant | 20/hr | Submit registration application |
| GET | `/api/v1/parties` | Public | 1000/hr | Public party register (paginated) |
| GET | `/api/v1/parties/{id}/status` | Public | 1000/hr | Party status |
| POST | `/api/v1/parties/{id}/annual-return` | Party Official+MFA | 5/year | File annual return |
| POST | `/api/v1/coalitions` | Party Official+MFA | 10/hr | Register coalition |
| POST | `/api/v1/disputes` | Party applicant | 10/hr | File registration dispute |

---

### M46 — Party Primaries Oversight

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/api/v1/primaries/opt-in` | Party Official+MFA | 5/election | Register party opt-in for oversight |
| GET | `/api/v1/primaries` | Public | 200/hr | List opted-in parties |
| POST | `/api/v1/results/{id}` | Party Official+MFA | 20/hr | Submit primary results |
| GET | `/api/v1/results/{id}` | Public (post-submit) | 200/hr | Primary results |
| POST | `/api/v1/disputes` | Party Candidate | 20/hr | File primary dispute |
| GET | `/api/v1/disputes/{id}` | Party Candidate / Admin | 100/hr | Dispute status |

---

### M47 — Nomination Fee Payment

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/payments/v1/fee-schedule` | Public | 1000/hr | Current fee schedule |
| POST | `/payments/v1/fees` | Candidate Portal+MFA | 20/hr | Initiate nomination fee payment |
| GET | `/payments/v1/fees/{id}` | Candidate Portal / RO+MFA | 200/hr | Payment status |
| POST | `/payments/v1/waivers` | Candidate Portal+MFA | 5/election | Apply for fee waiver |
| GET | `/payments/v1/waivers/{id}` | Candidate Portal / Admin | 200/hr | Waiver application status |
| POST | `/payments/v1/refunds` | Admin+MFA | 20/hr | Initiate refund |
| GET | `/payments/v1/refunds/{id}` | Admin | 100/hr | Refund status |
| POST | `/payments/v1/webhooks/mpesa` | Safaricom HMAC | 1000/s | M-Pesa payment callback |
| POST | `/payments/v1/webhooks/airtel` | Airtel HMAC | 1000/s | Airtel Money callback |

---

### M48 — Recall Election Workflow

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/recall/v1/petitions` | Petitioner+MFA | 5/year | Initiate recall petition |
| GET | `/recall/v1/petitions/{id}` | Public | 200/hr | Petition status |
| POST | `/recall/v1/petitions/{id}/signatures` | the Commission Officer+MTls | 200/hr | Submit signature batch for verification |
| GET | `/recall/v1/petitions/{id}/threshold` | Public | 200/hr | Threshold status |
| POST | `/recall/v1/petitions/{id}/challenge` | Respondent+MFA | 10/petition | Legal challenge hold |
| GET | `/recall/v1/elections/{id}` | Public | 200/hr | Recall election status |
| POST | `/recall/v1/elections` | Commissioner+MFA+dual+HSM | 2/petition | Declare recall election |

---

### M49 — Referendum Mechanics

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/referendum/v1/referenda` | Commissioner+MFA+dual | 2/cycle | Configure referendum |
| GET | `/referendum/v1/referenda/{id}` | Public | 1000/hr | Referendum configuration |
| GET | `/referendum/v1/referenda/{id}/results` | Public (post-declaration) | unlimited | Referendum results by county |
| POST | `/referendum/v1/referenda/{id}/county-result` | CRO+MFA+HSM | 47/referendum | Submit county result |
| GET | `/referendum/v1/referenda/{id}/national-result` | Public (post-declaration) | unlimited | National result summary |
| POST | `/referendum/v1/referenda/{id}/declare` | Commissioner+MFA+M-of-N+HSM | 1/referendum | Declare national result |
| POST | `/referendum/v1/webhooks/gazette` | Kenya Gazette API | 100/s | Gazette publication confirmation |

---

### M50 — By-Election Operations

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/by-election/v1/elections` | Commissioner+MFA+dual | 10/year | Create by-election |
| GET | `/by-election/v1/elections/{id}` | Public | 200/hr | By-election configuration |
| PATCH | `/by-election/v1/elections/{id}` | Admin+MFA | 20/hr | Update status |
| GET | `/by-election/v1/elections/{id}/cost-report` | Auditor / Parliament | 50/hr | Cost report |
| POST | `/by-election/v1/elections/{id}/activate-staff` | Admin+MFA | 5/election | Trigger staff activation event |
| GET | `/by-election/v1/elections/{id}/readiness` | Admin+MFA | 20/hr | Election-readiness checklist |
| POST | `/by-election/v1/webhooks/recall` | M48 internal | 10/s | Recall election trigger callback |

---

### M51 — Result Declaration and Certification

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/declarations/v1/` | RO+MFA | 20/hr | Initiate declaration workflow |
| GET | `/declarations/v1/{id}/checklist` | RO+MFA | 100/hr | Pre-declaration checklist |
| POST | `/declarations/v1/{id}/legal-review` | Legal Officer+MFA | 10/hr | Legal sign-off |
| POST | `/declarations/v1/{id}/sign` | RO+HSM | 5/hr | HSM-signed declaration |
| GET | `/declarations/v1/{id}/certificate` | Public (post-declaration) | unlimited | Certificate of Election PDF |
| GET | `/declarations/v1/{id}/petition-window` | the Commission Legal+MFA | 100/hr | Petition window status |
| POST | `/declarations/v1/{id}/petition` | Judiciary API key | 10/hr | Record petition lodged |

**Events:** `declaration.made`, `certificate.issued`.

---

### M52 — Election Calendar and Gazette

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/calendar/v1/elections` | Public | unlimited | Election calendar |
| POST | `/calendar/v1/elections` | Commissioner+MFA+dual | 5/year | Create election event |
| GET | `/calendar/v1/events` | Public | unlimited | Upcoming statutory events |
| GET | `/calendar/v1/gazette-notices` | Public | 200/hr | Gazette notice registry |
| POST | `/calendar/v1/gazette-notices` | Admin+MFA+dual | 20/hr | Create gazette notice |
| GET | `/calendar/v1/gazette-notices/{id}` | Public | unlimited | Notice detail |
| GET | `/calendar/v1/deadlines/{election_id}` | Public | 200/hr | Statutory deadline schedule |
| POST | `/calendar/v1/webhooks/gazette` | Kenya Gazette HMAC | 100/s | Gazette publication confirmation |

---

### M53 — Voter ID Card

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/voter-id/v1/batches` | Admin+MFA+dual | 5/year | Create card production batch |
| GET | `/voter-id/v1/batches/{id}` | Admin | 100/hr | Batch status |
| GET | `/voter-id/v1/cards/{voter_id}` | RO+MFA | 100/hr | Card status for voter |
| POST | `/voter-id/v1/cards/{voter_id}/collected` | the Commission Officer+biometric | 100/hr | Mark card collected |
| POST | `/voter-id/v1/cards/{voter_id}/replace` | Admin+MFA | 20/hr | Issue replacement card |
| POST | `/voter-id/v1/verify` | Public (kiosk) | 1000/s | Verify card QR code |
| POST | `/voter-id/v1/webhooks/printer` | Al Ghurair HMAC | 100/s | Printer delivery confirmation |

---

### M54 — Election Budget and Expenditure

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/budget/v1/budgets` | Finance Officer+MFA+dual | 2/election | Create election budget |
| GET | `/budget/v1/budgets/{id}` | Finance Officer / Parliament+JWT | 100/hr | Budget detail |
| POST | `/budget/v1/budgets/{id}/lines` | Finance Officer+MFA | 50/hr | Add budget line |
| POST | `/budget/v1/requisitions` | Programme Officer+MFA | 100/hr | Create procurement requisition |
| GET | `/budget/v1/requisitions/{id}` | Finance Officer / Auditor | 100/hr | Requisition status |
| POST | `/budget/v1/vouchers` | Finance Officer+MFA+dual | 100/hr | Create payment voucher |
| GET | `/budget/v1/vouchers/{id}` | Finance Officer / Auditor | 100/hr | Voucher detail |
| GET | `/budget/v1/reports/burn-rate` | Finance Officer+MFA | 20/hr | Budget burn rate |
| POST | `/budget/v1/webhooks/ntgov` | National Treasury HMAC | 100/s | Treasury release confirmation |
| GET | `/budget/v1/public/summary` | Public (post-election) | 200/hr | Published budget summary |

---

### M55 — HR and Payroll (Temporary Staff)

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/hr/v1/applications` | Applicant (public) | 20/day/IP | Submit staff application |
| GET | `/hr/v1/applications/{id}` | Applicant / HR Officer | 100/hr | Application status |
| POST | `/hr/v1/contracts` | HR Officer+MFA+dual | 50/hr | Issue employment contract |
| GET | `/hr/v1/contracts/{id}` | Staff+MFA / HR Officer | 100/hr | Contract detail |
| POST | `/hr/v1/payroll-runs` | Finance Officer+MFA+dual | 5/election | Create payroll run |
| GET | `/hr/v1/payroll-runs/{id}` | Finance Officer / Auditor | 50/hr | Payroll run detail |
| POST | `/hr/v1/payroll-runs/{id}/approve` | Commissioner+MFA+dual | 5/election | Approve payroll |
| GET | `/hr/v1/payroll-runs/{id}/lines` | Finance Officer+MFA | 100/hr | Payroll lines (PII-protected) |
| POST | `/hr/v1/webhooks/mpesa-payroll` | Safaricom HMAC | 1000/s | M-Pesa bulk payment confirmation |
| GET | `/hr/v1/reports/staffing` | Admin+MFA | 20/hr | Staffing level report |

---

### M56 — County-National Coordination

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/coord/v1/mous` | Admin+MFA | 100/hr | County MoU registry |
| POST | `/coord/v1/mous` | Commissioner+MFA+dual | 47/election | Upload signed County MoU |
| POST | `/coord/v1/data-uploads` | County Officer+mTLS | 50/hr | Upload county coordination data |
| GET | `/coord/v1/data-uploads/{id}` | Admin | 100/hr | Upload status |
| POST | `/coord/v1/incidents` | County Officer+MFA | 100/hr | Log cross-jurisdiction incident |
| GET | `/coord/v1/incidents` | Admin+MFA | 200/hr | Incident list |
| PATCH | `/coord/v1/incidents/{id}` | Admin+MFA | 100/hr | Update incident |
| GET | `/coord/v1/stream` | War-room | 10/user | SSE county coordination status |
| POST | `/coord/v1/ministerial-approvals` | Minister+MFA | 5/hr | Log ministerial approval |
| GET | `/coord/v1/approvals/{id}` | Admin / Auditor | 100/hr | Approval detail |
| POST | `/coord/v1/webhooks/county-gov` | County Gov HMAC | 100/s | County government status webhook |
| GET | `/coord/v1/reports/readiness` | Admin+MFA | 10/hr | County readiness report |
| GET | `/coord/v1/counties/{code}/liaison` | Admin+MFA | 100/hr | County liaison officer detail |

---

### M57 — International Observer Coordination

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/observers/v1/missions` | Public | 200/hr | Accredited observer missions |
| POST | `/observers/v1/missions` | Admin+MFA | 20/hr | Create observer mission |
| GET | `/observers/v1/missions/{id}` | Mission Lead+JWT | 200/hr | Mission detail |
| POST | `/observers/v1/missions/{id}/mou` | Admin+MFA | 5/mission | Upload signed MoU |
| POST | `/observers/v1/deployment-plans` | Mission Lead+JWT | 20/hr | Submit deployment plan |
| GET | `/observers/v1/deployment-plans/{id}` | Admin | 100/hr | Deployment plan |
| POST | `/observers/v1/briefings` | Admin+MFA | 20/hr | Schedule briefing |
| POST | `/observers/v1/briefings/{id}/attendance` | Observer+JWT | 100/hr | Record attendance |
| POST | `/observers/v1/reports` | Mission Lead+JWT | 10/mission | Submit mission report |
| GET | `/observers/v1/reports/{id}` | Public (post-publish) | 200/hr | Published mission report |
| POST | `/observers/v1/joint-statements` | Multi-mission leads | 5/election | Publish joint statement |
| GET | `/observers/v1/joint-statements/{id}` | Public | 200/hr | Joint statement |
| GET | `/observers/v1/accreditation-stats` | Public | 200/hr | Observer numbers by organisation type |
| POST | `/observers/v1/webhooks/mfa` | MFA Kenya HMAC | 100/s | Ministry of Foreign Affairs vetting callback |

---

### M58 — Parliamentary and Judiciary Reporting

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/statutory/v1/reports` | the Commission Legal+MFA+dual | 20/hr | Create statutory report |
| GET | `/statutory/v1/reports/{id}` | Parliament+JWT / AG / Public (after tabling) | 100/hr | Report detail |
| POST | `/statutory/v1/reports/{id}/sign` | Commissioner+HSM | 5/hr | HSM-sign report |
| GET | `/statutory/v1/testimony/{id}` | the Commission Legal+MFA / Court+JWT | 50/hr | Testimony workspace |
| POST | `/statutory/v1/testimony/{id}/exhibit` | the Commission Legal+MFA | 50/hr | Upload exhibit |
| POST | `/statutory/v1/judiciary-handoff/{id}` | the Commission Legal+MFA | 20/hr | Deliver to Judiciary CTS |
| GET | `/statutory/v1/sc-preregistration` | the Commission Legal+MFA | 100/hr | Supreme Court pre-registration items |
| POST | `/statutory/v1/sc-preregistration` | the Commission Legal+MFA+dual | 10/hr | Register SC pre-registration item |
| POST | `/statutory/v1/webhooks/parliament` | Parliament HMAC | 100/s | Parliamentary confirmation callback |
| GET | `/statutory/v1/webhooks/judiciary-cts` | Judiciary CTS HMAC | 100/s | CTS case status webhook |
| POST | `/statutory/v1/webhooks/ag` | AG Office HMAC | 100/s | Attorney-General office callback |
| GET | `/statutory/v1/reports` | Admin+MFA | 200/hr | All statutory reports |
| GET | `/statutory/v1/public` | Public | unlimited | Publicly tabled reports |

---

### M59 — Auditor-General Interface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/oag/v1/engagements` | OAG Officer+JWT | 10/year | Create audit engagement |
| GET | `/oag/v1/engagements/{id}` | OAG Officer+JWT | 100/hr | Engagement detail |
| POST | `/oag/v1/evidence/{id}` | the Commission Legal+MFA | 50/hr | Submit evidence extract |
| GET | `/oag/v1/evidence/{id}` | OAG Officer+JWT | 50/hr | Evidence extract (access logged) |
| POST | `/oag/v1/queries/{id}` | OAG Officer+JWT | 50/hr | Submit audit query |
| POST | `/oag/v1/responses/{query_id}` | the Commission Finance+MFA | 50/hr | Submit management response |
| GET | `/oag/v1/follow-up` | OAG Officer+JWT | 100/hr | Outstanding follow-up items |
| POST | `/oag/v1/access-log` | System | automatic | Log every OAG access |
| GET | `/oag/v1/access-log/{engagement_id}` | Commissioner+MFA | 50/hr | Full access log |
| POST | `/oag/v1/package-signed` | OAG HSM+JWT | 5/engagement | Sign off evidence package |
| GET | `/oag/v1/public/reports` | Public | unlimited | Published OAG reports on the Commission |
| POST | `/oag/v1/webhooks/oag` | OAG HMAC | 100/s | OAG platform webhook |
| GET | `/oag/v1/dashboards/expenditure` | OAG Officer+JWT | 100/hr | Expenditure dashboard (M54 integration) |

---

### M60 — Medical First Response Coordination

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/medical/v1/units` | War-room | 200/hr | Medical unit deployment |
| POST | `/medical/v1/incidents` | Medical Officer+mTLS | 50/hr | Log medical incident |
| GET | `/medical/v1/incidents/{id}` | War-room / MoH+JWT | 100/hr | Incident detail |
| POST | `/medical/v1/incidents/{id}/escalate` | Medical Officer+MFA | 20/hr | Escalate to NDMU |
| GET | `/medical/v1/signals` | MoH+JWT | 100/hr | Epidemic signals |
| POST | `/medical/v1/webhooks/ndmu` | NDMU HMAC | 100/s | NDMU coordination webhook |
| GET | `/medical/v1/reports/coverage` | Admin+MFA | 10/hr | Medical coverage report |

---

### M61 — Secrecy Assurance Testing

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/secrecy/v1/engagements` | Commissioner+MFA+dual | 3/election | Create red-team engagement |
| GET | `/secrecy/v1/engagements/{id}` | Commissioner+MFA | 50/hr | Engagement detail |
| POST | `/secrecy/v1/findings` | Red Team Lead+JWT | 50/hr | Submit finding |
| GET | `/secrecy/v1/findings` | Commissioner+MFA | 100/hr | Findings list |
| PATCH | `/secrecy/v1/findings/{id}` | the Commission ICT+MFA | 50/hr | Mark remediated |
| POST | `/secrecy/v1/attestations` | Commissioner+HSM | 2/election | Sign secrecy attestation |
| GET | `/secrecy/v1/attestations/{id}` | Public (post-publication) | 200/hr | Published attestation |
| POST | `/secrecy/v1/rts-analysis` | ML Engineer+MFA | 10/hr | Submit RTS metadata analysis run |
| GET | `/secrecy/v1/rts-analysis/{id}` | Auditor+MFA | 50/hr | Analysis results |
| POST | `/secrecy/v1/webhooks/critical` | System | automatic | Critical finding escalation webhook |
| GET | `/secrecy/v1/public/summary` | Public | 200/hr | Public secrecy assurance summary |
| POST | `/secrecy/v1/critical-escalation` | Red Team Lead+JWT | 5/hr | Emergency critical finding escalation |
| GET | `/secrecy/v1/remediation-status` | Commissioner+MFA | 50/hr | Finding remediation dashboard |

---

### M62 — Transmission Legal Basis Audit

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/transmission/v1/events` | Auditor+MFA | 200/hr | Transmission event log |
| GET | `/transmission/v1/events/{id}` | Auditor+MFA | 100/hr | Event detail + verdict |
| GET | `/transmission/v1/rules` | Public | 200/hr | Published compliance rules |
| POST | `/transmission/v1/rules` | Admin+MFA+dual | 10/hr | Update compliance rule |
| GET | `/transmission/v1/verdicts` | Auditor+MFA | 200/hr | Verdict register |
| POST | `/transmission/v1/verdicts/{id}/escalate` | Senior Auditor+MFA | 20/hr | Escalate non-compliant verdict |
| GET | `/transmission/v1/section39` | Auditor+MFA / Court+JWT | 50/hr | Section 39 compliance records |
| GET | `/transmission/v1/bundles/{id}` | Auditor / Court+JWT | 50/hr | Evidence bundle for non-compliance |
| POST | `/transmission/v1/webhooks/kit` | Kit mTLS | 10/s/kit | Kit reports its own transmission metadata |
| GET | `/transmission/v1/dashboard` | Admin+MFA | 100/hr | Compliance dashboard |
| POST | `/transmission/v1/compliance-run` | System (scheduled) | 1/form | Triggered per Form34A received |
| GET | `/transmission/v1/noncompliant` | Auditor+MFA | 100/hr | Non-compliant transmission list |
| GET | `/transmission/v1/public/report` | Public (post-election) | 200/hr | Published compliance report |

---

### M63 — Ballot Serial Chain

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/ballot-serial/v1/assignments/{station_code}` | RO+MFA / Kit mTLS | 100/hr | Serial range for station |
| POST | `/ballot-serial/v1/receipt` | Station Officer+biometric | 1/station | Record receipt of ballot bundle |
| GET | `/ballot-serial/v1/receipt/{id}` | RO+MFA | 100/hr | Receipt detail |
| POST | `/ballot-serial/v1/spoilt` | Kit mTLS | 50/hr | Record spoilt ballot |
| GET | `/ballot-serial/v1/spoilt/{station_code}` | RO+MFA | 50/hr | Spoilt ballot log for station |
| POST | `/ballot-serial/v1/deposit` | Ops+biometric | 5/station | Record post-poll ballot deposit |
| GET | `/ballot-serial/v1/reconciliation/{station_code}` | RO+MFA | 50/hr | Reconciliation: assigned vs. used vs. spoilt |
| POST | `/ballot-serial/v1/alert` | System | automatic | Alert on reconciliation mismatch |

---

### M64 — Voter List Public Inspection

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/roll/v1/versions` | Public | 200/hr | List published roll versions |
| GET | `/roll/v1/versions/{id}` | Public | 200/hr | Roll version + Merkle root |
| POST | `/roll/v1/objections` | Public+OTP | 3/day/IP | File objection on roll entry |
| GET | `/roll/v1/objections/{id}` | Objector+OTP / Admin | 100/hr | Objection status |
| PATCH | `/roll/v1/objections/{id}` | the Commission Officer+MFA | 50/hr | Update objection |
| POST | `/roll/v1/corrections` | Public+OTP | 3/day/IP | File correction claim |
| GET | `/roll/v1/corrections/{id}` | Requester+OTP / Admin | 100/hr | Correction status |

---

### M65 — Diaspora Ballot Physical Logistics

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/logistics/v1/posts` | Admin+MFA | 200/hr | Mail-in ballot post records |
| POST | `/logistics/v1/containers` | Logistics Officer+biometric | 20/hr | Create container record |
| GET | `/logistics/v1/containers/{id}` | Admin+MFA | 100/hr | Container detail + custody |
| POST | `/logistics/v1/containers/{id}/custody` | Logistics Officer+biometric | 20/hr | Custody event |
| GET | `/logistics/v1/alerts` | War-room | 200/hr | Late / missing container alerts |
| POST | `/logistics/v1/webhooks/courier` | Courier Provider HMAC | 500/s | Courier tracking webhook |

---

### M66 — Election Financial Forensics

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/forensics/v1/cycles` | Senior Analyst+MFA+dual | 3/election | Create analysis cycle |
| GET | `/forensics/v1/cycles/{id}` | Analyst+MFA | 100/hr | Analysis cycle detail |
| GET | `/forensics/v1/estimates` | Analyst+MFA | 200/hr | Constituency-level cash flow estimates |
| POST | `/forensics/v1/referrals` | Senior Analyst+MFA+dual | 10/hr | Generate ODPP referral packet |
| GET | `/forensics/v1/referrals/{id}` | Auditor / ODPP+JWT | 50/hr | Referral packet |
| POST | `/forensics/v1/public-report` | Commissioner+MFA+dual | 1/election | Publish public forensics report |
| GET | `/forensics/v1/public-report/{id}` | Public | unlimited | Published forensics report |

---

### M67 — Party Symbol and Colour Registry

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| POST | `/symbols/v1/registrations` | Party Official+MFA | 5/election | Submit symbol registration |
| GET | `/symbols/v1/registrations/{id}` | Party Official / Public | 200/hr | Registration status |
| PATCH | `/symbols/v1/registrations/{id}` | Vetting Officer+MFA | 20/hr | Approve / reject |
| GET | `/symbols/v1/colours` | Public | 1000/hr | Party colour registry |
| POST | `/symbols/v1/ballot-design` | Admin+MFA+dual | 1/election | Generate ballot design package |
| GET | `/symbols/v1/ballot-design/{id}` | Admin / Print Vendor mTLS | 20/hr | Ballot design package |
| POST | `/symbols/v1/webhooks/orpp` | ORPP HMAC | 100/s | ORPP registration confirmation |

---

### M68 — Referendum Explainer Platform

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/explainer/v1/questions` | Public | unlimited | Referendum questions in all languages |
| POST | `/explainer/v1/versions` | Content Editor+MFA | 20/hr | Create explainer version |
| GET | `/explainer/v1/versions/{id}` | Public | unlimited | Explainer content |
| POST | `/explainer/v1/versions/{id}/approve` | Commissioner+MFA+dual | 5/hr | Approve explainer for publication |
| POST | `/explainer/v1/fact-checks` | Analyst+MFA | 20/hr | Annotate fact-check |
| GET | `/explainer/v1/fact-checks` | Public | 200/hr | Published fact-checks |
| GET | `/explainer/v1/formats/{language}` | Public | unlimited | Format-specific explainer (SMS, audio, video) |

---

### M69 — Election Data Long-Term Archive

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/archive/v1/collections` | Public (restricted collections require auth) | 200/hr | Browse archive |
| GET | `/archive/v1/collections/{id}` | Public / Researcher+JWT | 200/hr | Collection metadata + attestation links |
| GET | `/archive/v1/collections/{id}/download` | Researcher+JWT (embargo enforced) | 10/hr | Download archive |
| POST | `/archive/v1/ingest` | System (scheduled) | 10/day | Ingest new data source |
| POST | `/archive/v1/attestations` | Archivist+MFA | 5/year | Create preservation attestation |
| GET | `/archive/v1/attestations/{id}` | Public | 200/hr | Attestation detail |
| POST | `/archive/v1/research-access` | Researcher | 5/year | Apply for restricted access |
| GET | `/archive/v1/sources` | Public | 200/hr | Data source registry |
| POST | `/archive/v1/webhooks/knads` | KNADS HMAC | 100/s | Kenya National Archives deposit confirmation |

---

### Platform APIs (IAM, PKI, Inventory, MDM, Geo-fence, SIM, Audit, Compliance, Support, Training, Dispatch)

#### IAM (P04)

| Method | Path | Auth | Notes |
|---|---|---|---|
| POST | `/iam/v1/auth/login` | Public | Password + MFA challenge |
| POST | `/iam/v1/auth/mfa` | Session token | Complete MFA challenge |
| POST | `/iam/v1/auth/refresh` | Refresh token | Refresh access token |
| POST | `/iam/v1/auth/logout` | Session token | Invalidate session |
| GET | `/iam/v1/users/{user_id}` | Admin+MFA | User profile |
| POST | `/iam/v1/users` | Admin+MFA+dual | Create user |
| POST | `/iam/v1/users/{user_id}/roles` | Admin+MFA+dual | Grant role |
| DELETE | `/iam/v1/users/{user_id}/roles/{role_id}` | Admin+MFA+dual | Revoke role |
| POST | `/iam/v1/users/{user_id}/lock` | Admin+MFA+dual | Suspend account |
| POST | `/iam/v1/policy/evaluate` | Service mTLS | OPA policy check |

#### PKI (P04)

| Method | Path | Auth | Notes |
|---|---|---|---|
| POST | `/pki/v1/kits/{kit_id}/certificate` | Kit Provisioning Admin+MFA | Issue per-kit certificate |
| POST | `/pki/v1/kits/{kit_id}/certificate/revoke` | Admin+MFA+dual | Revoke; propagates ≤ 60 s |
| GET | `/pki/v1/crl` | Public+mTLS | Certificate Revocation List |
| POST | `/pki/v1/ocsp` | Public+mTLS | OCSP responder |
| POST | `/pki/v1/attestation` | Kit mTLS | Boot attestation submission |

#### MDM (O01)

| Method | Path | Auth | Notes |
|---|---|---|---|
| GET | `/mdm/v1/kits/{kit_id}/config` | Kit mTLS | Kit fetches config |
| PUT | `/mdm/v1/kits/{kit_id}/config` | Admin+MFA+dual | Push config |
| POST | `/mdm/v1/kits/{kit_id}/firmware/update` | Admin+MFA+dual | OTA update |
| POST | `/mdm/v1/kits/{kit_id}/wipe` | Admin+MFA+dual | Remote wipe |
| POST | `/mdm/v1/kits/{kit_id}/lock` | Admin+MFA | Remote lock |

#### Audit (P05)

| Method | Path | Auth | Notes |
|---|---|---|---|
| GET | `/audit/v1/events` | Auditor+MFA | Query audit events |
| GET | `/audit/v1/events/{id}/proof` | Auditor / Public (public events) | Merkle inclusion proof |
| GET | `/audit/v1/merkle-roots/{constituency_id}` | Public | Latest root |
| POST | `/audit/v1/investigations` | Auditor+MFA | Open investigation |

#### Compliance (P06)

| Method | Path | Auth | Notes |
|---|---|---|---|
| GET | `/compliance/v1/dpia` | ODPC Officer + the Commission | List DPIAs |
| POST | `/compliance/v1/breach-notifications` | Admin+MFA+dual | File breach notification |
| POST | `/compliance/v1/sar` | Public+OTP | Subject access request |
| GET | `/compliance/v1/sar/{id}` | Requester / Admin | SAR status |
| GET | `/compliance/v1/retention/rules` | Admin+MFA | Retention rules |
| POST | `/compliance/v1/retention/run` | Admin+MFA+dual | Manual retention sweep |

#### Support (O02)

| Method | Path | Auth | Notes |
|---|---|---|---|
| POST | `/support/v1/tickets` | Public / Kit / Ops | Open ticket |
| GET | `/support/v1/tickets/{id}` | Support + Requester | Ticket status |
| PATCH | `/support/v1/tickets/{id}` | Support | Update ticket |
| POST | `/support/v1/tickets/{id}/escalate` | Support+MFA | Escalate severity |

#### Personnel Dispatch (O05)

| Method | Path | Auth | Notes |
|---|---|---|---|
| POST | `/dispatch/v1/assignments` | Admin+MFA | Assign staff |
| POST | `/dispatch/v1/attendance` | Staff (mobile) | GPS-verified check-in |
| GET | `/dispatch/v1/attendance/{staff_id}` | Admin | Attendance record |

---

## 5. Cross-Cutting Concerns

### 5.1 Correlation ID propagation

Every request must carry `X-Request-Id`. The gateway assigns one if absent. All downstream calls propagate the same header. Logs, traces, and error responses include the correlation ID. CloudEvents `id` field carries the same UUID7.

### 5.2 Distributed tracing

OpenTelemetry spans are created at the gateway for every request. Services instrument all DB queries, Kafka produce/consume, and HSM calls. Traces exported to Tempo. Spans are sampled at 100% for P0/P1 operations; 10% otherwise.

### 5.3 Structured logging

All logs are JSON with mandatory fields: `timestamp`, `level`, `service`, `trace_id`, `span_id`, `request_id`, `actor_id`, `action`. Logs stream to Loki via OpenTelemetry collector.

### 5.4 Event provenance

Every Kafka event carries:
- `source`: originating service (`/commission/bvr-svc`)
- `subject`: entity ID affected
- `time`: RFC 3339 UTC
- `signature`: HSM ECDSA-P384 over the CloudEvents envelope
- `iemecid`: the Commission message correlation ID (same as `X-Request-Id` that triggered the event)

### 5.5 Retry and back-off

Clients implement exponential backoff with jitter: initial 100 ms, multiplier 2×, cap 30 s, max 5 attempts. Idempotency keys ensure safety on retry. Kit-side retry queue is persisted to encrypted local storage and survives device reboot.

### 5.6 Server-Sent Events (SSE)

Real-time feeds (ECC stream, queue transparency, kit telemetry) use SSE over HTTPS (`Content-Type: text/event-stream`). Clients reconnect on disconnect using `Last-Event-ID` for gapless resumption. SSE is preferred over WebSocket for unidirectional server-push; WebSocket is used only for M16 ECC war-room where bidirectional control is needed.

---

## 6. Cross-Module Orchestration

Key choreography chains (event-driven, not request-driven):

| Trigger | Emitted by | Consumed by | Effect |
|---|---|---|---|
| `voter.enrolled` | M02 BVR | M03 ABIS | 1:N biometric dedup triggered |
| `session.opened` | M06 BVI | M16 ECC | War-room shows station as ACTIVE |
| `voter.authenticated` | M06 BVI | M44 Turnout, M26 Anomaly | Turnout count updated; anomaly check triggered |
| `form34a.submitted` | M10 RTS | M26 Anomaly, M14 Reproducibility, Audit | Parallel: anomaly check; bundle update; Merkle log |
| `form34a.published` | M10 RTS | M11 Portal, M13 Verifier | CDN cache invalidated; verifier shows result |
| `snapshot.published` | M11 Portal | M14 Reproducibility, M69 Archive, Escrow | Bundle regenerated; archive notified |
| `candidate.gazetted` | M12 CRMS | M10 RTS, M39 Ballot, M63 Serial | Form templates updated; print order authorised |
| `declaration.made` | M51 | M52 Gazette, M40 Court IT, M58 Parliament | Gazette notice created; court bundle ready; statutory report triggered |
| `anomaly.alert` (CRITICAL) | M26 | M16 ECC, M15 IAB | P0 alert in war-room; IAB escalation possible |
| `kit.attestation` (mismatch) | M01/M05 | P04 PKI, M16 ECC | Certificate revocation; war-room alert |
| `nomination_fee.confirmed` | M47 | M12 CRMS | Candidate clearance gate opened |
| `register.gazetted` | M02 BVR | M35 CVR, M64 Roll Inspection | CVR baseline frozen; public inspection period opened |
| `bundle.ready` | M40 Court IT | M58 Parliament, Judiciary CTS | Evidence served to court within 24-hr SLA |

---

## 7. Event Bus Catalogue (Kafka Topics)

All topics use CloudEvents 1.0 envelope. Key schema: every value schema registered in Confluent Schema Registry (on-prem). Topic ACLs enforced at broker level (mTLS client identity = SPIFFE SVID).

| Topic | Key schema | Value schema | Retention | Partitions | Owners | Consumers |
|---|---|---|---|---|---|---|
| `bvr.enrollments` | kit_id | VoterEnrolledEvent v1 | 7 days hot; permanent archive | 290 (one/constituency) | M02 | M03, P05, M69 |
| `bvr.updates` | voter_id | VoterUpdatedEvent v1 | 7 days hot | 47 | M02 | M06, M35, P05 |
| `bvr.transfers` | voter_id | VoterTransferredEvent v1 | 7 days hot | 47 | M02 | M06, M35, P05 |
| `cvr.events` | voter_id | VoterRegisterEvent v1 | permanent | 47 | M35 | M36, M64, P05 |
| `cvr.snapshots` | snapshot_id | RegisterSnapshotEvent v1 | permanent | 1 | M35 | M64, M69, P05 |
| `polling.sessions` | station_code | SessionEvent v1 | permanent | 290 | M06 | M10, M16, M44, P05 |
| `polling.identifications` | station_code | IdentificationEvent v1 | permanent | 55393 | M06 | M26, M44, P05 |
| `polling.distress` | kit_serial | DistressEvent v1 | permanent | 47 | M06 | M16, KNPS liaison, P05 |
| `polling.queue` | station_code | QueueSnapshotEvent v1 | 90 days | 290 | M19 | M13, M16, M44 |
| `rts.forms` | form_id | Form34AEvent v1 | permanent | 290 | M10 | M11, M14, M21, M26, P05 |
| `rts.publication` | snapshot_id | PublicationEvent v1 | permanent | 1 | M10 | M11, M13, M14, M69 |
| `kit.attestation` | kit_serial | AttestationEvent v1 | permanent | 47 | M01/M05/M08 | P04, M16, P05 |
| `kit.custody` | kit_id | CustodyEvent v1 | permanent | 47 | M20 | P05, M16 |
| `kit.telemetry` | kit_serial | TelemetryEvent v1 | 90 days | 290 | M01/M05/M08 | M16, M26 |
| `anomaly.alerts` | alert_id | DetectionAlertEvent v1 | 15 years | 47 | M26 | M16, M15, P05 |
| `ecc.alerts` | alert_id | ECCAlertEvent v1 | 15 years | 1 | M16 | M15, P05 |
| `grievances` | case_id | GrievanceEvent v1 | 15 years | 47 | M23 | M16, P05, M13 |
| `audit.events` | event_id | AuditEvent v1 | permanent | 290 | P05 | Merkle builder, Escrow, M69 |
| `boundaries` | boundary_code | BoundaryEvent v1 | permanent | 47 | M18 | All map consumers |
| `elections` | election_id | ElectionEvent v1 | permanent | 1 | M52 | All modules |
| `candidates` | candidate_id | CandidateEvent v1 | permanent | 47 | M12 | M10, M39, M63, M27 |
| `payments` | payment_id | PaymentEvent v1 | 15 years | 47 | M47 | M12, P05 |
| `declarations` | declaration_id | DeclarationEvent v1 | permanent | 47 | M51 | M52, M40, M58, P05 |
| `security.incidents` | incident_id | SecurityIncidentEvent v1 | permanent | 1 | Various | War-room, SIEM, P05 |
| `queue.snapshots` | station_code | QueueSnapshotEvent v1 | 90 days | 290 | M19 | M13, M16 |

Total topics: **25** enumerated above, plus per-module internal topics (dead-letter queues, retry topics) not publicly exposed.

---

## 8. Webhook and Callback Catalogue

Outbound HTTP webhooks to third-party systems. All webhooks are HMAC-SHA256 signed with a per-partner secret. Delivery uses exponential backoff with max 5 retries. Failed deliveries are queued in a durable outbox (PostgreSQL transactional outbox pattern).

| System | Module | Direction | Event | Auth |
|---|---|---|---|---|
| M-Pesa (Safaricom) | M47, M55 | Outbound + Inbound | Payment initiation + confirmation | OAuth 2.0 client credentials |
| Airtel Money | M47 | Outbound + Inbound | Payment + confirmation | API key + HMAC |
| KRA (Kenya Revenue Authority) | M35 | Inbound | Dormancy batch | mTLS client cert |
| CBK (Central Bank of Kenya) | M22, M66 | Inbound | Financial intelligence feed | mTLS + API key |
| FRC (Financial Reporting Centre) | M22, M66 | Inbound | SAR / cash movement | mTLS + HMAC |
| NRB (National Registration Bureau) | M35 | Inbound | ID correction batch | mTLS |
| CRS (Civil Registration Service) | M35 | Inbound | Death notification batch | mTLS |
| IPRS (Integrated Population Registry) | M04 | Outbound | Identity verification query | REST + mTLS |
| ORPP (Registrar of Political Parties) | M45, M67 | Outbound + Inbound | Party registration status | HMAC |
| Kenya Gazette | M52, M39, M51 | Outbound | Gazette notice publication | REST + API key |
| KNADS (Kenya National Archives) | M69 | Outbound + Inbound | Deposit + accession | REST + API key |
| Judiciary CTS | M58, M40 | Outbound + Inbound | Evidence delivery + case status | mTLS + JWT |
| Attorney General | M58 | Outbound | Statutory report delivery | mTLS + signed payload |
| Parliament (National Assembly / Senate) | M58, M41 | Outbound | Report tabling notification | REST + HMAC |
| Auditor-General (OAG) | M59 | Inbound | Audit query + access | JWT (OAG-issued) |
| NDMU (National Disaster Management Unit) | M60 | Outbound + Inbound | Medical incident escalation | REST + HMAC |
| MoH (Ministry of Health) | M60 | Outbound | Epidemic signal | REST + HMAC |
| Ministry of Foreign Affairs | M57 | Inbound | Observer vetting callback | HMAC |
| KNPS (Kenya National Police Service) | M17, M23 | Outbound | Safety incident referral | mTLS |
| DCI (Directorate of Criminal Investigations) | M22, M23 | Outbound | Evidence referral | mTLS + signed |
| ODPP (Office of DPP) | M22, M66 | Outbound | Prosecution referral packet | mTLS + signed |
| ODPC (Office of Data Protection Commissioner) | P06 | Outbound | Breach notification | REST + structured payload |
| EACC (Ethics & Anti-Corruption Commission) | M22 | Outbound | Bribery tip referral | mTLS |
| KEOG / ELOG / IFES / Carter Center / EU EOM / AU | M57 | Outbound | Observer credential confirmation | REST + API key |
| Al Ghurair (ballot/card printer) | M39, M53 | Outbound + Inbound | Print order + delivery confirmation | mTLS + HMAC |
| Safaricom USSD / SMS | M13, M23, M35 | Inbound | USSD/SMS intake | API key + HMAC |
| Airtel USSD / SMS | M13, M23 | Inbound | USSD/SMS intake | API key + HMAC |
| Telkom SMS | M13, M23 | Inbound | SMS intake | API key + HMAC |
| WhatsApp Business API | M13, M23 | Outbound + Inbound | Notification + intake | HMAC + token |
| IPFS | M11, M14 | Outbound | Merkle root + bundle pinning | IPFS HTTP API |
| GitHub | M11, M14 | Outbound | Merkle root publication | GitHub API + token |
| Safaricom M-Pesa (payroll) | M55 | Outbound | Bulk payroll disbursement | OAuth 2.0 |

Total outbound webhook targets: **31 systems**.

---

## 9. Deprecation and Versioning Policy

- URI versioning: `/{service}/v{N}/`. N increments on breaking changes only.
- Non-breaking additions (new optional fields, new optional query params, new endpoints): no version increment; documented in changelog.
- Deprecation announced via `Deprecation: <ISO date>` response header on every deprecated endpoint, from announcement date.
- `Sunset: <ISO date>` header added six months before removal.
- Minimum 12 months from `Deprecation` to removal.
- N-2 version support: at any given time, at most 3 major versions of any endpoint are active.
- Schema evolution in Kafka: backward-compatible only within a version; new major version requires new topic with old topic in parallel for 6 months.
- Deprecation log published in `CHANGELOG.md` in the source repository and via a `/deprecation-notices` endpoint on each service.

---

## 10. Compliance Mapping

### 10.1 DPA 2019 §26 — Personal data endpoints

| Endpoint category | Data touched | Lawful basis | Access logging |
|---|---|---|---|
| `POST /bvr/v1/enrolments` | Voter PII + biometrics | Explicit consent | Yes — full payload hash |
| `GET /bvr/v1/voters/{id}` | Voter PII (role-filtered) | Public task | Yes — who accessed when |
| `POST /vprs/v1/applications` | Voter PII | Legitimate interest | Yes |
| `POST /cvr/v1/family-notifications` | Voter ID hash | Public task | Yes |
| `GET /bvv/v1/register/lookup` | Voter PII (own data only) | Consent (self-service) | Yes |
| `POST /compliance/v1/sar` | Requester PII | Legal obligation | Yes |
| `GET /oag/v1/evidence/{id}` | May contain PII | Legal obligation | Mandatory — access log |
| `GET /hr/v1/payroll-runs/{id}/lines` | Staff PII (salary) | Contract | Yes |
| `GET /court/v1/bundles/{id}/items/{id}` | May contain voter PII | Legal obligation | Mandatory — access log |

### 10.2 Elections Act evidentiary endpoints

Endpoints that return or accept Elections Act evidentiary materials require full audit logging with Merkle-anchored proof:
- All `form34*` endpoints (M10)
- `GET /portal/v1/elections/{id}/stations/{station_id}` (M11)
- `GET /reproducibility/v1/bundles/{id}` (M14)
- All `declarations/*` endpoints (M51)
- All `court/v1/bundles/*` endpoints (M40)
- All `rla/v1/runs/*` endpoints (M21)

---

## 11. Gaps

The following API surfaces are referenced in module specifications but not fully defined:

| Gap | Referenced in | Notes |
|---|---|---|
| Judiciary CTS inbound REST API | M40, M58 | Depends on Judiciary CTS API contract not yet finalised; provisional HMAC webhook placeholder |
| IPRS real-time query API | M04, M35 | IPRS API format subject to DCI / NIS data-sharing MoU; provisional REST + mTLS placeholder |
| ABIS external vendor API | M03 | Depends on chosen engine (IDEMIA / Innovatrics / Thales); SDK-wrapped, no direct REST exposure |
| Parliament e-tabling API | M58, M41 | Parliament IT system integration; provisional email + structured payload fallback |
| KEBS/OEM warranty feed API | O04 | Mechrevo / OEM API not yet specified |
| County Government IT APIs | M56 | 47 county government IT systems have inconsistent API maturity; REST + CSV fallback required |
| PPRA (Public Procurement Authority) | M54 | Procurement compliance webhook not yet specified |
| KEOG / ELOG / Carter Center data exchange | M57 | Data exchange format TBD; provisional JSON + signed email |
| LoRa mesh fallback gateway API | M08, M10 | Field resilience layer; LoRa gateway API spec deferred to network infrastructure phase |
| IPFS pinning service internal API | M11, M14 | On-prem IPFS node; API confirmed as standard IPFS HTTP API v0 |

---

## 12. Standards and Formats Reference

| Standard | Application |
|---|---|
| OpenAPI 3.1 | All REST endpoint definitions |
| RFC 7807 Problem Details | All error responses |
| CloudEvents 1.0 | All Kafka event envelopes |
| OAuth 2.1 / OIDC | Human principal authentication |
| SPIFFE / SPIRE | Service-to-service identity |
| mTLS (TLS 1.3) | All service-to-service and kit-to-server channels |
| PKCS#11 / KMIP | HSM interface |
| RFC 3161 | Trusted timestamp for audit events |
| UUID7 | All IDs |
| RFC 3339 UTC | All timestamps |
| HTTP/2 | Preferred; HTTP/3 where supported |
| gRPC + Protobuf 3 | ABIS internal, high-throughput internal calls |
| Server-Sent Events (SSE) | Real-time server-push (M16, M19) |
| WebSocket | War-room bidirectional (M16 ECC) |
| HMAC-SHA256 | Webhook payload signing |
| JSON Schema Draft 2020-12 | Schema validation in all services |
