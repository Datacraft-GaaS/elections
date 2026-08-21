# eel-api-server-rs

Opinionated axum server builder that wires auth, SpiceDB, rate limiting, idempotency, RFC 9457 error bodies, response signing, and load shedding into every eel Rust service.

## Purpose in context

Each of the 18 eel services needs the same cross-cutting middleware stack. This crate provides `ApiServerBuilder`, which assembles all shared Tower layers from the other `eel-*-rs` packages into a single composable builder. Services add their own routes and call `.serve()`. The resulting server conforms to ADR-0011 (API conventions) out of the box.

## Public modules

| Module | Description |
|---|---|
| `builder` | `ApiServerBuilder` - fluent server builder |
| `server` | `ApiServer` - started server handle |
| `middleware` | Individual middleware layers (auth, idempotency, signing, rate-limit) |
| `problem` | `Problem`, `ProblemType` - RFC 9457 problem detail types |
| `response` | Response helpers (JSON+CBOR content negotiation) |
| `routes` | Built-in routes: `/healthz`, `/readyz`, `/metrics` |

## Key types

- `ApiServerBuilder` - configure auth, SpiceDB, rate limits, load shedding, TLS
- `ApiServer` - running server; call `.shutdown()` for graceful drain
- `Problem` - RFC 9457 `application/problem+json` response body
- `ProblemType` - typed problem URIs per ADR-0011
- `ApiServerError` - builder and startup errors

## Layout

```
eel-api-server-rs/
  src/
    lib.rs
    builder.rs
    server.rs
    middleware/
    problem.rs
    response.rs
    routes.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_api_server_rs::ApiServerBuilder;

#[tokio::main]
async fn main() {
    let server = ApiServerBuilder::new("rts-backend")
        .jwks_url("https://auth.example.local/realms/eel/protocol/openid-connect/certs")
        .spicedb_endpoint("http://spicedb:50051")
        .load_shedding(Default::default())
        .routes(rts_routes())
        .build()
        .await
        .unwrap();

    server.serve("0.0.0.0:8080").await.unwrap();
}
```

## Config / env

Inherits all env vars from `eel-auth-rs`, `eel-secrets-rs`, `eel-loadshedding-rs`, and `eel-observability-rs`. Additional:

| Variable | Description |
|---|---|
| `EEL_SIGNING_KEY_ID` | HSM key ID for response signing |
| `EEL_RATE_LIMIT_RPS` | Global rate limit requests/second |

## Dependencies

All `eel-*-rs` crates in this workspace, plus `axum`, `tower`, `tower-http`, `tonic`, `tokio`, `serde`, `serde_json`, `serde_cbor`.

## Build

```bash
cargo build -p eel-api-server-rs
```

## Test

```bash
cargo nextest run -p eel-api-server-rs
```

## Related docs

- `docs/adr/0011-api-conventions.md` - tier rules, RFC 9457, signing headers, Idempotency-Key
- `packages/service-template-rs/README.md` - cargo-generate template that uses this crate
- `packages/openapi-contracts/README.md` - specs that drive client generation

## Limitations

gRPC (tonic) and REST (axum) share the same port using HTTP/2 content-type routing. Some clients (proxies, load balancers) may not support this multiplexing. The Kubernetes ingress in `deploy/` is pre-configured for this topology.
