# eel-abis-fhe-match

Homomorphic-encrypted biometric matching for the eel ABIS stack.

## Modules

| Module | Scheme | Operation |
|---|---|---|
| `context` | CKKS/BFV/BGV/TFHE | Context config + V031 row serialisation |
| `match_ckks` | CKKS | 1:1 cosine similarity on 512-d face embeddings |
| `match_bfv` | BFV | 1:1 Hamming distance on iris bitstrings |
| `batch_1n` | CKKS | 1:N gallery search via SIMD slot packing |
| `policy` | -- | 128-bit security + noise budget + depth ceiling |

## Design

No external FHE library is pulled in. The toy implementation runs on scaled
integers with correct semantics for small parameters, enabling tests without
a C++ SEAL dependency. Production plugs a `CkksMatchTrait` implementation
backed by a real CKKS library.

## Tests

Run with: `cargo test -p eel-abis-fhe-match`

20+ unit tests plus proptest invariants.
