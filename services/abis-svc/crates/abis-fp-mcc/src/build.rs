//! Build MCC cylinders from a fingerprint template.
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010, Section III-A, eq. (1)-(5).

use std::f32::consts::PI;

use abis_matcher::modality::fingerprint::{FingerprintTemplate, Minutia};

use crate::cylinder::MinutiaCylinder;
use crate::params::MccParams;

/// Build one cylinder per minutia in `template`.
///
/// Cylinders that do not meet the validity threshold `params.tau_psi` are
/// included but flagged `valid = false`; callers may skip them during
/// global-similarity consolidation.
pub fn build_mcc(template: &FingerprintTemplate, params: &MccParams) -> Vec<MinutiaCylinder> {
    let ns = params.ns as usize;
    let nd = params.nd as usize;
    let cell_count = ns * ns * nd;
    let byte_count = (cell_count + 7) / 8;
    let cell_size = (2.0 * params.radius) / ns as f32;

    template
        .minutiae
        .iter()
        .enumerate()
        .map(|(i, mi)| {
            build_cylinder(
                i,
                mi,
                &template.minutiae,
                params,
                ns,
                nd,
                cell_count,
                byte_count,
                cell_size,
            )
        })
        .collect()
}

fn build_cylinder(
    minutia_ref: usize,
    mi: &Minutia,
    minutiae: &[Minutia],
    params: &MccParams,
    ns: usize,
    nd: usize,
    cell_count: usize,
    byte_count: usize,
    cell_size: f32,
) -> MinutiaCylinder {
    let mut cells = vec![0u8; byte_count];
    let mut popcount = 0u32;

    // Precompute directional bin centers (in radians).
    // phi_j = 2*PI*(j + 0.5)/Nd as per Cappelli 2010 eq. (2).
    let dir_centers: Vec<f32> = (0..nd)
        .map(|j| 2.0 * PI * (j as f32 + 0.5) / nd as f32)
        .collect();

    let theta_i = (mi.theta * PI) / 180.0; // convert to radians
    let cos_i = theta_i.cos();
    let sin_i = theta_i.sin();

    // Half-grid offset so that cells are symmetric around the minutia.
    let half = ns as f32 / 2.0;

    for k in 0..ns {
        for l in 0..ns {
            // Cell center in the rotated frame of minutia mi.
            let ck = (k as f32 - half + 0.5) * cell_size;
            let cl = (l as f32 - half + 0.5) * cell_size;

            // Rotate to image coordinates.
            let px = mi.x as f32 + ck * cos_i - cl * sin_i;
            let py = mi.y as f32 + ck * sin_i + cl * cos_i;

            for j in 0..nd {
                let phi_j = dir_centers[j];
                let contrib = sum_contributions(minutiae, mi, px, py, phi_j, params);

                if contrib >= params.mu_psi {
                    let idx = (k * ns + l) * nd + j;
                    let byte = idx / 8;
                    let bit = idx % 8;
                    cells[byte] |= 1 << bit;
                    popcount += 1;
                }
            }
        }
    }

    // Validity: fraction of set bits relative to total spatial cells.
    let total_spatial = ns * ns;
    let valid = total_spatial == 0 || (popcount as f32 / cell_count as f32) >= params.tau_psi;

    MinutiaCylinder {
        minutia_ref,
        radius: params.radius,
        ns: params.ns,
        nd: params.nd,
        cells,
        popcount,
        valid,
    }
}

/// Accumulated contribution to cell (px, py, phi_j) from all minutiae other than mi.
///
/// Cappelli 2010 eq. (3): C_{m_i}(k,l,j) = psi( sum_{m_t in T\{m_i}} G_s * G_d )
fn sum_contributions(
    minutiae: &[Minutia],
    mi: &Minutia,
    px: f32,
    py: f32,
    phi_j: f32,
    params: &MccParams,
) -> f32 {
    let radius_sq = params.radius * params.radius;
    let mut acc = 0.0f32;

    for mt in minutiae {
        // Skip the reference minutia itself.
        if std::ptr::eq(mi as *const _, mt as *const _) {
            continue;
        }
        let dx = mt.x as f32 - px;
        let dy = mt.y as f32 - py;
        let dist_sq = dx * dx + dy * dy;

        // Only consider neighbours within the cylinder radius.
        if dist_sq > radius_sq {
            continue;
        }

        let g_s = gaussian(dist_sq.sqrt(), params.sigma_s);

        let theta_t = (mt.theta * std::f32::consts::PI) / 180.0;
        let angle_diff = circular_diff(theta_t, phi_j);
        let g_d = gaussian(angle_diff, params.sigma_d);

        acc += g_s * g_d;
    }
    acc
}

/// Isotropic Gaussian: exp(-x^2 / (2*sigma^2)).
#[inline]
fn gaussian(x: f32, sigma: f32) -> f32 {
    let s2 = 2.0 * sigma * sigma;
    if s2 <= 0.0 {
        return 0.0;
    }
    (-(x * x) / s2).exp()
}

/// Minimum circular distance between two angles (both in radians).
#[inline]
fn circular_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(2.0 * std::f32::consts::PI);
    if d > std::f32::consts::PI {
        2.0 * std::f32::consts::PI - d
    } else {
        d
    }
}
