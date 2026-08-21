# abis-attest

Confidential compute attestation verification. Validates Intel TDX or AMD SEV-SNP attestation quotes, manages nonce registry to prevent replay attacks, and coordinates with HSM signer for countersignatures.

## What it does

- Quote verification: Parses TDX or SEV-SNP attestation quote headers; validates signature chain against golden TCB info stored in OpenBao
- Nonce registry: One-time nonce stored in `eel_attestation_nonces` per quote; prevents replay attacks within attestation validity window (typically 24h)
- SPIRE integration: Requests x509-SVID for pod identity; verifies pod's mTLS certificate against Kubernetes SPIRE agent
- HSM signer coordination: Calls HSM signer service (port 8444) to countersign attestation hash; stores signature in `eel_enclave_attestations`
- Pod startup: Attestation quote obtained at container entrypoint; blocks pod readiness probe until quote cached and nonce stored

## Public API surface

```rust
pub struct AttestationQuote {
    pub protocol: AttestationProtocol,
    pub quote: Vec<u8>,
    pub nonce: String,
}

pub enum AttestationProtocol { TDX, SEVSNS, Software }

pub struct AttestationClaim {
    pub enclave_signer: String,
    pub enclave_hash: String,
    pub tcb_version: String,
}

pub async fn verify_quote(quote: AttestationQuote) -> Result<AttestationClaim>;
pub async fn register_nonce(nonce: &str) -> Result<()>;
pub async fn get_cached_quote() -> Result<AttestationQuote>;
pub async fn refresh_quote_if_stale() -> Result<()>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (quote parsing, nonce validation, TCB version checking)
cargo test --lib --package eel-abis-attest

# Integration tests (SPIRE mTLS, HSM signer mock, PG nonce registry)
cargo test --test '*' --package eel-abis-attest -- --test-threads=1
```

## Bench commands

```bash
# Quote verification latency (TDX and SEV-SNP)
cargo bench --package eel-abis-attest -- verify_quote

# Nonce lookup (1M stored nonces)
cargo bench --package eel-abis-attest -- nonce_lookup
```

## Dependencies

- `reqwest` - HTTP client for TDX/SEV-SNP quote fetch and HSM signer calls
- `sqlx` - PostgreSQL nonce registry and quote cache writes
- `sha2` - Hash quote for immutability check
- `ed25519-dalek` - Verify SPIRE SVID signature chain
- `eel-crypto-rs` - HSM integration utilities

## Related docs

- [THREAT-MODEL.md](../../docs/THREAT-MODEL.md) - Attestation attack vectors, replay prevention, supply chain risks
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Attestation quote flow diagram, pod startup sequence
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Manual quote refresh, nonce registry cleanup, HSM certificate rotation
