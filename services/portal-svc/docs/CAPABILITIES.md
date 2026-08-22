# portal-svc capabilities and algorithms

Comprehensive capability catalogue for the public election portal. Every capability lists
the algorithm, the source reference (paper, ISO/IETF/W3C/statutory standard), and the
crate or component that implements it.

## Contents

1. Modules and capabilities overview
2. End-to-end publication pipelines
3. Results publication stack (M11)
4. Election calendar and gazette stack (M52)
5. Voter list inspection stack (M64)
6. Referendum explainer stack (M68)
7. Merkle proof and inclusion verification
8. Feed and subscription channels
9. Anti-enumeration and privacy controls
10. LiveView real-time results dashboard
11. Operational excellence
12. Standards and interoperability
13. Validation and accessibility harnesses
14. Data lifecycle and security
15. API surface
16. Summary

---

## 1. Modules and capabilities overview

| Module | Slug | Size | Primary crate | Notable risk |
|---|---|---|---|---|
| M11 | public-portal | L | crates/portal-api | petition-window peak; cache poisoning |
| M52 | election-calendar-gazette | M | crates/gazette | Gov Printer API; statutory rule accuracy |
| M64 | voter-list-inspection | M | crates/inspection | PII enumeration; T-60 race condition |
| M68 | referendum-explainer | L | astro/explainer + py/ | WCAG AAA; translation quality |

Tier: T2. Availability SLO: 99.9% (election cycle E-7 to E+30). Non-election: 99.0%.

---

## 2. End-to-end publication pipelines

### 2.1 Results publication (M11)

```
rts.snapshot_published event
  -> Kafka consumer (eel.rts.snapshot.v1, exactly-once)
  -> portal-api ingest: upsert snapshot_index, generate leaf_hash
  -> cosign sign tarball + append RFC 6962 inclusion proof to transparency log
  -> CDN invalidation + eel:counts PubSub broadcast
  -> CountsLive / SseController push to viewers
```

State machine: `Received -> Indexed -> ProofGenerated -> Published -> { Archived | Disputed }`.

### 2.2 Gazette notice publication (M52)

```
gov_printer webhook or manual import
  -> statutory_rule engine: derive effective_date, notice_type, gazette_reference
  -> gazette crate persists CalendarEvent + GazetteNotice
  -> RSS 2.0 + Atom 1.0 feeds regenerated
  -> eel.portal.gazette_notice.published.v1 emitted
```

### 2.3 Voter list inspection (M64)

```
bvr.register.frozen event
  -> Roll snapshot frozen (no further writes accepted; T-60 race gate enforced)
  -> Objection / CorrectionClaim accepted via REST
  -> AutoReferralPacket generated for eCourt on deadline expiry
```

### 2.4 Referendum explainer (M68)

```
M49 referendum question content
  -> 11-language translation via NLLB-200 sidecar (py/tts_pipeline.py)
  -> TTS audio generation per language
  -> WCAG 2.2 AAA audit gated on CI
  -> Astro 5 SSR page with accessible HTML + embedded audio player
```

---

## 3. Results publication stack (M11)

### 3.1 Tarball signing and publication

Crate: `crates/portal-api/src/publication.rs`.

- Tarball: gzip archive of Form34 PDFs + JSON summary per constituency.
- Signing: cosign sign-blob with Sigstore Rekor Transparency Log upload (RFC 6962 STH inclusion proof).
- Verification widget: WebCrypto SHA-256 in-browser hash check against published digest; implemented in `astro/src/components/VerifyWidget.astro` using `SubtleCrypto.digest("SHA-256", ...)`.
- Inclusion proof delivery SLO: p95 <= 30 s (SLA.md FR-M11-019).

### 3.2 CDN cache strategy (migrations/V1__cache_control_hints.sql)

| Period | max-age | s-maxage |
|---|---|---|
| T+0 to T+7 (petition window) | 30 s | 30 s |
| T+8 to T+30 | 300 s | 300 s |
| T+31+ (archive) | 3600 s | 86400 s |

Cache-Control hints stored in `eel_portal_cache_hints` (V1 migration). Invalidation triggered by `eel.rts.snapshot_published.v1` consumer.

### 3.3 Snapshot index (migrations/V2__snapshot_index.sql)

`eel_portal_snapshot_index` table: constituency_id, leaf_hash, merkle_path JSONB, published_at, proof_verified_at, tarball_url.

### 3.4 Viewer session metrics (migrations/V3__viewer_session_metrics.sql)

