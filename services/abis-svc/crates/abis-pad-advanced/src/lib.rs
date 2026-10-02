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

//! eel-abis-pad-advanced: iBeta Level 2 Presentation Attack Detection.
//!
//! Provides multi-modal PAD covering fingerprint (multispectral, LivDet-family),
//! face (challenge-response, passive texture, deepfake), iris (pupil dynamics),
//! and voice (replay, synthesis, conversion) modalities.
//!
//! Enforces iBeta Level 1 and Level 2 PAI coverage via `PadPolicy`.
#![warn(missing_docs)]

pub mod challenge_response;
pub mod deepfake;
pub mod iris_pad;
pub mod livdet;
pub mod multispectral;
pub mod passive_face;
pub mod policy;
pub mod voice_pad;

pub use challenge_response::{ChallengeSequence, ChallengeVerifier, FaceChallenge};
pub use deepfake::{DeepfakeDetector, DeepfakeResult};
pub use iris_pad::{IrisPadResult, IrisPadVerifier};
pub use livdet::{LivDetResult, LivDetVerifier};
pub use multispectral::{MultispectralCapture, MultispectralPadResult};
pub use passive_face::{PassiveFaceResult, PassiveFaceVerifier};
pub use policy::{PadPolicy, PolicyOutcome};
pub use voice_pad::{VoicePadResult, VoicePadVerifier};

/// Common error type for all PAD modules.
#[derive(Debug, thiserror::Error)]
pub enum PadError {
    #[error("replay attack detected: nonce {nonce} already seen")]
    ReplayAttack { nonce: String },
    #[error("below liveness threshold: score {score:.4} < {threshold:.4}")]
    BelowThreshold { score: f64, threshold: f64 },
    #[error("policy violation: {detail}")]
    PolicyViolation { detail: String },
    #[error("invalid input: {detail}")]
    InvalidInput { detail: String },
}

/// Liveness decision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Decision {
    Live,
    Spoof,
    Uncertain,
}
