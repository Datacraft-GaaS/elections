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

//! The §39 legal-basis rules engine.
//!
//! The crate is described as "§39 rules engine, per-event compliance verdicts,
//! tlog-hashed (WP-5.1)", and every one of those three claims was absent: the
//! six `/legal-audit/*` handlers answered from `vec![]` or
//! `Default::default()`, `get_verdict` returned an **empty** `ComplianceVerdict`
//! for any record id, and `generate_report` answered `202 Accepted` with no body.
//! An empty verdict is the dangerous shape here: `passed` is `false` by default,
//! so an unevaluated record reads as a *failed* compliance check with a rule id
//! of `""` -- a verdict that looks authoritative and means nothing.
//!
//! Rules are matched on the record's declared `legal_basis` against each rule's
//! `applies_to` set. A record that declares no basis at all is non-compliant,
//! because there is no instrument to justify the transmission.
//!
//! The verdict hash is a real hash chain over prior verdicts rather than a
//! per-record digest: it commits each verdict to everything recorded before it,
//! so removing or reordering a verdict changes every subsequent hash. It is a
//! local chain over this process's store, **not** a transparency-log inclusion
//! proof -- there is no append-only log in this workspace to submit to. That
//! distinction is carried on the verdict itself as `tlog_inclusion_proven: false`.

use sha2::{Digest, Sha256};

use crate::models::{ComplianceVerdict, LegalBasisRule, PartySummary, TransmissionRecord};

/// The built-in §39 rule set.
///
/// `applies_to` lists the statutory instruments a transmission may declare as its
/// legal basis. Section references are to the Constitution of Kenya 2010 and the
/// the Commission and OAG enabling statutes this service audits against.
pub fn rules() -> Vec<LegalBasisRule> {
    vec![
        LegalBasisRule {
            rule_id: "s39-elections-act".to_string(),
            section_ref: "Art. 39(2)(a)".to_string(),
            description: "Results transmitted under the Elections Act".to_string(),
            applies_to: vec![
                "s39-elections-act".to_string(),
                "COMMISSION-2024-07".to_string(),
            ],
        },
        LegalBasisRule {
            rule_id: "s39-constitution".to_string(),
            section_ref: "Art. 39(2)(b)".to_string(),
            description: "Constitutory transmission of electoral records".to_string(),
            applies_to: vec!["s39-constitution".to_string()],
        },
        LegalBasisRule {
            rule_id: "oag-mandate".to_string(),
            section_ref: "OAG Act s.7".to_string(),
            description: "Transmission to the Office of the Auditor-General".to_string(),
            applies_to: vec!["OAG-2024-11".to_string()],
        },
        LegalBasisRule {
            rule_id: "data-protection".to_string(),
            section_ref: "DPA 2019 s.31".to_string(),
            description: "Personal data processed under the Data Protection Act".to_string(),
            applies_to: vec!["DPA-2019-04".to_string()],
        },
    ]
}

/// Rules that apply to a record, by its declared legal basis.
fn matching_rules(record: &TransmissionRecord) -> Vec<LegalBasisRule> {
    rules()
        .into_iter()
        .filter(|r| r.applies_to.iter().any(|a| a == &record.legal_basis))
        .collect()
}

/// Rules a record was evaluated against but did not satisfy.
///
/// A record is non-compliant when it declares **no** basis (there is no
/// instrument to justify it), and when the basis it declares is unknown to the
/// rule set. A known basis with no matching rule still passes: the rule set
/// does not know that instrument, which is not evidence of a breach.
fn failed_rules(record: &TransmissionRecord) -> Vec<LegalBasisRule> {
    if record.legal_basis.trim().is_empty() {
        return rules();
    }
    if matching_rules(record).is_empty() {
        // Unknown instrument: flag it explicitly so an unrecognised basis is
        // visible rather than silently compliant.
        return vec![LegalBasisRule {
            rule_id: "unrecognised-basis".to_string(),
            section_ref: record.legal_basis.clone(),
            description: format!(
                "declared legal basis `{}` is not in the §39 rule set",
                record.legal_basis
            ),
            applies_to: vec![record.legal_basis.clone()],
        }];
    }
    Vec::new()
}

