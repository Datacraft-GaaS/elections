# eel-abis-cert-fido

FIDO Biometric Component Certification harness for the eel ABIS stack.

## Modules

| Module | Function |
|---|---|
| `evidence` | EvidencePack assembler: PAD report + FMR/FNMR report + tester signature |
| `matrix` | PAI matrix (Type x Species x Medium x Detector) with outcome validation |

## Output

`EvidencePack` contains all artefacts required for a FIDO Biometric Component
Certification submission: PAD species coverage, PAI matrix, FMR/FNMR at the
target operating point, and a tester signature (SHA-256 stub; production uses
an HSM-backed Ed25519 key).

## Tests

Run with: `cargo test -p eel-abis-cert-fido`

12+ tests covering assembly validation, signature determinism, PAI matrix deduplication.
