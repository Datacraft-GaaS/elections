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

//! Online competency exam engine (workforce-lifecycle §3.2).
//!
//! Domain types and PURE scoring/sampling logic. Sampling mirrors the
//! governance-svc RLA `draw_sample_positions` technique: SHA-256 in counter
//! mode over a seed digest, rejection sampling without replacement, so any
//! observer can recompute a candidate's paper from `(bank, blueprint, seed)`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RoleLevel {
    Po,
    Dpo,
    Clerk,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionSpec {
    pub name: String,
    pub question_count: u32,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamBlueprint {
    pub blueprint_id: String,
    pub role_code: RoleLevel,
    pub sections: Vec<SectionSpec>,
    pub pass_mark_pct: f32,
    pub duration_secs: i64,
    pub attempts_allowed: u32,
    pub active_from: Option<DateTime<Utc>>,
    pub active_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub question_id: String,
    pub role_level: RoleLevel,
    pub section: String,
    pub difficulty: u8,
    /// BCP-47 tag, "en" or "sw".
    pub language: String,
    pub body: String,
    pub options: Vec<String>,
    pub correct_indexes: Vec<usize>,
    pub active_from: Option<DateTime<Utc>>,
    pub active_to: Option<DateTime<Utc>>,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Issued,
    InProgress,
    Submitted,
    Scored,
    Voided,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamSession {
    pub session_id: String,
    pub application_id: String,
    pub blueprint_id: String,
    pub attempt: u32,
    pub seed_hex: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub question_ids: Vec<String>,
    pub receipt_hash_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamResult {
    pub session_id: String,
    pub application_id: String,
    pub blueprint_id: String,
    pub attempt: u32,
    /// (section name, percentage 0..=100)
    pub section_scores: Vec<(String, f32)>,
    pub total_pct: f32,
    pub passed: bool,
    /// Signature anchoring to the transparency log is out of scope here; this
    /// carries the receipt hash value itself.
    pub signed_receipt_hash_hex: String,
    /// Cohort channel marker: "digital" for online sessions, "paper" for
    /// OMR-style batch imports (§3.2 offline cohort fallback).
    #[serde(default)]
    pub channel: String,
}

/// Default per-role blueprints seeded into the store at boot. Pass marks and
/// durations follow §3.2 examples (PO 80%, clerk 70%; PO/DPO differ in counts).
pub fn default_blueprints() -> Vec<ExamBlueprint> {
    fn sections(specs: &[(&str, u32, f32)]) -> Vec<SectionSpec> {
        specs
            .iter()
            .map(|(name, count, weight)| SectionSpec {
                name: (*name).to_string(),
                question_count: *count,
                weight: *weight,
            })
            .collect()
    }
    vec![
        ExamBlueprint {
            blueprint_id: "bp-po-v1".into(),
            role_code: RoleLevel::Po,
            sections: sections(&[
                ("law_procedure", 6, 0.35),
                ("ethics_secrecy", 4, 0.25),
                ("biometric_operation", 4, 0.20),
                ("incident_handling", 3, 0.20),
            ]),
            pass_mark_pct: 80.0,
            duration_secs: 5400,
            attempts_allowed: 2,
            active_from: None,
            active_to: None,
        },
        ExamBlueprint {
            blueprint_id: "bp-dpo-v1".into(),
            role_code: RoleLevel::Dpo,
            sections: sections(&[
                ("law_procedure", 5, 0.40),
                ("ethics_secrecy", 5, 0.35),
                ("biometric_operation", 3, 0.25),
            ]),
            pass_mark_pct: 75.0,
            duration_secs: 4500,
            attempts_allowed: 2,
            active_from: None,
            active_to: None,
        },
        ExamBlueprint {
            blueprint_id: "bp-clerk-v1".into(),
            role_code: RoleLevel::Clerk,
            sections: sections(&[
                ("law_procedure", 4, 0.40),
                ("ethics_secrecy", 3, 0.30),
                ("biometric_operation", 3, 0.30),
            ]),
            pass_mark_pct: 70.0,
            duration_secs: 3600,
            attempts_allowed: 3,
            active_from: None,
            active_to: None,
        },
    ]
}

fn is_active(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    from.is_none_or(|f| now >= f) && to.is_none_or(|t| now <= t)
}

impl Question {
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        is_active(self.active_from, self.active_to, now)
    }
}

impl ExamBlueprint {
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        is_active(self.active_from, self.active_to, now)
    }
}

/// Deterministic draw of `n` items without replacement using SHA-256 in
/// counter mode over the seed digest (same PRNG construction as the RLA
/// engine's `draw_sample_positions`). When fewer than `n` candidates exist
/// every candidate is drawn, in personalised order.
fn seeded_draw<T, F>(candidates: &[T], n: usize, key: F, seed_bytes: &[u8]) -> Vec<usize>
where
    F: Fn(&T) -> &str,
{
    let seed_digest = Sha256::digest(seed_bytes);
    let total = candidates.len();
    if total == 0 || n == 0 {
        return Vec::new();
    }
    let mut seen: HashSet<u64> = HashSet::new();
    let mut out = Vec::with_capacity(n.min(total));
    let mut counter = 0u64;
    while out.len() < n.min(total) {
        // hash(seed_digest || counter_be) -> uniform index over candidates.
        let mut h = Sha256::new();
        h.update(seed_digest);
        h.update(counter.to_be_bytes());
        counter += 1;
        let bytes = h.finalize();
        let idx = u64::from_be_bytes(bytes[..8].try_into().expect("8 bytes")) % total as u64;

        if !seen.insert(idx) {
            continue; // rejection sampling keeps the draw without replacement
        }
        debug_assert!(key(&candidates[idx as usize]).len() <= u64::MAX as usize);
        out.push(idx as usize);
    }
    out
}

/// Sample question ids for a session: for each blueprint section (in declared
/// order) filter the bank by active-at-now questions matching the blueprint's
/// role level and the section name, then draw exactly `question_count` ids in
/// seed-personalised order.
pub fn sample_questions(
    bank: &[Question],
    blueprint: &ExamBlueprint,
    seed_hex: &str,
) -> Vec<String> {
    let now = Utc::now();
    // Seed is expected hex-encoded; fall back to raw bytes so callers passing
    // an opaque string still get a deterministic stream.
    let seed_bytes = hex::decode(seed_hex).unwrap_or_else(|_| seed_hex.as_bytes().to_vec());

    let mut sampled = Vec::new();
    for section in &blueprint.sections {
        let eligible: Vec<&Question> = bank
            .iter()
            .filter(|q| {
                q.role_level == blueprint.role_code
                    && q.section == section.name
                    && q.is_active_at(now)
            })
            .collect();
        for idx in seeded_draw(
            &eligible,
            section.question_count as usize,
            |q| &q.question_id,
            &seed_bytes,
        ) {
            sampled.push(eligible[idx].question_id.clone());
        }
    }
    sampled
}

/// Canonical receipt hash: SHA-256 over the canonical (key-sorted) JSON of the
/// result WITHOUT the signature field. Anyone with the result can recompute it.
fn receipt_hash(result: &ExamResult) -> String {
    let payload = serde_json::json!({
        "session_id": result.session_id,
        "application_id": result.application_id,
        "blueprint_id": result.blueprint_id,
        "attempt": result.attempt,
        "section_scores": result.section_scores,
        "total_pct": result.total_pct,
        "passed": result.passed,
    });
    let canonical = serde_json::to_string(&payload).expect("result serialises");
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

/// Score a submitted session against its blueprint. Multi-select questions are
/// all-or-nothing: the answer set must equal the correct set exactly. Section
/// percentages are weighted by the blueprint weights; passed iff
/// `total_pct >= pass_mark_pct`. Answers absent from `answers` count as wrong.
pub fn score_session(
    session: &ExamSession,
    blueprint: &ExamBlueprint,
    bank: &[Question],
    answers: &HashMap<String, Vec<usize>>,
) -> ExamResult {
    let by_id: HashMap<&str, &Question> =
        bank.iter().map(|q| (q.question_id.as_str(), q)).collect();

    let mut section_scores = Vec::with_capacity(blueprint.sections.len());
    let mut weight_sum = 0.0f32;
    let mut weighted_total = 0.0f32;
    for section in &blueprint.sections {
        let sampled_in_section: Vec<&&Question> = session
            .question_ids
            .iter()
            .filter_map(|id| by_id.get(id.as_str()))
            .filter(|q| q.section == section.name)
            .collect();
        let correct = sampled_in_section
            .iter()
            .filter(|q| {
                answers.get(&q.question_id).is_some_and(|given| {
                    let mut g = given.clone();
                    g.sort_unstable();
                    g.dedup();
                    let mut c = q.correct_indexes.clone();
                    c.sort_unstable();
                    c.dedup();
                    g == c
                })
            })
            .count() as f32;
        let pct = if sampled_in_section.is_empty() {
            0.0
        } else {
            correct / sampled_in_section.len() as f32 * 100.0
        };
        section_scores.push((section.name.clone(), pct));
        weighted_total += pct * section.weight;
        weight_sum += section.weight;
    }
    let total_pct = if weight_sum > 0.0 {
        weighted_total / weight_sum
    } else {
        0.0
    };
    let passed = total_pct >= blueprint.pass_mark_pct;

    let mut result = ExamResult {
        session_id: session.session_id.clone(),
        application_id: session.application_id.clone(),
        blueprint_id: session.blueprint_id.clone(),
        attempt: session.attempt,
        section_scores,
        total_pct,
        passed,
        signed_receipt_hash_hex: String::new(),
        channel: "digital".to_string(),
    };
    result.signed_receipt_hash_hex = receipt_hash(&result);
    result
}

/// Receipt hash for a pre-built result (used by the paper batch import where
/// no digital session/answers exist).
pub fn compute_receipt_hash(result: &ExamResult) -> String {
    receipt_hash(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bp(section_counts: &[(&str, u32, f32)]) -> ExamBlueprint {
        ExamBlueprint {
            blueprint_id: "bp-test".into(),
            role_code: RoleLevel::Po,
            sections: section_counts
                .iter()
                .map(|(name, count, weight)| SectionSpec {
                    name: (*name).to_string(),
                    question_count: *count,
                    weight: *weight,
                })
                .collect(),
            pass_mark_pct: 50.0,
            duration_secs: 600,
            attempts_allowed: 2,
            active_from: None,
            active_to: None,
        }
    }

    fn q(id: &str, section: &str, correct: Vec<usize>, active: bool) -> Question {
        let window = if active {
            (None, None)
        } else {
            (Some(Utc::now() + chrono::Duration::days(1)), None)
        };
        Question {
            question_id: id.into(),
            role_level: RoleLevel::Po,
            section: section.into(),
            difficulty: 2,
            language: "en".into(),
            body: format!("body {id}"),
            options: vec!["a".into(), "b".into(), "c".into()],
            correct_indexes: correct,
            active_from: window.0,
            active_to: window.1,
            version: 1,
        }
    }

    fn bank_for(counts: &[(&str, u8)], active: bool) -> Vec<Question> {
        let mut out = Vec::new();
        for (section, n) in counts {
            for i in 0..*n {
                out.push(q(&format!("{section}-{i:02}"), section, vec![0], active));
            }
        }
        out
    }

    #[test]
    fn sampling_is_deterministic_per_seed_and_differs_across_seeds() {
        let b = bp(&[("law", 3, 1.0)]);
        let bank = bank_for(&[("law", 10)], true);
        let s1 = sample_questions(&bank, &b, "deadbeef");
        let s2 = sample_questions(&bank, &b, "deadbeef");
        let s3 = sample_questions(&bank, &b, "cafebabe");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3, "different seeds must personalise order");
        assert_eq!(s1.len(), 3);
        assert_eq!(s1.iter().collect::<HashSet<_>>().len(), 3, "no duplicates");
    }

    #[test]
    fn sampling_excludes_inactive_questions_and_respects_section_counts() {
        let b = bp(&[("law", 2, 0.5), ("ethics", 1, 0.5)]);
        let mut bank = bank_for(&[("law", 5), ("ethics", 3)], true);
        bank.push(q("law-inactive", "law", vec![0], false));
        let sampled = sample_questions(&bank, &b, "00ff");
        assert_eq!(sampled.len(), 3);
        assert!(!sampled.contains(&"law-inactive".to_string()));
        assert_eq!(
            sampled.iter().filter(|id| id.starts_with("law-")).count(),
            2
        );
        assert_eq!(
            sampled
                .iter()
                .filter(|id| id.starts_with("ethics-"))
                .count(),
            1
        );
    }

    #[test]
    fn sampling_with_insufficient_bank_returns_all_available() {
        let b = bp(&[("law", 5, 1.0)]);
        let bank = bank_for(&[("law", 2)], true);
        let sampled = sample_questions(&bank, &b, "ab");
        assert_eq!(sampled.len(), 2);
    }

    fn session(ids: Vec<String>) -> ExamSession {
        ExamSession {
            session_id: "sess-1".into(),
            application_id: "app-1".into(),
            blueprint_id: "bp-test".into(),
            attempt: 1,
            seed_hex: "ab".into(),
            status: SessionStatus::InProgress,
            started_at: Utc::now(),
            deadline: Utc::now() + chrono::Duration::seconds(600),
            question_ids: ids,
            receipt_hash_hex: None,
        }
    }

    #[test]
    fn all_or_nothing_multi_select_scoring() {
        let mut b = bp(&[("sec", 2, 1.0)]);
        b.pass_mark_pct = 60.0;
        let bank = vec![
            q("m1", "sec", vec![0, 2], true),
            q("m2", "sec", vec![1], true),
        ];
        let s = session(vec!["m1".into(), "m2".into()]);
        // m1 exact match scores; partial [0] must NOT score; m2 wrong.
        let mut answers = HashMap::new();
        answers.insert("m1".to_string(), vec![0, 2]);
        answers.insert("m2".to_string(), vec![0]);
        let r = score_session(&s, &b, &bank, &answers);
        assert_eq!(r.section_scores, vec![("sec".to_string(), 50.0)]);
        assert!(!r.passed);

        let mut answers = HashMap::new();
        answers.insert("m1".to_string(), vec![0]); // partial subset
        answers.insert("m2".to_string(), vec![1]);
        let r = score_session(&s, &b, &bank, &answers);
        assert_eq!(r.section_scores, vec![("sec".to_string(), 50.0)]);
    }

    #[test]
    fn pass_boundary_is_exact_pass_mark() {
        let mut b = bp(&[("sec", 4, 1.0)]);
        b.pass_mark_pct = 75.0;
        let bank = bank_for(&[("sec", 4)], true);
        let ids = bank.iter().map(|q| q.question_id.clone()).collect();
        let s = session(ids);

        let mut answers = HashMap::new();
        for i in 0..3 {
            answers.insert(format!("sec-{i:02}"), vec![0]);
        }
        let r = score_session(&s, &b, &bank, &answers);
        assert_eq!(r.total_pct, 75.0);
        assert!(r.passed, "exactly at pass mark must pass");

        let mut answers = HashMap::new();
        for i in 0..2 {
            answers.insert(format!("sec-{i:02}"), vec![0]);
        }
        let r = score_session(&s, &b, &bank, &answers);
        assert!(!r.passed);
    }

    #[test]
    fn weights_drive_total_across_sections() {
        let b = bp(&[("a", 1, 0.25), ("bb", 2, 0.75)]);
        let bank = vec![
            q("a-00", "a", vec![0], true),
            q("bb-00", "bb", vec![0], true),
            q("bb-01", "bb", vec![0], true),
        ];
        let s = session(vec!["a-00".into(), "bb-00".into(), "bb-01".into()]);
        let mut answers = HashMap::new();
        answers.insert("a-00".to_string(), vec![0]); // 100% of a
        answers.insert("bb-00".to_string(), vec![0]); // bb: 1/2 = 50%
        let r = score_session(&s, &b, &bank, &answers);
        assert_eq!(r.section_scores[0], ("a".to_string(), 100.0));
        assert_eq!(r.section_scores[1], ("bb".to_string(), 50.0));
        assert!((r.total_pct - 62.5).abs() < 1e-6);
        assert!(r.passed); // pass_mark 50

        // Receipt hash is stable and independent of the signature field.
        let h1 = compute_receipt_hash(&r);
        let mut r2 = r.clone();
        r2.signed_receipt_hash_hex = "tampered".into();
        assert_eq!(h1, compute_receipt_hash(&r2));
    }

    #[test]
    fn unanswered_questions_score_zero() {
        let b = bp(&[("sec", 2, 1.0)]);
        let bank = bank_for(&[("sec", 2)], true);
        let s = session(bank.iter().map(|q| q.question_id.clone()).collect());
        let r = score_session(&s, &b, &bank, &HashMap::new());
        assert_eq!(r.total_pct, 0.0);
        assert!(!r.passed);
    }

    #[test]
    fn other_role_level_questions_are_never_sampled() {
        let b = bp(&[("law", 2, 1.0)]);
        let mut bank = bank_for(&[("law", 4)], true);
        let mut clerk_q = q("clerk-only", "law", vec![0], true);
        clerk_q.role_level = RoleLevel::Clerk;
        bank.push(clerk_q);
        let sampled = sample_questions(&bank, &b, "77");
        assert!(!sampled.contains(&"clerk-only".to_string()));
    }
}
