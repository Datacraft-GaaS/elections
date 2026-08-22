# zk-tally

Zero-knowledge proof of correct tally aggregation for the eel election platform.

Proves that the sum of 34A station-level vote counts equals the published national tally without revealing individual station tallies. Implemented as a Groth16 SNARK over BLS12-381.

## Algorithm citations

- Groth, Jens. "On the Size of Pairing-Based Non-Interactive Arguments." CRYPTO 2016. Springer LNCS 9666, pp. 305-326. <https://eprint.iacr.org/2016/260>

- Bunz, B., Bootle, J., Boneh, D., Poelstra, A., Wuille, P., Maxwell, G. "Bulletproofs: Short Proofs for Confidential Transactions and More." IEEE Symposium on Security and Privacy 2018. <https://eprint.iacr.org/2017/1066>

## Proof size

A Groth16 proof over BLS12-381 is 192 bytes (2 G1 + 1 G2 point, compressed). This fits in a single QR code alongside the tally data.

## Usage

```rust
use zk_tally::{TallyLeaf, PublishedTally, setup, TallyProver, TallyVerifier};

let leaves = vec![
    TallyLeaf { station_id: "KE-01-0001".into(), votes: 1234 },
    TallyLeaf { station_id: "KE-01-0002".into(), votes: 5678 },
];
let published = PublishedTally { total_votes: 6912 };
let params = setup::generate_params(2).unwrap();
let proof = TallyProver::prove(&leaves, &published, &params).unwrap();
assert!(TallyVerifier::verify(&proof, &params.vk(), &published).unwrap());
```

## Security

The `setup::generate_params` function uses a single random toxic waste value and is NOT safe for production. Production deployments must use output from a multi-party computation (MPC) ceremony using the `CeremonyHook` interface.

## Crate structure

| Module | Purpose |
|---|---|
| `circuit` | R1CS constraint system (TallyCircuit) |
| `prover` | Groth16 prover (TallyProver) |
| `verifier` | Groth16 verifier (TallyVerifier) |
| `setup` | Trusted setup + M-of-N ceremony stubs |
| `serialise` | 192-byte proof serialisation |
| `error` | Error types |
