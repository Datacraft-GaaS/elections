//! MinutiaCylinder: the 3-D binary descriptor built around each minutia.
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010, Section III-A.
//!
//! A cylinder is indexed as C[k][l][j] where:
//!   k, l in {0..Ns-1} are the spatial cell indices,
//!   j   in {0..Nd-1} is the directional layer.
//! The cylinder is stored as a packed bit-vector of length Ns*Ns*Nd bits,
//! row-major: index = (k * Ns + l) * Nd + j.

use serde::{Deserialize, Serialize};

/// Packed 3-D binary cylinder descriptor for one minutia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutiaCylinder {
    /// Index of the source minutia in the template.
    pub minutia_ref: usize,
    /// Cylinder radius used at build time (pixels).
    pub radius: f32,
    /// Spatial grid size (Ns).
    pub ns: u8,
    /// Directional layers (Nd).
    pub nd: u8,
    /// Packed bits: length = ceil(Ns * Ns * Nd / 8).
    pub cells: Vec<u8>,
    /// Count of set bits (pre-computed for fast similarity).
    pub popcount: u32,
    /// Whether this cylinder meets the validity threshold tau_psi.
    pub valid: bool,
}

impl MinutiaCylinder {
    /// Total number of cells.
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.ns as usize * self.ns as usize * self.nd as usize
    }

    /// Get the bit at flat cell index `idx`.
    #[inline]
    pub fn bit(&self, idx: usize) -> bool {
        let byte = idx / 8;
        let bit = idx % 8;
        byte < self.cells.len() && (self.cells[byte] >> bit) & 1 == 1
    }

    /// Set the bit at flat cell index `idx`.
    #[inline]
    pub fn set_bit(&mut self, idx: usize) {
        let byte = idx / 8;
        let bit = idx % 8;
        if byte < self.cells.len() {
            self.cells[byte] |= 1 << bit;
        }
    }

    /// Dot product with another cylinder (number of shared set bits).
    pub fn dot(&self, other: &Self) -> u32 {
        self.cells
            .iter()
            .zip(other.cells.iter())
            .map(|(&a, &b)| (a & b).count_ones())
            .sum()
    }
}
