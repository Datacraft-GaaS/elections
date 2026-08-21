use crate::domain::VeinTemplate;

pub struct VeinQuality {
    /// Standard deviation of pixel values divided by 128, clamped to [0, 1].
    pub contrast: f32,
    /// Vein branch points per 100 pixels.
    pub branch_density: f32,
    /// Overall quality score [0, 100].
    pub overall: f32,
}

const PIXEL_THRESHOLD: u8 = 128;

pub fn score_quality(template: &VeinTemplate) -> VeinQuality {
    let contrast = pixel_contrast(&template.pattern_bitmap);
    let branches = count_branch_points(&template.pattern_bitmap, template.width, template.height);
    let n = template.pattern_bitmap.len();
    let branch_density = if n == 0 {
        0.0
    } else {
        (branches as f32 / n as f32) * 100.0
    };

    // Weight: contrast 50%, branch density (capped at 5 per 100px) 50%.
    let branch_norm = (branch_density / 5.0).clamp(0.0, 1.0);
    let overall = ((contrast * 0.5 + branch_norm * 0.5) * 100.0).clamp(0.0, 100.0);

    VeinQuality {
        contrast,
        branch_density,
        overall,
    }
}

fn pixel_contrast(bitmap: &[u8]) -> f32 {
    let n = bitmap.len();
    if n == 0 {
        return 0.0;
    }
    let mean: f64 = bitmap.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
    let variance: f64 = bitmap
        .iter()
        .map(|&v| (v as f64 - mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let std_dev = variance.sqrt() as f32;
    (std_dev / 128.0).clamp(0.0, 1.0)
}

pub fn count_branch_points(bitmap: &[u8], width: u16, height: u16) -> u32 {
    let w = width as usize;
    let h = height as usize;
    if w < 3 || h < 3 {
        return 0;
    }
    let mut count = 0u32;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = bitmap[y * w + x];
            if center <= PIXEL_THRESHOLD {
                continue;
            }
            let neighbors = [
                bitmap[(y - 1) * w + (x - 1)],
                bitmap[(y - 1) * w + x],
                bitmap[(y - 1) * w + (x + 1)],
                bitmap[y * w + (x - 1)],
                bitmap[y * w + (x + 1)],
                bitmap[(y + 1) * w + (x - 1)],
                bitmap[(y + 1) * w + x],
                bitmap[(y + 1) * w + (x + 1)],
            ];
            let above_thresh = neighbors.iter().filter(|&&v| v > PIXEL_THRESHOLD).count();
            if above_thresh >= 3 {
                count += 1;
            }
        }
    }
    count
}
