# VPRS (M04)

Module M04 (`vprs`) owned by `vprs-svc`. Voter Pre-Registration System: a PWA that lets citizens pre-fill their biographic data online before attending a BVR station, reducing kit-side data-entry time. Issues a signed QR handoff token that the BVR kit redeems to pre-populate the enrolment form. Includes IPRS circuit-breaker, enumeration defences, and OPA hard-close at T-60 / T-30 freeze gates.

## Contents

1. What M04 does
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

## 1. What M04 does

M04 decouples biographic data entry from biometric capture. A citizen visits the VPRS PWA, enters their details, optionally validates against IPRS, and receives a time-limited QR code. At the BVR station, the enrolment clerk scans the QR; M01 (bvr-device) redeems the token against M04 to pre-populate all 22 biographic fields. Biometrics are never captured by M04.

M04 also enforces the gazetted pre-registration window via OPA policy; after T-60 (60 days before election), new pre-registration applications are hard-closed.

## 2. Data plane

Crates: `services/vprs-svc/crates/vprs-domain/`, `vprs-api/`. Migrations: `services/vprs-svc/migrations/` (no SQL file listing visible; directory confirmed present).

| Aggregate | Purpose | Data class |
|---|---|---|
| **VprsApplication** | application_id (UUID7), national_id_hash, 22 biographic fields, status (DRAFT/SUBMITTED/VALIDATED/EXPIRED/REDEEMED), iprs_match, otp_verified_at | A |
| **HandoffToken** | token_id, application_id, signed_qr_payload (Ed25519), expires_at (T+48 h), redeemed_at | B |
| **OtpSession** | session_id, application_id, otp_hash, attempt_count, expires_at | C |
| **AuditEvent** | event_id, actor (citizen hash / officer), action, ts_hlc | C |

