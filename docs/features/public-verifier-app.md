# Public verifier app (M13)

Module M13 (`public-verifier-app`) owned by `field-mobile-svc`. 10M-concurrent voter verification surface via mobile-bff (Rust), USSD shortcode, WhatsApp webhook, IVR, and PWA; binary integrity check on the mobile app; clone detection for fraudulent app impersonation.

## Contents

1. What M13 does
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

## 1. What M13 does

M13 is the high-scale citizen verification layer. On election day and in the days before, citizens use it to confirm their registration status and polling station via their preferred channel: mobile app (PWA), USSD shortcode (`*860#` or designated), WhatsApp, or IVR phone call. The mobile-bff (Rust, `crates/mobile-bff/`) is designed to sustain 10M concurrent requests by serving cached, signed snapshots and deferring live register lookups to a circuit-breaker-guarded path. Clone detection (`crates/mobile-bff/src/handlers/clone_detect.rs`) flags fraudulent apps impersonating the official verifier.

## 2. Data plane

Migrations `V1`-`V6` under `services/field-mobile-svc/migrations/`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **VerificationQuery** | query_id, channel (PWA/USSD/WA/IVR), national_id_hash, response_status, response_ts, source_ip_hash | C |
| **ParticipationTokenLookup** | lookup_id, token_hash, redeemed, ts (V2) | C |
| **GrievanceIntake** | grievance_id, channel, description_hash, status (OPEN/RESOLVED), filed_at | B |
| **UssdSession** | session_id, msisdn_hash, step, state, ts (V6) | C |
| **AuditShadow** | event_id, actor_hash, action, ts (V4) | C |

No voter PII stored by M13; national_id_hash and msisdn_hash only.

## 3. Ingest sources

