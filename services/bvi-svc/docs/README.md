# bvi-svc -- Biometric Voter Identification Service

WP-3.7 | Modules M06 (bvi-backend) + M07 (bvv)

## Purpose

Central authority for polling-day operations:

- Per-station register-extract config bundles (bvi-config crate)
- High-volume biometric identification event ingest (bvi-turnout crate, Kafka topic `eel.bvi.identification.v1`, 64 partitions)
- Live turnout rollups and SSE feed
- Kit certificate issue and revocation (<= 60 s propagation)
- Honey-mode routing for revoked kits (EC-SEC-14)
- BVV public voter-register lookup with constant-time response (EC-SEC-10)
- Objections workflow

## Crates

| crate | binary | role |
|---|---|---|
| bvi-api | bvi-api | HTTP REST + gRPC server; all OpenAPI paths bound |
| bvi-config | bvi-config | Signed per-station config bundle producer |
| bvi-turnout | bvi-turnout | Kafka consumer, rollup worker, honey-mode router |

## Elixir surface

`ex/bvv_web` -- Phoenix LiveView for BVV public lookup and objections.

## Astro surface

`astro/bvv` -- Static shell wrapping the LiveView app.

## Running locally

```
cargo build --release --workspace
cargo test --workspace
mix deps.get && mix compile      # from ex/bvv_web/
pnpm install && pnpm build       # from astro/bvv/
```

## SLA

See docs/SLA.md.
