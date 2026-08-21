# eel-governance-court-interface

M40 Election Court IT Interface: immutable evidence bundle creation, eCourt API integration, bundle signing via HSM, and petitioner access control.

## What it does

Assembles election evidence bundles for court petitions: results sheets (Form 34A), observer reports, transmission logs, RLA audit trails, forensics flagged anomalies. Once created and signed, bundles are immutable and pinned to IPFS via transparency-svc. Petitioners download via court-issued access tokens; eCourt API notified on bundle readiness.

Fits in governance-svc as the evidence layer. Runs on port 8092; integrates with eCourt SOAP API (mTLS via PKCS#11 client cert). Circuit breaker pattern: on eCourt timeout, bundles queue locally and replay on recovery.

## Public API

`eel_governance_court_interface::bundle::EvidenceBundle` aggregate: `fn add_item(&mut self, item: BundleItem) -> Result<()>`, `fn seal(&mut self, signer: &HsmSigner) -> Result<Signature>`, `fn verify_petitioner_access(&self, token: JwtToken) -> Result<AccessLevel>`.

Endpoints:
- `POST /v1/governance/court/petitions` - file petition
- `GET /v1/governance/court/petitions/{id}` - fetch metadata
- `POST /v1/governance/court/petitions/{id}/evidence-bundles` - attach bundle
- `GET /v1/governance/court/petitions/{id}/evidence-bundles/{bundle_id}` - download (multipart/form-data, resumable)

Multipart downloads chunk at 1 MB; client resumption via `Range` header support. Signatures stored as detached files (`.asc` files alongside bundle).

## Feature flags

`#[cfg(feature = "hsm-integration")]` for live HSM signer; uses in-memory Ed25519 stub for tests.

## Test commands

```bash
cargo test -p eel-governance-court-interface --all-features
cargo test -p eel-governance-court-interface --lib bundle_sealing
cargo test -p eel-governance-court-interface --test integration_court -- --include-ignored
```

Mock eCourt API via pytest-httpserver stub; verify circuit breaker state transitions.

## Bench commands

```bash
cargo bench -p eel-governance-court-interface bundle_sign_15_items
```

Bundle signing (HSM call overhead) should complete in < 500ms per bundle.

## Dependencies of note

- `tokio`: async eCourt SOAP client, circuit breaker
- `prost`: protobuf evidence item serialization
- `serde_json`: witness testimony JSON parsing
- `uuid6`: bundle IDs
- `tower-http`: multipart form handling

## Related docs

- `docs/api-reference.md#court` (eCourt endpoint contract)
- `docs/architecture.md#evidence-flow` (bundle lifecycle diagram)
- `migration_002_evidence_bundles.sql` (schema: petitions, bundles, items, signatures)
