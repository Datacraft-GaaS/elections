//! Trusted setup for the TallyCircuit Groth16 proving system.
//!
//! Production use requires a per-cycle powers-of-tau ceremony with M-of-N
//! participants. This module provides:
//!   - `generate_params`: in-process random setup for testing and CI.
//!   - `CeremonyHook`: interface stubs for production M-of-N ceremony.
//!
//! Security note: `generate_params` uses a single random toxic waste value.
//! The resulting CRS is NOT suitable for production -- use a multi-party
//! computation ceremony (e.g. Zcash powers-of-tau format) instead.
//!
//! Reference: Ben-Sasson et al., "Scalable, Transparent, and Post-Quantum Secure
//! Computational Integrity", IACR 2018/046.

use ark_bls12_381::{Bls12_381, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, ProvingKey};
use rand::thread_rng;

use crate::{
    circuit::{TallyCircuit, MAX_STATIONS},
    ZkError,
};

/// Proving key pair for the TallyCircuit.
pub struct TallyParams {
    pub(crate) pk: ProvingKey<Bls12_381>,
}

impl TallyParams {
    /// Return the verifying key.
    pub fn vk(&self) -> ark_groth16::VerifyingKey<Bls12_381> {
        self.pk.vk.clone()
    }
}

/// Generate Groth16 CRS parameters for a circuit with `n_stations` leaves.
///
/// Uses locally-generated random toxic waste -- NOT safe for production.
/// `n_stations` defaults to `MAX_STATIONS` when `None` is passed.
pub fn generate_params(n_stations: impl Into<Option<usize>>) -> Result<TallyParams, ZkError> {
    let n = n_stations.into().unwrap_or(MAX_STATIONS);
    let circuit = TallyCircuit::<Fr> {
        leaves: vec![0u64; n],
        published_total: 0,
        max_stations: n,
        _phantom: std::marker::PhantomData,
    };
    let mut rng = thread_rng();
    let pk = Groth16::<Bls12_381>::generate_random_parameters_with_reduction(circuit, &mut rng)
        .map_err(|e| ZkError::Setup(e.to_string()))?;
    Ok(TallyParams { pk })
}

/// Stub interface for production M-of-N ceremony integration.
///
/// Each participant calls `contribute` in sequence; the final coordinator
/// calls `finalise` to produce the proving key. Toxic waste from each
/// contribution is immediately discarded by the participant.
pub trait CeremonyHook: Send + Sync {
    /// Apply this participant's contribution to the accumulator.
    fn contribute(&self, accumulator: &[u8]) -> Result<Vec<u8>, ZkError>;

    /// Finalise the ceremony and return the serialized proving key.
    fn finalise(&self, accumulator: &[u8]) -> Result<Vec<u8>, ZkError>;

    /// Verify a specific participant's contribution (for transcript audit).
    fn verify_contribution(&self, before: &[u8], after: &[u8]) -> Result<bool, ZkError>;
}
