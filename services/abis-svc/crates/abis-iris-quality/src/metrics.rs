//! Individual ISO/IEC 29794-6 quality metrics.
//!
//! Metrics:
//! - Sharpness: Laplacian variance on the iris annulus
//! - Contrast: histogram spread (90th - 10th percentile) on the iris annulus
//! - Dilation: pupil/iris radius ratio (ideal 0.3-0.5)
//! - Iris/sclera ratio: fraction of annulus visible (should be > 0.7)
//! - Gaze angle: angular offset of pupil from limbus centre (deg)
//! - Occlusion: fraction of iris annulus masked as occluded

use eel_abis_iris_segment::{Circle, GrayImage, IrisMask, IrisSegmentation};

/// Compute Laplacian variance (sharpness) over iris annulus pixels.
///
/// High Laplacian variance indicates a sharp, in-focus image.
pub fn sharpness(image: &GrayImage, seg: &IrisSegmentation) -> f32 {
    let values = annulus_pixels(image, seg);
    if values.len() < 9 {
        return 0.0;
    }

    // Approximate Laplacian with 3x3 kernel sum, using horizontal neighbours only
    // (simplified for efficiency on the normalised annulus).
    let w = image.width;
    let h = image.height;
    let mut lap_sum = 0.0f64;
    let mut count = 0u32;

    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            if !seg.mask.get(x, y) {
                continue;
            }
            if !seg.limbus.contains(x as f32, y as f32) || seg.pupil.contains(x as f32, y as f32) {
                continue;
            }
            let center = image.get(x, y) as f64;
            let neighbors = image.get(x + 1, y) as f64
                + image.get(x - 1, y) as f64
                + image.get(x, y + 1) as f64
                + image.get(x, y - 1) as f64;
            let lap = (4.0 * center - neighbors).abs();
            lap_sum += lap;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }

    // Normalise to [0, 1] by dividing by max possible Laplacian (4 * 255 = 1020).
    ((lap_sum / count as f64) / 1020.0).min(1.0) as f32
}

/// Contrast: (90th percentile - 10th percentile) / 255 of annulus pixel values.
pub fn contrast(image: &GrayImage, seg: &IrisSegmentation) -> f32 {
    let mut values = annulus_pixels(image, seg);
    if values.is_empty() {
        return 0.0;
    }
    values.sort_unstable();
    let p10 = values[values.len() / 10] as f32;
    let p90 = values[values.len() * 9 / 10] as f32;
    ((p90 - p10) / 255.0).clamp(0.0, 1.0)
}

/// Dilation: pupil radius / limbus radius.
/// Ideal range is 0.3-0.5. Values outside [0.1, 0.8] indicate abnormal dilation.
pub fn dilation(seg: &IrisSegmentation) -> f32 {
    if seg.limbus.radius < 1.0 {
        return 0.0;
    }
    (seg.pupil.radius / seg.limbus.radius).clamp(0.0, 1.0)
}

/// Gaze angle: angular offset of pupil centre from limbus centre (degrees).
/// Frontal gaze = 0 degrees; values > 15 degrees indicate off-axis gaze.
pub fn gaze_angle(seg: &IrisSegmentation) -> f32 {
    let dx = seg.pupil.cx - seg.limbus.cx;
    let dy = seg.pupil.cy - seg.limbus.cy;
    let dist = (dx * dx + dy * dy).sqrt();
    // Normalise by limbus radius, then convert to approximate angle.
    if seg.limbus.radius < 1.0 {
        return 0.0;
    }
    let ratio = (dist / seg.limbus.radius).clamp(0.0, 1.0);
    // asin(ratio) gives angle in radians; convert to degrees.
    ratio.asin().to_degrees()
}

/// Occlusion percentage: fraction of iris annulus pixels marked as occluded.
pub fn occlusion_pct(seg: &IrisSegmentation) -> f32 {
    seg.mask.occlusion_fraction()
}