`eel_portal_viewer_sessions`: session_id, constituency_id, started_at, proof_fetched BOOLEAN, channel (cdn/sse/ussd).

---

## 4. Election calendar and gazette stack (M52)

### 4.1 Statutory rule engine

Crate: `crates/gazette/src/rule_engine.rs`.

Drives gazette notice derivation from election calendar milestones. Rules encode Kenya Elections Act timetables: nomination deadlines, campaign period start/end, polling day, results declaration window, petition window open/close. Rule evaluation: `StatutoryRule { trigger_event, offset_days, notice_type } -> GazetteNotice`. Deterministic: same trigger input always yields the same notice date.

### 4.2 Calendar events and notices

| Entity | Table | Notes |
|---|---|---|
| CalendarEvent | eel_calendar_events | election_id, event_type, scheduled_date, statutory_authority |
| GazetteNotice | eel_gazette_notices | notice_id, gazette_ref, effective_date, notice_text, rule_id |
| StatutoryRule | eel_statutory_rules | rule_id, trigger_event, offset_days, notice_type |

### 4.3 Feed generation

Crate: `crates/gazette/src/feeds.rs`.

- RSS 2.0: `<rss version="2.0">` with `<item>` per notice (title, link, description, pubDate in RFC 822 format).
- Atom 1.0 (RFC 4287): `<feed>` with `<entry>` per notice; `<updated>` in RFC 3339; `<id>` as tag URI per RFC 4151.
- Feeds regenerated on each notice publication; ETag + Last-Modified cache headers.

---

## 5. Voter list inspection stack (M64)

### 5.1 Roll snapshot and freeze

Crate: `crates/inspection/src/roll.rs`.

Trigger: `bvr.register.frozen` event (Kafka, at-least-once idempotent). T-60 gate: roll snapshot accepted only if no later snapshot has arrived for the same election_id. Subsequent `bvr.register.frozen` events within 60 s of a prior are idempotent no-ops (prevents T-60 race condition per ADR-0012).

Entities:
- `eel_roll_snapshots`: snapshot_id, election_id, frozen_at, voter_count, roll_hash SHA-256.
- `eel_objections`: objection_id, voter_id_hash (SHA-256; no plaintext PII in this table), filed_at, status, assigned_officer.
- `eel_correction_claims`: claim_id, voter_id_hash, correction_type, evidence_url, status, auto_referral_sent_at.

### 5.2 Constant-time voter lookup

Crate: `crates/inspection/src/lookup.rs`.

All lookups are O(1) in constant wall-clock time via indexed hash lookup. The response body size and timing are padded to a fixed envelope to prevent timing-oracle attacks (Elections Act §6A privacy constraint). Daily per-IP lookup cap enforced at API gateway (default 50 lookups/IP/day) to prevent enumeration; cap stored in `eel_lookup_rate_limits` with daily reset.

### 5.3 Objection and correction workflow

Filing: citizen POSTs `POST /v1/portal/inspection/objections` with voter_id (hashed on ingest), objection_type (NAME_INCORRECT, DECEASED, DUPLICATE, WRONG_STATION), and evidence_doc_url (MinIO presigned PUT URL).

Lifecycle: `Filed -> UnderReview -> { Resolved | Rejected | AutoReferred }`. AutoReferral to eCourt generated by scheduled job when deadline_at approaches with status still UnderReview. AutoReferralPacket: objection_id, case_summary (no PII plaintext), statutory_deadline, referee_court_id; delivered via governance-svc M47 eCourt integration.

Officers assigned by workload-balanced round-robin from `eel_objection_officers` pool. Assignment triggers `eel.portal.objection.assigned.v1` event.

---

## 6. Referendum explainer stack (M68)

### 6.1 Multi-language TTS pipeline

Python worker: `py/tts_pipeline.py`.

- Translation: NLLB-200 (No Language Left Behind, Meta AI 2022) for Kiswahili, Kikuyu, Luhya, Luo, Kalenjin, Kamba, Somali, Meru, Giriama, Maasai, and English (11 languages matching Elections Act §33 requirement).
- TTS synthesis: VITS (Kim et al. ICML 2021) or eSpeak-NG fallback for languages without neural TTS data.
- Audio stored in MinIO; URL signed for CDN delivery.

### 6.2 Accessibility

- WCAG 2.2 Level AAA for M68 (AA for M11, M52, M64). ISO/IEC 40500:2012 (identical normative text to WCAG 2.0).
- axe-core automated audit runs in CI (`tests/axe_wcag.js`).
- Manual VPAT review gated before each referendum release.
- Alt-text on all images; ARIA landmarks on all page regions; colour contrast ratio >= 7:1 (AAA).

