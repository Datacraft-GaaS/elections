# eel-abis-cancelable

ISO/IEC 24745 cancelable biometrics for the eel ABIS stack.

## Modules

| Module | Function |
|---|---|
| `biohash` | BioHashing: irreversible feature x user-key to bit string |
| `random_projection` | Johnson-Lindenstrauss user-specific projection |
| `fuzzy_vault` | Polynomial-encoded template with chaff-point protection |
| `fuzzy_commit` | BCH-based fuzzy commitment over template bitstrings |
| `revocation` | Append-only revocation ledger with event emission |
| `renewal` | Key rotation: revoke old, derive new, emit renewal record |

## Cancelability

Revocation rotates the user key. Because the BioHash and random projection are
keyed, the old and new templates are computationally unlinkable: distinguishing
them requires breaking SHA-256.

## Tests

Run with: `cargo test -p eel-abis-cancelable`

25+ tests covering: irreversibility, fuzzy vault threshold, JL distance
preservation, revocation ledger append, renewal unlinkability.
