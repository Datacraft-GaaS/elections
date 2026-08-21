use eel_abis_cert_frvt::{
    frvt_1v1::{Frvt1v1Submission, VerificationPair},
    frvt_1vn::Frvt1vNSubmission,
    frvt_pad::FrvtPadSubmission,
    frvt_quality::FrvtQualitySubmission,
};
use proptest::prelude::*;

// ---- 1:1 ----

fn identical_pair(n: usize) -> VerificationPair {
    VerificationPair {
        enrolment_template: vec![0xABu8; n],
        probe_template: vec![0xABu8; n],
        ground_truth_match: true,
    }
}

fn impostor_pair(n: usize) -> VerificationPair {
    VerificationPair {
        enrolment_template: vec![0xFFu8; n],
        probe_template: vec![0x00u8; n],
        ground_truth_match: false,
    }
}

#[test]
fn frvt_1v1_identical_pairs_high_tar() {
    let sub = Frvt1v1Submission::new("eel-face", "1.0").unwrap();
    let pairs: Vec<_> = (0..50).map(|_| identical_pair(64)).collect();
    let report = sub.run(&pairs, 0.99).unwrap();
    assert!(report.tar_at_far_1e3 >= 0.0 && report.tar_at_far_1e3 <= 1.0);
    assert_eq!(report.genuine_pairs, 50);
    assert_eq!(report.impostor_pairs, 0);
}

#[test]
fn frvt_1v1_empty_pairs_ok() {
    let sub = Frvt1v1Submission::new("eel-face", "1.0").unwrap();
    let report = sub.run(&[], 0.5).unwrap();
    assert_eq!(report.genuine_pairs, 0);
}

#[test]
fn frvt_1v1_empty_algorithm_id_errors() {
    assert!(Frvt1v1Submission::new("", "1.0").is_err());
}

#[test]
fn frvt_1v1_algorithm_id_accessor() {
    let sub = Frvt1v1Submission::new("eel-iris", "2.0").unwrap();
    assert_eq!(sub.algorithm_id(), "eel-iris");
}

// ---- 1:N ----

#[test]
fn frvt_1vn_rank1_in_range() {
    let sub = Frvt1vNSubmission::new("eel-face");
    let report = sub.run(1_000, 50_000);
    assert!(report.rank1_id_rate >= 0.0 && report.rank1_id_rate <= 1.0);
}

#[test]
fn frvt_1vn_zero_gallery_rank1_zero() {
    let sub = Frvt1vNSubmission::new("eel-face");
    let report = sub.run(0, 100);
    assert_eq!(report.rank1_id_rate, 0.0);
}

#[test]
fn frvt_1vn_gallery_size_stored() {
    let sub = Frvt1vNSubmission::new("eel-face");
    let report = sub.run(500, 10_000);
    assert_eq!(report.gallery_size, 500);
}

// ---- Quality ----

#[test]
fn frvt_quality_high_quality_scores() {
    let sub = FrvtQualitySubmission;
    let scores = vec![80.0, 85.0, 90.0, 95.0];
    let report = sub.run(&scores).unwrap();
    assert!(report.mean_quality >= 80.0 && report.mean_quality <= 100.0);
    assert!(report.correlation_with_fmr >= 0.8);
}

#[test]
fn frvt_quality_out_of_range_errors() {
    let sub = FrvtQualitySubmission;
    assert!(sub.run(&[50.0, 101.0]).is_err());
}

#[test]
fn frvt_quality_empty_scores() {
    let sub = FrvtQualitySubmission;
    let report = sub.run(&[]).unwrap();
    assert_eq!(report.sample_count, 0);
    assert_eq!(report.mean_quality, 0.0);
}

// ---- PAD ----

#[test]
fn frvt_pad_acer_is_average() {
    let sub = FrvtPadSubmission::new("eel-pad");
    let report = sub.run(0.02, 0.08, vec!["printed-photo".into()]);
    assert!((report.acer - 0.05).abs() < 1e-9);
}

#[test]
fn frvt_pad_species_stored() {
    let sub = FrvtPadSubmission::new("eel-pad");
    let report = sub.run(0.01, 0.05, vec!["silicone".into(), "gelatin".into()]);
    assert_eq!(report.species_tested.len(), 2);
}

// ---- Proptest ----

proptest! {
    #[test]
    fn prop_quality_mean_in_range(
        scores in proptest::collection::vec(0.0f64..100.0f64, 1..50),
    ) {
        let sub = FrvtQualitySubmission;
        let report = sub.run(&scores).unwrap();
        prop_assert!(report.mean_quality >= 0.0 && report.mean_quality <= 100.0);
        prop_assert_eq!(report.sample_count, scores.len() as u64);
    }

    #[test]
    fn prop_1vn_rank1_bounded(gallery in 0usize..1_000_000usize) {
        let sub = Frvt1vNSubmission::new("eel");
        let report = sub.run(gallery, 1000);
        prop_assert!(report.rank1_id_rate >= 0.0 && report.rank1_id_rate <= 1.0);
    }
}
