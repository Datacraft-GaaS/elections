# ADR 0005: Cryptography & HSM

Status: accepted
Date: 2026-08-21

## Context

Election integrity requires HSM-anchored root of trust for kit provisioning, PKI, secret sealing, transparency-log signing, ceremony workflows, and result declaration. `sw/ARCHITECTURE.md:215` calls for FIPS 140-3 L3+; `sw/expect.md:37` says 140-2 L3. Discrepancy deferred to procurement.

## Decision

- **HSM abstraction**: **PKCS#11 + KMIP**. Vendor (Thales Luna 7 vs Utimaco CryptoServer) is swappable. Rust `cryptoki` crate + Python `python-pkcs11`. No vendor-specific SDK code in application services.
- **Level**: **FIPS 140-3 L3+** target; 140-2 L3 acceptable if procurement forces it. Codified as ADR-0005-a placeholder to close pre-award.
- **Key ceremony**: M-of-N (Shamir 3-of-5) across three named custodians (IEBC Chair rep, CJ rep, AG rep, plus two operational alternates). Recorded in `docs/runbooks/hsm-key-ceremony.md`. Every ceremony operation writes to the transparency log (ADR-0017).
- **Root usage**: HSM root signs (a) SPIRE trust bundle, (b) OpenBao seal, (c) Signed Tree Heads for the transparency log at 1s/60s/5min cadence, (d) result declarations under M-of-N ceremony, (e) kit provisioning ceremony credentials.
- **Producer signing on the bus**: producers use **short-lived SPIFFE-issued Ed25519 keys** (rotated hourly, HSM-anchored trust chain). HSM does not sign individual events; only tree heads. See ADR-0015.
- **Transport crypto**: **TLS 1.3 only** with **PQ hybrid ML-KEM + X25519** (via `oqs` provider for OpenSSL 3 / rustls with PQ providers). No TLS 1.2 anywhere.
- **At-rest crypto**: **AES-256-GCM** for volume-level (LUKS + HSM-sealed keys); PostgreSQL TDE via `cryptsetup` + `pgcrypto` for column-level where a data-class-A field is stored (per ADR-0016).
- **Artifact signing**: **cosign** with keyless via SPIFFE workload identity for CI artifacts; keyed for release artifacts using HSM-anchored key.
- **Verification libraries**: single Rust crate `packages/eel-crypto-rs` wraps PKCS#11, Ed25519 verify, ML-KEM+X25519, cosign verify; wrapped for Python and Elixir via PyO3/Rustler.

## Consequences

Positive: vendor-agnostic HSM path; PQ hybrid ready for the long tail of the 15-year retention window; per-workload Ed25519 keys make event-signing throughput unbounded; single shared crypto crate eliminates per-service reinvention.

Negative: PKCS#11 abstraction adds indirection versus native vendor SDK. PQ hybrid support in libraries is younger than classical TLS; pin OpenSSL/rustls versions carefully. Ceremony logistics for M-of-N key custody are real operational overhead.

## Alternatives rejected

- Native Thales / Utimaco SDKs in application code (vendor lock-in).
- Classical TLS 1.3 only (no PQ hybrid), inadequate for a 15-year audit window given known harvest-now-decrypt-later posture of state adversaries.
- Software-only crypto (no HSM), non-starter for election-critical roots.

## Related

ADR-0004 (SPIRE + cert-manager rooted here), ADR-0007 (OpenBao sealed here), ADR-0015 (producer signing), ADR-0017 (STH signing).
