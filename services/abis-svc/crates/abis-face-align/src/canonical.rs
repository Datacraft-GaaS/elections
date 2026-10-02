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

//! Canonical 5-point landmark positions for the 112x112 ArcFace template.
//!
//! Reference: Deng et al. (2019). ArcFace, CVPR 2019, Appendix A.
//! These coordinates are the standard targets used by InsightFace / ArcFace
//! preprocessing to produce the 112x112 aligned crop.

use crate::landmarks::FiveLandmarks;

/// ArcFace canonical 5-point targets in a 112x112 image.
/// Order: left_eye, right_eye, nose, left_mouth, right_mouth.
pub const CANONICAL_5_RAW: [(f32, f32); 5] = [
    (38.2946, 51.6963), // left eye
    (73.5318, 51.5014), // right eye
    (56.0252, 71.7366), // nose tip
    (41.5493, 92.3655), // left mouth corner
    (70.7299, 92.2041), // right mouth corner
];

/// Canonical landmark positions as a `FiveLandmarks` value.
pub fn canonical_five() -> FiveLandmarks {
    FiveLandmarks::new(
        CANONICAL_5_RAW[0],
        CANONICAL_5_RAW[1],
        CANONICAL_5_RAW[2],
        CANONICAL_5_RAW[3],
        CANONICAL_5_RAW[4],
    )
}

/// Flat array of canonical 5 landmark positions (10 floats).
pub const CANONICAL_5: [f32; 10] = [
    CANONICAL_5_RAW[0].0,
    CANONICAL_5_RAW[0].1,
    CANONICAL_5_RAW[1].0,
    CANONICAL_5_RAW[1].1,
    CANONICAL_5_RAW[2].0,
    CANONICAL_5_RAW[2].1,
    CANONICAL_5_RAW[3].0,
    CANONICAL_5_RAW[3].1,
    CANONICAL_5_RAW[4].0,
    CANONICAL_5_RAW[4].1,
];
