# repro-bundle

Reproducibility bundle generator for M14, producing bit-identical tally bundles from primary election evidence.

Assembles Form 34A/B/C images, biometric templates, voter register snapshots, and transparency log proofs into a cryptographically sealed reproducibility bundle. Enables independent auditors to verify election tallies without direct access to protected systems. Generates determistic binary outputs (merkle roots, signed bundle hashes) for bit-level reproduction verification.

Fits into transparency-svc as the reproducibility assurance module: RTS publishes forms, transparency log holds them, repro-bundle seals and verifies them.

## Public API

Exposed modules via lib.rs:
- bundle_builder: BundleBuilder, build(election_id, forms, register_snapshot, tlog_proof) -> Bundle
- evidence_pack: EvidencePack, add_form34(form), add_biometric(biometric), finalize() -> PackHash
- verification: BundleVerifier, verify(bundle, expected_hash) -> VerificationResult
- manifest: Manifest, serialize_json() -> JsonManifest

## Feature flags

None currently. Supports reproducibility-friendly binary serialization (postcard).

## Testing

Unit tests (hash computation, serialization):
```bash
cargo test --lib
```

Evidence pack tests:
```bash
cargo test evidence_pack::
```

Bundle round-trip (serialize, verify):
```bash
cargo test verification::tests
```

Integration tests (mock election data):
```bash
cargo nextest run --features test-fixtures
```

## Benchmarks

Bundle generation latency (if benches/ exists):
```bash
cargo bench bundle_build_10k_forms
```

Tracks packing time for 10k Form 34 entries.

## Dependencies of note

Workspace:
- eel-crypto-rs: Hash functions (SHA-256), signature verification
- eel-events-rs: Form event deserialization

External:
- sha2: SHA-256 hashing
- serde, postcard: Deterministic serialization
- ed25519-dalek: Signature verification
- tokio-util: Streaming large bundles

## Related docs

See services/transparency-svc/docs/bundle-schema.md for reproducibility bundle format spec.
See services/transparency-svc/docs/evidence-collection.md for evidence gathering procedures.
