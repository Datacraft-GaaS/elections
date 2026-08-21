# eel-auth-rs

JWT verification against Keycloak JWKS, SpiceDB ReBAC authorization, and audit event emission for eel Rust services.

## Purpose in context

Every API request entering an eel service must be authenticated and authorized. This crate provides: a JWKS verifier with kid-miss refetch and in-memory cache; a SpiceDB gRPC client with fail-closed semantics; a `Subject` extraction type carrying claims and purpose scopes; and an `AuditSink` trait that downstream code uses to record access events.

## Public modules

| Module | Description |
|---|---|
| `jwks` | `JwksVerifier` - JWT verify with cached JWKS (`keycloak` feature) |
| `subject` | `Subject` - authenticated caller with roles, scopes, service account flag |
| `spicedb` | `SpiceDbClient`, `Decision`, `Consistency` (`spicedb` feature) |
| `audit` | `AuditEvent`, `AuditSink`, `NoopSink`, `LogSink` |

## Key types

- `JwksVerifier` - verifies RS256/ES256 JWT; refetches JWKS on kid miss with back-off
- `Subject` - `{ sub, roles, scopes, purpose, is_service }` extracted from validated claims
- `SpiceDbClient` - `check()` returns `Decision::Allow | Deny`; fail-closed: network error => Deny
- `AuditSink` - trait; production impl emits to the outbox; `LogSink` writes to tracing
- `AuthError` - unified error type

## Layout

```
eel-auth-rs/
  src/
    lib.rs
    jwks.rs        # keycloak feature
    subject.rs
    spicedb.rs     # spicedb feature
    audit.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_auth_rs::{JwksVerifier, SpiceDbClient, Consistency, LogSink};

let verifier = JwksVerifier::new("https://auth.example.local/realms/eel/protocol/openid-connect/certs").await?;
let subject = verifier.verify(&jwt_token)?;

let spicedb = SpiceDbClient::new("http://spicedb:50051", preshared_key).await?;
let decision = spicedb.check(
    &subject,
    "ballot_serial/2027-general",
    "read",
    Consistency::MinimizeLatency,
).await?;

match decision {
    eel_auth_rs::Decision::Allow => { /* proceed */ }
    eel_auth_rs::Decision::Deny => return Err(AuthError::Forbidden),
}
```

## Config / env

| Variable | Description |
|---|---|
| `KEYCLOAK_JWKS_URL` | Full JWKS endpoint URL |
| `SPICEDB_ENDPOINT` | gRPC endpoint, e.g. `http://spicedb:50051` |
| `SPICEDB_PRESHARED_KEY` | SpiceDB preshared key (from OpenBao) |

## Dependencies

`jsonwebtoken`, `reqwest`, `moka`, `prost`, `tonic`, `tokio`, `serde`, `thiserror`, `tracing`.

## Features

| Feature | Default |
|---|---|
| `keycloak` | yes |
| `spicedb` | yes |
| `test-utils` | no (provides mock verifier) |

## Build

```bash
cargo build -p eel-auth-rs
```

## Test

```bash
cargo nextest run -p eel-auth-rs
cargo nextest run -p eel-auth-rs --features test-utils
```

## Related docs

- `docs/adr/0004-iam.md` - Keycloak realm design, scope taxonomy
- `packages/authz-schema/README.md` - SpiceDB `.zed` schemas per bounded context
- `packages/eel-api-server-rs/README.md` - axum middleware that wires this crate into the request pipeline

## Limitations

The JWKS cache has a configurable TTL (default 5 min). After key rotation the first request with the new key ID triggers a synchronous refetch; subsequent requests use the refreshed cache. SpiceDB availability is critical: any gRPC error causes Deny, so SpiceDB must be in the resilience budget.
