# eel-kit-cert-transparency

CT-log-style transparency for kit certificate issuance in the eel election integrity platform.

## Overview

Every field kit in the eel deployment receives a SPIFFE identity certificate from a SPIRE + step-ca
pipeline. `eel-kit-cert-transparency` ensures that no certificate exists outside the transparency
log: the cert is appended to the log before it is delivered to the requester, and the kit verifies
its own cert's inclusion at boot time.

The Merkle structure follows RFC 6962 "Certificate Transparency" (Laurie, Langley, Kasper 2013),
using the SHA-256 leaf-hash and node-hash domain separation defined there. The Merkle primitives
are provided by `eel-crypto-rs` (`packages/eel-crypto-rs/src/merkle.rs`).

The architecture is modelled after Google Trillian but kept intentionally simple: the log is an
append-only in-memory structure with persistence delegated to the caller (e.g. a PostgreSQL-backed
service). Consistency proofs between Signed Tree Heads (STHs) let auditors verify that older STHs
are prefixes of newer ones.

## References

- RFC 6962: Ben Laurie, Adam Langley, Emilia Kasper, "Certificate Transparency" (2013).
  https://www.rfc-editor.org/rfc/rfc6962
- Google Trillian: https://github.com/google/trillian
- SPIRE: https://spiffe.io/docs/latest/spire-about/

## Technical integration

The log is linked into the biometric edge service (`services/biometric-edge-svc`). The issuance
hook (`src/issuance_hook.rs`) is called synchronously inside the step-ca issuance handler.

Auditors can verify consistency between any two published STHs using the consistency proof endpoint
(`GET /v1/kct/sth` + `GET /v1/kct/inclusion-proof/{cert_hash}`).

The IAG (`docs/governance/independent-assurance-group.md`) appends its attestations to
`services/transparency-svc/crates/tlog-witness`; the kit-cert log entries are cross-referenced
there.

## Modules

| Module | Purpose |
|--------|---------|
| `entry` | `KitCertEntry` leaf type |
| `log` | `KitCertTransparencyLog`, STH, inclusion + consistency proofs |
| `issuance_hook` | Intercept and log every cert before delivery |
| `verifier` | Kit boot-time and auditor inclusion verification |
| `api` | Axum HTTP API (`/v1/kct/*`) |

## Running tests

```bash
cargo test -p eel-kit-cert-transparency
```
