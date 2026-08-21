//! FHE context configuration and serialisation.
//!
//! Matches the V031 eel_fhe_contexts row shape:
//!   scheme, polynomial_degree, plaintext_modulus, security_level_bits,
//!   noise_budget_initial, circuit_depth_max.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("scheme {0:?} is not supported")]
    UnsupportedScheme(FheScheme),
    #[error("polynomial_degree must be a power of 2 >= 1024")]
    InvalidPolyDegree,
    #[error("security_level_bits must be >= 80")]
    InsufficientSecurity,
    #[error("circuit_depth_max must be >= 1")]
    InvalidDepth,
    #[error("JSON serialisation error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Supported FHE schemes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FheScheme {
    Ckks,
    Bfv,
    Bgv,
    Tfhe,
}

/// FHE context matching the eel_fhe_contexts database row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FheContext {
    pub scheme: FheScheme,
    pub polynomial_degree: usize,
    pub plaintext_modulus: u64,
    pub security_level_bits: u32,
    pub noise_budget_initial: u32,
    pub circuit_depth_max: u32,
}

/// Serialised form for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedContext {
    pub json: String,
}

impl FheContext {
    /// Constructs a validated FHE context.
    ///
    /// # Errors
    /// Returns errors for invalid polynomial degrees or insufficient security.
    pub fn new(
        scheme: FheScheme,
        polynomial_degree: usize,
        plaintext_modulus: u64,
        security_level_bits: u32,
        noise_budget_initial: u32,
        circuit_depth_max: u32,
    ) -> Result<Self, ContextError> {
        if polynomial_degree < 1024 || !polynomial_degree.is_power_of_two() {
            return Err(ContextError::InvalidPolyDegree);
        }
        if security_level_bits < 80 {
            return Err(ContextError::InsufficientSecurity);
        }
        if circuit_depth_max < 1 {
            return Err(ContextError::InvalidDepth);
        }
        Ok(Self {
            scheme,
            polynomial_degree,
            plaintext_modulus,
            security_level_bits,
            noise_budget_initial,
            circuit_depth_max,
        })
    }

    /// CKKS preset suitable for 512-d face matching.
    pub fn ckks_face_preset() -> Result<Self, ContextError> {
        Self::new(FheScheme::Ckks, 8192, 0, 128, 300, 20)
    }

    /// BFV preset for iris Hamming distance.
    pub fn bfv_iris_preset() -> Result<Self, ContextError> {
        Self::new(FheScheme::Bfv, 4096, 65537, 128, 200, 10)
    }

    /// Serialises to JSON for persistence.
    ///
    /// # Errors
    /// Returns a JSON error on serialisation failure.
    pub fn serialise(&self) -> Result<SerializedContext, ContextError> {
        let json = serde_json::to_string(self)?;
        Ok(SerializedContext { json })
    }

    /// Deserialises from a persisted JSON blob.
    ///
    /// # Errors
    /// Returns a JSON error on deserialisation failure.
    pub fn deserialise(sc: &SerializedContext) -> Result<Self, ContextError> {
        let ctx = serde_json::from_str(&sc.json)?;
        Ok(ctx)
    }
}
