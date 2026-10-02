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

//! Compact proof serialisation for embedding in the portal and verifier-rs.
//!
//! A Groth16 proof over BLS12-381 consists of:
//!   - A (G1 point): 48 bytes compressed
//!   - B (G2 point): 96 bytes compressed
//!   - C (G1 point): 48 bytes compressed
//!   Total: 192 bytes
//!
//! This fits well within the 512-byte target for embedding in a QR code or
//! transparency log leaf alongside the tally data.

use ark_bls12_381::Bls12_381;
use ark_groth16::Proof;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::{prover::Groth16Proof, ZkError};

/// Serialise a proof to its canonical compressed byte representation.
///
/// The output is 192 bytes for BLS12-381 Groth16.
pub fn compress(proof: &Groth16Proof) -> Result<Vec<u8>, ZkError> {
    let mut buf = Vec::new();
    proof
        .inner
        .serialize_compressed(&mut buf)
        .map_err(|e| ZkError::Serialise(e.to_string()))?;
    Ok(buf)
}

/// Deserialise a proof from compressed bytes.
pub fn decompress(bytes: &[u8]) -> Result<Groth16Proof, ZkError> {
    let inner = Proof::<Bls12_381>::deserialize_compressed(bytes)
        .map_err(|e| ZkError::Serialise(e.to_string()))?;
    Ok(Groth16Proof { inner })
}

/// Serialise a proof to a hex string.
pub fn to_hex(proof: &Groth16Proof) -> Result<String, ZkError> {
    let bytes = compress(proof)?;
    Ok(hex::encode(bytes))
}

/// Deserialise a proof from a hex string.
pub fn from_hex(s: &str) -> Result<Groth16Proof, ZkError> {
    let bytes = hex::decode(s).map_err(|e| ZkError::Serialise(e.to_string()))?;
    decompress(&bytes)
}
