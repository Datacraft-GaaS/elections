use eel_governance_parallel_manual_count::{
    compute_divergence,
    divergence::PlatformTally,
    ingest::{CandidateCount, ManualCountStore, ManualCountSubmission},
    plan::SamplePlan,
    rla_feed::build_rla_input,
    sample::{default_strata, neyman_allocate, sample_stations, SAMPLE_TARGET, TOTAL_STATIONS},
};

// --- sample tests ---

#[test]
fn sample_deterministic_same_seed() {
    let strata = default_strata();
    let a = sample_stations(&strata, 42);
    let b = sample_stations(&strata, 42);
    let ids_a: Vec<_> = a.iter().map(|s| &s.station_id).collect();
    let ids_b: Vec<_> = b.iter().map(|s| &s.station_id).collect();
    assert_eq!(ids_a, ids_b);
}

#[test]
fn sample_different_seeds_different() {
    let strata = default_strata();
    let a: std::collections::HashSet<_> =
        sample_stations(&strata, 1).into_iter().map(|s| s.station_id).collect();
    let b: std::collections::HashSet<_> =
        sample_stations(&strata, 999_999).into_iter().map(|s| s.station_id).collect();
    // Different seeds should produce meaningfully different sets (some overlap is fine,
    // but they should not be identical).
    assert_ne!(a, b);
}

#[test]
fn sample_count_is_target() {
    let strata = default_strata();
    let sampled = sample_stations(&strata, 7);
    assert_eq!(sampled.len(), SAMPLE_TARGET as usize);
}

#[test]
fn sample_no_duplicates_within_stratum() {
    let strata = default_strata();
    let sampled = sample_stations(&strata, 13);
    for stratum in &strata {
        let ids: Vec<_> = sampled
            .iter()
            .filter(|s| s.stratum == stratum.name)
            .map(|s| &s.station_id)
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "duplicates found in stratum {}",
            stratum.name
        );
    }
}

#[test]
fn default_strata_sum_to_total() {
    let strata = default_strata();
    let total: u32 = strata.iter().map(|s| s.station_count).sum();
    assert_eq!(total, TOTAL_STATIONS);
}

#[test]
fn neyman_allocation_sums_to_target() {
    let strata = default_strata();
    let allocs = neyman_allocate(&strata, SAMPLE_TARGET);
    let sum: u32 = allocs.iter().sum();
    assert_eq!(sum, SAMPLE_TARGET);
}

#[test]
fn stratum_balance_nairobi() {
    // Nairobi: 2645 / 46229 * 462 ~= 26.4, so expected floor 26 or ceil 27.
    let strata = default_strata();
    let allocs = neyman_allocate(&strata, SAMPLE_TARGET);
    let nairobi_idx = strata.iter().position(|s| s.name == "Nairobi").unwrap();
    let nairobi_alloc = allocs[nairobi_idx];
    assert!(
        nairobi_alloc >= 26 && nairobi_alloc <= 27,
        "Nairobi allocation {} out of expected range 26-27",
        nairobi_alloc
    );
}

// --- plan tests ---

#[test]
fn plan_generate_contains_station_check() {
    let plan = SamplePlan::generate("2027-08-09", 55);
    let first_id = plan.sampled_stations[0].station_id.clone();
    assert!(plan.contains_station(&first_id));
    assert!(!plan.contains_station("STN-99999"));
}

#[test]
fn plan_station_ids_len() {
    let plan = SamplePlan::generate("2027-08-09", 55);
    assert_eq!(plan.station_ids().len(), SAMPLE_TARGET as usize);
}

// --- ingest tests ---

async fn make_submission(station_id: &str) -> ManualCountSubmission {
    ManualCountSubmission {
        station_id: station_id.to_string(),
        stratum: "Nairobi".to_string(),
        submitted_by: "officer-001".to_string(),
        submitted_at: "2027-08-12T10:00:00Z".to_string(),
        candidates: vec![CandidateCount {
            candidate_id: "C1".to_string(),
            manual_count: 100,
        }],
    }
}

