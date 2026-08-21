use crate::domain::PeriocularTemplate;

pub struct PeriocularQuality {
    /// Laplacian variance normalized to [0, 100].
    pub sharpness_score: f32,
    /// 100 * (1 - occlusion_ratio).
    pub occlusion_score: f32,
    /// Weighted average of sub-scores.
    pub overall: f32,
}

/// Sharpness variance that maps to a score of 100.
const SHARP_SATURATION: f32 = 500.0;

pub fn score_quality(template: &PeriocularTemplate) -> PeriocularQuality {
    let sharpness_score = (template.sharpness / SHARP_SATURATION * 100.0).clamp(0.0, 100.0);
    let occlusion_score = ((1.0 - template.occlusion_ratio) * 100.0).clamp(0.0, 100.0);
    let overall = (sharpness_score * 0.5 + occlusion_score * 0.5).clamp(0.0, 100.0);
    PeriocularQuality {
        sharpness_score,
        occlusion_score,
        overall,
    }
}

/// Apply the Laplacian kernel [0,1,0; 1,-4,1; 0,1,0] and return the variance of the response.
pub fn laplacian_variance(patch: &[u8], width: usize, height: usize) -> f32 {
    if width < 3 || height < 3 {
        return 0.0;
    }

    let n = (width - 2) * (height - 2);
    let mut responses = Vec::with_capacity(n);

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let center = patch[y * width + x] as f32;
            let top = patch[(y - 1) * width + x] as f32;
            let bottom = patch[(y + 1) * width + x] as f32;
            let left = patch[y * width + (x - 1)] as f32;
            let right = patch[y * width + (x + 1)] as f32;
            let resp = top + bottom + left + right - 4.0 * center;
            responses.push(resp);
        }
    }

    let mean: f32 = responses.iter().sum::<f32>() / n as f32;
    let variance: f32 = responses.iter().map(|&r| (r - mean).powi(2)).sum::<f32>() / n as f32;
    variance
}
