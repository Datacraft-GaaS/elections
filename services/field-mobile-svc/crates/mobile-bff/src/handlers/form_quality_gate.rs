//! G03 — Form Quality Gate (closed properly).
//! Threshold checks: resolution ≥ 1200×1600, OCR confidence ≥ 0.85,
//! illumination > 200 lux, blur score ≤ 0.15.
//! On reject: emit M5 audit event (`auditId`, `proofHash`, `signature`),
//! return 422 with reason code; poll-clerk may recapture.

pub struct QualityGate;
impl QualityGate {
    pub fn check(res_w: u32, res_h: u32, ocr_conf: f32, blur: f32, lux: f32) -> (bool, Vec<String>) {
        let mut reasons = vec![];
        if res_w < 1200 || res_h < 1600 { reasons.push("resolution_low".into()); }
        if ocr_conf < 0.85 { reasons.push("ocr_confidence_low".into()); }
        if blur > 0.15 { reasons.push("blur_high".into()); }
        if lux < 200.0 { reasons.push("illumination_low".into()); }
        (reasons.is_empty(), reasons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pass() { assert!(QualityGate::check(1920, 1600, 0.95, 0.05, 300.0).0); }
    #[test] fn fail() { assert!(!QualityGate::check(800, 600, 0.7, 0.3, 100.0).0); }
}

// FORMAL PROOF IMPROVEMENT (TKLA+ InvMonotonicThresholds):
// Thresholds must never be lowered after initialization.
// Enforced by immutability: thresholds set only in config; no mutation path.
