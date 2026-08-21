use abis_matcher::adjudication::{is_ambiguous, open_adjudication_case, record_decision};
use abis_matcher::batch::run_dedup_batch;
use abis_matcher::domain::{
    AdjudicationDecision, AdjudicationStatus, MatchOutcome, MatchResult, ModalityScore,
    TemplateIndex,
};
use abis_matcher::fmr_fnmr::compute_fmr_fnmr;
use abis_matcher::{MatchPipeline, MockMatcher};

#[test]
fn mock_matcher_deterministic() {
    let m = MockMatcher;
    let r1 = m.match_11("probe-abc", "gallery-xyz", 0.8);
    let r2 = m.match_11("probe-abc", "gallery-xyz", 0.8);
    assert_eq!(r1.confidence_score, r2.confidence_score);
    assert_eq!(r1.outcome, r2.outcome);
}

#[test]
fn mock_matcher_self_match_is_high() {
    let m = MockMatcher;
    let r = m.match_11("voter-001", "voter-001", 0.8);
    assert!(
        r.confidence_score >= 0.9,
        "self-match score={}",
        r.confidence_score
    );
    assert_eq!(r.outcome, MatchOutcome::Match);
}

#[test]
fn is_ambiguous_thresholds() {
    assert!(is_ambiguous(0.6, 0.5, 0.8));
    assert!(is_ambiguous(0.5, 0.5, 0.8));
    assert!(!is_ambiguous(0.8, 0.5, 0.8)); // upper is exclusive
    assert!(!is_ambiguous(0.4, 0.5, 0.8));
    assert!(!is_ambiguous(0.9, 0.5, 0.8));
}

#[test]
fn fmr_fnmr_mock() {
    let m = MockMatcher;
    let genuine: Vec<(String, String)> = (0..1000)
        .map(|i| (format!("voter-{i}"), format!("voter-{i}")))
        .collect();
    // Impostors: pairs where probe and gallery are different and far apart in hash space.
    // Use widely spaced IDs so their sha256 bytes differ enough.
    let impostors: Vec<(String, String)> = (0..1000)
        .map(|i| {
            (
                format!("probe-{i}"),
                format!("gallery-impostor-{}", i + 100_000),
            )
        })
        .collect();

    let harness = compute_fmr_fnmr(&m, &genuine, &impostors, 0.8);
    assert_eq!(harness.total_genuine, 1000);
    assert_eq!(harness.total_impostor, 1000);
    assert!(harness.fnmr <= 0.001, "FNMR too high: {}", harness.fnmr);
    // FMR may not be perfectly zero due to hash collisions, but must be very low.
    assert!(harness.fmr <= 0.05, "FMR too high: {}", harness.fmr);
}

#[test]
fn batch_dedup_empty() {
    let m = MockMatcher;
    let cases = run_dedup_batch(&m, &[], 0.8);
    assert!(cases.is_empty());
}

#[test]
fn adjudication_case_open_and_decide() {
    let fake_result = MatchResult {
        result_id: "result-1".to_string(),
        request_id: "req-1".to_string(),
        outcome: MatchOutcome::Adjudicate,
        confidence_score: 0.65,
        modality_scores: vec![ModalityScore {
            modality: "fingerprint".to_string(),
            score: 0.65,
        }],
    };
    let mut case = open_adjudication_case(&fake_result, vec!["voter-1".into(), "voter-2".into()]);
    assert_eq!(case.status, AdjudicationStatus::Open);

    let decision = AdjudicationDecision {
        case_id: case.case_id.clone(),
        decision: "no_match".to_string(),
        reason_code: "BIOMETRIC_DISTINCT".to_string(),
        officer_id: "officer-99".to_string(),
        decided_at: "2026-01-01T00:00:00Z".to_string(),
    };
    record_decision(&mut case, decision);
    assert_eq!(case.status, AdjudicationStatus::Resolved);
    assert_eq!(case.decision.as_deref(), Some("no_match"));
}