#[tokio::test]
async fn ingest_submit_accepted() {
    let store = ManualCountStore::new();
    let sub = make_submission("STN-00001").await;
    store.submit(sub).await.unwrap();
    assert_eq!(store.count().await, 1);
}

#[tokio::test]
async fn ingest_duplicate_rejected() {
    let store = ManualCountStore::new();
    let sub1 = make_submission("STN-00002").await;
    let sub2 = make_submission("STN-00002").await;
    store.submit(sub1).await.unwrap();
    let result = store.submit(sub2).await;
    assert!(result.is_err(), "second submit should be rejected");
}

// --- divergence tests ---

fn make_sub(station_id: &str, counts: &[(&str, u64)]) -> ManualCountSubmission {
    ManualCountSubmission {
        station_id: station_id.to_string(),
        stratum: "Central".to_string(),
        submitted_by: "officer-002".to_string(),
        submitted_at: "2027-08-12T11:00:00Z".to_string(),
        candidates: counts
            .iter()
            .map(|(id, n)| CandidateCount {
                candidate_id: id.to_string(),
                manual_count: *n,
            })
            .collect(),
    }
}

fn make_tally(station_id: &str, counts: &[(&str, u64)]) -> PlatformTally {
    PlatformTally {
        station_id: station_id.to_string(),
        candidates: counts
            .iter()
            .map(|(id, n)| CandidateCount {
                candidate_id: id.to_string(),
                manual_count: *n,
            })
            .collect(),
    }
}

#[test]
fn divergence_zero_when_counts_match() {
    let subs = vec![make_sub("STN-00010", &[("C1", 200), ("C2", 150)])];
    let tallies = vec![make_tally("STN-00010", &[("C1", 200), ("C2", 150)])];
    let report = compute_divergence(&subs, &tallies);
    assert_eq!(report.station_reports[0].max_abs_divergence, 0);
    assert_eq!(report.mean_abs_divergence, 0.0);
}

#[test]
fn divergence_nonzero_detected() {
    let subs = vec![make_sub("STN-00011", &[("C1", 205)])];
    let tallies = vec![make_tally("STN-00011", &[("C1", 200)])];
    let report = compute_divergence(&subs, &tallies);
    assert_eq!(report.station_reports[0].candidate_divergences[0].delta, 5);
}

#[test]
fn divergence_report_stations_with_divergence_count() {
    let subs = vec![
        make_sub("STN-00020", &[("C1", 100)]),
        make_sub("STN-00021", &[("C1", 110)]),
        make_sub("STN-00022", &[("C1", 100)]),
    ];
    let tallies = vec![
        make_tally("STN-00020", &[("C1", 95)]),  // diverges
        make_tally("STN-00021", &[("C1", 100)]), // diverges
        make_tally("STN-00022", &[("C1", 100)]), // exact match
    ];
    let report = compute_divergence(&subs, &tallies);
    assert_eq!(report.stations_with_divergence, 2);
}

// --- rla_feed tests ---

#[test]
fn rla_input_build_contains_report_id() {
    let subs = vec![make_sub("STN-00030", &[("C1", 50)])];
    let tallies = vec![make_tally("STN-00030", &[("C1", 50)])];
    let report = compute_divergence(&subs, &tallies);
    let rla_input = build_rla_input(&report);
    assert_eq!(rla_input.report_id, report.report_id);
    assert_eq!(rla_input.source, "parallel-manual-count");
}

#[test]
fn divergence_mean_abs_computed() {
    // Station A: C1 delta=5, Station B: C1 delta=3.
    // All deltas: [5, 3]. Mean = 4.0.
    let subs = vec![
        make_sub("STN-00040", &[("C1", 105)]),
        make_sub("STN-00041", &[("C1", 103)]),
    ];
    let tallies = vec![
        make_tally("STN-00040", &[("C1", 100)]),
        make_tally("STN-00041", &[("C1", 100)]),
    ];
    let report = compute_divergence(&subs, &tallies);
    assert!(
        (report.mean_abs_divergence - 4.0).abs() < 1e-9,
        "expected mean 4.0, got {}",
        report.mean_abs_divergence
    );
}
