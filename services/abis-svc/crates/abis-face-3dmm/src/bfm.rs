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

//! Basel Face Model parameter types and the Model3dmm trait.
//!
//! BFM parameter shape:
//! - 85 PCA shape coefficients (identity component)
//! - 64 expression PCA coefficients (FACS-like blendshapes)
//! - 3 Euler angles for pose (pitch, yaw, roll) in degrees
//! - 3 translation parameters (tx, ty, tz) in normalised face space
//!
//! Reference: Paysan et al. (2009). AVSS 2009.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 3DMM fitting error.
#[derive(Debug, Error)]
pub enum DmmError {
    #[error("insufficient landmarks: need at least 5, got {0}")]
    TooFewLandmarks(usize),
    #[error("image dimensions too small")]
    ImageTooSmall,
    #[error("model fitting diverged")]
    FitDiverged,
}

/// 3-D landmark in model space.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Landmark {
    pub x: f32,
    pub y: f32,
}

impl Landmark {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Camera projection parameters (weak-perspective approximation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera3d {
    /// Scale factor (focal length proxy).
    pub scale: f32,
    /// Translation in image coordinates (pixels).
    pub tx: f32,
    pub ty: f32,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            scale: 1.0,
            tx: 56.0,
            ty: 56.0,
        }
    }
}

/// Head pose in 3-D (Euler angles, degrees, ZYX convention).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose3d {
    /// Pitch (up/down) in degrees.
    pub pitch: f32,
    /// Yaw (left/right) in degrees.
    pub yaw: f32,
    /// Roll (tilt) in degrees.
    pub roll: f32,
}

impl Default for Pose3d {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }
    }
}

impl Pose3d {
    /// Rotate the pose to a frontal view (zero yaw and pitch, preserve roll).
    pub fn frontal(&self) -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            roll: self.roll,
        }
    }

    /// Returns true when the pose is within the given degree tolerances.
    pub fn is_near_frontal(&self, max_deg: f32) -> bool {
        self.pitch.abs() < max_deg && self.yaw.abs() < max_deg
    }
}

/// Result of fitting a 3DMM to a face image.
///
/// Coefficients follow the BFM2017 convention:
/// - 80 shape PCA components + 5 texture PCA components (simplified to 85 total)
/// - 64 expression components
///
/// Reference: Deng et al. (2019). CVPRW 2019.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fit3dmm {
    /// Shape identity PCA coefficients (85-d).
    pub shape_coeffs: Vec<f32>,
    /// Expression PCA coefficients (64-d).
    pub expr_coeffs: Vec<f32>,
    /// Recovered head pose.
    pub pose: Pose3d,
    /// Camera/projection parameters.
    pub camera: Camera3d,
}

impl Fit3dmm {
    /// Construct with explicit coefficients.
    pub fn new(
        shape_coeffs: Vec<f32>,
        expr_coeffs: Vec<f32>,
        pose: Pose3d,
        camera: Camera3d,
    ) -> Self {
        Self {
            shape_coeffs,
            expr_coeffs,
            pose,
            camera,
        }
    }

    /// Build a neutral/identity fit (all coefficients zero, frontal pose).
    pub fn identity() -> Self {
        Self {
            shape_coeffs: vec![0.0; 85],
            expr_coeffs: vec![0.0; 64],
            pose: Pose3d::default(),
            camera: Camera3d::default(),
        }
    }

    /// Returns a copy of this fit with the pose set to frontal.
    pub fn to_frontal(&self) -> Self {
        Self {
            pose: self.pose.frontal(),
            ..self.clone()
        }
    }
}

/// Trait for 3D Morphable Model implementations.
pub trait Model3dmm {
    /// Fit the model to an image given detected 2-D landmarks.
    fn fit(
        &self,
        image_pixels: &[u8],
        image_width: u32,
        image_height: u32,
        landmarks: &[Landmark],
    ) -> Result<Fit3dmm, DmmError>;

    /// Render the fit from a given view pose, returning RGB pixel buffer
    /// (width x height x 3).
    fn render(&self, fit: &Fit3dmm, view: &Pose3d, width: u32, height: u32) -> Vec<u8>;
}

/// Deterministic stub implementation for tests (no real model data required).
///
/// Fit extracts pose from the landmark centroid and roll from the inter-eye
/// line; all coefficients are zero.
#[derive(Debug, Clone, Default)]
pub struct StubModel3dmm;

impl StubModel3dmm {
    pub fn new() -> Self {
        Self
    }
}

impl Model3dmm for StubModel3dmm {
    fn fit(
        &self,
        _image_pixels: &[u8],
        _image_width: u32,
        _image_height: u32,
        landmarks: &[Landmark],
    ) -> Result<Fit3dmm, DmmError> {
        if landmarks.len() < 5 {
            return Err(DmmError::TooFewLandmarks(landmarks.len()));
        }
        // Estimate roll from the first two landmark pair (eye surrogates).
        let dx = landmarks[1].x - landmarks[0].x;
        let dy = landmarks[1].y - landmarks[0].y;
        let roll = dy.atan2(dx).to_degrees();

        // Estimate yaw from horizontal centroid offset vs. image centre.
        let cx: f32 = landmarks.iter().map(|l| l.x).sum::<f32>() / landmarks.len() as f32;
        let cy: f32 = landmarks.iter().map(|l| l.y).sum::<f32>() / landmarks.len() as f32;
        let yaw = (cx - 56.0) * 0.5; // heuristic
        let pitch = (cy - 56.0) * 0.3;

        Ok(Fit3dmm {
            shape_coeffs: vec![0.0; 85],
            expr_coeffs: vec![0.0; 64],
            pose: Pose3d { pitch, yaw, roll },
            camera: Camera3d {
                scale: 1.0,
                tx: cx,
                ty: cy,
            },
        })
    }

    fn render(&self, _fit: &Fit3dmm, _view: &Pose3d, width: u32, height: u32) -> Vec<u8> {
        // Stub: render a uniform grey image.
        vec![128u8; (width * height * 3) as usize]
    }
}