/// Evaluate one record into its verdicts.
///
/// `first_index` is this record's position in the **global** verdict sequence,
/// which may start above zero when earlier records produced verdicts first. The
/// position is hashed into each verdict, so it must advance across the whole
/// sequence rather than restarting per record — otherwise two records at the
/// same offset would produce identical hashes for the same rule and outcome.
pub fn evaluate_record(
    record: &TransmissionRecord,
    first_index: usize,
    prior: &str,
    evaluated_at: String,
) -> Vec<ComplianceVerdict> {
    let failed = failed_rules(record);
    // Every applicable rule is judged, so a record with two applicable rules
    // yields two verdicts rather than one verdict standing in for both.
    let mut applicable = matching_rules(record);
    if applicable.is_empty() && failed.is_empty() {
        applicable.push(LegalBasisRule {
            rule_id: "no-applicable-rule".to_string(),
            section_ref: String::new(),
            description: "no §39 rule applies to this transmission".to_string(),
            applies_to: vec![record.legal_basis.clone()],
        });
    }

    let failed_ids: Vec<String> = failed.iter().map(|f| f.rule_id.clone()).collect();
    let mut out: Vec<ComplianceVerdict> = Vec::new();
    // Each verdict chains to the one before it *within* this record too, not
    // just to the record before, so a multi-verdict record is an ordered chain
    // rather than N verdicts all pointing at the same predecessor.
    let mut running = prior.to_string();
    for (offset, rule) in applicable.into_iter().chain(failed).enumerate() {
        // A rule the record was matched against passes; a rule it failed is
        // present in `failed_ids` and does not. `unrecognised-basis` is always
        // a failure by construction.
        let passed = rule.rule_id != "unrecognised-basis" && !failed_ids.contains(&rule.rule_id);
        let v = verdict_for(
            record,
            &rule,
            passed,
            first_index + offset,
            &running,
            &evaluated_at,
        );
        running = v.verdict_hash.clone();
        out.push(v);
    }
    out
}

fn verdict_for(
    record: &TransmissionRecord,
    rule: &LegalBasisRule,
    passed: bool,
    verdict_index: usize,
    prior: &str,
    evaluated_at: &str,
) -> ComplianceVerdict {
    let mut h = Sha256::new();
    // The chain: prior hash, then every field that identifies this verdict.
    h.update(prior.as_bytes());
    h.update(b"\x1f");
    h.update(prior.len().to_string().as_bytes());
    h.update(b"\x1e");
    h.update(record.record_id.as_bytes());
    h.update(b"\x1e");
    h.update(record.event_type.as_bytes());
    h.update(b"\x1e");
    h.update(record.payload_hash.as_bytes());
    h.update(b"\x1e");
    h.update(rule.rule_id.as_bytes());
    h.update(b"\x1e");
    h.update(rule.section_ref.as_bytes());
    h.update(b"\x1e");
    h.update([u8::from(passed)].as_slice());
    h.update(b"\x1e");
    h.update(verdict_index.to_string().as_bytes());
    let verdict_hash = hex::encode(h.finalize());

    ComplianceVerdict {
        verdict_id: format!("vd-{}", &verdict_hash[..16]),
        record_id: record.record_id.clone(),
        rule_id: rule.rule_id.clone(),
        section_ref: rule.section_ref.clone(),
        passed,
        // The chain head, not a bare digest of this record.
        verdict_hash,
        // Explicitly not an inclusion proof: no append-only log exists here.
        tlog_inclusion_proven: false,
        evaluated_at: evaluated_at.to_string(),
    }
}

/// The chain head for a verdict list: the last verdict's hash, or the genesis
/// value for an empty list.
pub fn chain_head(verdicts: &[ComplianceVerdict]) -> String {
    verdicts
        .last()
        .map(|v| v.verdict_hash.clone())
        .unwrap_or_else(|| "genesis".to_string())
}

/// Check that every verdict id is derived from its own hash, returning the index
/// of the first that is not.
///
/// Verifying the *chain* itself needs the source records, because the hash covers
/// fields the stored verdict does not carry; [`verify_chain_with_records`] does
/// the full re-derivation. This is the cheap invariant that catches a verdict id
/// edited after the fact.
pub fn verify_chain(verdicts: &[ComplianceVerdict]) -> Option<usize> {
    verdicts.iter().position(|v| {
        let hash = v.verdict_hash.as_str();
        hash.len() < 16 || v.verdict_id != format!("vd-{}", &hash[..16])
    })
}

/// Re-derive every verdict hash from the records and return the index of the
/// first that does not reproduce, or `None` if the chain verifies end to end.
///
/// This is the check that gives the chain its value: it recomputes rather than
/// trusting the stored hash, so a record or verdict edited after evaluation is
/// caught at the position it was tampered.
pub fn verify_chain_with_records(
    verdicts: &[ComplianceVerdict],
    records: &[TransmissionRecord],
) -> Option<usize> {
    let mut prior = "genesis".to_string();
    for (index, v) in verdicts.iter().enumerate() {
        let Some(record) = records.iter().find(|r| r.record_id == v.record_id) else {
            return Some(index);
        };
        let Some(rule) = rules()
            .into_iter()
            .chain(failed_rules(record))
            .find(|r| r.rule_id == v.rule_id)
        else {
            return Some(index);
        };
        let expected = verdict_for(record, &rule, v.passed, index, &prior, &v.evaluated_at);
        if expected.verdict_hash != v.verdict_hash {
            return Some(index);
        }
        prior = v.verdict_hash.clone();
    }
    None
}

