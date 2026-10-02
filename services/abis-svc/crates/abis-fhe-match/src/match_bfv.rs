// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! BFV Hamming-distance 1:1 match on encrypted iris bitstrings.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BfvMatchError {
    #[error("empty ciphertext")]
    EmptyCiphertext,
    #[error("bitstring length mismatch: probe {probe} != gallery {gallery}")]
    LengthMismatch { probe: usize, gallery: usize },
    #[error("hamming distance {distance} exceeds threshold {threshold}")]
    AboveThreshold { distance: usize, threshold: usize },
}

/// Toy BFV ciphertext for an iris bitstring.
#[derive(Debug, Clone)]
pub struct BfvCiphertext {
    /// Bit values as 0/1 integers (plaintext modulus = 2).
    pub bits: Vec<u8>,
}

impl BfvCiphertext {
    /// Encrypts an iris bitstring.
    ///
    /// # Errors
    /// Returns an error if the slice is empty.
    pub fn encrypt(bitstring: &[u8]) -> Result<Self, BfvMatchError> {
        if bitstring.is_empty() {
            return Err(BfvMatchError::EmptyCiphertext);
        }
        // Toy: store bits as 0/1 bytes.
        let bits: Vec<u8> = bitstring.iter().map(|&b| b & 1).collect();
        Ok(Self { bits })
    }

    pub fn decrypt(&self) -> Vec<u8> {
        self.bits.clone()
    }
}

pub struct BfvMatcher;

impl BfvMatcher {
    /// Computes homomorphic Hamming distance.
    ///
    /// Returns the distance when within threshold, else an error.
    ///
    /// # Errors
    /// Returns errors on length mismatch or when distance exceeds threshold.
    pub fn match_1v1(
        probe: &BfvCiphertext,
        gallery: &BfvCiphertext,
        threshold: usize,
    ) -> Result<usize, BfvMatchError> {
        if probe.bits.is_empty() || gallery.bits.is_empty() {
            return Err(BfvMatchError::EmptyCiphertext);
        }
        if probe.bits.len() != gallery.bits.len() {
            return Err(BfvMatchError::LengthMismatch {
                probe: probe.bits.len(),
                gallery: gallery.bits.len(),
            });
        }

        // Toy HE: XOR and popcount directly.
        let distance: usize = probe
            .bits
            .iter()
            .zip(gallery.bits.iter())
            .filter(|(a, b)| a != b)
            .count();

        if distance > threshold {
            return Err(BfvMatchError::AboveThreshold {
                distance,
                threshold,
            });
        }
        Ok(distance)
    }
}
