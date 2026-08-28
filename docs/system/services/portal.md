# portal-svc

## 1. Overview

portal-svc is the public-facing results portal (module M11): signed result snapshots with
Merkle proofs, bulk snapshot downloads, a transparency dashboard with client-verifiable
inclusion proofs, historical append-only snapshot access, an RSS feed, live counts via
Elixir SSE/LiveView, gazette/calendar publishing, and voter-roll inspection/objection APIs.

- **Runtime**: Rust axum (`crates/portal-api`, plus `gazette` and `inspection` router
  crates), Astro static site (`astro/public`), Elixir Phoenix (`ex/portal_live`),
  Python FastAPI TTS service (`py/tts`).
- **Port**: portal-api binds `0.0.0.0:8090`, hard-coded
  (services/portal-svc/crates/portal-api/src/main.rs:5–11). Ports for ex/portal_live,
  py/tts, and the gazette/inspection routers are not set in code (not verified; Elixir/
  FastAPI defaults apply when run standalone).
- **Entrypoints**: crates/portal-api/src/main.rs; ex/portal_live/lib/portal_live/application.ex;
  py/tts/src/eel_tts/main.py.
- Peak target: 5M visitors/hour, p95 <= 200 ms, p99 <= 500 ms (TRACEABILITY.yaml FR-M11-022).

## 2. Ownership & Boundaries

Ground truth: services/portal-svc/TRACEABILITY.yaml (M11, WP-3.9) and
docs/architecture/services/portal.md.

| M-ID | Module | Implementation |
|---|---|---|
| M11 | public-results-portal | astro/public site, crates/portal-api |
| — | gazette calendar/notices | crates/gazette (not mounted in portal-api — see gaps) |
| — | roll inspection/objections/corrections | crates/inspection (not mounted in portal-api) |

- **Data ownership**: snapshot_index (append-only), cache_control_hints,
  viewer_session_metrics. No PII in any table (enforced by migration comments and
  FR-M11-026 "no PII returned from any endpoint", crates/portal-api/src/lib.rs:9).
- **Upstream**: rts-svc — consumes `eel.rts.snapshot_published.v1` to populate
  snapshot_index (migration V2 header comment); consumer code not present in this crate.
- **Downstream**: none (leaf service). Edge/WAF/CDN tiering per ADR-0010/ADR-0011.

## 3. Repository Layout

```
services/portal-svc/
  TRACEABILITY.yaml         # FR-M11 requirement → implementation map
  crates/
    portal-api/             # public snapshots API (lib.rs routes+handlers, main.rs)
    gazette/src/api.rs      # /calendar + /gazette + /subscriptions router
    inspection/src/api.rs   # /rolls + /objections + /corrections + auto-referral
  astro/public/             # Astro site: pages index/constituency/station/transparency/
                            # history, rss.xml.ts, i18n, tests
  astro/explainer/          # explainer site (playwright tests)
  ex/portal_live/           # LiveView live counts + SSE controller
  py/tts/                   # text-to-speech render API for accessibility
  migrations/V1..V3*.sql    # cache_control_hints, snapshot_index, viewer_session_metrics
  tests/                    # axe_wcag.js (a11y), rate_limit_test.rs
  loadtest/k6_portal.js     # k6 thresholds for FR-M11-022
  deploy/service.yaml       # kustomize descriptor
```

## 4. Capabilities & Functions

### portal-api (mounted at /api/v1/portal)

Routes: crates/portal-api/src/lib.rs:206–212 (`pub fn app()`).

| Method | Path | Handler (lib.rs line) | Purpose |
|---|---|---|---|
| GET | /api/v1/portal/health | `health` :137 | Liveness JSON |
| GET | /api/v1/portal/snapshots/latest | `snapshots_latest` :147 | Latest signed snapshot summary |
| GET | /api/v1/portal/snapshots/{id} | `snapshot_by_id` :169 | Snapshot by UUID |
| GET | /api/v1/portal/snapshots/bulk.tar.gz | `snapshots_bulk` :191 | 302 to MinIO pre-signed URL |
| GET | /api/v1/portal/snapshots/{id}/verify | `snapshot_verify` :198 | Merkle inclusion proof + STH |

All snapshot responses carry signing/cache headers via `signing_headers()`
(lib.rs ~line 100–124): `X-Eel-Merkle-Root`, `X-Eel-Snapshot-Id`,
`X-Content-Type-Options: nosniff`.

### gazette crate (router not nested into portal-api app())

Routes: crates/gazette/src/api.rs:26–43 — `/calendar/events` (create/list),
calendar event detail/sign, `/gazette/notices` (create/list), notice sign (:39)/submit
(:40), `/subscriptions` create (:41)/delete (:42).

### inspection crate (router not nested into portal-api app())

Routes: crates/inspection/src/api.rs:23–36 — roll snapshots (create/get), `roll_lookup`,
objections file/get/status-patch, corrections file/get/decide,
`POST /corrections/auto-referral/run` (`run_auto_referral`, api.rs:208; logic in
src/auto_referral.rs).

### Astro public site

Pages (astro/public/src/pages/): `index.astro` (home dashboard, FR-M11-001),
`constituency/[code].astro` and `station/[code].astro` (drill-down, FR-M11-002/003),
`transparency.astro` (Merkle roots + WASM verifier-rs, FR-M11-006/011),
`history/index.astro` (FR-M11-007), `rss.xml.ts` (RSS feed, FR-M11-013).