/// Aggregate per-party compliance from a party's transmissions.
pub fn summarise_party(
    party_id: &str,
    party_name: &str,
    records: &[&TransmissionRecord],
    verdicts: &[ComplianceVerdict],
) -> PartySummary {
    let mut compliant = 0u32;
    let mut non_compliant = 0u32;
    for record in records {
        // A record is compliant only when every verdict it produced passed.
        let mut any = false;
        let mut all_passed = true;
        for v in verdicts.iter().filter(|v| v.record_id == record.record_id) {
            any = true;
            if !v.passed {
                all_passed = false;
            }
        }
        if any && all_passed {
            compliant += 1;
        } else {
            // No verdict at all is not compliant: an unevaluated transmission
            // must not be counted as a pass.
            non_compliant += 1;
        }
    }
    PartySummary {
        party_id: party_id.to_string(),
        party_name: party_name.to_string(),
        total_transmissions: records.len() as u32,
        compliant,
        non_compliant,
    }
}

/// Reject a verdict whose hash does not reproduce.
#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, basis: &str) -> TransmissionRecord {
        TransmissionRecord {
            record_id: id.to_string(),
            event_type: "results-transmission".to_string(),
            sender: "party-a".to_string(),
            recipient: "Electoral Authority".to_string(),
            transmitted_at: "2026-08-01T10:00:00Z".to_string(),
            payload_hash: "sha256:deadbeef".to_string(),
            legal_basis: basis.to_string(),
        }
    }

    #[test]
    fn a_known_basis_passes() {
        let v = evaluate_record(&record("r1", "s39-elections-act"), 0, "genesis", "t".into());
        assert!(!v.is_empty());
        assert!(v.iter().all(|x| x.passed), "{v:?}");
    }

    #[test]
    fn no_declared_basis_fails_every_rule() {
        let v = evaluate_record(&record("r2", ""), 0, "genesis", "t".into());
        assert!(
            v.iter().all(|x| !x.passed),
            "an unjustified transmission cannot pass"
        );
        assert!(!v.is_empty());
    }

    #[test]
    fn an_unknown_basis_is_flagged_not_silently_accepted() {
        let v = evaluate_record(
            &record("r3", "made-up-instrument"),
            0,
            "genesis",
            "t".into(),
        );
        assert!(
            v.iter()
                .any(|x| !x.passed && x.rule_id == "unrecognised-basis"),
            "{v:?}"
        );
    }

    #[test]
    fn the_verdict_hash_is_chained_to_the_prior_one() {
        let a = evaluate_record(&record("r1", "s39-elections-act"), 0, "genesis", "t".into());
        let b_onto_genesis =
            evaluate_record(&record("r2", "s39-elections-act"), 1, "genesis", "t".into());
        let b_onto_a = evaluate_record(
            &record("r2", "s39-elections-act"),
            1,
            &chain_head(&a),
            "t".into(),
        );
        assert_ne!(
            b_onto_genesis[0].verdict_hash, b_onto_a[0].verdict_hash,
            "the same verdict must hash differently depending on what precedes it"
        );
    }

    #[test]
    fn the_chain_verifies_end_to_end() {
        let r1 = record("r1", "s39-elections-act");
        let r2 = record("r2", "");
        let mut all: Vec<ComplianceVerdict> = Vec::new();
        let records = vec![r1.clone(), r2.clone()];
        for rec in [&r1, &r2] {
            let produced = evaluate_record(rec, all.len(), &chain_head(&all), "t".into());
            all.extend(produced);
        }
        assert_eq!(verify_chain_with_records(&all, &records), None);
    }

    #[test]
    fn tampering_with_a_record_breaks_the_chain_at_that_position() {
        let r1 = record("r1", "s39-elections-act");
        let produced = evaluate_record(&r1, 0, "genesis", "t".into());
        let mut records = vec![r1.clone()];
        records[0].payload_hash = "sha256:tampered".to_string();
        assert!(
            verify_chain_with_records(&produced, &records).is_some(),
            "an edited record must break its verdict hash"
        );
    }

    #[test]
    fn verdict_id_is_derived_from_the_hash() {
        let v = evaluate_record(&record("r1", "s39-elections-act"), 0, "genesis", "t".into());
        assert!(verify_chain(&v).is_none());
        let mut tampered = v.clone();
        tampered[0].verdict_id = "vd-0000000000000000".to_string();
        assert_eq!(verify_chain(&tampered), Some(0));
    }

    #[test]
    fn tlog_inclusion_is_never_claimed() {
        let v = evaluate_record(&record("r1", "s39-elections-act"), 0, "genesis", "t".into());
        assert!(v.iter().all(|x| !x.tlog_inclusion_proven));
    }

    #[test]
    fn an_unevaluated_record_is_not_counted_as_compliant() {
        let rec = record("r1", "s39-elections-act");
        let records = vec![&rec];
        let s = summarise_party("p1", "Party One", &records, &[]);
        assert_eq!(s.total_transmissions, 1);
        assert_eq!(s.compliant, 0);
        assert_eq!(s.non_compliant, 1);
    }

    #[test]
    fn a_party_with_a_failing_verdict_is_non_compliant() {
        let rec = record("r1", "");
        let v = evaluate_record(&rec, 0, "genesis", "t".into());
        let s = summarise_party("p1", "Party One", &[&rec], &v);
        assert_eq!(s.non_compliant, 1);
        assert_eq!(s.compliant, 0);
    }
}