---

## 7. Merkle proof and inclusion verification

### 7.1 Proof generation

Crate: `crates/portal-api/src/proof.rs`.

- Merkle tree: binary hash tree over Form34 leaf hashes. Leaf: `SHA-256(form_id || constituency_id || form_hash || published_at)`.
- Inclusion proof: sibling hashes path from leaf to root (RFC 6962 Section 2.1 format).
- Signed Tree Head (STH): root_hash, tree_size, timestamp, cosign signature (Ed25519).
- STH anchor published to Sigstore Rekor; transparency-svc STH consumed via `eel.transparency.sth_published.v1`.

| Algorithm | Reference |
|---|---|
| SHA-256 | FIPS 180-4, NIST 2015 |
| Binary Merkle tree + inclusion proof | RFC 6962 (Laurie, Langley, Kasper 2013) |
| cosign blob signing | Sigstore cosign; Rekor transparency log |
| In-browser SHA-256 | W3C Web Cryptography API (SubtleCrypto), W3C 2017 |

### 7.2 verifier-rs WASM proof viewer

Client-side Rust WASM crate (`crates/portal-api/wasm/verifier.rs` compiled to WASM via wasm-pack). Accepts tarball URL, fetches locally, hashes with SHA-256, verifies Merkle path against published STH without sending voter data to any server.

---

## 8. Feed and subscription channels

| Channel | Standard | Crate / component | Notes |
|---|---|---|---|
| RSS 2.0 | RSS 2.0 spec (Winer 2003) | crates/gazette | gazette notices and calendar |
| Atom 1.0 | RFC 4287 (Nottingham, Sayre 2005) | crates/gazette | structured feed with entry IDs |
| Server-Sent Events (SSE) | W3C EventSource API | ex/portal_live SseController | live counts to CountsLive |
| Phoenix LiveView | Elixir LiveView 1.0 | ex/portal_live | authenticated dashboard |
| Kafka consumer | CloudEvents 1.0 | crates/portal-api | eel.rts.snapshot.v1 topic |

SSE endpoint: `GET /v1/portal/stream`: delivers `eel:counts` events (aggregate counts only; no PII per EVENTS.md).

---

## 9. Anti-enumeration and privacy controls

| Control | Mechanism | Crate / component |
|---|---|---|
| Daily voter lookup cap | 50 lookups/IP/day; `eel_lookup_rate_limits` reset nightly | crates/inspection |
| Constant-time lookup response | Fixed-size padded envelope; timing attack prevention | crates/inspection/src/lookup.rs |
| PII separation | Objection and correction tables store voter_id_hash (SHA-256), not plaintext voter_id | crates/inspection |
| Tarball download redirect | MinIO presigned URL; no voter PII in portal DB | crates/portal-api |
| Rate-limit on proof endpoint | 100 proofs/IP/hour via tower-governor middleware | crates/portal-api |
| Audit log | eel.portal.snapshot_served.v1 events: aggregate counts only, no PII (EVENTS.md) | crates/portal-api |

---

## 10. LiveView real-time results dashboard

Component: `ex/portal_live/` (Elixir Phoenix LiveView).

- Consumes `eel:counts` PubSub topic (strictly aggregate; no PII; per EVENTS.md).
- Renders constituency-level vote counts; updates on each `eel.rts.snapshot_published.v1` event.
- 60 s configurable delay on `eel.bvi.turnout_rollup_updated.v1` (per EVENTS.md).
- Scale target: 5M concurrent viewers at p95 < 200 ms, p99 < 500 ms (SLA.md AC-M11-01).
- Connection draining via Phoenix presence with max_connections per node; horizontal scale via Kubernetes HPA (min 3, max 50 replicas on election night).

---

## 11. Operational excellence

| Capability | Mechanism |
|---|---|
| CDN failover | Origin loss: CDN continues serving signed snapshots; stale label shown; RTO <= 30 s (SLA.md FR-M11-019) |
| Bulk tarball download | MinIO presigned redirect; CDN caches tarball; avoids origin saturation |
| Proof generation queue | portal-api enqueues proof tasks; SELECT FOR UPDATE SKIP LOCKED for at-most-once delivery |
| §6A roll lock failure | Manual operator intervention via admin API; rejected late submissions queued for review |
| TTS pipeline fallback | Audio unavailable: text fallback rendered; async retry on TTS job failure |
| WCAG drift detection | axe-core CI check on M68 pages; alert on violation count > 0 |
| k6 load test | `loadtest/k6_portal.js` targets 5M/hr; 30 s sustained; run before each election-night scale rehearsal |
| Gazette feed caching | ETag + Last-Modified on RSS and Atom feeds; CDN caches per-notice granularity for 300 s |
| Objection deadline enforcement | Scheduled job at T+21 (petition window close): auto-generates AutoReferralPackets for unresolved corrections |
| WASM module caching | verifier-rs WASM binary served with `immutable` Cache-Control; versioned hash in URL for cache-busting on release |
| LiveView heartbeat | Phoenix heartbeat every 30 s; client reconnects with exponential backoff (cap 60 s) on disconnect |

