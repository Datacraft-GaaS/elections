# BVV (M07)

Module M07 (`bvv`) owned by `bvi-svc`. Biometric Voter Verification kiosk and web lookup: lets citizens confirm their registration status, polling station, and ward assignment before election day. Enforces a timing-constant response shape, daily lookup cap, and a voter-objection filing path.

## Contents

1. What M07 does
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

## 1. What M07 does

M07 is a read path over the voter register exposed to citizens and accredited agents. A citizen enters their national ID number (or hashes it client-side) and retrieves: confirmation of registration, assigned polling station, ward, constituency, and county. The response shape is timing-constant to prevent enumeration attacks. Citizens may also file an objection (challenge their own registration data) via the same surface.

M07 is not a biometric surface: no biometric data is captured or returned.

## 2. Data plane

Served from the BVI back-end read models; no additional M07-specific migrations. Consumer of `services/bvi-svc/migrations/V7__voter_lookup_audit.sql`.

| Aggregate (read model) | Purpose | Data class |
|---|---|---|
| **VoterLookupResult** | voter_id_hash, registration_status (REGISTERED/NOT_FOUND), polling_station_name, ward, constituency, county, last_verified_at | B (derived) |
| **VoterLookupAudit** | lookup_id, requester_type (CITIZEN / AGENT / KIOSK), national_id_hash, ts, outcome | C |
| **Objection** | objection_id, national_id_hash, reason_code, description, status (OPEN/RESOLVED/DISMISSED), filed_at, resolved_at | B |

Daily lookup cap per national_id_hash: 5 lookups/day (configurable); enforced in Valkey with PG-durable fallback.

## 3. Ingest sources

| Source | Notes |
|---|---|
| Citizen (web portal) | HTTPS; hashed national ID submitted client-side; no raw ID reaches server |
| Citizen (kiosk) | Touch-screen kiosk at the Commission offices and county hubs; same backend API |
| Accredited party agent | JWT + SpiceDB `PARTY_AGENT` role; uncapped lookup with audit |
| Register read models (BVR M02) | Voter registration status consumed from bvr-svc read projection |

## 4. Pipeline

Crates: `services/bvi-svc/crates/` (portal-api and inspection crates in `portal-svc` supply the kiosk frontend; bvi-svc supplies the lookup backend). Voter lookup routes: `services/bvi-svc/crates/` (identification routes + voter_lookup_audit).

```
citizen submits hashed national_id (client-side SHA-256 + salt)
      -> rate-limit check: <= 5 lookups/day per id_hash (Valkey + PG)
             exceeded -> 429 with retry-after; response shape identical to not-found
      -> timing-constant lookup against register read model:
             found     -> VoterLookupResult (status, station, ward, ...)
             not found -> identical response shape, status = NOT_FOUND
             (no timing difference between found and not-found)
      -> VoterLookupAudit entry appended (V7)
      -> response returned to citizen

objection filing path:
      citizen POST /bvv/v1/objections
        -> validate national_id_hash + OTP confirmation
        -> create Objection record (OPEN)
        -> notify Commission officer queue (bvr-admin)
        -> officer reviews -> resolves or dismisses
        -> citizen notified via SMS (Africa's Touching / gateway)
```

## 5. API surface

Contract: `packages/openapi-contracts/bvi/openapi.yaml` (shared with M06).

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /bvv/v1/lookup` | Public (rate-limited; national_id_hash query param) | Check registration status and polling station |
| `POST /bvv/v1/objections` | Public (OTP-verified) | File registration data objection |
| `GET /bvv/v1/objections/{id}` | Citizen (session cookie) | Track objection status |
| `GET /bvv/v1/lookup/audit` | Commission admin (JWT + SpiceDB) | Audit trail of all lookups |
| `PATCH /bvv/v1/objections/{id}` | Commission officer (JWT + SpiceDB) | Resolve or dismiss objection |

All responses carry `Cache-Control: no-store`; no voter data cached at edge.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| VPRS / BVV web portal (Astro) | `services/portal-svc/astro/` (shared portal) | Citizen web lookup + objection filing |
| Kiosk app | Touch-screen PWA at the Commission offices | In-person self-service lookup |
| USSD shortcode | `integrations/ussd-gateway/` (Africa's Touching) | Feature-phone citizen lookup via `*860#` or designated shortcode |

Languages: English + Kiswahili minimum; extensible to 11. WCAG 2.1 AA; mobile-first. Kiosk: large-text mode; audio readout; no biometric capture.

## 7. Integrity + anti-tamper

- **Timing-constant response** -- lookup handler adds random jitter and uses constant-time string comparison; `FOUND` vs `NOT_FOUND` responses are statistically indistinguishable in timing (EC-SEC-11 pattern).
- **Client-side hashing** -- raw national ID never transmitted; client salts and hashes before HTTP request.
- **Daily lookup cap** -- Valkey rate-limit keyed on `id_hash + /64 source IP`; PG-durable fallback prevents Valkey eviction from bypassing cap.
- **Audit log** -- every lookup appended to `eel_bvi_voter_lookup_audit`; requester type and outcome logged; no raw ID stored.
- **OTP gate on objection** -- citizen must OTP-verify phone number before objection is persisted; prevents anonymous spam.
- **No voter biographic fields returned** -- only status, polling station name, ward, constituency, county; no name, DOB, or ID number echoed back.
- **TLS 1.3** on all endpoints; `Secure` + `HttpOnly` + `SameSite=Strict` on all session cookies.

## 8. Kenya statute alignment

- **Elections Act s.6A (public inspection)** -- citizen right to verify registration and polling station assignment before election day; objection path provides formal challenge mechanism.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- client-side hashing; no raw ID stored; only aggregated fields returned; no biometrics.
- **Kenya Data Protection Act 2019 s.33 (consent)** -- citizen initiates every lookup; no passive data collection.
- **Elections (General) Regulations Reg. 21 (voter list public inspection)** -- digital inspection via BVV satisfies public inspection requirement alongside the printed voter list.

## 9. Honest caveats + external prerequisites

- **USSD shortcode** (`*860#` or designated) requires Safaricom / Africa's Touching commercial arrangement; adapter is written; live shortcode pending.
- **Kiosk hardware** deployment (the Commission offices, county hubs) is an operations prerequisite; software is ready.
- **Objection resolution SLA** -- no SLA is encoded in the spec; Commission officers resolve via bvr-admin queue; response time is an operational commitment.
- **Rate cap calibration** -- 5 lookups/day is a configurable default; the Commission may adjust based on legitimate multi-device citizen patterns.
- BVV does not verify biometrics; a citizen who finds their registration incorrect must visit a BVR station or use the formal objection path.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (timing-constant lookup, rate-limit logic, OTP validation, objection FSM) | nextest | M07-UNIT- |
| Property (response shape identical for found vs not-found under any timing probe) | proptest | M07-PROP- |
| Contract (portal consumer; USSD gateway consumer) | Pact | M07-PACT- |
| Integration (Valkey + PG rate-limit; bvr-svc read model double) | nextest + testcontainers | M07-INT- |
| Load (10M concurrent lookup scenario; USSD gateway flood) | k6 | M07-LOAD- |
| Security (enumeration probe; timing side-channel measurement; CAPTCHA bypass) | ZAP + manual timing | M07-SEC- |

Runbook (to create): `docs/runbooks/bvv-degradation.md` (USSD gateway down; rate-cap Valkey eviction; objection queue backlog).

Full spec: `docs/plans/modules/bvv.md`. Service crates: `services/bvi-svc/crates/`. Audit migration: `services/bvi-svc/migrations/V7__voter_lookup_audit.sql`.
