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

//! Pose estimation from pupil/limbus geometry.

use eel_abis_iris_segment::IrisSegmentation;
use serde::{Deserialize, Serialize};

/// Estimated gaze and tilt derived from pupil displacement within the limbus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoseEstimate {
    /// Horizontal gaze offset normalised by limbus radius (range approximately -1..1).
    pub gaze_x: f32,
    /// Vertical gaze offset normalised by limbus radius (range approximately -1..1).
    pub gaze_y: f32,
    /// Tilt angle in degrees, derived from atan2(gaze_y, gaze_x).
    pub tilt_deg: f32,
}

/// Estimates off-axis pose from iris segmentation geometry.
pub struct PoseCorrector;

impl PoseCorrector {
    /// Derive a pose estimate from the pupil/limbus displacement.
    ///
    /// `gaze_x` and `gaze_y` are normalised by limbus radius so they are
    /// sensor-independent. `tilt_deg` encodes the azimuthal direction of gaze.
    pub fn estimate(seg: &IrisSegmentation) -> PoseEstimate {
        let limbus_r = seg.limbus.radius;
        let gaze_x = if limbus_r > 0.0 {
            (seg.pupil.cx - seg.limbus.cx) / limbus_r
        } else {
            0.0
        };
        let gaze_y = if limbus_r > 0.0 {
            (seg.pupil.cy - seg.limbus.cy) / limbus_r
        } else {
            0.0
        };
        let tilt_deg = gaze_y.atan2(gaze_x).to_degrees();
        PoseEstimate {
            gaze_x,
            gaze_y,
            tilt_deg,
        }
    }

    /// Returns true when the off-axis angle is small enough to correct reliably.
    ///
    /// Correction is reliable when the normalised gaze offset is below 0.35 in
    /// both axes, corresponding to approximately +/-20 degrees.
    pub fn is_correctable(est: &PoseEstimate) -> bool {
        est.gaze_x.abs() < 0.35 && est.gaze_y.abs() < 0.35
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, IrisMask, IrisSegmentation};

    fn make_seg(px: f32, py: f32, lx: f32, ly: f32, lr: f32) -> IrisSegmentation {
        IrisSegmentation {
            pupil: Circle {
                cx: px,
                cy: py,
                radius: lr * 0.35,
            },
            limbus: Circle {
                cx: lx,
                cy: ly,
                radius: lr,
            },
            mask: IrisMask::all_valid(100, 100),
            iris_sclera_ratio: 0.5,
        }
    }

    #[test]
    fn frontal_gaze_zero() {
        let seg = make_seg(50.0, 50.0, 50.0, 50.0, 100.0);
        let est = PoseCorrector::estimate(&seg);
        assert_eq!(est.gaze_x, 0.0);
        assert_eq!(est.gaze_y, 0.0);
    }

    #[test]
    fn gaze_x_bounded() {
        // Pupil displaced 1.5x limbus radius -- gaze_x should be ~1.5, not clamped
        // but the test verifies it is finite and computable.
        let seg = make_seg(200.0, 50.0, 50.0, 50.0, 100.0);
        let est = PoseCorrector::estimate(&seg);
        assert!(est.gaze_x.is_finite());
        // Normalised offset equals displacement / radius
        assert!((est.gaze_x - 1.5).abs() < 1e-5);
    }

    #[test]
    fn gaze_y_bounded() {
        let seg = make_seg(50.0, 150.0, 50.0, 50.0, 100.0);
        let est = PoseCorrector::estimate(&seg);
        assert!(est.gaze_y.is_finite());
        assert!((est.gaze_y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn tilt_deg_finite() {
        let seg = make_seg(60.0, 55.0, 50.0, 50.0, 80.0);
        let est = PoseCorrector::estimate(&seg);
        assert!(est.tilt_deg.is_finite());
    }

    #[test]
    fn is_correctable_false_for_large_offset() {
        // Normalised offset 0.5 exceeds the 0.35 threshold.
        let seg = make_seg(100.0, 50.0, 50.0, 50.0, 100.0);
        let est = PoseCorrector::estimate(&seg);
        assert!((est.gaze_x - 0.5).abs() < 1e-5);
        assert!(!PoseCorrector::is_correctable(&est));
    }

    #[test]
    fn is_correctable_true_within_threshold() {
        let seg = make_seg(60.0, 50.0, 50.0, 50.0, 100.0);
        let est = PoseCorrector::estimate(&seg);
        // gaze_x = 0.1 < 0.35
        assert!(PoseCorrector::is_correctable(&est));
    }

    #[test]
    fn is_correctable_boundary_exactly_at_threshold() {
        // gaze_x = 0.35 is NOT < 0.35, so not correctable.
        let est = PoseEstimate {
            gaze_x: 0.35,
            gaze_y: 0.0,
            tilt_deg: 0.0,
        };
        assert!(!PoseCorrector::is_correctable(&est));
    }

    #[test]
    fn pose_estimate_serde_round_trip() {
        let est = PoseEstimate {
            gaze_x: 0.12,
            gaze_y: -0.08,
            tilt_deg: 33.7,
        };
        let json = serde_json::to_string(&est).expect("serialise");
        let back: PoseEstimate = serde_json::from_str(&json).expect("deserialise");
        assert!((back.gaze_x - est.gaze_x).abs() < 1e-6);
        assert!((back.gaze_y - est.gaze_y).abs() < 1e-6);
        assert!((back.tilt_deg - est.tilt_deg).abs() < 1e-4);
    }

    #[test]
    fn distinct_tilt_from_different_segs() {
        let seg_a = make_seg(60.0, 50.0, 50.0, 50.0, 100.0);
        let seg_b = make_seg(50.0, 60.0, 50.0, 50.0, 100.0);
        let est_a = PoseCorrector::estimate(&seg_a);
        let est_b = PoseCorrector::estimate(&seg_b);
        // One is displaced right, the other down: tilts must differ.
        assert!((est_a.tilt_deg - est_b.tilt_deg).abs() > 1.0);
    }

    #[test]
    fn zero_limbus_radius_does_not_panic() {
        let seg = IrisSegmentation {
            pupil: Circle {
                cx: 50.0,
                cy: 50.0,
                radius: 10.0,
            },
            limbus: Circle {
                cx: 50.0,
                cy: 50.0,
                radius: 0.0,
            },
            mask: IrisMask::all_valid(100, 100),
            iris_sclera_ratio: 0.0,
        };
        let est = PoseCorrector::estimate(&seg);
        assert_eq!(est.gaze_x, 0.0);
        assert_eq!(est.gaze_y, 0.0);
    }
}
