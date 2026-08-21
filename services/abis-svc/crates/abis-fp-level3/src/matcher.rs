//! Level-3 matcher: compares pore constellations, ridge edge shapes, incipient ridges.
//!
//! Reference: Jain, Chen, Demirkus, IEEE TPAMI 2007, Section IV.
//!
//! Matching algorithm:
//!   1. For each pair of corresponding minutiae (assumed pre-aligned), find the
//!      K nearest pores around each minutia.
//!   2. Compute Procrustes-like alignment on the K-nearest pore constellations.
//!   3. Score = mean pore-pair cosine similarity after alignment.
//!   4. Ridge-edge shape correlation adds a weighted bonus.
//!   5. Incipient ridge overlap adds a bonus.

use crate::pores::Pore;
use crate::quality::{Level3Quality, Level3QualityError};
use crate::ridge_edge::EdgePoint;

/// Number of nearest pores per minutia for constellation matching.
const K_NEAREST_PORES: usize = 5;
/// Positional tolerance for pore matching after alignment (pixels).
const PORE_MATCH_DIST_PX: f32 = 4.0;

/// Level-3 feature set for one fingerprint impression.
#[derive(Debug, Clone)]
pub struct Level3Features {
    pub pores: Vec<Pore>,
    pub edge_points: Vec<EdgePoint>,
    pub quality: Level3Quality,
}

/// Level-3 matcher.
pub struct Level3Matcher;

impl Level3Matcher {
    pub fn new() -> Self {
        Self
    }

    /// Compare pore constellations near corresponding minutia positions.
    ///
    /// `probe_minutiae` and `gallery_minutiae` are (x, y) positions.
    /// Returns an error if either image has insufficient resolution.
    pub fn compare(
        &self,
        probe: &Level3Features,
        gallery: &Level3Features,
        probe_minutiae: &[(f32, f32)],
        gallery_minutiae: &[(f32, f32)],
    ) -> Result<f32, Level3QualityError> {
        probe.quality.check_resolution()?;
        gallery.quality.check_resolution()?;

        if probe_minutiae.is_empty() || gallery_minutiae.is_empty() {
            return Ok(0.0);
        }

        let n_pairs = probe_minutiae.len().min(gallery_minutiae.len());
        let mut total_sim = 0.0f32;
        let mut pair_count = 0u32;

        for i in 0..n_pairs {
            let (px, py) = probe_minutiae[i];
            let (gx, gy) = gallery_minutiae[i];

            let probe_pores = k_nearest_pores(&probe.pores, px, py, K_NEAREST_PORES);
            let gallery_pores = k_nearest_pores(&gallery.pores, gx, gy, K_NEAREST_PORES);

            if probe_pores.is_empty() || gallery_pores.is_empty() {
                continue;
            }

            let sim = pore_constellation_similarity(&probe_pores, &gallery_pores);
            total_sim += sim;
            pair_count += 1;
        }

        if pair_count == 0 {
            return Ok(0.0);
        }

        let base_sim = total_sim / pair_count as f32;
        let edge_bonus = edge_shape_similarity(&probe.edge_points, &gallery.edge_points) * 0.15;
        Ok((base_sim + edge_bonus).clamp(0.0, 1.0))
    }
}

impl Default for Level3Matcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the K pores nearest to (cx, cy), with their positions relative to (cx, cy).
fn k_nearest_pores(pores: &[Pore], cx: f32, cy: f32, k: usize) -> Vec<(f32, f32)> {
    let mut with_dist: Vec<(f32, (f32, f32))> = pores
        .iter()
        .map(|p| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            (dx * dx + dy * dy, (dx, dy))
        })
        .collect();
    with_dist.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    with_dist.truncate(k);
    with_dist.into_iter().map(|(_, pos)| pos).collect()
}

/// Procrustes-inspired pore constellation similarity.
///
/// Aligns two sets of relative pore positions using centroid translation and
/// scale normalization, then measures fraction of matched pores.
fn pore_constellation_similarity(probe: &[(f32, f32)], gallery: &[(f32, f32)]) -> f32 {
    if probe.is_empty() || gallery.is_empty() {
        return 0.0;
    }

    // Center both constellations.
    let (pcx, pcy) = centroid(probe);
    let (gcx, gcy) = centroid(gallery);

    let probe_c: Vec<(f32, f32)> = probe.iter().map(|(x, y)| (x - pcx, y - pcy)).collect();
    let gallery_c: Vec<(f32, f32)> = gallery.iter().map(|(x, y)| (x - gcx, y - gcy)).collect();

    // Scale by RMS distance from centroid.
    let p_scale = rms_dist(&probe_c);
    let g_scale = rms_dist(&gallery_c);
    if p_scale < 1e-6 || g_scale < 1e-6 {
        return 0.0;
    }

    let probe_n: Vec<(f32, f32)> = probe_c
        .iter()
        .map(|(x, y)| (x / p_scale, y / g_scale))
        .collect();
    let gallery_n: Vec<(f32, f32)> = gallery_c
        .iter()
        .map(|(x, y)| (x / g_scale, y / g_scale))
        .collect();

    // Count matched pairs within scaled tolerance.
    let tol = PORE_MATCH_DIST_PX / p_scale.min(g_scale);
    let mut matched = 0usize;
    for pp in &probe_n {
        for gp in &gallery_n {
            let dx = pp.0 - gp.0;
            let dy = pp.1 - gp.1;
            if dx * dx + dy * dy <= tol * tol {
                matched += 1;
                break;
            }
        }
    }

    let max_possible = probe_n.len().max(gallery_n.len());
    matched as f32 / max_possible as f32
}

fn centroid(pts: &[(f32, f32)]) -> (f32, f32) {
    let n = pts.len() as f32;
    let sx: f32 = pts.iter().map(|(x, _)| x).sum();
    let sy: f32 = pts.iter().map(|(_, y)| y).sum();
    (sx / n, sy / n)
}

fn rms_dist(pts: &[(f32, f32)]) -> f32 {
    let sum: f32 = pts.iter().map(|(x, y)| x * x + y * y).sum();
    (sum / pts.len() as f32).sqrt()
}

/// Ridge edge shape correlation: mean absolute curvature difference.
fn edge_shape_similarity(probe: &[EdgePoint], gallery: &[EdgePoint]) -> f32 {
    if probe.is_empty() || gallery.is_empty() {
        return 0.0;
    }
    let n = probe.len().min(gallery.len());
    let diff: f32 = probe[..n]
        .iter()
        .zip(gallery[..n].iter())
        .map(|(p, g)| (p.curvature - g.curvature).abs())
        .sum::<f32>()
        / n as f32;
    (1.0 - diff).clamp(0.0, 1.0)
}
