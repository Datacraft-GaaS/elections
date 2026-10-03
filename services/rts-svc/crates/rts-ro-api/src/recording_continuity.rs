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

//! N5: recording-session continuity certification.
//!
//! Session-recording anchors are captured on-device every 60 s and each gets
//! a monotonically increasing sequence number AT CAPTURE TIME (not arrival
//! time). Over low-connectivity links those anchors reach this service LATE,
//! OUT OF ORDER, DUPLICATED, or IN BATCHES — so continuity is defined purely
//! over the captured sequence space: a gap is any missing seq within
//! `first..=last`, no matter when or in what grouping its neighbors arrived.
//! Certification is a pure function over the anchors known at call time;
//! re-running it after more batches land converges to the same verdict.

use serde::{Deserialize, Serialize};

/// One on-device capture anchor. `seq` is issued by the recorder as a
/// monotonic counter at capture time; arrival order here carries no meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorPoint {
    pub seq: u64,
    pub hash_hex: String,
}

/// Verdict on the anchor chain for one session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Certification {
    /// Every seq in first..=last present and session stopped cleanly.
    Certified { anchors: usize },
    /// At least one seq missing in first..=last (listed ascending).
    Gaps { missing_seqs: Vec<u64> },
    /// No anchors recorded even though the session was opened.
    NotStarted,
    /// Contiguous chain but no stop flag observed yet.
    Unclosed,
}

/// JSON request body for POST /api/v1/rts/ro/session-recording/certify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifyRequest {
    pub session_id: String,
    pub anchors: Vec<AnchorPoint>,
    pub first_seq: u64,
    pub last_seq: u64,
    pub opened: bool,
    pub stopped: bool,
}

/// Certify continuity of an anchor chain.
///
/// - Duplicates are ignored; input need not be sorted (batch arrivals).
/// - Any missing seq in `first..=last` is a gap regardless of WHEN batches
///   arrived -> [`Certification::Gaps`] (gap reporting wins over unclosed).
/// - Empty anchors on an opened session -> [`Certification::NotStarted`].
/// - Missing stop flag with otherwise-contiguous chain -> [`Certification::Unclosed`].
pub fn certify_session(
    anchors: &[AnchorPoint],
    first_seq: u64,
    last_seq: u64,
    opened: bool,
    stopped: bool,
) -> Certification {
    if !opened || anchors.is_empty() {
        return Certification::NotStarted;
    }

    // Deduplicate by seq; sort ascending. Arrival order/duplication irrelevant.
    let mut seqs: Vec<u64> = anchors.iter().map(|a| a.seq).collect();
    seqs.sort_unstable();
    seqs.dedup();

    let mut missing_seqs = Vec::new();
    for expected in first_seq..=last_seq {
        if !seqs.contains(&expected) {
            missing_seqs.push(expected);
        }
    }

    if !missing_seqs.is_empty() {
        return Certification::Gaps { missing_seqs };
    }
    if !stopped {
        return Certification::Unclosed;
    }
    Certification::Certified {
        anchors: seqs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(seq: u64) -> AnchorPoint {
        AnchorPoint {
            seq,
            hash_hex: format!("{seq:064x}"),
        }
    }

    #[test]
    fn contiguous_chain_certifies() {
        let anchors: Vec<AnchorPoint> = (1..=5).map(a).collect();
        assert_eq!(
            certify_session(&anchors, 1, 5, true, true),
            Certification::Certified { anchors: 5 }
        );
    }

    #[test]
    fn dropped_seq_listed() {
        let anchors: Vec<AnchorPoint> = vec![1, 2, 4, 5].into_iter().map(a).collect();
        assert_eq!(
            certify_session(&anchors, 1, 5, true, true),
            Certification::Gaps {
                missing_seqs: vec![3]
            }
        );
    }

    #[test]
    fn out_of_order_batch_arrival_certifies_identically_to_in_order() {
        // KEY low-connectivity property: batched/out-of-order/duplicated
        // arrival must yield exactly the same verdict as clean in-order feed.
        let in_order: Vec<AnchorPoint> = (1..=6).map(a).collect();
        let messy: Vec<AnchorPoint> = vec![
            a(4),
            a(1),
            a(4), // duplicate
            a(6),
            a(2),
            a(2), // duplicate
            a(5),
            a(3),
        ];
        let expect = Certification::Certified { anchors: 6 };
        assert_eq!(certify_session(&in_order, 1, 6, true, true), expect);
        assert_eq!(certify_session(&messy, 1, 6, true, true), expect);
    }

    #[test]
    fn gaps_identical_for_messy_vs_clean_arrival() {
        let clean: Vec<AnchorPoint> = vec![1, 2, 5, 6].into_iter().map(a).collect();
        let messy: Vec<AnchorPoint> = vec![a(6), a(2), a(1), a(5), a(1)];
        assert_eq!(
            certify_session(&clean, 1, 6, true, false),
            Certification::Gaps {
                missing_seqs: vec![3, 4]
            }
        );
        assert_eq!(
            certify_session(&messy, 1, 6, true, false),
            Certification::Gaps {
                missing_seqs: vec![3, 4]
            }
        );
    }

    #[test]
    fn unclosed_flagged_when_stop_missing() {
        let anchors: Vec<AnchorPoint> = (1..=4).map(a).collect();
        assert_eq!(
            certify_session(&anchors, 1, 4, true, false),
            Certification::Unclosed
        );
    }

    #[test]
    fn gap_report_wins_over_unclosed() {
        // Unclosed AND gapped still reports the gaps.
        let anchors: Vec<AnchorPoint> = vec![a(1), a(3)];
        assert_eq!(
            certify_session(&anchors, 1, 4, true, false),
            Certification::Gaps {
                missing_seqs: vec![2, 4]
            }
        );
    }

    #[test]
    fn empty_opened_is_not_started() {
        assert_eq!(
            certify_session(&[], 1, 10, true, false),
            Certification::NotStarted
        );
    }

    #[test]
    fn duplicates_harmless_on_certified_path() {
        let anchors: Vec<AnchorPoint> = vec![a(1), a(1), a(2), a(2), a(3)];
        assert_eq!(
            certify_session(&anchors, 1, 3, true, true),
            Certification::Certified { anchors: 3 }
        );
    }

    #[test]
    fn partial_batch_arrival_lists_all_not_yet_received_seqs() {
        // Device manifest says 1..=10 were captured; only a first batch
        // (3,4) has arrived so far -> everything else is still missing and
        // will clear as later batches land (pure re-run, no network needed).
        let anchors: Vec<AnchorPoint> = vec![a(3), a(4)];
        let expect: Vec<u64> = (1..=10u64).filter(|s| *s != 3 && *s != 4).collect();
        assert_eq!(
            certify_session(&anchors, 1, 10, true, false),
            Certification::Gaps {
                missing_seqs: expect
            }
        );
    }
}
