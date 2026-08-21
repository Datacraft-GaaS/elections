//! Landmark types: 2-D point, 5-point set, and 68-point dlib-style set.

use serde::{Deserialize, Serialize};

/// A single 2-D landmark in pixel coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Landmark {
    pub x: f32,
    pub y: f32,
}

impl Landmark {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Five canonical face landmarks:
/// left_eye, right_eye, nose, left_mouth, right_mouth.
///
/// Follows the RetinaFace / MTCNN 5-point convention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiveLandmarks {
    pub left_eye: Landmark,
    pub right_eye: Landmark,
    pub nose: Landmark,
    pub left_mouth: Landmark,
    pub right_mouth: Landmark,
}

impl FiveLandmarks {
    pub fn new(
        left_eye: (f32, f32),
        right_eye: (f32, f32),
        nose: (f32, f32),
        left_mouth: (f32, f32),
        right_mouth: (f32, f32),
    ) -> Self {
        Self {
            left_eye: Landmark::new(left_eye.0, left_eye.1),
            right_eye: Landmark::new(right_eye.0, right_eye.1),
            nose: Landmark::new(nose.0, nose.1),
            left_mouth: Landmark::new(left_mouth.0, left_mouth.1),
            right_mouth: Landmark::new(right_mouth.0, right_mouth.1),
        }
    }

    /// Convert to a flat [x, y, ...] array (10 floats).
    pub fn to_array(&self) -> [f32; 10] {
        [
            self.left_eye.x,
            self.left_eye.y,
            self.right_eye.x,
            self.right_eye.y,
            self.nose.x,
            self.nose.y,
            self.left_mouth.x,
            self.left_mouth.y,
            self.right_mouth.x,
            self.right_mouth.y,
        ]
    }
}

/// 68-point dlib-style landmarks (full face model).
/// Index convention follows the iBUG 300-W annotation scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landmarks68 {
    pub points: Vec<Landmark>,
}

impl Landmarks68 {
    /// Build from a flat slice of (x, y) pairs.
    pub fn from_pairs(pairs: &[(f32, f32)]) -> Option<Self> {
        if pairs.len() != 68 {
            return None;
        }
        Some(Self {
            points: pairs.iter().map(|&(x, y)| Landmark::new(x, y)).collect(),
        })
    }

    /// Extract the 5-point subset from the 68-point set.
    /// Indices: left eye centre ~ mean of points 36-41,
    ///          right eye centre ~ mean of points 42-47,
    ///          nose tip = point 30,
    ///          left mouth corner = point 48,
    ///          right mouth corner = point 54.
    ///
    /// Reference: Sagonas et al., "300 Faces In-The-Wild Challenge." ICCV 2013.
    pub fn to_five(&self) -> Option<FiveLandmarks> {
        if self.points.len() < 68 {
            return None;
        }
        let eye_mean = |range: std::ops::Range<usize>| -> Landmark {
            let n = (range.end - range.start) as f32;
            let sx: f32 = self.points[range.clone()].iter().map(|p| p.x).sum();
            let sy: f32 = self.points[range].iter().map(|p| p.y).sum();
            Landmark::new(sx / n, sy / n)
        };
        Some(FiveLandmarks {
            left_eye: eye_mean(36..42),
            right_eye: eye_mean(42..48),
            nose: self.points[30].clone(),
            left_mouth: self.points[48].clone(),
            right_mouth: self.points[54].clone(),
        })
    }
}
