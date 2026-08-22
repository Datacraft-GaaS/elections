//! Groth16 prover for the TallyCircuit.
//!
//! Reference: Groth, Jens. "On the Size of Pairing-Based Non-Interactive Arguments."
//! CRYPTO 2016. Springer LNCS 9666, pp. 305-326. <https://eprint.iacr.org/2016/260>

use ark_bls12_381::{Bls12_381, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use rand::thread_rng;

use crate::{
    circuit::{PublishedTally, TallyCircuit, TallyLeaf},
    setup::TallyParams,
    ZkError,
};

/// A Groth16 proof over BLS12-381, wrapping the arkworks proof type.
pub struct Groth16Proof {
    pub(crate) inner: ark_groth16::Proof<Bls12_381>,
}

/// Generates Groth16 proofs for the TallyCircuit.
pub struct TallyProver;

impl TallyProver {
    /// Prove that `sum(leaves) == published.total_votes`.
    ///
    /// `leaves` may have at most as many entries as the circuit was set up with.
    ///
    /// Returns `Err(ZkError::TallyMismatch)` if the leaves do not sum to `published.total_votes`.
    /// Returns `Err(ZkError::Prove)` if the proving system fails internally.
    pub fn prove(
        leaves: &[TallyLeaf],
        published: &PublishedTally,
        params: &TallyParams,
    ) -> Result<Groth16Proof, ZkError> {
        // Sanity-check: leaves must sum to the declared total.
        let leaves_sum: u64 = leaves.iter().map(|l| l.votes).sum();
        if leaves_sum != published.total_votes {
            return Err(ZkError::TallyMismatch {
                leaves_sum,
                published: published.total_votes,
            });
        }

        let max_stations = leaves.len();
        let circuit = TallyCircuit::<Fr>::new(leaves, published, max_stations)?;

        let mut rng = thread_rng();
        let proof = Groth16::<Bls12_381>::prove(&params.pk, circuit, &mut rng)
            .map_err(|e| ZkError::Prove(e.to_string()))?;

        Ok(Groth16Proof { inner: proof })
    }
}
