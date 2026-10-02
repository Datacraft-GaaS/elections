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

//! Matcher trait and per-modality implementations.

pub mod face_matcher;
pub mod fingerprint_matcher;
pub mod fusion;
pub mod iris_matcher;

pub use face_matcher::FaceMatcher;
pub use fingerprint_matcher::FingerprintMatcher;
pub use fusion::{FusionPolicy, FusionResult, MultiModalScore};
pub use iris_matcher::IrisMatcher;

use crate::modality::Template;

/// Comparison score returned by every matcher.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Score {
    /// Raw algorithm-native score (e.g. minutiae count, cosine similarity, 1-HD).
    pub raw: f32,
    /// Score normalised to [0.0, 1.0] by the matcher's decision function.
    pub normalised: f32,
    /// Whether `normalised` exceeds the configured decision threshold.
    pub threshold_pass: bool,
}

impl Score {
    pub fn new(raw: f32, normalised: f32, threshold: f32) -> Self {
        Self {
            raw,
            normalised,
            threshold_pass: normalised >= threshold,
        }
    }
}

/// Core matching contract.
pub trait Matcher<T: Template>: Send + Sync {
    /// Compare probe to gallery template and return a composite score.
    fn compare(&self, probe: &T, gallery: &T) -> Score;
}
