//! Groth16 verifier for the TallyCircuit.
//!
//! Verification is O(1) pairings regardless of the number of stations --
//! sub-second on any modern CPU.
//!
//! Reference: Groth, Jens. "On the Size of Pairing-Based Non-Interactive Arguments."
//! CRYPTO 2016. Springer LNCS 9666, pp. 305-326. <https://eprint.iacr.org/2016/260>

use ark_bls12_381::{Bls12_381, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, VerifyingKey};

use crate::{circuit::PublishedTally, prover::Groth16Proof, ZkError};

/// Verifies Groth16 proofs for the TallyCircuit.
pub struct TallyVerifier;

impl TallyVerifier {
    /// Verify that `proof` is valid for `published` under `vk`.
    ///
    /// Returns `Ok(true)` iff the proof is valid.
    /// Returns `Ok(false)` for an invalid proof (tampered or forged).
    /// Returns `Err(ZkError::Verify)` if the verifier itself errors.
    ///
    /// Complexity: O(1) pairing operations, independent of station count.
    /// Typical wall time: < 10 ms on a modern CPU.
    pub fn verify(
        proof: &Groth16Proof,
        vk: &VerifyingKey<Bls12_381>,
        published: &PublishedTally,
    ) -> Result<bool, ZkError> {
        // Public inputs: single field element (the declared national total).
        let public_inputs = vec![Fr::from(published.total_votes)];
        let result = Groth16::<Bls12_381>::verify(vk, &public_inputs, &proof.inner)
            .map_err(|e| ZkError::Verify(e.to_string()))?;
        Ok(result)
    }
}
