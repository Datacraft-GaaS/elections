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

//! Biometric modality types: fingerprint, face, iris.

pub mod face;
pub mod fingerprint;
pub mod iris;

pub use face::FaceTemplate;
pub use fingerprint::{FingerPosition, FingerprintTemplate, Minutia, MinutiaeType};
pub use iris::IrisTemplate;

/// Biometric modality discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Fingerprint,
    Face,
    Iris,
}

/// Core template contract every modality implements.
pub trait Template: Sized + Clone + Send + Sync {
    /// Which modality this template belongs to.
    fn modality() -> Modality;

    /// Normalised quality score in [0.0, 1.0].
    ///
    /// NFIQ2-style for fingerprint (ISO 29794-4), ICAO quality for face (ISO 29794-5),
    /// and Daugman-style usable-bits for iris (ISO 29794-6).
    fn quality(&self) -> f32;

    /// Raw feature bytes (e.g. serialised minutiae, embedding floats, IrisCode bits).
    fn feature_bytes(&self) -> Vec<u8>;

    /// Return a normalised copy of the template (zero-mean, unit-norm where applicable).
    fn normalise(&self) -> Self;
}
