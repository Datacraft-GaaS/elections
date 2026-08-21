//! PAI matrix: attack type x species x medium x detector expected outcome.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaiMatrixError {
    #[error("duplicate entry for type={attack_type:?} species={species:?} medium={medium:?}")]
    DuplicateEntry {
        attack_type: AttackType,
        species: PaiSpecies,
        medium: AttackMedium,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    Artefact,
    VideoReplay,
    DigitalManipulation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaiSpecies {
    Face,
    Fingerprint,
    Iris,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackMedium {
    Print,
    Screen,
    Silicone,
    Gelatin,
    Digital,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectedOutcome {
    Blocked,
    Passed,
}

/// One cell in the PAI matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaiMatrixEntry {
    pub attack_type: AttackType,
    pub species: PaiSpecies,
    pub medium: AttackMedium,
    pub expected_outcome: ExpectedOutcome,
    pub apcer: f64,
}

/// Full PAI coverage matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaiMatrix {
    pub entries: Vec<PaiMatrixEntry>,
}

impl PaiMatrix {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the matrix.
    ///
    /// # Errors
    /// Returns `PaiMatrixError::DuplicateEntry` if the combination already exists.
    pub fn add(&mut self, entry: PaiMatrixEntry) -> Result<(), PaiMatrixError> {
        let exists = self.entries.iter().any(|e| {
            e.attack_type == entry.attack_type
                && e.species == entry.species
                && e.medium == entry.medium
        });
        if exists {
            return Err(PaiMatrixError::DuplicateEntry {
                attack_type: entry.attack_type,
                species: entry.species,
                medium: entry.medium,
            });
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Returns all entries where the expected outcome is Blocked.
    pub fn blocked_entries(&self) -> impl Iterator<Item = &PaiMatrixEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.expected_outcome, ExpectedOutcome::Blocked))
    }

    /// Returns the maximum APCER across all entries.
    pub fn max_apcer(&self) -> f64 {
        self.entries.iter().map(|e| e.apcer).fold(0.0f64, f64::max)
    }
}

impl Default for PaiMatrix {
    fn default() -> Self {
        Self::new()
    }
}
