//! Face challenge-response PAD.
//!
//! Generates a random challenge sequence (blink, head-turn, smile, gaze-target),
//! verifies the captured response, and defends against replay via nonce binding.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Individual challenge types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaceChallenge {
    Blink,
    HeadTurnLeft,
    HeadTurnRight,
    Smile,
    GazeTarget { x: u8, y: u8 },
}

/// A sequence of challenges with a bound nonce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeSequence {
    pub steps: Vec<FaceChallenge>,
    /// Nonce binds the challenge to a single session; must be unique per issuance.
    pub nonce: String,
}

/// Result of verifying a challenge response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeVerifyResult {
    pub decision: Decision,
    pub steps_passed: usize,
    pub steps_total: usize,
}

/// Issues and verifies face challenge sequences.
///
/// Maintains a replay-defence nonce store. In production this store is backed
/// by a distributed cache with a TTL equal to the challenge window.
pub struct ChallengeVerifier {
    seen_nonces: Arc<Mutex<HashSet<String>>>,
    /// Minimum fraction of steps that must pass for a Live decision.
    pass_threshold: f64,
}

impl ChallengeVerifier {
    pub fn new(pass_threshold: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&pass_threshold) {
            return Err(PadError::InvalidInput {
                detail: "pass_threshold must be in [0, 1]".into(),
            });
        }
        Ok(Self {
            seen_nonces: Arc::new(Mutex::new(HashSet::new())),
            pass_threshold,
        })
    }

    /// Generates a deterministic-but-unpredictable sequence from a seed + nonce.
    ///
    /// # Errors
    /// Returns `PadError::ReplayAttack` if the nonce was previously issued.
    pub fn issue(&self, nonce: &str, length: usize) -> Result<ChallengeSequence, PadError> {
        let mut store = self
            .seen_nonces
            .lock()
            .map_err(|_| PadError::InvalidInput {
                detail: "nonce store lock poisoned".into(),
            })?;
        if store.contains(nonce) {
            return Err(PadError::ReplayAttack {
                nonce: nonce.to_owned(),
            });
        }
        store.insert(nonce.to_owned());

        let steps = generate_steps(nonce, length);
        Ok(ChallengeSequence {
            steps,
            nonce: nonce.to_owned(),
        })
    }

    /// Verifies that the provided responses satisfy the issued challenge.
    ///
    /// `responses` must be ordered to match `challenge.steps`.
    pub fn verify(
        &self,
        challenge: &ChallengeSequence,
        responses: &[FaceChallenge],
    ) -> ChallengeVerifyResult {
        let total = challenge.steps.len();
        let passed = challenge
            .steps
            .iter()
            .zip(responses.iter())
            .filter(|(expected, actual)| expected == actual)
            .count();

        let fraction = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };

        let decision = if fraction >= self.pass_threshold {
            Decision::Live
        } else {
            Decision::Spoof
        };

        ChallengeVerifyResult {
            decision,
            steps_passed: passed,
            steps_total: total,
        }
    }
}

/// Derives a challenge step sequence from nonce using SHA-256 as a PRF.
fn generate_steps(nonce: &str, length: usize) -> Vec<FaceChallenge> {
    let options: &[FaceChallenge] = &[
        FaceChallenge::Blink,
        FaceChallenge::HeadTurnLeft,
        FaceChallenge::HeadTurnRight,
        FaceChallenge::Smile,
        FaceChallenge::GazeTarget { x: 50, y: 50 },
        FaceChallenge::GazeTarget { x: 20, y: 80 },
        FaceChallenge::GazeTarget { x: 80, y: 20 },
    ];
    let mut steps = Vec::with_capacity(length);
    let mut hasher_input = nonce.as_bytes().to_vec();
    for i in 0..length {
        hasher_input.extend_from_slice(&(i as u64).to_le_bytes());
        let digest = Sha256::digest(&hasher_input);
        let idx = (digest[0] as usize) % options.len();
        steps.push(options[idx].clone());
        // Feed previous digest forward to avoid cycles.
        hasher_input = digest.to_vec();
    }
    steps
}
