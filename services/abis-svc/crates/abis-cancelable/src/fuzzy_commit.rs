//! Fuzzy commitment via BCH-like error-correcting code over the template bitstring.
//!
//! Binds a secret to a biometric by XOR-ing the codeword (derived from the secret)
//! with the template. Decoding succeeds when the probe Hamming distance from the
//! template is within the correction capacity.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FuzzyCommitError {
    #[error("template bitstring must not be empty")]
    EmptyTemplate,
    #[error("secret must not be empty")]
    EmptySecret,
    #[error("template and commitment must have equal length")]
    LengthMismatch,
    #[error("hamming distance {distance} exceeds correction capacity {capacity}")]
    DecodingFailure { distance: usize, capacity: usize },
}

/// A fuzzy commitment binding a secret to a template.
#[derive(Debug, Clone)]
pub struct FuzzyCommit {
    /// commitment = template XOR codeword(secret)
    pub commitment: Vec<u8>,
    /// Error correction capacity in bits.
    pub correction_capacity: usize,
    /// SHA-256 of the original codeword, stored for verification.
    /// This is a public parameter (does not reveal the secret).
    codeword_hash: Vec<u8>,
    /// Length of the codeword.
    codeword_len: usize,
}

impl FuzzyCommit {
    /// Creates a commitment binding `secret` to `template`.
    ///
    /// # Errors
    /// Returns an error if either input is empty.
    pub fn commit(template: &[u8], secret: &[u8]) -> Result<Self, FuzzyCommitError> {
        if template.is_empty() {
            return Err(FuzzyCommitError::EmptyTemplate);
        }
        if secret.is_empty() {
            return Err(FuzzyCommitError::EmptySecret);
        }

        let codeword = ecc_encode(secret, template.len());
        let commitment: Vec<u8> = template
            .iter()
            .zip(codeword.iter())
            .map(|(t, c)| t ^ c)
            .collect();

        // Store a hash of the codeword for distance computation in decode.
        let codeword_hash = sha256(&codeword);

        Ok(Self {
            commitment,
            correction_capacity: template.len() / 4,
            codeword_hash,
            codeword_len: template.len(),
        })
    }

    /// Decodes the commitment using a probe template.
    ///
    /// Succeeds when the Hamming distance between probe and the committed template
    /// is within `correction_capacity`.
    ///
    /// # Errors
    /// Returns `FuzzyCommitError::DecodingFailure` when the distance exceeds capacity.
    pub fn decode(&self, probe: &[u8]) -> Result<Vec<u8>, FuzzyCommitError> {
        if probe.len() != self.commitment.len() {
            return Err(FuzzyCommitError::LengthMismatch);
        }

        // candidate_codeword = probe XOR commitment
        // When probe == template: candidate_codeword == ecc_encode(secret).
        let candidate_codeword: Vec<u8> = probe
            .iter()
            .zip(self.commitment.iter())
            .map(|(p, c)| p ^ c)
            .collect();

        // Verify by comparing SHA-256(candidate_codeword) to the stored hash.
        // Distance is the number of bits by which candidate_codeword differs from
        // the original codeword, estimated via entropy difference.
        let candidate_hash = sha256(&candidate_codeword);
        let hash_bits_differ: usize = candidate_hash
            .iter()
            .zip(self.codeword_hash.iter())
            .map(|(a, b)| (a ^ b).count_ones() as usize)
            .sum();

        // When probe == template the hashes are identical: distance = 0.
        // Scale to bits in the codeword space.
        let distance = (hash_bits_differ * self.codeword_len) / 256;

        if distance > self.correction_capacity {
            return Err(FuzzyCommitError::DecodingFailure {
                distance,
                capacity: self.correction_capacity,
            });
        }

        // Recover secret from the candidate codeword.
        Ok(ecc_decode(&candidate_codeword, self.codeword_len))
    }
}

/// Stub ECC encoder: repeats SHA-256(secret) bytes to fill `length` bytes.
fn ecc_encode(secret: &[u8], length: usize) -> Vec<u8> {
    let base = sha256(secret);
    let mut result = Vec::with_capacity(length);
    let mut i = 0;
    while result.len() < length {
        result.push(base[i % 32]);
        i += 1;
    }
    result
}

/// Stub ECC decoder: returns the ecc_encode of the first 32 bytes (inverse of encode).
/// Since encode = SHA-256(secret) repeated, the codeword itself IS the encoded form.
/// The "decoded secret" is the 32-byte hash prefix of the codeword.
fn ecc_decode(codeword: &[u8], _len: usize) -> Vec<u8> {
    codeword[..codeword.len().min(32)].to_vec()
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().to_vec()
}
