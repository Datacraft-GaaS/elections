# portal-svc

Module M11 public results portal (WP-3.9). Three surfaces:

| Surface | Path | Purpose |
|---|---|---|
| Astro static site | astro/public/ | Pre-rendered signed results from WP-3.1 tarballs |
| Phoenix LiveView | ex/portal_live/ | Real-time counts via Kafka projections |
| Rust axum API | crates/portal-api/ | Snapshot, proof, and bulk-download endpoints |

## Quick start

```sh
# Rust API
cargo build --release -p portal-api
cargo test -p portal-api

# Elixir LiveView
cd ex/portal_live && mix deps.get && mix compile && mix test

# Astro static site
cd astro/public && pnpm install && pnpm build && pnpm test

# Load test (requires k6)
k6 run --vus 50 --duration 5m loadtest/k6_portal.js
```

## Design decisions

- ADR-0008: Astro for static-first surfaces, Phoenix LiveView for real-time.
- ADR-0010: WAF, rate limiting, and HSTS enforced at edge; portal-api is inside the mesh.
- ADR-0011: Five API tiers; snapshots are Tier 1 (signed-static).
- ADR-0017: All inclusion and consistency proofs sourced from the transparency log.

## Boundaries

WP-5.7 owns: crates/gazette, crates/inspection, astro/explainer, py/tts.
Do not modify those paths.

## SLAs

See docs/SLA.md.
