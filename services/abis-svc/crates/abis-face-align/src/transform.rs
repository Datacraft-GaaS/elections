//! Similarity transform: rotation + uniform scale + translation.
//!
//! Computes the least-squares similarity transform mapping a set of source
//! points onto target points using the Umeyama (1991) algorithm for the
//! 2-D case with 5 correspondences.
//!
//! Reference: Umeyama, S. (1991). "Least-squares estimation of transformation
//! parameters between two point patterns." IEEE TPAMI 13(4):376-380.

use crate::canonical::canonical_five;
use crate::landmarks::FiveLandmarks;
use thiserror::Error;

/// Alignment error.
#[derive(Debug, Error)]
pub enum AlignError {
    #[error("degenerate point configuration: scale is near zero")]
    DegenerateConfiguration,
    #[error("mismatched point counts")]
    MismatchedPoints,
}

/// A 2-D similarity transform: scale, rotation angle, translation (tx, ty).
/// Applies as: dst = scale * R(angle) * src + (tx, ty).
#[derive(Debug, Clone)]
pub struct SimilarityTransform {
    pub scale: f32,
    pub cos_a: f32,
    pub sin_a: f32,
    pub tx: f32,
    pub ty: f32,
}

impl SimilarityTransform {
    /// Compute the similarity transform from `src` 5 points to `dst` 5 points.
    ///
    /// Uses the closed-form Umeyama estimator for 2-D, 5 correspondences.
    pub fn from_five(src: &FiveLandmarks, dst: &FiveLandmarks) -> Result<Self, AlignError> {
        let src_arr = src.to_array();
        let dst_arr = dst.to_array();
        Self::fit(&src_arr, &dst_arr)
    }

    /// Compute the transform mapping detected `src` landmarks to the ArcFace
    /// canonical 112x112 template.
    pub fn to_canonical(src: &FiveLandmarks) -> Result<Self, AlignError> {
        Self::from_five(src, &canonical_five())
    }

    /// Fit a 2-D similarity transform from flat arrays of [x0,y0, x1,y1, ...]
    /// (10 floats = 5 points each).
    pub fn fit(src: &[f32], dst: &[f32]) -> Result<Self, AlignError> {
        if src.len() != dst.len() || src.len() % 2 != 0 {
            return Err(AlignError::MismatchedPoints);
        }
        let n = (src.len() / 2) as f32;

        // Centroid of source and destination.
        let (mu_sx, mu_sy) = src
            .chunks(2)
            .fold((0f32, 0f32), |acc, c| (acc.0 + c[0], acc.1 + c[1]));
        let (mu_dx, mu_dy) = dst
            .chunks(2)
            .fold((0f32, 0f32), |acc, c| (acc.0 + c[0], acc.1 + c[1]));
        let (mu_sx, mu_sy) = (mu_sx / n, mu_sy / n);
        let (mu_dx, mu_dy) = (mu_dx / n, mu_dy / n);

        // Variance of source (sigma^2).
        let var_s: f32 = src
            .chunks(2)
            .map(|c| {
                let dx = c[0] - mu_sx;
                let dy = c[1] - mu_sy;
                dx * dx + dy * dy
            })
            .sum::<f32>()
            / n;

        if var_s < 1e-9 {
            return Err(AlignError::DegenerateConfiguration);
        }

        // Cross-covariance matrix (2x2): S = (1/n) * sum (dst_i - mu_d)(src_i - mu_s)^T
        let mut s00 = 0f32;
        let mut s01 = 0f32;
        let mut s10 = 0f32;
        let mut s11 = 0f32;
        for (sc, dc) in src.chunks(2).zip(dst.chunks(2)) {
            let sx = sc[0] - mu_sx;
            let sy = sc[1] - mu_sy;
            let dx = dc[0] - mu_dx;
            let dy = dc[1] - mu_dy;
            s00 += dx * sx;
            s01 += dx * sy;
            s10 += dy * sx;
            s11 += dy * sy;
        }
        s00 /= n;
        s01 /= n;
        s10 /= n;
        s11 /= n;

        // For 2-D similarity, the rotation is extracted from the 2x2 cross-covariance.
        // R = [[cos, -sin], [sin, cos]] chosen to maximise trace(R^T S).
        // This reduces to atan2(s10 - s01, s00 + s11).
        let cos_a = s00 + s11;
        let sin_a = s10 - s01;
        let norm = (cos_a * cos_a + sin_a * sin_a).sqrt();
        let (cos_a, sin_a) = if norm > 1e-9 {
            (cos_a / norm, sin_a / norm)
        } else {
            (1.0f32, 0.0f32)
        };

        // Scale: c = (1 / var_s) * trace(R^T S).
        let scale = (cos_a * s00 + sin_a * s10 + cos_a * s11 - sin_a * s01) / var_s;

        // Translation: t = mu_d - scale * R * mu_s
        let tx = mu_dx - scale * (cos_a * mu_sx - sin_a * mu_sy);
        let ty = mu_dy - scale * (sin_a * mu_sx + cos_a * mu_sy);

        Ok(Self {
            scale,
            cos_a,
            sin_a,
            tx,
            ty,
        })
    }

    /// Apply transform to a single point.
    pub fn apply_point(&self, x: f32, y: f32) -> (f32, f32) {
        let rx = self.scale * (self.cos_a * x - self.sin_a * y) + self.tx;
        let ry = self.scale * (self.sin_a * x + self.cos_a * y) + self.ty;
        (rx, ry)
    }

    /// Inverse: map destination points back to source space.
    pub fn inverse(&self) -> Result<Self, AlignError> {
        let denom = self.scale * (self.cos_a * self.cos_a + self.sin_a * self.sin_a);
        if denom.abs() < 1e-9 {
            return Err(AlignError::DegenerateConfiguration);
        }
        let inv_scale = 1.0 / self.scale;
        let inv_cos = self.cos_a;
        let inv_sin = -self.sin_a;
        let inv_tx = -(inv_scale * (inv_cos * self.tx - inv_sin * self.ty));
        let inv_ty = -(inv_scale * (inv_sin * self.tx + inv_cos * self.ty));
        Ok(Self {
            scale: inv_scale,
            cos_a: inv_cos,
            sin_a: inv_sin,
            tx: inv_tx,
            ty: inv_ty,
        })
    }
}