### Elixir live counts

`GET /live/counts` LiveView (ex/portal_live/lib/portal_live/router.ex:20; component
mount at lib/portal_live/live/counts_live.ex:16 with stale-marking `handle_info`
:33/:45) and `GET /api/v1/portal/sse/counts` SSE stream
(router.ex:28; lib/portal_live/sse_controller.ex:12).

### Python TTS (accessibility)

`POST /tts/render` (py/tts/src/eel_tts/main.py:59, `render`) and `GET /healthz`
(:84). Adapter selection `_get_adapter` :27; synthesis backends in synth.py
(`synthesise` :17/:29/:59).

## 5. Data Model

PostgreSQL migrations (Refinery-managed):

| Table | Purpose | Key columns |
|---|---|---|
| cache_control_hints (V1) | Per-snapshot TTL hints for edge caches | snapshot_id PK, merkle_root, max_age_seconds, s_maxage_seconds, stale_while_revalidate, petition_day (TTL rules T+0..7 aggressive → T+31+ near-static per ADR-0010) |
| snapshot_index (V2) | Append-only lookup for /snapshots/latest | scope, scope_code, position_code, merkle_root, cosign_sig, tarball_url, published_at |
| viewer_session_metrics (V3) | Per-minute aggregate counters for SLO dashboards | bucket_at, endpoint, http_status_class, p95_ms/p99_ms, cache_hits/misses — explicitly no PII/IP/session IDs |

Stores besides PostgreSQL: MinIO object store for bulk tarballs (redirect target in
`snapshots_bulk`). Events: consumes `eel.rts.snapshot_published.v1` (documented);
emits none.

In-memory state only in gazette/inspection crates (`AppState` placeholders, e.g.
crates/gazette/src/api.rs:17–21).

## 6. Operation

- Build/run: `cargo run -p portal-api`; Astro site `pnpm build` under astro/public;
  Elixir `mix phx.server` in ex/portal_live; TTS `uvicorn`-style launch of py/tts.
- Env vars: none read by portal-api (port hard-coded to 8090, main.rs:6). Edge rate
  limiting/WAF/HSTS enforced upstream per ADR-0010 (documented in lib.rs:10–11).
- Test: `cargo test` (incl. tests/rate_limit_test.rs burst test, tests/rate_limit_test.rs:12);
  `axe_wcag.js` accessibility suite; Astro vitest suites (astro/public/src/test/);
  Playwright in astro/explainer; k6 thresholds in loadtest/k6_portal.js.
- Deploy: services/portal-svc/deploy/service.yaml (image harbor.eel.internal/eel/portal-svc).
- Degradation: cache-first serving with stale-while-revalidate hints; bulk download is a
  redirect so object-store failure degrades that endpoint only. Petition-window TTL rules
  reduce cache aggressiveness during petitions (V1 migration comments).

## 7. Safeguards & Security

- **Auth posture**: fully public read-only API by design ("No PII is returned from any
  endpoint", lib.rs:9; ADR-0011 public API surface). No auth middleware on any route —
  correct for this surface. Admin operations (publishing snapshots) belong to rts-svc and
  are not exposed here.
- **Rate limiting**: enforced at the edge per ADR-0010; application-level budget verified
  by tests/rate_limit_test.rs ("verifies portal-api enforces per-endpoint request budgets"
  header comment; test asserts no 5xx under burst, lines 4–16). The limiter itself lives
  outside this crate.
- **PII handling**: all three tables are documented no-PII (V3 migration comment:
  "No PII. No IP addresses. No individual session identifiers."). Roll inspection lookups
  would touch personal data — the inspection crate's `roll_lookup` handler exists but its
  data store is unimplemented, so PII handling there is (not verified).
- **Integrity**: every snapshot response carries Merkle root + signature headers
  (`signing_headers`); client-side verification path via WASM verifier on
  transparency.astro; proofs verifiable against STH signature (`MerkleProofResponse`,
  lib.rs:63–75).
- **Validation**: UUID parsing on `{id}` paths returns RFC-7807-style problem details
  (`ProblemDetail::bad_request`, lib.rs:96–113).
- **Audit logging**: not applicable to public reads; viewer metrics are aggregate-only.
- **Tests verifying safeguards**: rate_limit_test.rs, axe_wcag.js (WCAG), vitest snapshot
  tests (stale-label logic, i18n RSS titles per TRACEABILITY.yaml).
- **Known deferrals**: handlers return well-typed placeholder data ("Stub returns a
  well-typed placeholder", lib.rs:148–149) — no real Valkey/PostgreSQL reads; bulk URL is
  hard-coded; gazette and inspection routers are compiled but never mounted into the
  served app (`app()` nests only the five portal routes), so those capabilities are dead
  code as deployed; Kafka consumer populating snapshot_index not present here.

## 8. Observability

- Logging/tracing: `tracing_subscriber::fmt::init()` (main.rs:6); no OTel exporter or
  TraceLayer in this crate.
- Metrics/SLO: viewer_session_metrics table feeds Grafana dashboards (FR-M11-039) and
  burn-rate alerts (FR-M11-040, per V3 migration comments); population job not in this repo
  section.
- Health: `GET /api/v1/portal/health` returns `{ok, service, version}` (lib.rs:137–143);
  Elixir app has separate health route; TTS has `/healthz`.
