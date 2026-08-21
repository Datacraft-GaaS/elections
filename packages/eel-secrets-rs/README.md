# eel-secrets-rs

OpenBao (Vault-compatible) secrets client for the eel platform: dynamic Postgres credentials, transit encryption, JIT admin creds, and PKI leaf cert issuance (ADR-0007).

## Purpose in context

No service stores long-lived database passwords or encryption keys. This crate wraps the OpenBao HTTP API to issue short-lived dynamic Postgres credentials on startup, perform envelope encryption via the transit engine, obtain 4-hour admin credentials for break-glass procedures, and issue mTLS leaf certificates for service-to-service calls.

## Public modules

| Module | Description |
|---|---|
| `dynamic_creds` | `PgDynamicCred`, `PgCred` - leased Postgres credentials |
| `transit` | `Transit` - encrypt / decrypt via OpenBao transit engine |
| `admin_creds` | `AdminCred`, `issue_admin_cred()` - break-glass JIT credentials |
| `pki` | `LeafCert`, `issue_leaf_cert()` - mTLS leaf cert issuance |

## Key types

- `PgDynamicCred` - renews credentials before lease expiry; provides a `sqlx::PgConnectOptions`
- `Transit` - `encrypt(&plaintext) -> ciphertext`, `decrypt(&ciphertext) -> plaintext`
- `AdminCred` - credential with 4-hour TTL and renewal capability
- `LeafCert` - PEM bundle for a service identity
- `SecretsError` - unified error type

## Layout

```
eel-secrets-rs/
  src/
    lib.rs
    dynamic_creds.rs
    transit.rs
    admin_creds.rs
    pki.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_secrets_rs::{PgDynamicCred, Transit};

// Dynamic Postgres credentials (renewed automatically before expiry)
let cred = PgDynamicCred::new(
    "http://openbao:8200",
    "s.service-token",
    "database/creds/rts-backend",
).await?;
let pool = sqlx::PgPool::connect_with(cred.connect_options()).await?;

// Transit encrypt/decrypt
let transit = Transit::new("http://openbao:8200", "s.service-token", "eel-data");
let cipher = transit.encrypt(b"voter-id-plaintext").await?;
let plain  = transit.decrypt(&cipher).await?;
```

## Config / env

| Variable | Description |
|---|---|
| `OPENBAO_ADDR` | OpenBao server address |
| `OPENBAO_TOKEN` | Service token (bootstrapped by the pod's init container) |
| `OPENBAO_TRANSIT_KEY` | Transit key name for this service |

## Dependencies

`tokio`, `serde`, `serde_json`, `thiserror`, `tracing`, `chrono`, `uuid`, `async-trait`, `base64`, `reqwest`, `sqlx`, `tokio-util`.

## Build

```bash
cargo build -p eel-secrets-rs
```

## Test

```bash
cargo nextest run -p eel-secrets-rs
# Integration tests require a dev OpenBao instance:
just dev-up && cargo nextest run -p eel-secrets-rs --features integration
```

## Related docs

- `docs/adr/0007-secrets.md` - OpenBao role design, lease policies, rotation windows
- `platform/openbao/` - policy HCL files and role definitions
- `packages/eel-auth-rs/README.md` - SPICEDB_PRESHARED_KEY retrieved via this crate at startup

## Limitations

Credential renewal is asynchronous. If the renewal background task fails (network partition), requests will fail once the lease expires. Services must implement graceful degradation via `eel-resilience-rs` for this scenario.
