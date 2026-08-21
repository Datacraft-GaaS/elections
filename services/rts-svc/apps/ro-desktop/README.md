# ro-desktop

Returning Officer desktop app for Form 34A/B/C scanning, tally entry, and HSM-signed result declaration.

## Module

M09 (rts-ro) | rts-svc | WP-3.2

## Purpose

Tauri 2 desktop application used by Returning Officers (ROs) to scan Form 34A, B, and C
ballot result sheets. ICR (Intelligent Character Recognition) assists with handwritten
tally extraction. The RO validates extracted tallies, resolves flagged discrepancies,
signs the declaration with an HSM-backed key, and transmits to rts-svc. Sessions are
screen-recorded for non-repudiation. Must function reliably in contested election
scenarios where accuracy is legally binding.

## Contents

```
ro-desktop/
  src-tauri/        Rust backend (Tauri commands, HSM signing, ICR calls, session recording)
  src/              Frontend (TypeScript + React, form scanning UI)
  assets/           Form templates, static assets
  test/             Frontend unit tests
```

## Target Platform

Windows (primary - ROs use Commission-issued Windows laptops).
Compiled as signed MSI installer.

## Offline Capabilities

Form scan results buffered locally (AES-256-GCM encrypted) when network is unavailable.
HSM signing is offline-capable (PKCS#11 local device). Transmission is retried automatically
when connectivity restores.

## Attestation Profile

- Windows EV code signing certificate (from internal CA)
- HSM-backed RO signing key (Thales Luna 7 or YubiKey HSM 2 per station)

## i18n / WCAG

- Language: English (official language for legal declarations)
- WCAG: 2.2 AA

## Build Commands

```bash
cd services/rts-svc/apps/ro-desktop

# Development
cargo tauri dev

# Production (Windows)
cargo tauri build --target x86_64-pc-windows-msvc
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `RTS_API_URL` | rts-svc API for declaration submission |
| `HSM_SLOT` | PKCS#11 slot number for RO signing key |
| `HSM_PIN` | PKCS#11 PIN (prompted on launch, not stored) |
| `SESSION_RECORDING_PATH` | Local path for session video archive |
| `ICR_ENDPOINT` | rts-svc/py/rts-icr gRPC endpoint |
| `FORM_TEMPLATE_PATH` | Local path to Form 34 template overlays |

## Test Commands

```bash
cargo test                   # Rust unit tests
npm run test                 # Frontend unit tests
```

## Deployment Notes

Distributed as a signed MSI via internal Windows Update server or pre-loaded on
Commission-issued laptops. Not a Kubernetes pod. ROs receive the installer on a
provisioned USB drive before the election. HSM keys are provisioned at county
offices by the ICT team.

## SLA

- Form scan to signed transmission: p99 target per rts-svc ICR pipeline

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M09)
- rts-svc/py/rts-icr (ICR service)
- integrations/hsm-pkcs11
- docs/runbooks/returning-officer-rts.md

## Upstream / Downstream

- Upstream: rts-svc/py/rts-icr (ICR), integrations/hsm-pkcs11
- Downstream: rts-svc/crates/rts-ro-api (declaration submission)
