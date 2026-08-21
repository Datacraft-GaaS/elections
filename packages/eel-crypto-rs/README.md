# eel-crypto-rs

Cryptographic primitives for the eel platform: Ed25519, BLAKE3, RFC 8785, Merkle trees, PKCS#11 HSM pool, post-quantum hybrids, and TSA client.

## Purpose in context

Implements ADR-0005. Every signing, hashing, and key-management operation in the platform flows through this crate. Pure-Rust modules are always available; HSM and PQ features gate on optional feature flags.

## Public modules

| Module | Description |
|---|---|
| `ed25519` | Ed25519 sign / verify (pure Rust, `ed25519-dalek`) |
| `blake3` | BLAKE3 hashing |
| `rfc8785` | RFC 8785 JSON Canonicalization Scheme |
| `merkle` | RFC 6962 leaf/node hash, root computation |
| `constant_time` | `ct_eq_fixed` byte comparison |
| `pkcs11_pool` | PKCS#11 session pool (`pkcs11` feature) |
| `pq` | ML-DSA-65 hybrid, ML-KEM-768, SLH-DSA (`pq-hybrid` / `pq-kem` / `pq-slh` features) |
| `tsa` | RFC 3161 TSA client (`tsa` feature) |
| `cosign` | cosign bundle verification (`cosign` feature) |

## Key types

- `KeyRole` - enum naming every HSM-managed key purpose
- `CryptoError` - unified error type

## Layout

```
eel-crypto-rs/
  src/
    lib.rs          # public re-exports
    ed25519.rs
    blake3.rs
    rfc8785.rs
    merkle.rs
    constant_time.rs
    pkcs11_pool.rs  # feature-gated
    pq/             # feature-gated
    tsa.rs          # feature-gated
    cosign.rs       # feature-gated
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_crypto_rs::ed25519::{generate_keypair, sign, verify};
use eel_crypto_rs::blake3::hash;
use eel_crypto_rs::rfc8785::canonicalize;

let (sk, vk) = generate_keypair();
let payload = serde_json::json!({"type": "eel.rts.form34a_captured.v1"});
let canonical = canonicalize(&payload).unwrap();
let sig = sign(&sk, &canonical);
assert!(verify(&vk, &canonical, &sig).is_ok());

let digest = hash(b"some data");
```

## Features

| Feature | What it adds |
|---|---|
| `pkcs11` | PKCS#11 session pool (requires liboqs at link time) |
| `pq-hybrid` | ML-DSA-65 + ECDSA P-384 hybrid |
| `pq-kem` | ML-KEM-768 + X25519 key encapsulation |
| `pq-slh` | SLH-DSA archive signing |
| `tsa` | RFC 3161 TSA client (async, rustls) |
| `cosign` | cosign bundle verification |

## Dependencies

Pure-Rust dependencies only in the default build: `blake3`, `sha2`, `digest`, `subtle`, `hex`, `base64`, `rand_core`, `serde`, `serde_json`, `thiserror`, `tracing`.

## Build

```bash
cargo build -p eel-crypto-rs
# With post-quantum features:
cargo build -p eel-crypto-rs --features pq-hybrid,pq-kem
```

## Test

```bash
cargo nextest run -p eel-crypto-rs
cargo nextest run -p eel-crypto-rs --features pq-hybrid,tsa
```

## Related docs

- `docs/adr/0005-cryptography.md` - key roles, algorithm choices, rotation policy
- `packages/verifier-rs/README.md` - Merkle verifier built on top of this crate
- `packages/eel-events-rs/README.md` - signing of CloudEvent envelopes

## Limitations

PKCS#11 and PQ features require native dependencies not available in the default workspace build. CI builds the default feature set; hardware-gated features build in the HSM CI job only.
