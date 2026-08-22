# eel-rts-fhe-tally

CKKS homomorphic tally over encrypted Form34A leaves (HARD-B item 9).

## Algorithm

Implements the CKKS approximate homomorphic encryption scheme:

> Cheon, J. H., Kim, A., Kim, M., Song, Y.
> "Homomorphic Encryption for Arithmetic of Approximate Numbers."
> ASIACRYPT 2017. LNCS 10624, pp. 409-437.

Each polling station submits an encrypted Form34A leaf. The aggregator sums
all leaves homomorphically (no decryption). The final tally is decrypted once
using M-of-N threshold key shares held in HSMs.

Threshold secret sharing:

> Shamir, A. "How to Share a Secret."
> Communications of the ACM, vol. 22, no. 11, pp. 612-613, 1979.

Default: M=3, N=5 (3-of-5 HSM ceremony).

## Security parameters

| Parameter            | CI (toy) | Production |
|----------------------|----------|------------|
| poly_modulus_degree  | 32       | >= 8192    |
| security_level_bits  | 1        | >= 128     |
| Circuit depth        | 2        | 20         |

Production parameters are enforced by FhePolicy. Tests use FhePolicy::test()
to bypass the check.

## Performance caveats

At production parameters (degree 8192, 128-bit security):
- One homomorphic addition: ~0.5ms per slot vector (Cortex-X2).
- Full 47,500-station aggregation: ~24s single-thread, ~3s with 8 threads.
- Source: Microsoft SEAL benchmark suite, 2024.

The aggregation transcript (prove.rs) is a lightweight public proof of correct
aggregation verifiable without decryption keys. A full Groth16/Bulletproofs
ZK-SNARK is planned in HARD-A item 2.
