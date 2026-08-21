//! Spoof detection stub: flags inconsistency between recovered 3DMM pose and
//! expected sensor pose.
//!
//! A printed-photo or replay attack typically shows a planar object with
//! specular reflection patterns inconsistent with a real face surface. One
//! geometric cue: the 3DMM fit pose (recovered from landmarks) should agree
//! with the pose implied by sensor/camera metadata within a tolerance.
//!
//! Reference: Boulkenafet, Z. et al. (2017). "Face Anti-Spoofing Using
//! Texture and Depth Information." ICCV 2017 Workshop.

use crate::bfm::Pose3d;
use serde::{Deserialize, Serialize};

/// Outcome of the spoof detection decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpoofDecision {
    /// Image appears to be a live face.
    Live,
    /// Possible presentation attack: reason included.
    Attack(String),
}

/// Parameters for the pose-consistency spoof detector.
#[derive(Debug, Clone)]
pub struct SpoofDetector {
    /// Maximum tolerated absolute difference in yaw (degrees) before flagging.
    pub max_yaw_diff: f32,
    /// Maximum tolerated absolute difference in pitch (degrees).
    pub max_pitch_diff: f32,
}

impl Default for SpoofDetector {
    fn default() -> Self {
        Self {
            max_yaw_diff: 20.0,
            max_pitch_diff: 20.0,
        }
    }
}

impl SpoofDetector {
    pub fn new(max_yaw_diff: f32, max_pitch_diff: f32) -> Self {
        Self {
            max_yaw_diff,
            max_pitch_diff,
        }
    }

    /// Compare the 3DMM-recovered pose with an expected sensor pose.
    ///
    /// If the discrepancy exceeds the thresholds, the image is flagged as a
    /// possible presentation attack (PA). This is a geometric PAD heuristic;
    /// a full PAD system would combine texture + depth + temporal cues.
    pub fn check(&self, recovered: &Pose3d, expected: &Pose3d) -> SpoofDecision {
        let yaw_diff = (recovered.yaw - expected.yaw).abs();
        let pitch_diff = (recovered.pitch - expected.pitch).abs();

        if yaw_diff > self.max_yaw_diff {
            return SpoofDecision::Attack(format!(
                "yaw discrepancy {:.1} deg (max {:.1} deg)",
                yaw_diff, self.max_yaw_diff,
            ));
        }
        if pitch_diff > self.max_pitch_diff {
            return SpoofDecision::Attack(format!(
                "pitch discrepancy {:.1} deg (max {:.1} deg)",
                pitch_diff, self.max_pitch_diff,
            ));
        }
        SpoofDecision::Live
    }

    /// Convenience: check using a frontal expected pose (yaw=0, pitch=0).
    pub fn check_frontal(&self, recovered: &Pose3d) -> SpoofDecision {
        let frontal = Pose3d::default();
        self.check(recovered, &frontal)
    }
}
