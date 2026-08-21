//! Tests for consensus matching, policy, and calibration.

use abis_matcher::modality::fingerprint::{
    FingerPosition, FingerprintTemplate, Minutia, MinutiaeType,
};
use eel_abis_fp_consensus::{
    calibration::ScoreCalibrator,
    consensus::{ConsensusInput, ConsensusMatcher},
    policy::MinAgreementPolicy,
};

fn make_template(count: usize, offset_x: u16) -> FingerprintTemplate {
    let minutiae: Vec<Minutia> = (0..count)
        .map(|i| Minutia {
            x: offset_x.saturating_add(i as u16 * 10),
            y: i as u16 * 5,
            theta: 30.0 * i as f32,
            quality: 80,
            mtype: MinutiaeType::Ending,
        })
        .collect();
    FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
}

fn gray_pixels(w: u32, h: u32, val: u8) -> Vec<u8> {
    vec![val; (w * h) as usize]
}

fn striped_pixels(w: u32, h: u32, period: u32) -> Vec<u8> {
    (0..(w * h))
        .map(|i| {
            if (i % w / period) % 2 == 0 {
                50u8
            } else {
                200u8
            }
        })
        .collect()
}

fn binary_pixels(w: u32, h: u32, val: u8) -> Vec<u8> {
    vec![val; (w * h) as usize]
}

// ----- Policy -----

#[test]
fn test_policy_default_requires_3_of_5() {
    let p = MinAgreementPolicy::default();
    assert!(!p.check(2, 5));
    assert!(p.check(3, 5));
}

#[test]
fn test_policy_scales_for_fewer_matchers() {
    let p = MinAgreementPolicy::new(3);
    assert!(!p.check(1, 2));
    assert!(p.check(2, 2));
}

#[test]
fn test_policy_zero_total_false() {
    assert!(!MinAgreementPolicy::default().check(0, 0));
}

#[test]
fn test_policy_all_agree_passes() {
    let p = MinAgreementPolicy::default();
    assert!(p.check(5, 5));
}

// ----- Calibration -----

#[test]
fn test_calibrator_identity_monotone() {
    let cal = ScoreCalibrator::identity();
    let lo = cal.calibrate(0.1);
    let hi = cal.calibrate(0.9);
    assert!(hi > lo);
}

#[test]
fn test_calibrator_output_in_range() {
    let cal = ScoreCalibrator::identity();
    for i in 0..=10 {
        let v = cal.calibrate(i as f32 / 10.0);
        assert!(v >= 0.0 && v <= 1.0, "calibrated {v} out of [0,1]");
    }
}

#[test]
fn test_calibrator_fit_needs_min_samples() {
    assert!(ScoreCalibrator::fit(&[0.5], &[1.0]).is_err());
}

#[test]
fn test_calibrator_monotonic_flag() {
    let cal = ScoreCalibrator::new(5.0, -2.5);
    assert!(cal.is_monotonic());
}

#[test]
fn test_calibrator_negative_a_not_monotone() {
    let cal = ScoreCalibrator::new(-1.0, 0.0);
    assert!(!cal.is_monotonic());
}

#[test]
fn test_calibrator_fit_real_data() {
    let scores = vec![0.9f32, 0.85, 0.7, 0.2, 0.1, 0.15];
    let labels = vec![1.0f32, 1.0, 1.0, 0.0, 0.0, 0.0];
    let cal = ScoreCalibrator::fit(&scores, &labels).unwrap();
    assert!(cal.calibrate(0.9) > cal.calibrate(0.1));
}

// ----- ConsensusMatcher -----

#[test]
fn test_consensus_no_inputs_errors() {
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: None,
        minutiae_gallery: None,
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    assert!(m.compare(&input).is_err());
}

#[test]
fn test_consensus_minutiae_only_runs_bozorth_and_mcc() {
    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    assert_eq!(
        result.per_matcher.len(),
        2,
        "only bozorth3 and mcc should run"
    );
}

