# eel-abis-shielding

Irreversible shielded biometric templates for the eel ABIS stack.

## Modules

| Module | Function |
|---|---|
| `shield` | Bloom-filter shielded template construction and intersection-based verify |
| `params` | Parameter selection to meet target FMR/FNMR under shielded matching |

## Design

No plaintext template is ever stored or transmitted; only the Bloom filter
leaves the shielding step. Similarity is computed via Jaccard intersection
of two filters, making cross-scheme linking computationally infeasible.

## Tests

Run with: `cargo test -p eel-abis-shielding`

15+ tests covering determinism, cross-scheme unlinkability, FMR/FNMR bounds.