### 11.1 Deployment topology

Three-tier architecture:

```
[Astro 5 SSG] -> static HTML/JS/CSS/WASM -> [CDN (Cloudflare)]
                                                     |
                                             [portal-api (Rust, axum 0.8)]
                                                     |
                                       [Phoenix LiveView (Elixir)]
                                                     |
                                  [PostgreSQL 17 primary + 1 read replica]
                                  [Valkey cluster (registration cache)]
                                  [MinIO (tarballs, TTS audio)]
```

Astro 5 site built at CI time; static assets pushed to CDN. Portal-api handles all dynamic requests (proof generation, rate-limiting, event consumption). LiveView handles authenticated real-time views (CountsLive) and is deployed as a separate Kubernetes Deployment from portal-api.

HPA (portal-api): CPU 60 %, memory 75 %; min 3, max 50 replicas. HPA (LiveView): connections-per-pod metric; max 200 LiveView connections per pod; scale at 150 connections.

---

## 12. Standards and interoperability

| Standard | Coverage | Crate / component |
|---|---|---|
| WCAG 2.2 Level AA | M11, M52, M64 (all portal pages) | astro/, axe-core tests |
| WCAG 2.2 Level AAA | M68 referendum explainer | astro/explainer/, axe-core tests |
| ISO/IEC 40500:2012 | WCAG 2.0 normative equivalent; referenced in accessibility reports | astro/explainer/ |
| RFC 4287 Atom 1.0 | Gazette and calendar feed format | crates/gazette/src/feeds.rs |
| RSS 2.0 | Gazette notices feed | crates/gazette/src/feeds.rs |
| RFC 6962 | Merkle inclusion proof format; STH | crates/portal-api/src/proof.rs |
| W3C Web Cryptography API | In-browser SHA-256 verification | astro/src/components/VerifyWidget.astro |
| CloudEvents 1.0 | Event envelope for all consumed and emitted events | crates/portal-api |
| OpenAPI 3.1 | API contract; published at packages/openapi-contracts/portal/ | crates/portal-api |

---

## 13. Validation and accessibility harnesses

| Harness | Scope | Command |
|---|---|---|
| axe-core WCAG audit | M68 AAA; M11/M52/M64 AA | `cd tests && npm test axe_wcag.js` |
| Rust WCAG feature flag | Compile-time gate | `cargo test wcag:: --features wcag-audit` |
| Proof verification unit tests | Merkle path, STH anchor | `cargo test proof:: --lib` |
| k6 load test | 5M/hr peak, p99 latency | `cd loadtest && k6 run k6_portal.js --vus 5000 --duration 300s` |
| Inclusion proof SLO test | p95 <= 30 s end-to-end | `cargo test proof::slo --lib` |
| Cargo nextest integration | Full integration suite | `cargo nextest run` |
| Manual VPAT | M68 before each referendum release | Human review; checklist in docs/vpat-checklist.md |

---

## 14. Data lifecycle and security

### 14.1 Encryption

Form34 tarballs: encrypted at rest in MinIO (AES-256-GCM server-side). Voter-related hashes: SHA-256 one-way; plaintext voter IDs never stored in portal-svc tables. Gazette notice text: stored in plaintext (public documents by design); no PII in gazette content. TTS audio files: encrypted at rest in MinIO; MinIO presigned URLs expire after 24 h to limit distribution.

### 14.2 Audit

All snapshot serves logged via `eel.portal.snapshot_served.v1` (aggregate counts; no PII). Append-only audit trigger on `eel_portal_snapshot_index`. cosign signature on every tarball provides non-repudiation.

### 14.3 Retention

- `eel_portal_snapshot_index`: permanent (election archive).
- `eel_viewer_session_metrics` (V3): 90-day rolling partition; no PII columns.
- `eel_lookup_rate_limits`: reset daily; pruned at 30 days.
- `eel_objections`, `eel_correction_claims`: 7 years post-election per Elections Act §39(1C).

