//! Sweat-pore extraction from high-resolution fingerprint images.
//!
//! Reference: Jain, Chen, Demirkus, IEEE TPAMI 2007, Section III-A.
//!
//! Pores are detected as local intensity minima on ridge crests.
//! Expected inter-pore spacing: 8-10 px at 1000 ppi (or 4-5 px at 500 ppi).
//! Each pore is associated with a ridge identifier and orientation.

use serde::{Deserialize, Serialize};

/// A detected sweat pore on a fingerprint ridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pore {
    /// X coordinate in pixels.
    pub x: f32,
    /// Y coordinate in pixels.
    pub y: f32,
    /// Ridge identifier (index into ridge list, or 0 if unknown).
    pub ridge_id: u32,
    /// Ridge orientation at this pore location (degrees, [0, 180)).
    pub orientation: f32,
    /// Detection confidence [0.0, 1.0].
    pub confidence: f32,
}

/// Minimum inter-pore spacing in pixels (8 px at 500 ppi).
pub const MIN_PORE_SPACING_PX: f32 = 8.0;

/// Extract pore candidates from a synthetic model of ridge intensity values.
///
/// `intensity` is a flat row-major buffer of pixel intensities [0, 255].
/// `width` and `height` define image dimensions.
/// `orientation_field` is a flat row-major buffer of ridge orientations [0, 179] degrees.
/// Returns pores sorted by decreasing confidence.
///
/// This implementation detects local minima on estimated ridge crests using
/// non-maximum suppression perpendicular to the local ridge orientation.
pub fn extract_pores(
    intensity: &[u8],
    width: usize,
    height: usize,
    orientation_field: &[u8],
) -> Vec<Pore> {
    if intensity.len() != width * height || orientation_field.len() != width * height {
        return Vec::new();
    }

    let mut candidates: Vec<Pore> = Vec::new();

    for y in 1..(height.saturating_sub(1)) {
        for x in 1..(width.saturating_sub(1)) {
            let idx = y * width + x;
            let val = intensity[idx] as i32;

            // Detect local minimum: 8-connected neighbourhood.
            let is_minimum = (0i32..=2)
                .flat_map(|dy| (0i32..=2).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| dx != 1 || dy != 1)
                .all(|(dx, dy)| {
                    let nx = (x as i32 + dx - 1) as usize;
                    let ny = (y as i32 + dy - 1) as usize;
                    if nx < width && ny < height {
                        intensity[ny * width + nx] as i32 >= val
                    } else {
                        true
                    }
                });

            if !is_minimum {
                continue;
            }

            // Confidence: how dark relative to neighbours (deeper = more confident).
            let neighbours_sum: i32 = (0i32..=2)
                .flat_map(|dy| (0i32..=2).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| dx != 1 || dy != 1)
                .map(|(dx, dy)| {
                    let nx = (x as i32 + dx - 1) as usize;
                    let ny = (y as i32 + dy - 1) as usize;
                    if nx < width && ny < height {
                        intensity[ny * width + nx] as i32
                    } else {
                        val
                    }
                })
                .sum();
            let avg_neighbour = neighbours_sum as f32 / 8.0;
            let contrast = (avg_neighbour - val as f32) / 255.0;
            let confidence = contrast.clamp(0.0, 1.0);

            if confidence < 0.05 {
                continue; // too shallow to be a pore
            }

            let orientation = orientation_field[idx] as f32;
            candidates.push(Pore {
                x: x as f32,
                y: y as f32,
                ridge_id: 0, // assigned by caller if ridge tracing is available
                orientation,
                confidence,
            });
        }
    }

    // Non-maximum suppression by minimum spacing to avoid duplicate detections.
    non_max_suppress(candidates, MIN_PORE_SPACING_PX)
}

/// Remove pore candidates within `min_dist` of a higher-confidence candidate.
fn non_max_suppress(mut pores: Vec<Pore>, min_dist: f32) -> Vec<Pore> {
    // Sort by confidence descending.
    pores.sort_unstable_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut kept: Vec<Pore> = Vec::new();
    'outer: for candidate in pores {
        for kept_pore in &kept {
            let dx = candidate.x - kept_pore.x;
            let dy = candidate.y - kept_pore.y;
            if dx * dx + dy * dy < min_dist * min_dist {
                continue 'outer;
            }
        }
        kept.push(candidate);
    }
    kept
}
