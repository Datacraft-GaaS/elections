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

use eel_abis_cert_fido::{
    evidence::{EvidencePack, FmrFnmrReport, PadReport},
    matrix::{AttackMedium, AttackType, ExpectedOutcome, PaiMatrix, PaiMatrixEntry, PaiSpecies},
};
use proptest::prelude::*;

fn sample_matrix() -> PaiMatrix {
    let mut m = PaiMatrix::new();
    m.add(PaiMatrixEntry {
        attack_type: AttackType::Artefact,
        species: PaiSpecies::Face,
        medium: AttackMedium::Print,
        expected_outcome: ExpectedOutcome::Blocked,
        apcer: 0.01,
    })
    .unwrap();
    m.add(PaiMatrixEntry {
        attack_type: AttackType::VideoReplay,
        species: PaiSpecies::Face,
        medium: AttackMedium::Screen,
        expected_outcome: ExpectedOutcome::Blocked,
        apcer: 0.02,
    })
    .unwrap();
    m
}

fn sample_fmr_fnmr(fmr: f64, fnmr: f64) -> FmrFnmrReport {
    FmrFnmrReport {
        threshold: 0.5,
        fmr,
        fnmr,
        genuine_attempts: 10_000,
        impostor_attempts: 100_000,
    }
}

fn sample_pad_report() -> PadReport {
    PadReport {
        species_tested: 8,
        species_passed: 8,
        max_apcer: 0.01,
        max_bpcer: 0.05,
    }
}

// ---- Evidence pack ----

#[test]
fn evidence_pack_assembles_within_targets() {
    let pack = EvidencePack::assemble(
        "eel-face-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.001, 0.01),
        sample_matrix(),
        0.01,
        0.05,
    )
    .unwrap();
    assert_eq!(pack.component_id, "eel-face-v1");
    assert!(!pack.tester_signature.is_empty());
}

#[test]
fn evidence_pack_fmr_exceeded_errors() {
    let result = EvidencePack::assemble(
        "eel-face-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.05, 0.01),
        sample_matrix(),
        0.01, // target FMR
        0.05,
    );
    assert!(result.is_err());
}

#[test]
fn evidence_pack_fnmr_exceeded_errors() {
    let result = EvidencePack::assemble(
        "eel-face-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.001, 0.10),
        sample_matrix(),
        0.01,
        0.05, // target FNMR
    );
    assert!(result.is_err());
}

#[test]
fn evidence_pack_empty_matrix_errors() {
    let result = EvidencePack::assemble(
        "eel-face-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.001, 0.01),
        PaiMatrix::new(),
        0.01,
        0.05,
    );
    assert!(result.is_err());
}

#[test]
fn evidence_signature_is_deterministic() {
    let p1 = EvidencePack::assemble(
        "eel-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.001, 0.01),
        sample_matrix(),
        0.01,
        0.05,
    )
    .unwrap();
    let p2 = EvidencePack::assemble(
        "eel-v1",
        sample_pad_report(),
        sample_fmr_fnmr(0.001, 0.01),
        sample_matrix(),
        0.01,
        0.05,
    )
    .unwrap();
    assert_eq!(p1.tester_signature, p2.tester_signature);
}

// ---- PAI matrix ----

#[test]
fn matrix_add_entry_success() {
    let mut m = PaiMatrix::new();
    m.add(PaiMatrixEntry {
        attack_type: AttackType::Artefact,
        species: PaiSpecies::Fingerprint,
        medium: AttackMedium::Silicone,
        expected_outcome: ExpectedOutcome::Blocked,
        apcer: 0.005,
    })
    .unwrap();
    assert_eq!(m.entries.len(), 1);
}

#[test]
fn matrix_duplicate_entry_errors() {
    let mut m = PaiMatrix::new();
    let entry = PaiMatrixEntry {
        attack_type: AttackType::Artefact,
        species: PaiSpecies::Iris,
        medium: AttackMedium::Print,
        expected_outcome: ExpectedOutcome::Blocked,
        apcer: 0.01,
    };
    m.add(entry.clone()).unwrap();
    assert!(m.add(entry).is_err());
}

#[test]
fn matrix_max_apcer_computed() {
    let m = sample_matrix();
    assert!((m.max_apcer() - 0.02).abs() < 1e-9);
}

#[test]
fn matrix_blocked_entries_count() {
    let m = sample_matrix();
    assert_eq!(m.blocked_entries().count(), 2);
}

#[test]
fn matrix_default_is_empty() {
    let m = PaiMatrix::default();
    assert_eq!(m.entries.len(), 0);
}

// ---- Proptest ----

proptest! {
    #[test]
    fn prop_evidence_signature_not_empty(
        fmr in 0.0f64..0.009f64,
        fnmr in 0.0f64..0.04f64,
    ) {
        let pack = EvidencePack::assemble(
            "eel-test",
            sample_pad_report(),
            sample_fmr_fnmr(fmr, fnmr),
            sample_matrix(),
            0.01,
            0.05,
        ).unwrap();
        prop_assert!(!pack.tester_signature.is_empty());
    }
}
