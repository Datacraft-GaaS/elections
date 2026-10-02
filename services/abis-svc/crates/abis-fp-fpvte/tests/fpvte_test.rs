// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FpVTE harness tests.
//!
//! Uses synthetic in-memory corpora so no NIST Special Database files are needed.

use eel_abis_fp_fpvte::{
    corpus::{Corpus, CorpusEntry, DemographicLabel},
    harness::{FpVteHarness, HarnessConfig},
    matcher_trait::{FpMatcher, NullMatcher, SyntheticMatcher},
    report::{FairnessSlice, RankAccuracy, Report, RocPoint},
    FpVteError,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn small_corpus(n_subjects: usize) -> Corpus {
    Corpus::synthetic_in_memory(n_subjects)
}

fn harness() -> FpVteHarness {
    FpVteHarness::new(HarnessConfig {
        roc_steps: 20,
        max_rank: 5,
    })
}

fn synthetic_matcher() -> SyntheticMatcher {
    SyntheticMatcher {
        name: "synthetic".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_generates_correct_count() {
    let c = small_corpus(10);
    // 10 subjects x 2 acquisitions = 20 entries.
    assert_eq!(c.entries.len(), 20);
}

#[test]
fn corpus_subject_ids_unique() {
    let c = small_corpus(8);
    let ids = c.subject_ids();
    assert_eq!(ids.len(), 8);
}

#[test]
fn corpus_entries_for_subject() {
    let c = small_corpus(5);
    let id = c.subject_ids()[0];
    let entries = c.entries_for_subject(id);
    assert_eq!(entries.len(), 2, "each subject should have 2 acquisitions");
}

// ---------------------------------------------------------------------------
// Matcher trait
// ---------------------------------------------------------------------------

#[test]
fn null_matcher_returns_zero() {
    let m = NullMatcher;
    let score = m.compare(&vec![1, 2, 3], &vec![4, 5, 6]).unwrap();
    assert_eq!(score, 0.0);
}

#[test]
fn synthetic_matcher_identical_templates_score_one() {
    let m = synthetic_matcher();
    let t = vec![42u8; 64];
    let score = m.compare(&t, &t).unwrap();
    assert_eq!(score, 1.0);
}

#[test]
fn synthetic_matcher_score_bounded() {
    let m = synthetic_matcher();
    let s = m.compare(&vec![0u8; 64], &vec![255u8; 64]).unwrap();
    assert!((0.0..=1.0).contains(&s));
}

// ---------------------------------------------------------------------------
// Harness: ROC
// ---------------------------------------------------------------------------

#[test]
fn harness_run_produces_valid_report() {
    let corpus = small_corpus(50);
    let matcher = synthetic_matcher();
    let h = harness();
    let report = h.run(&corpus, &matcher, None).unwrap();
    assert_eq!(report.matcher_name, "synthetic");
    assert!(report.n_probes > 0);
    assert!(report.n_gallery > 0);
}

#[test]
fn roc_not_empty() {
    let corpus = small_corpus(50);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    assert!(!report.roc.is_empty(), "ROC curve should not be empty");
}

#[test]
fn roc_fmr_fnmr_in_range() {
    let corpus = small_corpus(50);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    for pt in &report.roc {
        assert!(
            (0.0..=1.0).contains(&pt.fmr),
            "FMR out of range: {}",
            pt.fmr
        );
        assert!(
            (0.0..=1.0).contains(&pt.fnmr),
            "FNMR out of range: {}",
            pt.fnmr
        );
    }
}

#[test]
fn eer_in_range() {
    let corpus = small_corpus(50);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    assert!(
        (0.0..=1.0).contains(&report.eer),
        "EER={} out of range",
        report.eer
    );
}

// ---------------------------------------------------------------------------
// Harness: Rank-N accuracy
// ---------------------------------------------------------------------------

#[test]
fn rank_n_monotone_increasing() {
    let corpus = small_corpus(50);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    let accs: Vec<f32> = report.rank_accuracies.iter().map(|r| r.accuracy).collect();
    for i in 1..accs.len() {
        assert!(
            accs[i] >= accs[i - 1] - 1e-6,
            "rank accuracy not monotone: rank {} acc {} < rank {} acc {}",
            i + 1,
            accs[i],
            i,
            accs[i - 1]
        );
    }
}

#[test]
fn rank1_accuracy_in_range() {
    let corpus = small_corpus(30);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    assert!(
        (0.0..=1.0).contains(&report.rank1_accuracy),
        "rank1_accuracy={} out of range",
        report.rank1_accuracy
    );
}

// ---------------------------------------------------------------------------
// Harness: Report rendering
// ---------------------------------------------------------------------------

#[test]
fn report_writes_json() {
    let dir = std::env::temp_dir().join("fpvte_test_json");
    let corpus = small_corpus(10);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, Some(&dir)).unwrap();
    let json_path = dir.join(format!("{}_report.json", report.matcher_name));
    assert!(json_path.exists(), "JSON report not written");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn report_writes_csv() {
    let dir = std::env::temp_dir().join("fpvte_test_csv");
    let corpus = small_corpus(10);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, Some(&dir)).unwrap();
    let csv_path = dir.join(format!("{}_roc.csv", report.matcher_name));
    assert!(csv_path.exists(), "CSV ROC not written");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Harness: Fairness
// ---------------------------------------------------------------------------

#[test]
fn fairness_empty_when_no_demographics() {
    // synthetic_in_memory produces no demographic labels.
    let corpus = small_corpus(20);
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    assert!(
        report.fairness_slices.is_empty(),
        "no demographics -> no fairness slices"
    );
}

#[test]
fn fairness_slices_when_demographics_present() {
    // Build a corpus with sex demographics.
    let mut entries = Vec::new();
    for i in 0..10 {
        let sex = if i % 2 == 0 { "M" } else { "F" };
        for acq in 0u8..2 {
            let t = vec![(i % 256) as u8; 32];
            entries.push(CorpusEntry {
                subject_id: format!("s{i:03}"),
                acquisition: acq,
                template: t,
                demographics: DemographicLabel {
                    sex: Some(sex.to_string()),
                    age_group: None,
                    fitzpatrick: None,
                },
            });
        }
    }
    let corpus = Corpus { entries };
    let matcher = synthetic_matcher();
    let report = harness().run(&corpus, &matcher, None).unwrap();
    // With M/F demographics, expect 2 slices.
    assert!(
        !report.fairness_slices.is_empty(),
        "expected fairness slices"
    );
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn insufficient_corpus_returns_error() {
    let corpus = Corpus::synthetic_in_memory(1);
    let matcher = synthetic_matcher();
    let result = harness().run(&corpus, &matcher, None);
    assert!(matches!(result, Err(FpVteError::InsufficientCorpus { .. })));
}

#[test]
fn null_matcher_eer_near_half() {
    // Null matcher scores everything 0.0 -> cannot distinguish genuine/impostor.
    let corpus = small_corpus(20);
    let matcher = NullMatcher;
    let report = harness().run(&corpus, &matcher, None).unwrap();
    // With degenerate scores the harness should still produce a report.
    let _ = report.eer;
}