#[test]
fn test_consensus_explanation_present() {
    let probe = make_template(8, 0);
    let gallery = make_template(8, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    assert!(!result.explanation.is_empty());
}

#[test]
fn test_consensus_per_matcher_names_present() {
    let probe = make_template(8, 0);
    let gallery = make_template(8, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    for r in &result.per_matcher {
        assert!(!r.name.is_empty());
    }
}

#[test]
fn test_consensus_rejects_when_only_1_of_5_agrees() {
    let mut m = ConsensusMatcher::default();
    // bozorth always votes match (threshold 0), rest never pass.
    m.thresholds[0] = 0.0;
    m.thresholds[1] = 2.0;
    m.thresholds[2] = 2.0;
    m.thresholds[3] = 2.0;
    m.thresholds[4] = 2.0;
    m.policy = MinAgreementPolicy::new(3);

    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let pix = striped_pixels(64, 64, 5);
    let bin = binary_pixels(64, 64, 0);
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: Some(&pix),
        image_gallery: Some(&pix),
        image_width: 64,
        image_height: 64,
        binary_probe: Some(&bin),
        binary_gallery: Some(&bin),
    };
    let result = m.compare(&input).unwrap();
    assert!(
        !result.decision,
        "consensus must reject when only 1 of 5 matchers agrees"
    );
}

#[test]
fn test_consensus_accepts_when_all_thresholds_zero() {
    let mut m = ConsensusMatcher::default();
    m.thresholds = [0.0, 0.0, 0.0, 0.0, 0.0];
    m.policy = MinAgreementPolicy::new(3);

    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let pix = striped_pixels(64, 64, 5);
    let bin = binary_pixels(64, 64, 1);
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: Some(&pix),
        image_gallery: Some(&pix),
        image_width: 64,
        image_height: 64,
        binary_probe: Some(&bin),
        binary_gallery: Some(&bin),
    };
    let result = m.compare(&input).unwrap();
    // With threshold 0, all matchers vote yes.
    assert!(
        result.decision,
        "consensus must accept when all thresholds are 0"
    );
}

#[test]
fn test_consensus_fused_score_in_range() {
    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    assert!(
        result.fused >= 0.0 && result.fused <= 1.0,
        "fused score out of [0,1]: {}",
        result.fused
    );
}

#[test]
fn test_consensus_5_sub_matchers_with_all_inputs() {
    let mut m = ConsensusMatcher::default();
    m.thresholds = [0.0; 5];
    let probe = make_template(5, 0);
    let gallery = make_template(5, 0);
    let pix = striped_pixels(64, 64, 5);
    let bin = binary_pixels(64, 64, 1);
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: Some(&pix),
        image_gallery: Some(&pix),
        image_width: 64,
        image_height: 64,
        binary_probe: Some(&bin),
        binary_gallery: Some(&bin),
    };
    let result = m.compare(&input).unwrap();
    assert_eq!(
        result.per_matcher.len(),
        5,
        "all 5 sub-matchers should produce results"
    );
}

#[test]
fn test_consensus_per_matcher_calibrated_in_range() {
    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let m = ConsensusMatcher::default();
    let pix = gray_pixels(64, 64, 128);
    let bin = binary_pixels(64, 64, 1);
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: Some(&pix),
        image_gallery: Some(&pix),
        image_width: 64,
        image_height: 64,
        binary_probe: Some(&bin),
        binary_gallery: Some(&bin),
    };
    let result = m.compare(&input).unwrap();
    for r in &result.per_matcher {
        assert!(
            r.calibrated_score >= 0.0 && r.calibrated_score <= 1.0,
            "{} calibrated_score {} out of range",
            r.name,
            r.calibrated_score
        );
    }
}

#[test]
fn test_consensus_different_templates_do_not_crash() {
    let probe = make_template(10, 0);
    let gallery = make_template(10, 200); // different positions
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    assert!(m.compare(&input).is_ok());
}

#[test]
fn test_consensus_policy_reported_in_decision() {
    let mut m = ConsensusMatcher::default();
    // All matchers vote yes but policy requires 6 (impossible).
    m.thresholds = [0.0; 5];
    m.policy = MinAgreementPolicy::new(6); // requires 6 of N

    let probe = make_template(10, 0);
    let gallery = make_template(10, 0);
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    // Only 2 matchers ran (bozorth + MCC) - can't meet min_votes=6.
    assert!(!result.decision);
}

#[test]
fn test_consensus_explanation_contains_fused() {
    let probe = make_template(5, 0);
    let gallery = make_template(5, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    assert!(result.explanation.contains("fused="));
}

#[test]
fn test_consensus_explanation_contains_votes() {
    let probe = make_template(5, 0);
    let gallery = make_template(5, 0);
    let m = ConsensusMatcher::default();
    let input = ConsensusInput {
        minutiae_probe: Some(&probe),
        minutiae_gallery: Some(&gallery),
        image_probe: None,
        image_gallery: None,
        image_width: 0,
        image_height: 0,
        binary_probe: None,
        binary_gallery: None,
    };
    let result = m.compare(&input).unwrap();
    assert!(result.explanation.contains("votes="));
}
