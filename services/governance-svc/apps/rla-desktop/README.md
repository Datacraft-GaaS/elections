# rla-desktop

Offline-capable Risk-Limiting Audit desktop tool for post-election ballot auditing.

## Module

M21 (rla-audit-tool) | governance-svc | WP-4.6

## Purpose

Tauri 2 desktop application used by auditors to conduct risk-limiting audits (RLA) after
election results are published. Loads a ballot manifest from USB or local file, fetches
entropy from the NIST Randomness Beacon (or uses a locally cached value when offline),
runs the SHANGRLA algorithm to compute the audit sample, and guides auditors through
physical ballot comparison. Signs the completed audit run record with an HSM-backed key
and exports the result to the governance-svc API and/or a signed PDF for Supreme Court
submission.

## Contents

```
rla-desktop/
  src-tauri/          Rust backend (Tauri commands, HSM signing, gRPC to py/shangrla)
  src/                Frontend (TypeScript + React, audit UI)
  assets/             Static assets
  test/               Frontend unit tests
```

## Target Platform

Windows (primary, auditors use Windows laptops), macOS and Linux supported.
Compiled installers: `.msi` (Windows), `.dmg` (macOS), `.deb` (Linux).

## Offline Capabilities

Fully offline. The ballot manifest is loaded from a local file or USB drive. The NIST
beacon value is cached locally for audit reproducibility. Network is only needed to
submit the signed audit record to governance-svc on completion.

## Attestation Profile

- Windows: EV code signing certificate, signed MSI via internal CA
- macOS: Apple notarization (Developer ID)
- Linux: `.deb` with GPG-signed Release file

## Build Commands

```bash
cd services/governance-svc/apps/rla-desktop

# Development (hot reload)
cargo tauri dev

# Production build (current platform)
cargo tauri build

# Cross-platform (requires target toolchains installed)
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target x86_64-unknown-linux-gnu
```

## Configuration / Env Variables

| Variable | Description | Default |
|---|---|---|
| `NIST_BEACON_URL` | NIST Randomness Beacon v2 endpoint | `https://beacon.nist.gov/beacon/2.0/pulse/last` |
| `HSM_SLOT_LABEL` | PKCS#11 slot label for audit signing key | `rla-audit` |
| `HSM_PIN` | PKCS#11 slot PIN | (prompt) |
| `SHANGRLA_GRPC_URL` | py/shangrla gRPC endpoint | `http://localhost:50061` |
| `GOVERNANCE_API_URL` | governance-svc API for audit record submission | required |
| `SHANGRLA_ALGO_VERSION` | Algorithm version string (audit reproducibility) | `shangrla-2.1` |

## Test Commands

```bash
# Rust unit tests
cargo test

# Frontend unit tests
npm run test

# End-to-end (requires connected SoftHSMv2)
SOFTHSM2_CONF=platform/softhsm/softhsm2.conf cargo tauri build --debug
```

## Deployment Notes

Distributed as a signed installer package via internal secure download portal.
Not a Kubernetes pod - runs on auditor laptops only.
Auditors must have the HSM key provisioned before the audit window opens.

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M21)
- governance-svc/py/shangrla (SHANGRLA algorithm)
- integrations/nist-beacon (entropy source)
- integrations/hsm-pkcs11 (signing key)
- docs/runbooks/rla-audit-procedure.md

## Upstream / Downstream

- Upstream: governance-svc/py/shangrla (gRPC), integrations/nist-beacon
- Downstream: governance-svc API (audit record), Supreme Court PDF submission
