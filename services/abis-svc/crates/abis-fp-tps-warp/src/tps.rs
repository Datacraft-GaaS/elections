//! Thin-Plate Spline fitting and evaluation.
//!
//! Reference: F. L. Bookstein, "Principal warps," IEEE TPAMI 1989.
//!
//! Given N source control points {p_i} and N target points {q_i}, the TPS
//! finds coefficients {w_i} and affine coefficients {a} such that:
//!
//!   f(x,y) = a0 + a1*x + a2*y + sum_i w_i * U(||p - p_i||)
//!
//! where U(r) = r^2 * log(r^2) (the TPS radial basis function).
//!
//! The system is solved by building the K matrix (N+3 x N+3) and solving
//! a linear system via Gaussian elimination.

use crate::{Point, TpsError};

/// The TPS kernel U(r) = r^2 * ln(r^2), with U(0)=0.
fn tps_kernel(r2: f64) -> f64 {
    if r2 < f64::EPSILON {
        0.0
    } else {
        r2 * r2.ln()
    }
}

/// A fitted TPS warp from N source control points to N target control points.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TpsWarp {
    /// Source control points.
    pub control_points: Vec<Point>,
    /// TPS radial basis weights (length N), one per control point, for each output axis.
    pub weights_x: Vec<f64>,
    pub weights_y: Vec<f64>,
    /// Affine coefficients [a0, a1, a2] for x and y output.
    pub affine_x: [f64; 3],
    pub affine_y: [f64; 3],
}

impl TpsWarp {
    /// Fit a TPS from `source` to `target` control points.
    ///
    /// Requires at least 3 control points.
    pub fn fit(source: &[Point], target: &[Point]) -> Result<Self, TpsError> {
        let n = source.len();
        if n < 3 {
            return Err(TpsError::TooFewPoints(n));
        }
        if target.len() != n {
            return Err(TpsError::PointCountMismatch {
                src: n,
                target: target.len(),
            });
        }

        let m = n + 3; // system size

        // Build K matrix (top-left N x N block = RBF).
        let mut k = vec![0.0f64; m * m];
        for i in 0..n {
            for j in 0..n {
                let dx = source[i].x - source[j].x;
                let dy = source[i].y - source[j].y;
                let r2 = dx * dx + dy * dy;
                k[i * m + j] = tps_kernel(r2);
            }
        }
        // P block: top-right N x 3 and its transpose.
        for i in 0..n {
            k[i * m + n] = 1.0;
            k[i * m + n + 1] = source[i].x;
            k[i * m + n + 2] = source[i].y;
            k[n * m + i] = 1.0;
            k[(n + 1) * m + i] = source[i].x;
            k[(n + 2) * m + i] = source[i].y;
        }
        // Bottom-right 3x3 block stays zero.

        // RHS vectors.
        let mut rhs_x = vec![0.0f64; m];
        let mut rhs_y = vec![0.0f64; m];
        for i in 0..n {
            rhs_x[i] = target[i].x;
            rhs_y[i] = target[i].y;
        }

        // Solve K * c = rhs via Gaussian elimination with partial pivoting.
        let cx = gaussian_solve(&mut k.clone(), &mut rhs_x, m).ok_or(TpsError::SingularSystem)?;
        let k_copy = {
            let mut k2 = vec![0.0f64; m * m];
            for i in 0..n {
                for j in 0..n {
                    let dx = source[i].x - source[j].x;
                    let dy = source[i].y - source[j].y;
                    let r2 = dx * dx + dy * dy;
                    k2[i * m + j] = tps_kernel(r2);
                }
                k2[i * m + n] = 1.0;
                k2[i * m + n + 1] = source[i].x;
                k2[i * m + n + 2] = source[i].y;
                k2[n * m + i] = 1.0;
                k2[(n + 1) * m + i] = source[i].x;
                k2[(n + 2) * m + i] = source[i].y;
            }
            k2
        };
        let cy =
            gaussian_solve(&mut k_copy.clone(), &mut rhs_y, m).ok_or(TpsError::SingularSystem)?;

        Ok(TpsWarp {
            control_points: source.to_vec(),
            weights_x: cx[..n].to_vec(),
            weights_y: cy[..n].to_vec(),
            affine_x: [cx[n], cx[n + 1], cx[n + 2]],
            affine_y: [cy[n], cy[n + 1], cy[n + 2]],
        })
    }

    /// Apply the fitted TPS to a single point.
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let n = self.control_points.len();
        let mut wx = self.affine_x[0] + self.affine_x[1] * x + self.affine_x[2] * y;
        let mut wy = self.affine_y[0] + self.affine_y[1] * x + self.affine_y[2] * y;
        for i in 0..n {
            let dx = x - self.control_points[i].x;
            let dy = y - self.control_points[i].y;
            let r2 = dx * dx + dy * dy;
            let u = tps_kernel(r2);
            wx += self.weights_x[i] * u;
            wy += self.weights_y[i] * u;
        }
        (wx, wy)
    }

    /// Create an identity TPS (apply returns the input unchanged).
    ///
    /// Uses a triangle of control points at infinity-equivalent positions.
    pub fn identity() -> Result<Self, TpsError> {
        let src = vec![
            Point::new(0.0, 0.0),
            Point::new(1000.0, 0.0),
            Point::new(500.0, 1000.0),
        ];
        Self::fit(&src, &src)
    }
}

/// Gaussian elimination with partial pivoting.  Returns solution vector or None if singular.
fn gaussian_solve(a: &mut [f64], b: &mut [f64], n: usize) -> Option<Vec<f64>> {
    for col in 0..n {
        // Partial pivot.
        let mut max_row = col;
        let mut max_val = a[col * n + col].abs();
        for row in (col + 1)..n {
            let v = a[row * n + col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None; // singular
        }
        if max_row != col {
            for j in 0..n {
                a.swap(col * n + j, max_row * n + j);
            }
            b.swap(col, max_row);
        }
        let pivot = a[col * n + col];
        for row in (col + 1)..n {
            let factor = a[row * n + col] / pivot;
            for j in col..n {
                let v = a[col * n + j] * factor;
                a[row * n + j] -= v;
            }
            let bv = b[col] * factor;
            b[row] -= bv;
        }
    }
    // Back substitution.
    let mut x = vec![0.0f64; n];
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i * n + j] * x[j];
        }
        if a[i * n + i].abs() < 1e-12 {
            return None;
        }
        x[i] = sum / a[i * n + i];
    }
    Some(x)
}
