# hsm-signer

PKCS#11-backed Ed25519 + ML-DSA hybrid signing service for transparency log Signed Tree Heads.

Provides cryptographic signing operations for the transparency log, coordinating with HSM (Hardware Security Module) via PKCS#11 interface. Produces dual-signature Signed Tree Heads (Ed25519 for immediate verification, ML-DSA for post-quantum resilience). Handles key rotation, audit logging, and fallback to graceful degradation on HSM unavailability.

Fits into transparency-svc as the cryptographic core: the main log appends leaves, hsm-signer produces the authenticated STH, and the five-channel publisher distributes it.

## Public API

Exposed modules via lib.rs:
- signing: SigningService trait, sign_sth(tree_size, leaf_hash, timestamp) -> (Ed25519Sig, MLDSASig)
- key_management: KeyRotation, KeyCeremony, break_glass_recovery
- audit: AuditLog, log_sign_operation(key_id, tree_size, timestamp, status)
- transport: PKCS#11Session, session_pool for thread-safe access

## Feature flags

None currently defined. Signing algorithms (Ed25519, ML-DSA) always compiled.

## Testing

Unit tests:
```bash
cargo test --lib
```

Signing correctness (verifies Ed25519 + ML-DSA signatures):
```bash
cargo test signing::tests
```

Key rotation tests:
```bash
cargo test key_management::rotation
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires live HSM or PKCS#11 simulator):
```bash
cargo test --test '*hsm*' -- --ignored  # Use SOFTHSM for CI
```

## Benchmarks

Signing latency (if benches/ exists):
```bash
cargo bench --bench signing_latency
```

Track Ed25519 and ML-DSA signing time across key rotations.

## Dependencies of note

Workspace:
- eel-auth-rs: HSM session credential management
- eel-observability-rs: Tracing for signing operations
- eel-crypto-rs: Cryptographic utilities

External:
- pkcs11 (crates.io): PKCS#11 FFI bindings
- ed25519-dalek: Ed25519 signing/verification
- ml-dsa (pending): ML-DSA post-quantum signing
- tokio: Async runtime
- sqlx: Audit log persistence to PostgreSQL

## Related docs

See crates/transparency-svc/docs/key-ceremony.md for HSM key ceremony procedures.
See crates/transparency-svc/docs/signing-slo.md for latency SLOs and monitoring.
See docs/runbooks/hsm-key-rotation.md for operational runbook.