| Source | Notes |
|---|---|
| PWA (mobile browser) | HTTPS; client-side hashing before submission; service worker for offline shell |
| USSD gateway (Safaricom / Africa's Touching) | `integrations/ussd-gateway/`; `*860#`; USSD session state in V6 |
| WhatsApp Business API | `packages/openapi-contracts/whatsapp/webhook.yaml`; webhook POST from Meta |
| IVR (Africa's Touching voice) | `packages/openapi-contracts/ivr/webhook.yaml`; audio prompt + DTMF input |
| BVV read model (M07) | Cached verification results from bvi-svc read projection |
| Participation token (M14) | Token hash lookup for bug-bounty and transparency participation |

## 4. Pipeline

Crates: `services/field-mobile-svc/crates/mobile-bff/src/handlers/verify.rs`, `clone_detect.rs`, `sse_handler.rs`, `grievance.rs`.

```
citizen submits national_id_hash via any channel
      -> mobile-bff rate-limit: channel-specific caps (Valkey + PG fallback)
      -> cache-first lookup (Valkey):
             cache hit  -> return signed cached result (max-age 5 min)
             cache miss -> forward to BVV M07 read model (circuit-breaker guarded)
      -> response shape: registration_status, polling_station_name, ward,
                          constituency, county
             NOT_FOUND -> identical response shape (timing-constant)
      -> log VerificationQuery (hashed only)
      -> SSE push on result (sse_handler.rs): live channel notifies PWA clients

USSD path:
      ussd-gateway POST -> UssdSession state machine (V6)
      step 1: prompt national_id last 6 digits
      step 2: verify + respond (2 SMS segments max)
      session timeout 120 s; state stored in Valkey

WhatsApp path:
      Meta webhook POST -> parse structured message
      -> route to same verify handler
      -> respond via WhatsApp Business API reply

IVR path:
      Africa's Touching webhook POST -> parse DTMF input
      -> route to same verify handler
      -> text-to-speech response in EN / SW

clone detection:
      mobile-bff/src/handlers/clone_detect.rs:
      -> binary hash of PWA app bundle compared against signed canonical hash
         (published by portal-svc tarball)
      -> request from non-canonical binary -> flagged; ECC alert
      -> rate-spike from single source -> clone signal; IP hash logged
```

## 5. API surface

Contracts: `packages/openapi-contracts/field-mobile-bff/bff.yaml`, `verifier/verify.yaml`, `verifier/ussd.yaml`, `verifier/whatsapp.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /verify/v1/check` | Public (rate-limited) | PWA / API verification lookup |
| `POST /ussd/v1/verify` | USSD gateway (secret) | USSD session step handler |
| `POST /wa/v1/verify` | WhatsApp webhook (Meta HMAC) | WhatsApp message handler |
| `POST /ivr/v1/verify` | IVR webhook (Africa's Touching secret) | IVR DTMF handler |
| `POST /verify/v1/grievance` | Public (OTP-verified) | File verification grievance |
| `GET /verify/v1/participation-token/{hash}` | Public | Participation token lookup (M14) |
| `GET /mobile-bff/v1/health` | Internal | Health check |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| PWA verifier | `services/field-mobile-svc/apps/` (Flutter / React PWA) | Primary mobile verification surface |
| USSD `*860#` | `integrations/ussd-gateway/` via Africa's Touching | Feature-phone citizens |
| WhatsApp Bot | `packages/openapi-contracts/whatsapp/` | WhatsApp Business API integration |
| IVR (`packages/openapi-contracts/ivr/`) | Africa's Touching voice | Audio verification for non-literate citizens |
| Grievance intake | Same PWA or USSD | File verification complaint |

Languages: English + Kiswahili on all channels; IVR also supports Kikuyu, Luo, Kamba (configurable TTS locale set). WCAG 2.1 AA on PWA. USSD / IVR numeric steps <= 5 (EC-HUM-10).

## 7. Integrity + anti-tamper

- **Cache-first with signed snapshots** -- Valkey cache serves signed BVV results; Merkle proof available for any cached result; cache key includes signature version.
- **Clone detection** -- `clone_detect.rs` compares incoming PWA binary hash against signed canonical hash from portal tarball; mismatch triggers ECC alert; fraudulent app users get canonical redirect.
- **Timing-constant response** -- NOT_FOUND and FOUND responses are indistinguishable in shape and timing; no enumeration signal.
- **USSD session integrity** -- session state stored server-side (Valkey + PG fallback); MSISDN hash only; no raw phone number stored.
- **WhatsApp HMAC verification** -- Meta webhook payload HMAC-SHA256 verified on every inbound message; replayed or tampered messages rejected.
- **IVR webhook signature** -- Africa's Touching webhook secret verified on every request.
- **Participation token integrity** -- token hash lookup returns only boolean (valid/invalid); no token plaintext echoed.
- **Audit shadow** (`V4__audit_shadows.sql`) -- every verification query logged with channel, ts, and result (hashed only).
- **Rate-limit defence** -- Valkey keyed on `id_hash + channel + /64 IP`; flood triggers clone-detect escalation; Alertmanager page.

## 8. Kenya statute alignment

- **Elections Act s.6A (public inspection)** -- citizen right to verify registration status is provided via all channels.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- national_id_hash and msisdn_hash only; no raw IDs or phone numbers stored; results are aggregated civic data, not personal data.
- **Kenya Data Protection Act 2019 s.33 (consent)** -- citizen initiates every query; no passive profiling.
- **Communications Authority of Kenya (USSD/IVR)** -- USSD shortcode and IVR number allocations are regulatory prerequisites; adapters ready; allocations pending.

## 9. Honest caveats + external prerequisites

- **10M concurrent** SLO requires CDN edge capacity for the PWA static shell and Valkey cluster scaled for peak read throughput; origin mobile-bff targets <= 50k concurrent connections at origin.
- **Africa's Touching + Meta commercial arrangements** for USSD, WhatsApp, and IVR are external prerequisites; adapters and doubles are ready; live credentials pending.
- **Communications Authority USSD shortcode allocation** (`*860#` or designated) is a regulatory prerequisite; timeline outside this repo.
- **Clone detection accuracy** depends on the canonical binary hash being published to the portal tarball before app distribution; timing gap between tarball publication and app store availability must be managed operationally.
- IVR TTS voice quality for Kenyan indigenous languages depends on the TTS engine vendor; Africa's Touching TTS locale support must be confirmed for Kikuyu, Luo, and Kamba.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (verify handler, USSD FSM, WA HMAC verify, clone detect, rate-limit) | nextest | M13-UNIT- |
| Property (timing-constant response; rate-limit enforced at any concurrency) | proptest | M13-PROP- |
| Contract (BVV M07 lookup; USSD gateway; WA webhook; IVR webhook) | Pact | M13-PACT- |
| Integration (Valkey + PG; BVV double; USSD gateway double) | nextest + testcontainers | M13-INT- |
| Load (10M simulated lookups; USSD flood; WA batch) | k6 + Gatling | M13-LOAD- |
| Security (enumeration probe; timing side-channel; HMAC bypass; clone inject) | ZAP + manual | M13-SEC- |

Runbooks (to create): `docs/runbooks/field-mobile-svc-degradation.md` (USSD gateway down; WA webhook down; Valkey eviction; clone alert).

Full spec: `docs/plans/modules/public-verifier-app.md`. Crates: `services/field-mobile-svc/crates/mobile-bff/`, `dispatch-api/`. Migrations: `services/field-mobile-svc/migrations/V1`-`V6`.
