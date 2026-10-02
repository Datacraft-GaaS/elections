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

//! The `FpMatcher` trait: uniform interface for all fingerprint matchers.
//!
//! Any matcher that implements this trait can be evaluated by the FpVTE harness,
//! enabling head-to-head comparison of Bozorth3, MCC, SourceAFIS, DL, and
//! consensus matchers.

use crate::FpVteError;

/// A fingerprint template as raw bytes.
pub type Template = Vec<u8>;

/// Pluggable fingerprint matcher.
///
/// Implementors should be thread-safe (`Send + Sync`) so the harness can
/// parallelise the 1:N gallery probe.
pub trait FpMatcher: Send + Sync {
    /// Unique name for this matcher (used in reports).
    fn name(&self) -> &str;

    /// Compare two templates.  Returns a similarity score in [0, 1].
    fn compare(&self, probe: &Template, gallery: &Template) -> Result<f32, FpVteError>;
}

/// A no-op matcher that always returns 0.0 (useful as a baseline sentinel).
pub struct NullMatcher;

impl FpMatcher for NullMatcher {
    fn name(&self) -> &str {
        "null"
    }

    fn compare(&self, _probe: &Template, _gallery: &Template) -> Result<f32, FpVteError> {
        Ok(0.0)
    }
}

/// A deterministic synthetic matcher: returns cosine-similarity-like score
/// derived from the first bytes of both templates.
pub struct SyntheticMatcher {
    pub name: String,
}

impl FpMatcher for SyntheticMatcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn compare(&self, probe: &Template, gallery: &Template) -> Result<f32, FpVteError> {
        // Simple synthetic score: fraction of matching bytes in the first 64 bytes.
        let len = probe.len().min(gallery.len()).min(64).max(1);
        let matching = probe[..len]
            .iter()
            .zip(gallery[..len].iter())
            .filter(|(a, b)| a == b)
            .count();
        Ok(matching as f32 / len as f32)
    }
}