### 14.4 Kit attestation dependency

Portal-api verifies cosign signatures from transparency-svc before publishing any proof. Unsigned or unverified STHs are rejected.

### 14.5 Gazette notice immutability

`eel_gazette_notices` has an append-only trigger (INSERT only; UPDATE and DELETE blocked). Correction or supersession of a notice is recorded as a new notice with notice_type `SUPERSEDES` and a reference to the superseded notice_id. Superseded notices remain publicly accessible with a `SUPERSEDED` label in the API response; RSS and Atom feeds include both the original and superseding entries.

### 14.6 Per-request PII isolation

Portal-api applies a `PiiGuard` middleware layer (tower middleware in `crates/portal-api/src/middleware/pii_guard.rs`) on all routes that touch voter-related data. PiiGuard:
1. Verifies no plaintext voter_id appears in request path or body (regex scan on raw bytes).
2. Ensures response body contains no column named voter_id, national_id, or full_name.
3. Emits a `eel.portal.pii_guard.violation.v1` alert to ecc-svc if either check fails.
This provides defence-in-depth beyond schema-level hashing.

---

## 15. API surface

### 15.1 REST (`crates/portal-api`, axum 0.8, port 8080)

- `GET /v1/portal/results/{constituency_id}`: live Form34 results.
- `GET /v1/portal/proof/{leaf_hash}`: Merkle inclusion proof + STH anchor.
- `GET /v1/portal/gazette/notices`: search and list official notices (full-text, date range).
- `GET /v1/portal/gazette/notices/{id}/feed`: RSS 2.0 or Atom 1.0 (Accept header).
- `GET /v1/portal/inspection/roll`: §6A voter roll snapshot (frozen; no plaintext PII).
- `GET /v1/portal/inspection/objections`: public objection and correction records.
- `POST /v1/portal/inspection/objections`: file §6A objection.
- `GET /v1/portal/referendum/{id}/explainer`: explainer content + TTS URLs by language.
- `GET /v1/portal/stream`: SSE live counts (aggregate only).
- `GET /healthz` + `/readyz` + `/openapi.json`.

Every read endpoint enforces rate limits per /64 subnet. Idempotency-Key header required on all POST endpoints (backed by `eel_idempotency_keys`). All responses include `X-Proof-Hash` header for cache integrity.

### 15.2 Events (CloudEvents 1.0 via outbox)

Consumed:
- `eel.rts.snapshot_published.v1` (eel.rts.snapshot.v1, exactly-once)
- `eel.bvi.turnout_rollup_updated.v1` (eel.bvi.turnout.v1, at-least-once idempotent)

Emitted:
- `eel.portal.snapshot_served.v1` (eel.portal.audit.v1, at-least-once idempotent)
- `eel.portal.gazette_notice.published.v1`
- `eel.portal.objection.filed.v1`

### 15.3 LiveView dashboard (Elixir Phoenix, port 4000)

`ex/portal_live/`: authenticated role `portal_viewer`; SPIRE SVID required for admin routes. Public results available without authentication.

---

## 16. Summary

Total footprint: 3 Rust crates (portal-api, gazette, inspection) + 1 Elixir LiveView app + 1 Astro 5 frontend + 1 Python TTS pipeline + 3 PostgreSQL migrations (V1-V3) + WASM verifier. Covers M11 (live results, cosign publication, inclusion proofs), M52 (gazette + calendar + Atom/RSS feeds), M64 (§6A voter list inspection with anti-enumeration controls), and M68 (WCAG 2.2 AAA referendum explainer with 11-language TTS). Key production inputs remaining: Gov Printer API credential for M52 automated ingestion; final VPAT sign-off for M68 before referendum activation.

### 16.1 Inter-service dependencies

| Upstream service | Data consumed | Failure mode |
|---|---|---|
| rts-svc | `eel.rts.snapshot_published.v1` (M11 live counts) | Counts frozen at last snapshot; stale label shown on LiveView |
| bvr-svc | `bvr.register.frozen` (M64 roll snapshot) | Roll snapshot not updated; existing frozen snapshot served |
| transparency-svc | STH for cosign proof verification (M11) | Proof publication blocked; tarball still available without proof |
| biometric-edge-svc | (indirect via rts-svc) | No direct dependency |
| ai-platform-svc | NLLB-200 translation via ml/vault (M68) | TTS generation fails; text fallback rendered; async retry queued |
| governance-svc | AutoReferralPacket delivery (M64) | Packets queued locally; retry on governance-svc recovery |
