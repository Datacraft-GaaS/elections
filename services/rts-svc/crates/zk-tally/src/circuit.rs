//! TallyCircuit: R1CS constraint system for correct tally aggregation.
//!
//! The circuit enforces:
//!   sum(leaf_votes[i] for i in 0..n) == published_total
//!
//! where each `leaf_votes[i]` is a private input (station 34A count) and
//! `published_total` is a public input (the declared national tally).
//!
//! The circuit handles up to `MAX_STATIONS` leaves (padded with zeros).
//!
//! Reference: Groth, "On the Size of Pairing-Based Non-Interactive Arguments",
//! CRYPTO 2016.

use ark_ff::PrimeField;
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Maximum number of stations supported by a single circuit instance.
///
/// Corresponds to the total number of polling stations in Kenya (2022 figure).
pub const MAX_STATIONS: usize = 46_229;

/// A single polling station tally leaf.
#[derive(Debug, Clone)]
pub struct TallyLeaf {
    /// Station identifier (e.g. "KE-01-0001"). Not committed to the circuit,
    /// used only for logging and audit.
    pub station_id: String,

    /// Number of votes cast at this station. Private witness input.
    pub votes: u64,
}

/// The declared published tally (national total). This is the public input.
#[derive(Debug, Clone)]
pub struct PublishedTally {
    /// Total valid votes across all stations.
    pub total_votes: u64,
}

/// R1CS circuit for verifying that the sum of station votes equals the published total.
///
/// `F` is the scalar field of the pairing-friendly curve (BLS12-381 Fr).
/// Private inputs: `leaves` (station vote counts, padded with zeros to `max_stations`).
/// Public input: `published_total`.
pub struct TallyCircuit<F: PrimeField> {
    /// Private witness: vote counts per station (padded to `max_stations` with 0).
    pub leaves: Vec<u64>,

    /// Public input: declared national total.
    pub published_total: u64,

    /// Maximum number of stations (circuit size). Must equal the value used at setup.
    pub max_stations: usize,

    pub _phantom: std::marker::PhantomData<F>,
}

impl<F: PrimeField> TallyCircuit<F> {
    /// Construct a circuit from leaves and a published tally.
    ///
    /// Pads `leaves` with zeros if fewer than `max_stations` are provided.
    /// Returns an error if more than `max_stations` leaves are supplied.
    pub fn new(
        leaves: &[TallyLeaf],
        published: &PublishedTally,
        max_stations: usize,
    ) -> Result<Self, crate::ZkError> {
        if leaves.len() > max_stations {
            return Err(crate::ZkError::TooManyLeaves(leaves.len(), max_stations));
        }
        let mut padded: Vec<u64> = leaves.iter().map(|l| l.votes).collect();
        padded.resize(max_stations, 0);
        Ok(Self {
            leaves: padded,
            published_total: published.total_votes,
            max_stations,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for TallyCircuit<F> {
    /// Generate R1CS constraints.
    ///
    /// Constraint structure:
    ///   1. Allocate each leaf as a private witness variable.
    ///   2. Accumulate the sum via addition constraints.
    ///   3. Allocate the published total as a public input.
    ///   4. Assert sum == published_total.
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Accumulate sum of all leaf vote counts as private witnesses.
        let mut sum = FpVar::<F>::new_constant(cs.clone(), F::zero())?;
        for &votes in &self.leaves {
            let leaf_var = FpVar::<F>::new_witness(cs.clone(), || Ok(F::from(votes)))?;
            sum += &leaf_var;
        }

        // Allocate the published total as a public input.
        let published_var =
            FpVar::<F>::new_input(cs.clone(), || Ok(F::from(self.published_total)))?;

        // Enforce: sum == published_total.
        sum.enforce_equal(&published_var)?;

        Ok(())
    }
}
