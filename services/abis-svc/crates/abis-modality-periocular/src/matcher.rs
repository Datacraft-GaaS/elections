use crate::domain::PeriocularTemplate;

pub const DEFAULT_THRESHOLD: f32 = 0.45;

pub struct Score {
    pub raw: f32,
    pub chi_sq_distance: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
}

pub fn match_periocular(probe: &PeriocularTemplate, gallery: &PeriocularTemplate) -> Score {
    let chi_sq = chi_square(&probe.lbp_histogram.bins, &gallery.lbp_histogram.bins);
    // Convert distance to similarity: higher distance means less similar.
    let raw = (-chi_sq / 2.0).exp().clamp(0.0, 1.0);
    Score {
        raw,
        chi_sq_distance: chi_sq,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
    }
}

pub fn chi_square(h1: &[f32], h2: &[f32]) -> f32 {
    const EPS: f32 = 1e-10;
    h1.iter()
        .zip(h2.iter())
        .map(|(&a, &b)| {
            let diff = a - b;
            diff * diff / (a + b + EPS)
        })
        .sum()
}