/// Collect pixel values from the iris annulus (inside limbus, outside pupil, mask valid).
fn annulus_pixels(image: &GrayImage, seg: &IrisSegmentation) -> Vec<u8> {
    let mut values = Vec::new();
    for y in 0..image.height {
        for x in 0..image.width {
            if seg.limbus.contains(x as f32, y as f32)
                && !seg.pupil.contains(x as f32, y as f32)
                && seg.mask.get(x, y)
            {
                values.push(image.get(x, y));
            }
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, GrayImage, IrisMask, IrisSegmentation};

    fn make_seg(pr: f32, lr: f32, w: u32, h: u32) -> (GrayImage, IrisSegmentation) {
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let pupil = Circle::new(cx, cy, pr);
        let limbus = Circle::new(cx, cy, lr);
        let mask = IrisMask::all_valid(w, h);
        let data: Vec<u8> = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < pr {
                        30
                    } else if r < lr {
                        150
                    } else {
                        220
                    }
                })
            })
            .collect();
        let image = GrayImage::from_raw(w, h, data);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.9,
        };
        (image, seg)
    }

    #[test]
    fn sharpness_in_range() {
        let (img, seg) = make_seg(25.0, 70.0, 200, 200);
        let s = sharpness(&img, &seg);
        assert!((0.0..=1.0).contains(&s), "sharpness {s}");
    }

    #[test]
    fn contrast_in_range() {
        let (img, seg) = make_seg(25.0, 70.0, 200, 200);
        let c = contrast(&img, &seg);
        assert!((0.0..=1.0).contains(&c), "contrast {c}");
    }

    #[test]
    fn dilation_ideal_range() {
        let (_, seg) = make_seg(25.0, 70.0, 200, 200);
        let d = dilation(&seg);
        assert!((0.0..=1.0).contains(&d), "dilation {d}");
        // 25/70 ~ 0.357, should be in ideal range.
        assert!(d > 0.2 && d < 0.6, "dilation {d} not in ideal range");
    }

    #[test]
    fn gaze_angle_frontal_is_zero() {
        let (_, seg) = make_seg(25.0, 70.0, 200, 200);
        let g = gaze_angle(&seg);
        assert!(g.abs() < 1.0, "frontal gaze should be ~0 degrees, got {g}");
    }

    #[test]
    fn gaze_angle_off_axis() {
        let pupil = Circle::new(110.0, 100.0, 25.0); // 10px offset
        let limbus = Circle::new(100.0, 100.0, 70.0);
        let mask = IrisMask::all_valid(200, 200);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.9,
        };
        let g = gaze_angle(&seg);
        assert!(g > 0.0, "off-axis gaze should be > 0 degrees, got {g}");
    }

    #[test]
    fn occlusion_zero_with_full_mask() {
        let (_, seg) = make_seg(25.0, 70.0, 200, 200);
        let o = occlusion_pct(&seg);
        assert!(o < 0.01, "full mask should have ~0 occlusion, got {o}");
    }

    #[test]
    fn occlusion_one_with_zero_mask() {
        let pupil = Circle::new(100.0, 100.0, 25.0);
        let limbus = Circle::new(100.0, 100.0, 70.0);
        let mask = IrisMask::zeroed(200, 200);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.9,
        };
        let o = occlusion_pct(&seg);
        assert!(
            (o - 1.0).abs() < 0.01,
            "zero mask should have 100% occlusion, got {o}"
        );
    }

    #[test]
    fn sharpness_higher_for_high_contrast_edges() {
        // Create image with sharp edges (black/white border at iris boundary).
        let w = 200u32;
        let h = 200u32;
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let pr = 25.0f32;
        let lr = 70.0f32;
        let data_sharp: Vec<u8> = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < pr {
                        0
                    } else if r < lr {
                        255
                    } else {
                        0
                    }
                })
            })
            .collect();
        let data_flat = vec![128u8; (w * h) as usize];
        let img_sharp = GrayImage::from_raw(w, h, data_sharp);
        let img_flat = GrayImage::from_raw(w, h, data_flat);
        let pupil = Circle::new(cx, cy, pr);
        let limbus = Circle::new(cx, cy, lr);
        let mask = IrisMask::all_valid(w, h);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask: mask.clone(),
            iris_sclera_ratio: 0.9,
        };
        let seg2 = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.9,
        };
        let s_sharp = sharpness(&img_sharp, &seg);
        let s_flat = sharpness(&img_flat, &seg2);
        assert!(
            s_sharp > s_flat,
            "sharp image should have higher sharpness: {s_sharp} vs {s_flat}"
        );
    }
}
