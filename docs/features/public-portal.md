# Public portal (M11)

Module M11 (`public-portal`) owned by `portal-svc`. Citizen-facing public website: static signed tarballs, Phoenix LiveView live result counts, a Rust/WASM Merkle proof viewer, published Merkle roots, petition-window cache, WCAG AA, and RSS/Atom feeds.

## Contents

1. What M11 does
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

## 1. What M11 does

M11 is the primary public-facing output channel of the system. It publishes live election results (turnout, tallies) as they are declared, lets citizens verify Merkle inclusion proofs for any tally or register snapshot, provides voter registration lookups (BVV M07 integration), displays the gazette calendar, and offers a petition-window status cache. Static content is distributed as signed tarballs so anyone can independently verify the portal's integrity.

## 2. Data plane

Migrations `V1`-`V3` under `services/portal-svc/migrations/`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **TallySnapshot** | snapshot_id, scope, totals, Merkle root, HSM sig, published_at (consumed from M10) | A (read-only) |
| **RegisterSnapshot** | root_hash, seq, HSM sig, gazette_notice_id (consumed from M02/CVR) | A (read-only) |
| **CacheControlHint** | path, max_age, stale_while_revalidate, signed_at | C |
| **ViewerSessionMetric** | session_id, path, latency_ms, ts (V3) | C |
| **GazetteFeed** | gazette_id, notice_type, published_at, atom_entry | B |

Portal-svc does not own voter data; it consumes signed snapshots from M10, M02, and CVR.

## 3. Ingest sources

| Source | Notes |
|---|---|
| M10 rts-backend | `eel.rts.result_published.v1` Kafka event; TallySnapshot served at CDN edge |
| M02 bvr-backend | `eel.bvr.register_snapshot.v1`; gazette notice; Merkle root |
| M06 bvi-backend | `eel.bvi.turnout_updated.v1`; live turnout rollup |
| Gov Printer gazette | `integrations/gov-printer/`; gazette PDF + notice ID |
| Petition window API | Internal status endpoint; cached with configurable TTL |

## 4. Pipeline

Crates: `services/portal-svc/crates/portal-api/`, `gazette/`, `inspection/`. Astro static site: `services/portal-svc/astro/`.

```
ingest eel.rts.result_published.v1 (Kafka)
      -> portal-api: store TallySnapshot (V2__snapshot_index.sql)
      -> generate static HTML page per scope (Astro SSG)
      -> sign static tarball (Ed25519, portal key from P06):
             sha256sums.txt + tarball.sig included in release
      -> upload to CDN (S3-compatible); set Cache-Control (V1)
      -> LiveView channel (Phoenix LiveView):
             push TallySnapshot update to all connected sessions
             over WebSocket / SSE
      -> WASM proof viewer (verifier-rs compiled to WASM):
             citizen pastes Merkle proof JSON
             WASM verifies inclusion proof client-side (no server call)
      -> RSS/Atom feed updated on every TallySnapshot change
             eel.portal.gazette_feed_updated.v1

voter lookup integration (BVV M07):
      citizen -> portal -> /bvv/v1/lookup (M06/M07 API)
      response cached at portal layer with max-age controlled by V1

petition-window cache:
      portal queries internal petition-window status on gazette event
      result cached; displayed on petition page with expiry countdown
```

## 5. API surface

Contract: `packages/openapi-contracts/portal/public.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /portal/v1/results/{scope}` | Public (rate-limited) | Live tally snapshot for scope |
| `GET /portal/v1/results/{scope}/proof` | Public | Download Merkle inclusion proof JSON |
| `GET /portal/v1/register/snapshots` | Public | List register snapshots with Merkle roots |
| `GET /portal/v1/register/snapshots/{id}/proof` | Public | Merkle inclusion proof for register snapshot |
| `GET /portal/v1/gazette` | Public | Gazette calendar feed (RSS/Atom) |
| `GET /portal/v1/petition` | Public | Petition window status + countdown |
| `GET /portal/v1/turnout/{scope}` | Public | Near-real-time turnout rollup |
| `GET /portal/v1/health` | Internal | Health check |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Static portal (Astro) | `services/portal-svc/astro/` | Main public website; results, gazette, voter lookup |
| Phoenix LiveView live counts | `services/portal-svc/ex/` | Real-time result updates over WebSocket |
| Verifier-rs WASM | `services/portal-svc/` (WASM bundle from `verification/`) | Client-side Merkle proof verification |
| RSS/Atom feed | `/portal/v1/gazette.atom` | Machine-readable gazette and result updates |

