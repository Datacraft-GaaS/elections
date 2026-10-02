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

//! Local similarity metric between two MCC cylinders.
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010, Section III-B, eq. (8).
//!
//! sim(C_a, C_b) = dot(C_a, C_b) / sqrt(popcount(C_a) * popcount(C_b))
//! Returns 0.0 when either cylinder has zero set bits.

use crate::cylinder::MinutiaCylinder;

/// Cosine-like local similarity between two cylinders over their binary bit vectors.
///
/// This is the LSS (Local Similarity Sort) metric from Cappelli 2010 eq. (8).
pub fn local_similarity(a: &MinutiaCylinder, b: &MinutiaCylinder) -> f32 {
    if a.popcount == 0 || b.popcount == 0 {
        return 0.0;
    }
    let dot = a.dot(b) as f32;
    let denom = (a.popcount as f32 * b.popcount as f32).sqrt();
    dot / denom
}
