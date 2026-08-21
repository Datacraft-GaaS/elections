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