Languages: English + Kiswahili (minimum); extensible to 11. WCAG 2.2 AA across all pages. Mobile-first responsive layout. High-contrast mode. No authentication required for public pages.

## 7. Integrity + anti-tamper

- **Signed static tarballs** -- every portal release signed with Ed25519 (portal key from P06); `sha256sums.txt` + `tarball.sig` included; any citizen or observer can verify the tarball independently.
- **Verifier-rs WASM** -- Merkle proof verification runs entirely client-side (compiled from `verification/`); server cannot forge a valid proof for a leaf not in the tree.
- **Cache poisoning defence** -- cache key includes snapshot signature version (`V1__cache_control_hints.sql`); CDN serves signed content only; stale poisoned content rejected by WASM verifier.
- **No voter PII on portal** -- only aggregate tallies and scoped turnout counts; no voter IDs, names, or biometrics.
- **HSM-signed snapshots consumed** -- portal only serves TallySnapshots and RegisterSnapshots that carry valid HSM signatures; unsigned content rejected at ingest.
- **LiveView session binding** -- CSRF token + session binding on all Phoenix LiveView channels; replay and hijack mitigations from `packages/eel-resilience-rs`.
- **RSS/Atom feed signature** -- each feed entry includes the snapshot's HSM signature digest; feed readers can verify freshness.

## 8. Kenya statute alignment

- **Elections Act s.39 (public results)** -- M11 is the official digital publication channel for declared results; static tarballs provide the non-repudiable archive.
- **Elections Act s.6A (public inspection)** -- voter lookup (BVV) and register snapshot proofs available via portal satisfy digital public inspection.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- no PII on portal; only aggregate data.
- **Gazette Act / Kenya Gazette** -- gazette feed integrates with Gov Printer; gazette notice IDs from M02 CVR linked in portal feed.
- **Elections (Technology) Regulations 2017** -- signed tarballs + Merkle proof viewer provide KEBS-certifiable public verifiability.

## 9. Honest caveats + external prerequisites

- **CDN provider** (S3-compatible endpoint) is an infrastructure prerequisite; tarball upload path in `integrations/` targets a configurable endpoint; specific provider not locked.
- **Gov Printer XML schema** for gazette notice is documented in `integrations/gov-printer/`; exact schema locked at first live batch.
- **Petition-window status** is sourced from an internal endpoint; the legal petition window dates are defined by gazette notice, not by code; the portal displays what the gazette says.
- **WASM verifier-rs** is compiled from `verification/`; a separate Cargo workspace; build must be included in portal release CI.
- **5M concurrent viewers** SLO (EC-SCL-01) requires CDN edge capacity provisioned before election day; origin handles <= 500 req/s; >= 95% edge-cached.
- LiveView real-time updates require WebSocket connectivity; fallback is static page reload (SSG pages remain valid even without WebSocket).

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (Merkle proof verifier, gazette feed builder, cache hint generator) | nextest | M11-UNIT- |
| Contract (M10 result consumer; M02 register snapshot consumer; M06 turnout consumer) | Pact | M11-PACT- |
| Integration (tarball sign + verify; WASM proof verify; LiveView channel push) | nextest + testcontainers | M11-INT- |
| Load (5M concurrent; CDN edge vs origin; SSE fan-out at 1M connections) | k6 + Gatling | M11-LOAD- |
| Accessibility (WCAG 2.2 AA; axe-core; NVDA; 150% pseudo-locale) | axe-core + manual | M11-A11Y- |
| Security (cache poisoning probe; WASM tampering; signed tarball verification) | ZAP + manual | M11-SEC- |

Runbooks (to create): `docs/runbooks/portal-svc-degradation.md` (CDN origin overload; LiveView fan-out spike; tarball sign failure).

Full spec: `docs/plans/modules/public-portal.md`. Service crates: `services/portal-svc/crates/portal-api/`, `gazette/`, `inspection/`. Astro: `services/portal-svc/astro/`. Migrations: `services/portal-svc/migrations/V1`-`V3`.
