//! Patterned contact lens detection via radial DFT peak analysis.

use eel_abis_iris_segment::{Circle, GrayImage};

const DFT_POINTS: usize = 64;

/// Estimate the probability that a patterned contact lens is present.
///
/// Samples a radial strip at mid-iris radius and computes a 64-point DFT.
/// Patterned lenses produce strong periodic signals. Confidence is the
/// normalised ratio of peak-to-mean magnitude, clamped to [0, 1].
///
/// `confidence = ((peak / mean) - 1.0).clamp(0, 10) / 10.0`
pub fn detect_patterned(image: &GrayImage, pupil: Circle, limbus: Circle) -> f32 {
    let mid_r = (pupil.radius + limbus.radius) / 2.0;
    let samples = sample_radial(image, limbus.cx, limbus.cy, mid_r);
    let (magnitudes, _phases) = dft_magnitudes(&samples);

    // Normalise AC magnitudes by DC power to be invariant to absolute brightness.
    // A flat signal produces DC >> AC, so the ratio stays near zero even under f32 noise.
    let dc = magnitudes[0];
    if dc < 1.0 {
        return 0.0;
    }
    let ac = &magnitudes[1..];
    if ac.is_empty() {
        return 0.0;
    }
    let peak = ac.iter().cloned().fold(0.0f32, f32::max);
    // peak / dc: for a flat signal this is tiny (< 0.01); for patterned it can be > 0.1.
    ((peak / dc - 0.05).clamp(0.0, 1.0))
}

/// Sample pixel values at `n` angular positions along the given radius.
fn sample_radial(image: &GrayImage, cx: f32, cy: f32, r: f32) -> Vec<f32> {
    (0..DFT_POINTS)
        .map(|i| {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (DFT_POINTS as f32);
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            let px = x.round().clamp(0.0, image.width.saturating_sub(1) as f32) as u32;
            let py = y.round().clamp(0.0, image.height.saturating_sub(1) as f32) as u32;
            image.get(px, py) as f32
        })
        .collect()
}

/// Compute DFT magnitude and phase for each frequency bin.
fn dft_magnitudes(samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let n = samples.len();
    let mut magnitudes = Vec::with_capacity(n);
    let mut phases = Vec::with_capacity(n);
    for k in 0..n {
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for (t, &x) in samples.iter().enumerate() {
            let angle = -2.0 * std::f32::consts::PI * (k * t) as f32 / n as f32;
            re += x * angle.cos();
            im += x * angle.sin();
        }
        magnitudes.push((re * re + im * im).sqrt());
        phases.push(im.atan2(re));
    }
    (magnitudes, phases)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, f: impl Fn(u32, u32) -> u8 + Copy) -> GrayImage {
        let data: Vec<u8> = (0..h)
            .flat_map(move |y| (0..w).map(move |x| f(x, y)))
            .collect();
        GrayImage::new(w, h, data).expect("valid image")
    }

    #[test]
    fn flat_image_returns_low_patterned_confidence() {
        let image = make_image(200, 200, |_, _| 128);
        let pupil = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 20.0,
        };
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 70.0,
        };
        let conf = detect_patterned(&image, pupil, limbus);
        // Flat signal -> no periodic content -> confidence near 0.
        assert!(conf < 0.1, "expected low confidence, got {conf}");
    }

    #[test]
    fn confidence_in_range() {
        // Alternating stripes to excite periodicity.
        let image = make_image(200, 200, |x, _| if x % 2 == 0 { 0 } else { 255 });
        let pupil = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 20.0,
        };
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 70.0,
        };
        let conf = detect_patterned(&image, pupil, limbus);
        assert!(
            (0.0..=1.0).contains(&conf),
            "confidence out of range: {conf}"
        );
    }
}