Voter OTP delivered via SMS (Africa's Talking gateway); IPRS validation against `integrations/iprs/` with circuit breaker.

## 3. Ingest sources

| Source | Notes |
|---|---|
| Citizen self-service (PWA) | HTTPS; no auth before OTP; anti-enumeration via constant-time response and CAPTCHA |
| IPRS (optional validation) | `integrations/iprs/`; circuit breaker via `packages/eel-resilience-rs`; IPRS down does not block submission |
| BVR kit (token redemption) | gRPC SVID mTLS; M01 calls `GET /vprs/v1/handoff/{token}` once; token marked REDEEMED |
| OPA (freeze gate) | Hard-close policy loaded from OPA bundle; evaluated at transaction time |

## 4. Pipeline

Crates: `services/vprs-svc/crates/vprs-api/src/routes/`, `vprs-domain/src/application.rs`, `vprs-domain/src/otp.rs`.

```
citizen submits 22 biographic fields (HTTPS, CAPTCHA)
      -> input validation (NFC normalise, DOB >= 18, MRZ check-digit)
      -> OPA hard-close check: reject if inside T-60 freeze window
      -> enumeration defence: constant-time response; no field-level
         error that reveals register membership
      -> IPRS lookup (optional, circuit-breaker guarded):
             match     -> iprs_match = CONFIRMED
             no-match  -> iprs_match = UNCONFIRMED, application continues
             IPRS down -> iprs_match = UNAVAILABLE, application continues
      -> OTP generation -> SMS via Africa's Talking gateway
      -> citizen enters OTP (rate-limited: 3 attempts, 10-min lockout)
      -> VprsApplication status -> VALIDATED
      -> sign HandoffToken (Ed25519, 48 h TTL)
      -> emit QR code to citizen (PWA display + print)
      -> BVR kit scans QR -> POST /vprs/v1/handoff/{token}
      -> token verified (signature, TTL, not-redeemed)
      -> biographic fields returned to M01
      -> token status -> REDEEMED; no re-use
      -> emit eel.vprs.application_submitted.v1
             eel.vprs.handoff_redeemed.v1 (Kafka eel.vprs.v1)
```

## 5. API surface

Contract: `packages/openapi-contracts/vprs/portal.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /vprs/v1/applications` | Public (CAPTCHA) | Submit pre-registration application |
| `POST /vprs/v1/applications/{id}/otp/verify` | Public (rate-limited) | Verify OTP; returns signed token on success |
| `GET /vprs/v1/applications/{id}/status` | Citizen (session cookie) | Check application status |
| `GET /vprs/v1/handoff/{token}` | BVR kit (SVID mTLS) | Redeem token; return pre-filled fields |
| `GET /vprs/v1/health` | Internal | Health check |

All write endpoints enforce `Idempotency-Key`; GET /handoff is idempotent by token state (REDEEMED after first call).

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Citizen PWA | `services/vprs-svc/ex/` (Elixir Phoenix LiveView or static PWA) | Pre-registration form + QR display + status check |
| Enrolment clerk (kit) | M01 bvr-kit app | QR scan + form pre-population |

Languages: English + Kiswahili minimum; extensible to 11. WCAG 2.1 AA; mobile-first responsive. Offline PWA shell with service worker (biographic fields cacheable; token generation requires connectivity).

## 7. Integrity + anti-tamper

- **HandoffToken signed with Ed25519** (kit_pki key from P04); BVR kit verifies signature before accepting pre-fill; tampered tokens rejected.
- **Single-use token** -- `REDEEMED` state set atomically on first successful redemption; subsequent calls return 409.
- **OPA hard-close** -- pre-registration window enforced at transaction time; no bypass possible via race.
- **Enumeration defence** -- constant-time response shape for all application lookups; no field-level distinction between "not found" and "exists"; CAPTCHA + rate-limit on submission.
- **OTP rate-limit** -- 3 attempts then 10-min lockout; OTP hash stored (not plaintext); bcrypt-hashed attempt.
- **IPRS circuit breaker** -- IPRS outage does not block citizens; `iprs_match = UNAVAILABLE` noted in application record; BVR clerk sees flag at station.
- **Audit log** -- every application creation, OTP event, and token redemption logged to T1-audit with actor hash and HLC timestamp.
- **TLS 1.3 minimum** on all endpoints; PQ-hybrid where supported; no sensitive cookies without `Secure` + `HttpOnly` + `SameSite=Strict`.

## 8. Kenya statute alignment

- **Elections Act s.6A (enrolment window)** -- OPA hard-close enforces the gazetted pre-registration window; T-60 freeze prevents post-deadline applications.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- biometric data never captured; national ID number hashed in storage; OTP hash not plaintext.
- **Kenya Data Protection Act 2019 s.31 (security)** -- field validation, parameterised queries, TLS 1.3, OTP rate-limiting.
- **Kenya Data Protection Act 2019 s.33 (consent)** -- citizen initiates application; OTP confirms intent; no passive data collection.

## 9. Honest caveats + external prerequisites

- **Africa's Touching SMS gateway** commercial arrangement required for OTP delivery; adapter written and double available; live allocation pending.
- **IPRS MoU** is an external prerequisite; double is shipped under `integrations/iprs/`; live IPRS endpoint must be provisioned by IEBC.
- **OPA bundle distribution** depends on P04 (IAM platform); OPA policy source lives in `config/opa/`; bundle must be published before G3.
- VPRS is optional: citizens who do not pre-register enrol directly at the BVR station with no degradation to register integrity.
- Migration files for `vprs-svc` are confirmed present (`services/vprs-svc/migrations/`) but SQL filenames were not enumerated in the available listing; file count unverified.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (OTP hash, token signing/verify, OPA policy eval, enumeration-defence response shape) | nextest | M04-UNIT- |
| Property (token never redeemable twice; freeze gate respected at any concurrency) | proptest | M04-PROP- |
| Contract (M01 handoff redemption; IPRS double) | Pact | M04-PACT- |
| Integration (OTP flow; IPRS double; OPA server) | nextest + testcontainers | M04-INT- |
| E2E (citizen PWA -> QR -> kit redemption on staging) | Playwright | M04-E2E- |
| Security (OWASP ASVS L2; enumeration probe; CAPTCHA bypass) | ZAP + manual | M04-SEC- |

Runbook (to create): `docs/runbooks/vprs-svc-degradation.md` (IPRS down; SMS gateway down; OPA bundle stale).

Full spec: `docs/plans/modules/vprs.md`. Service crates: `services/vprs-svc/crates/vprs-api/`, `vprs-domain/`.
