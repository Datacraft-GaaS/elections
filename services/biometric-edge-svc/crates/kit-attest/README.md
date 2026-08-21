# kit-attest

Kit attestation validation. Parses Android Integrity API verdicts and iOS DeviceCheck tokens, verifies signatures against platform keys, and coordinates with SPIRE to issue short-lived SVIDs for kit identity.

## What it does

- Attestation parsing: Decodes Android Integrity verdictToken (JWT) and iOS DeviceCheck token (opaque binary); extracts device claims (model, API level, hardware hash)
- Signature verification: Validates verdictToken signature against Google's public keys (cached, refreshed daily); validates DeviceCheck token signature via Apple's attestation service
- Risk assessment: Extracts device risk signals (bootloader unlock, compromised OS, unsigned APK); categorizes kit as Trusted, Conditional, or Untrusted
- SPIRE integration: Calls SPIRE agent to issue x509-SVID with kit serial as Subject Alternative Name; SVID valid for 1 hour (renewed on sync)
- Attestation cache: Caches verdict in `eel_kit_attestations` table (serial, timestamp, status, claims, risk_level); enables offline verification
- Nonce validation: Prevents replay attacks; compares nonce in verdict against stored nonce per kit per attestation window

## Public API surface

```rust
pub struct AttestationToken {
    pub platform: Platform,
    pub token: Vec<u8>,
    pub nonce: String,
}

pub enum Platform { Android, iOS }

pub struct AttestationVerdict {
    pub kit_serial: String,
    pub status: AttestationStatus,
    pub device_model: String,
    pub api_level: u32,
    pub hardware_hash: String,
    pub risk_level: RiskLevel,
}

pub enum AttestationStatus { Trusted, Conditional, Untrusted }
pub enum RiskLevel { Low, Medium, High }

pub async fn verify_attestation(token: AttestationToken) -> Result<AttestationVerdict>;
pub async fn issue_spire_svid(kit_serial: &str) -> Result<String>;
pub async fn refresh_spire_svid(kit_serial: &str) -> Result<String>;
pub async fn get_cached_verdict(kit_serial: &str) -> Result<Option<AttestationVerdict>>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (JWT parsing, signature verification, risk assessment)
cargo test --lib --package eel-kit-attest

# Integration tests (SPIRE agent integration, attestation cache)
cargo test --test '*' --package eel-kit-attest -- --test-threads=1

# Property tests (nonce uniqueness, replay prevention)
cargo test --package eel-kit-attest -- --nocapture
```

## Bench commands

```bash
# Android Integrity verification (100 verdictTokens)
cargo bench --package eel-kit-attest -- verify_android

# iOS DeviceCheck verification (100 tokens)
cargo bench --package eel-kit-attest -- verify_ios

# SPIRE SVID issuance latency
cargo bench --package eel-kit-attest -- issue_svid

# Cached verdict lookup (1M entries)
cargo bench --package eel-kit-attest -- cache_lookup
```

## Dependencies

- `jsonwebtoken` - JWT parsing and signature verification for Android Integrity
- `reqwest` - HTTP client for Google Play API (public key download), Apple attestation service
- `sqlx` - Attestation cache reads/writes
- `sha2` - Hardware hash verification
- `tonic` - gRPC client for SPIRE agent
- `eel-crypto-rs` - HSM utilities for key management

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Kit attestation flow, SPIRE SVID issuance, nonce management
- [THREAT-MODEL.md](../../docs/THREAT-MODEL.md) - Replay attack prevention, device tamper detection, platform security review
