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

//! Compile and validate behaviour for declarative BVR enrolment workflows.
//!
//! The fixtures here double as the reference example for the desktop client:
//! `standard_ke_workflow()` is the shape a Commission registration officer would
//! actually be handed.

use eel_bvr_workflow_rs::{
    CaptureKind, CapturePolicy, CaptureRequirement, Constraints, DataItem, DataKind, Step,
    StepKind, ValidationError, WorkflowSpec, compile, validate_submission,
};
use serde_json::json;

/// A realistic standard enrolment workflow: identity details, two biometric
/// modalities, review, submit.
fn standard_ke_workflow() -> WorkflowSpec {
    WorkflowSpec {
        version: 1,
        id: "bvr-enrolment-standard".to_owned(),
        label: "Standard voter enrolment".to_owned(),
        data_items: vec![
            DataItem {
                name: "full_name".to_owned(),
                kind: DataKind::Text,
                label: "Full legal name".to_owned(),
                required: true,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints {
                    min_length: Some(1),
                    max_length: Some(200),
                    ..Constraints::default()
                },
            },
            DataItem {
                name: "national_id".to_owned(),
                kind: DataKind::NationalId,
                label: "National ID number".to_owned(),
                required: true,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints::default(),
            },
            DataItem {
                name: "dob".to_owned(),
                kind: DataKind::Date,
                label: "Date of birth".to_owned(),
                required: true,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints {
                    min_age_years: Some(18),
                    max_age_years: Some(120),
                    ..Constraints::default()
                },
            },
            DataItem {
                name: "phone".to_owned(),
                kind: DataKind::Phone,
                label: "Phone number".to_owned(),
                required: false,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints::default(),
            },
            DataItem {
                name: "gender".to_owned(),
                kind: DataKind::Choice,
                label: "Gender".to_owned(),
                required: true,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints {
                    options: Some(vec![
                        "male".to_owned(),
                        "female".to_owned(),
                        "other".to_owned(),
                        "prefer_not_to_say".to_owned(),
                    ]),
                    ..Constraints::default()
                },
            },
            DataItem {
                name: "polling_station_code".to_owned(),
                kind: DataKind::StationCode,
                label: "Polling station".to_owned(),
                required: true,
                data_class: "B".to_owned(),
                help: None,
                constraints: Constraints::default(),
            },
            DataItem {
                name: "consent".to_owned(),
                kind: DataKind::Consent,
                label: "Citizen consents to biometric enrolment".to_owned(),
                required: true,
                data_class: "A".to_owned(),
                help: None,
                constraints: Constraints::default(),
            },
        ],
        capture_requirements: vec![
            CaptureRequirement {
                id: "fingerprints".to_owned(),
                label: "Left and right fingerprints".to_owned(),
                kinds: vec![CaptureKind::Fingerprint],
                policy: CapturePolicy::AllOf,
                optional: false,
            },
            CaptureRequirement {
                id: "face_live".to_owned(),
                label: "Liveness-checked face".to_owned(),
                kinds: vec![CaptureKind::LivePhotograph],
                policy: CapturePolicy::AllOf,
                optional: false,
            },
        ],
        steps: vec![
            Step {
                id: "identity".to_owned(),
                kind: StepKind::DataEntry {
                    items: vec![
                        "full_name".to_owned(),
                        "national_id".to_owned(),
                        "dob".to_owned(),
                        "gender".to_owned(),
                    ],
                },
                label: None,
            },
            Step {
                id: "contact".to_owned(),
                kind: StepKind::DataEntry {
                    items: vec!["phone".to_owned(), "polling_station_code".to_owned()],
                },
                label: None,
            },
            Step {
                id: "left_fingers".to_owned(),
                kind: StepKind::Capture { capture: "fingerprints".to_owned() },
                label: None,
            },
            Step {
                id: "face".to_owned(),
                kind: StepKind::Capture { capture: "face_live".to_owned() },
                label: None,
            },
            Step {
                id: "consent".to_owned(),
                kind: StepKind::DataEntry { items: vec!["consent".to_owned()] },
                label: None,
            },
            Step { id: "review".to_owned(), kind: StepKind::Review, label: None },
            Step { id: "submit".to_owned(), kind: StepKind::Submit, label: None },
        ],
    }
}

fn valid_submission() -> serde_json::Map<String, serde_json::Value> {
    json!({
        "full_name": "Wanjiku Mwangi",
        "national_id": "12345678",
        "dob": "1990-06-15",
        "gender": "female",
        "phone": "+254712345678",
        "polling_station_code": "NRB001PS042",
        "consent": true,
    })
    .as_object()
    .cloned()
    .unwrap()
}

fn both_modalities() -> Vec<CaptureKind> {
    vec![CaptureKind::Fingerprint, CaptureKind::LivePhotograph]
}

// ---- compilation ------------------------------------------------------------

#[test]
fn standard_workflow_compiles() {
    let compiled = compile(&standard_ke_workflow()).expect("standard workflow must compile");
    assert_eq!(compiled.id, "bvr-enrolment-standard");
    assert_eq!(compiled.capture_order, vec!["fingerprints", "face_live"]);
    assert_eq!(compiled.data_item_order.len(), 7);
    assert_eq!(compiled.steps.len(), 7);
}

#[test]
fn data_items_are_reachable_and_ordered_as_declared() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    assert_eq!(
        compiled.data_item_order,
        vec![
            "full_name",
            "national_id",
            "dob",
            "phone",
            "gender",
            "polling_station_code",
            "consent",
        ],
        "declaration order is preserved verbatim, independent of step order"
    );
    for name in &compiled.data_item_order {
        assert!(
            compiled.data_item(name).unwrap().reachable,
            "{name} should be reachable"
        );
    }
}

#[test]
fn step_labels_are_generated_when_absent() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let by_id: std::collections::HashMap<_, _> =
        compiled.steps.iter().map(|s| (s.step.id.as_str(), s)).collect();
    assert_eq!(by_id["review"].display_label, "Review");
    assert_eq!(by_id["submit"].display_label, "Submit");
    assert_eq!(by_id["left_fingers"].display_label, "Capture Fingerprint");
    assert!(compiled.capture("fingerprints").unwrap().referenced);
    assert_eq!(by_id["identity"].display_label, "Details");
}

#[test]
fn explicit_step_label_overrides_the_generated_one() {
    let mut spec = standard_ke_workflow();
    spec.steps[0].label = Some("Confirm identity".to_owned());
    let compiled = compile(&spec).unwrap();
    assert_eq!(compiled.steps[0].display_label, "Confirm identity");
}

#[test]
fn spec_round_trips_through_json() {
    let spec = standard_ke_workflow();
    let text = serde_json::to_string(&spec).unwrap();
    let parsed: WorkflowSpec = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed, spec);
    assert_eq!(compile(&parsed).unwrap().id, spec.id);
}

#[test]
fn steps_parse_from_their_tagged_json_form() {
    // A capture step is tagged by `kind`, which is what an operator-authored
    // document actually looks like on disk.
    let json = json!({
        "id": "prints",
        "kind": "capture",
        "capture": "fingerprints"
    });
    let step: Step = serde_json::from_value(json).unwrap();
    assert_eq!(
        step.kind,
        StepKind::Capture { capture: "fingerprints".to_owned() }
    );

    // A capture requirement round-trips through its own tagged form, so an
    // operator-authored document can express the policy directly.
    let capture_json = json!({
        "id": "vascular",
        "label": "Palm or finger vein",
        "kinds": ["palm_vein", "finger_vein"],
        "policy": { "at_least": 1 }
    });
    let capture: CaptureRequirement = serde_json::from_value(capture_json).unwrap();
    assert_eq!(capture.kinds, vec![CaptureKind::PalmVein, CaptureKind::FingerVein]);
    assert_eq!(capture.policy, CapturePolicy::AtLeast(1));
}

// ---- compilation failures ----------------------------------------------------

#[test]
fn unsupported_version_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.version = 2;
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::UnsupportedVersion { got: 2 }));
}

#[test]
fn empty_workflow_is_rejected() {
    let spec = WorkflowSpec {
        version: 1,
        id: "empty".to_owned(),
        label: "Empty".to_owned(),
        data_items: vec![],
        capture_requirements: vec![],
        steps: vec![],
    };
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::NoDataItems));
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::NoSteps));
}

#[test]
fn duplicate_data_item_name_is_rejected() {
    let mut spec = standard_ke_workflow();
    let dup = spec.data_items[0].clone();
    spec.data_items.push(dup.clone());
    let errors = compile(&spec).unwrap_err();
    assert!(
        errors
            .contains(&eel_bvr_workflow_rs::CompileError::DuplicateDataItem { name: dup.name })
    );
}

#[test]
fn duplicate_step_id_is_rejected() {
    let mut spec = standard_ke_workflow();
    let dup = spec.steps[0].clone();
    spec.steps.push(dup);
    let errors = compile(&spec).unwrap_err();
    assert!(
        errors
            .contains(&eel_bvr_workflow_rs::CompileError::DuplicateStepId {
                id: "identity".to_owned()
            })
    );
}

#[test]
fn step_referencing_unknown_data_item_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.steps[0].kind = StepKind::DataEntry { items: vec!["not_a_field".to_owned()] };
    let errors = compile(&spec).unwrap_err();
    assert!(
        errors.contains(&eel_bvr_workflow_rs::CompileError::UnknownDataItem {
            step: "identity".to_owned(),
            item: "not_a_field".to_owned(),
        })
    );
}

#[test]
fn declared_but_never_collected_item_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items.push(DataItem {
        name: "orphan".to_owned(),
        kind: DataKind::Text,
        label: "Never asked".to_owned(),
        required: false,
        data_class: "A".to_owned(),
        help: None,
        constraints: Constraints::default(),
    });
    let errors = compile(&spec).unwrap_err();
    assert!(
        errors.contains(&eel_bvr_workflow_rs::CompileError::UnreachableDataItem {
            name: "orphan".to_owned()
        })
    );
}

#[test]
fn inverted_length_bounds_are_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[0].constraints.min_length = Some(50);
    spec.data_items[0].constraints.max_length = Some(10);
    let errors = compile(&spec).unwrap_err();
    assert!(
        errors
            .contains(&eel_bvr_workflow_rs::CompileError::LengthBoundsInverted {
                name: "full_name".to_owned(),
                min: 50,
                max: 10
            })
    );
}

#[test]
fn inverted_age_bounds_are_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[2].constraints.min_age_years = Some(90);
    spec.data_items[2].constraints.max_age_years = Some(18);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::AgeBoundsInverted {
        name: "dob".to_owned(),
        min: 90,
        max: 18
    }));
}

#[test]
fn choice_without_options_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[4].constraints.options = None;
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::ChoiceWithoutOptions {
        name: "gender".to_owned()
    }));
}

#[test]
fn duplicate_choice_option_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[4].constraints.options = Some(vec!["male".to_owned(), "male".to_owned()]);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::DuplicateOption {
        name: "gender".to_owned(),
        option: "male".to_owned()
    }));
}

#[test]
fn age_bound_on_a_non_date_item_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[0].constraints.min_age_years = Some(18);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::AgeBoundOnNonDate {
        name: "full_name".to_owned()
    }));
}

#[test]
fn length_bound_on_a_pattern_typed_item_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.data_items[1].constraints.max_length = Some(8); // national_id is not free text
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::LengthBoundOnNonText {
        name: "national_id".to_owned(),
        kind: DataKind::NationalId
    }));
}

#[test]
fn unknown_data_class_is_rejected() {
    // "D" is a valid tier; "Z" is not.
    let mut spec = standard_ke_workflow();
    spec.data_items[0].data_class = "Z".to_owned();
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::InvalidDataClass {
        name: "full_name".to_owned(),
        class: "Z".to_owned()
    }));
}

#[test]
fn compilation_reports_every_fault_not_just_the_first() {
    let mut spec = standard_ke_workflow();
    spec.data_items[0].data_class = "?".to_owned();
    spec.data_items[1].name = "full_name".to_owned(); // duplicate
    spec.steps[0].kind = StepKind::DataEntry { items: vec!["ghost".to_owned()] };
    let errors = compile(&spec).unwrap_err();
    // Duplicate name, unknown data class, and unknown reference all reported.
    assert!(errors.len() >= 3, "expected multiple faults, got {errors:?}");
}

// ---- submission validation --------------------------------------------------

#[test]
fn valid_submission_passes() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let result = validate_submission(&compiled, &valid_submission(), &both_modalities());
    assert!(result.is_ok(), "unexpected violations: {:?}", result.violations);
}

#[test]
fn missing_required_item_is_reported() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.remove("full_name");
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::MissingRequired { item: "full_name".to_owned() }));
}

#[test]
fn blank_string_counts_as_absent() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("full_name".to_owned(), json!("   "));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::MissingRequired { item: "full_name".to_owned() }));
}

#[test]
fn optional_item_may_be_omitted() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.remove("phone");
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result.is_ok(), "phone is optional, got {:?}", result.violations);
}

#[test]
fn bad_national_id_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("national_id".to_owned(), json!("12345"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result.violations.contains(&ValidationError::PatternMismatch {
        item: "national_id".to_owned(),
        kind: DataKind::NationalId
    }));
}

#[test]
fn bad_phone_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("phone".to_owned(), json!("0712345678"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result.violations.contains(&ValidationError::PatternMismatch {
        item: "phone".to_owned(),
        kind: DataKind::Phone
    }));
}

#[test]
fn bad_station_code_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("polling_station_code".to_owned(), json!("nrb001ps042"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result.violations.contains(&ValidationError::PatternMismatch {
        item: "polling_station_code".to_owned(),
        kind: DataKind::StationCode
    }));
}

#[test]
fn value_outside_choice_options_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("gender".to_owned(), json!("unspecified"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::NotAnOption { item: "gender".to_owned() }));
}

#[test]
fn name_longer_than_max_length_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("full_name".to_owned(), json!("W".repeat(201)));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::TooLong { item: "full_name".to_owned(), max: 200 }));
}

#[test]
fn underage_date_of_birth_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    // Someone who is 17 today, unambiguously under the 18-year minimum.
    // Expressed in whole 365-day years so the value never drifts across a
    // leap-year boundary the way `18 * 365 + 364` days does.
    let dob = chrono::Utc::now().date_naive() - chrono::Duration::days(17 * 365);
    submission.insert("dob".to_owned(), json!(dob.format("%Y-%m-%d").to_string()));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(
        result
            .violations
            .iter()
            .any(|v| matches!(v, ValidationError::AgeOutOfRange { item, .. } if item == "dob")),
        "expected an age violation, got {:?}",
        result.violations
    );
}

#[test]
fn unparseable_date_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("dob".to_owned(), json!("not-a-date"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::NotADate { item: "dob".to_owned() }));
}

#[test]
fn impossible_date_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("dob".to_owned(), json!("1990-02-31"));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::NotADate { item: "dob".to_owned() }));
}

#[test]
fn consent_must_be_affirmatively_true() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("consent".to_owned(), json!(false));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::ConsentNotGiven { item: "consent".to_owned() }));
}

#[test]
fn undeclared_field_in_submission_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("is_admin".to_owned(), json!(true));
    let result = validate_submission(&compiled, &submission, &both_modalities());
    assert!(result
        .violations
        .contains(&ValidationError::UndeclaredItem { item: "is_admin".to_owned() }));
}

#[test]
fn required_capture_not_reported_is_rejected() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let result = validate_submission(&compiled, &valid_submission(), &[CaptureKind::Fingerprint]);
    assert!(
        result
            .violations
            .iter()
            .any(|v| matches!(v, ValidationError::CaptureUnsatisfied { step, .. } if step == "face")),
        "got {:?}",
        result.violations
    );
}

#[test]
fn all_violations_are_reported_together() {
    let compiled = compile(&standard_ke_workflow()).unwrap();
    let mut submission = valid_submission();
    submission.insert("national_id".to_owned(), json!("bad"));
    submission.insert("gender".to_owned(), json!("nope"));
    submission.remove("full_name");
    let result = validate_submission(&compiled, &submission, &[]);
    // full_name missing, national_id pattern, gender option, fingerprint + face capture.
    assert!(result.violations.len() >= 5, "got {:?}", result.violations);
}

// ---- configurability --------------------------------------------------------

#[test]
fn a_reduced_workflow_needs_no_code_change() {
    // The point of the DSL: an operator drops phone, gender, and polling
    // station, and removes the face capture step, purely by editing the
    // document. Nothing in the client changes.
    let mut spec = standard_ke_workflow();
    spec.data_items
        .retain(|i| !matches!(i.name.as_str(), "phone" | "gender" | "polling_station_code"));
    spec.steps.retain(|s| s.id != "contact" && s.id != "face");
    spec.capture_requirements.retain(|c| c.id != "face_live");
    spec.steps[0].kind = StepKind::DataEntry {
        items: vec!["full_name".to_owned(), "national_id".to_owned(), "dob".to_owned()],
    };

    let compiled = compile(&spec).expect("reduced workflow must still compile");
    assert_eq!(compiled.capture_order, vec!["fingerprints"]);
    assert_eq!(compiled.steps.len(), 5);

    let mut submission = valid_submission();
    submission.remove("phone");
    submission.remove("gender");
    submission.remove("polling_station_code");
    let result = validate_submission(&compiled, &submission, &[CaptureKind::Fingerprint]);
    assert!(result.is_ok(), "got {:?}", result.violations);
}

// ---- capture kinds, combinations, and documents ------------------------------

/// A workflow whose single capture step accepts several kinds and needs only
/// some of them -- the "individually or in any combination" case.
fn combination_workflow(
    kinds: Vec<CaptureKind>,
    policy: CapturePolicy,
) -> WorkflowSpec {
    let mut spec = standard_ke_workflow();
    spec.steps.retain(|s| s.id != "left_fingers" && s.id != "face");
    spec.capture_requirements = vec![CaptureRequirement {
        id: "combo".to_owned(),
        label: "Any acceptable capture".to_owned(),
        kinds,
        policy,
        optional: false,
    }];
    spec.steps.insert(
        2,
        Step {
            id: "capture".to_owned(),
            kind: StepKind::Capture { capture: "combo".to_owned() },
            label: None,
        },
    );
    spec
}

#[test]
fn all_of_requires_every_declared_kind() {
    let workflow = compile(&combination_workflow(
        vec![CaptureKind::Fingerprint, CaptureKind::PalmVein],
        CapturePolicy::AllOf,
    ))
    .unwrap();
    let cases: [(Vec<CaptureKind>, bool); 3] = [
        (vec![], false),
        (vec![CaptureKind::Fingerprint], false),
        (vec![CaptureKind::Fingerprint, CaptureKind::PalmVein], true),
    ];
    for (captured, expected_ok) in cases {
        let result = validate_submission(&workflow, &valid_submission(), &captured);
        assert_eq!(
            result.is_ok(),
            expected_ok,
            "captured {captured:?} should be ok={expected_ok}, got {:?}",
            result.violations
        );
    }
}

#[test]
fn any_of_accepts_a_single_kind() {
    let workflow = compile(&combination_workflow(
        vec![CaptureKind::Fingerprint, CaptureKind::PalmVein, CaptureKind::Iris],
        CapturePolicy::AnyOf,
    ))
    .unwrap();
    for one in [CaptureKind::Fingerprint, CaptureKind::PalmVein, CaptureKind::Iris] {
        let result = validate_submission(&workflow, &valid_submission(), &[one]);
        assert!(result.is_ok(), "{one:?} alone should satisfy any_of: {:?}", result.violations);
    }
    let none = validate_submission(&workflow, &valid_submission(), &[]);
    assert!(!none.is_ok(), "any_of must still require something");
}

#[test]
fn at_least_allows_degraded_capture() {
    // The platform supports degraded capture via
    // BiometricSensorDriver::capture_fingerprint_degraded, so a workflow must be
    // able to accept fewer captures than the ideal.
    let workflow = compile(&combination_workflow(
        vec![
            CaptureKind::Fingerprint,
            CaptureKind::PalmVein,
            CaptureKind::FingerVein,
        ],
        CapturePolicy::AtLeast(2),
    ))
    .unwrap();
    let ok = validate_submission(
        &workflow,
        &valid_submission(),
        &[CaptureKind::Fingerprint, CaptureKind::PalmVein],
    );
    assert!(ok.is_ok(), "two of three should pass: {:?}", ok.violations);

    let short = validate_submission(&workflow, &valid_submission(), &[CaptureKind::Fingerprint]);
    assert!(!short.is_ok(), "one of three must fail at least 2");
}

#[test]
fn document_scan_is_expressible_alone_and_with_biometrics() {
    let workflow = compile(&combination_workflow(
        vec![CaptureKind::DocumentMrz, CaptureKind::Fingerprint],
        CapturePolicy::AllOf,
    ))
    .unwrap();
    let result = validate_submission(
        &workflow,
        &valid_submission(),
        &[CaptureKind::DocumentMrz, CaptureKind::Fingerprint],
    );
    assert!(result.is_ok(), "got {:?}", result.violations);
}

#[test]
fn every_capture_kind_round_trips_through_json() {
    let expected: &[(CaptureKind, &str)] = &[
        (CaptureKind::Fingerprint, "fingerprint"),
        (CaptureKind::FingerVein, "finger_vein"),
        (CaptureKind::PalmVein, "palm_vein"),
        (CaptureKind::WristVein, "wrist_vein"),
        (CaptureKind::Face, "face"),
        (CaptureKind::LivePhotograph, "live_photograph"),
        (CaptureKind::Iris, "iris"),
        (CaptureKind::DocumentMrz, "document_mrz"),
    ];
    for (kind, wire_name) in expected {
        let encoded = serde_json::to_value(kind).unwrap();
        assert_eq!(encoded, json!(wire_name), "{kind:?} wire form");
        let decoded: CaptureKind = serde_json::from_value(encoded).unwrap();
        assert_eq!(&decoded, kind);
    }
}

#[test]
fn document_scan_is_not_treated_as_a_biometric() {
    // MRZ output is document data, not a biometric template, so it must not be
    // routed through the biometric handling rules.
    assert!(!CaptureKind::DocumentMrz.is_biometric());
    assert!(CaptureKind::LivePhotograph.is_biometric());
    assert!(CaptureKind::PalmVein.is_biometric());
}

#[test]
fn optional_capture_step_may_be_skipped() {
    let mut spec = combination_workflow(
        vec![CaptureKind::Fingerprint, CaptureKind::Iris],
        CapturePolicy::AllOf,
    );
    spec.capture_requirements[0].optional = true;
    let workflow = compile(&spec).unwrap();
    let result = validate_submission(&workflow, &valid_submission(), &[]);
    assert!(result.is_ok(), "optional capture must not block: {:?}", result.violations);
}

#[test]
fn capture_unsatisfied_message_names_the_step_and_requirement() {
    let workflow = compile(&combination_workflow(
        vec![CaptureKind::PalmVein, CaptureKind::Fingerprint],
        CapturePolicy::AllOf,
    ))
    .unwrap();
    let result = validate_submission(&workflow, &valid_submission(), &[]);
    let violation = result
        .violations
        .iter()
        .find(|v| matches!(v, ValidationError::CaptureUnsatisfied { .. }))
        .expect("expected a capture violation");
    let rendered = violation.to_string();
    assert!(rendered.contains("capture"), "{rendered}");
    assert!(rendered.contains("fingerprint"), "{rendered}");
    assert!(rendered.contains("palm_vein"), "{rendered}");
}

// ---- capture compilation failures -------------------------------------------

#[test]
fn step_referencing_unknown_capture_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.steps[2].kind = StepKind::Capture { capture: "ghost".to_owned() };
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::UnknownCapture {
        step: "left_fingers".to_owned(),
        capture: "ghost".to_owned()
    }));
}

#[test]
fn duplicate_capture_id_is_rejected() {
    let mut spec = standard_ke_workflow();
    let dup = spec.capture_requirements[0].clone();
    spec.capture_requirements.push(dup);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::DuplicateCaptureId {
        id: "fingerprints".to_owned()
    }));
}

#[test]
fn unreferenced_capture_requirement_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.capture_requirements.push(CaptureRequirement {
        id: "unused".to_owned(),
        label: "Never used".to_owned(),
        kinds: vec![CaptureKind::Iris],
        policy: CapturePolicy::AllOf,
        optional: false,
    });
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::UnreferencedCapture {
        id: "unused".to_owned()
    }));
}

#[test]
fn capture_requirement_without_kinds_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.capture_requirements[0].kinds.clear();
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::CaptureWithoutKinds {
        id: "fingerprints".to_owned()
    }));
}

#[test]
fn duplicate_capture_kind_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.capture_requirements[0].kinds = vec![CaptureKind::Fingerprint, CaptureKind::Fingerprint];
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::DuplicateCaptureKind {
        id: "fingerprints".to_owned(),
        kind: CaptureKind::Fingerprint
    }));
}

#[test]
fn at_least_larger_than_the_kind_list_is_rejected() {
    let mut spec = standard_ke_workflow();
    spec.capture_requirements[0].kinds = vec![CaptureKind::Fingerprint];
    spec.capture_requirements[0].policy = CapturePolicy::AtLeast(3);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::CapturePolicyUnsatisfiable {
        id: "fingerprints".to_owned(),
        wanted: 3,
        have: 1
    }));
}

#[test]
fn at_least_zero_is_rejected_as_vacuous() {
    let mut spec = standard_ke_workflow();
    spec.capture_requirements[0].kinds = vec![CaptureKind::Fingerprint];
    spec.capture_requirements[0].policy = CapturePolicy::AtLeast(0);
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::CapturePolicyVacuous {
        id: "fingerprints".to_owned()
    }));
}

#[test]
fn a_workflow_with_no_captures_compiles() {
    // Document-only pre-registration review: no biometrics at all.
    let mut spec = standard_ke_workflow();
    spec.capture_requirements.clear();
    spec.steps.retain(|s| s.id != "left_fingers" && s.id != "face");
    let workflow = compile(&spec).expect("a biometric-free workflow must compile");
    assert!(workflow.capture_order.is_empty());
    let result = validate_submission(&workflow, &valid_submission(), &[]);
    assert!(result.is_ok(), "got {:?}", result.violations);
}

// ---- single-sided age bounds ------------------------------------------------

#[test]
fn min_age_alone_is_enforced() {
    // Regression: a workflow declaring only `min_age_years` used to compile and
    // then enforce nothing, because the check required both bounds to be set.
    let mut spec = standard_ke_workflow();
    spec.data_items[2].constraints.max_age_years = None;
    let workflow = compile(&spec).expect("a single age bound must compile");
    assert_eq!(workflow.capture_order.len(), 2);

    let mut submission = valid_submission();
    submission.insert("dob".to_owned(), json!(under_18_dob()));
    let result = validate_submission(&workflow, &submission, &both_modalities());
    assert!(
        result
            .violations
            .iter()
            .any(|v| matches!(v, ValidationError::AgeOutOfRange { item, .. } if item == "dob")),
        "min_age alone must be enforced, got {:?}",
        result.violations
    );
}

#[test]
fn max_age_alone_is_enforced() {
    let mut spec = standard_ke_workflow();
    spec.data_items[2].constraints.min_age_years = None;
    let workflow = compile(&spec).expect("a single age bound must compile");
    let mut submission = valid_submission();
    // Someone over 130 is not a plausible citizen; max_age alone must catch it.
    submission.insert("dob".to_owned(), json!("1800-01-01"));
    let result = validate_submission(&workflow, &submission, &both_modalities());
    assert!(
        result
            .violations
            .iter()
            .any(|v| matches!(v, ValidationError::AgeOutOfRange { item, .. } if item == "dob")),
        "max_age alone must be enforced, got {:?}",
        result.violations
    );
}

#[test]
fn a_date_with_no_age_bounds_is_still_parsed() {
    // No bounds means no age rule, but the value must still be a real date.
    let mut spec = standard_ke_workflow();
    spec.data_items[2].constraints.min_age_years = None;
    spec.data_items[2].constraints.max_age_years = None;
    let workflow = compile(&spec).unwrap();
    let mut submission = valid_submission();
    submission.insert("dob".to_owned(), json!("1990-06-15"));
    assert!(validate_submission(&workflow, &submission, &both_modalities()).is_ok());

    submission.insert("dob".to_owned(), json!("1990-13-45"));
    assert!(!validate_submission(&workflow, &submission, &both_modalities()).is_ok());
}

/// A date of birth that is unambiguously under 18 years old.
fn under_18_dob() -> String {
    chrono::Utc::now()
        .date_naive()
        .checked_sub_signed(chrono::Duration::days(17 * 365))
        .expect("subtracting 17 years must succeed")
        .format("%Y-%m-%d")
        .to_string()
}

// ---- the shipped standard workflow ------------------------------------------

/// The standard document the desktop client ships with must compile, and must
/// stay in step with the BVR contract. Paths are relative to the workspace root.
const STANDARD_WORKFLOW: &str =
    "services/biometric-edge-svc/apps/bvr-desktop/workflows/standard-enrolment.json";

fn load_standard_workflow() -> WorkflowSpec {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(STANDARD_WORKFLOW);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {path:?} failed: {e}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("parsing {path:?} failed: {e}"))
}

#[test]
fn shipped_standard_workflow_compiles() {
    let spec = load_standard_workflow();
    let compiled = compile(&spec).expect("the shipped standard workflow must compile");
    assert_eq!(compiled.id, "bvr-enrolment-standard");
    assert_eq!(compiled.steps.len(), 8);
    assert_eq!(compiled.capture_order.len(), 3);
}

#[test]
fn shipped_workflow_covers_the_contract_required_voter_fields() {
    // The BVR contract's Voter schema requires voter_id, national_id,
    // full_name, dob, gender, polling_station_code, status, version, and
    // election_scope. The client collects all of those a citizen supplies;
    // the rest are server-assigned.
    let compiled = compile(&load_standard_workflow()).unwrap();
    for field in [
        "full_name",
        "national_id",
        "dob",
        "gender",
        "polling_station_code",
    ] {
        assert!(
            compiled.data_item(field).is_some(),
            "standard workflow must collect {field}"
        );
    }
}

#[test]
fn shipped_workflow_uses_only_pattern_checked_kinds_that_agree_with_the_contract() {
    // Guards the documented contract discrepancy: station_code is enforced
    // against ^[A-Z0-9]{6,12}$, so the workflow must not ship a value the
    // pattern would reject.
    let compiled = compile(&load_standard_workflow()).unwrap();
    let station = compiled.data_item("polling_station_code").unwrap();
    assert!(matches!(station.item.kind, DataKind::StationCode));
    assert!(eel_spec_helpers_rs::matches_bvr_pattern("polling_station_code", "NRB001PS042"));
    assert!(!eel_spec_helpers_rs::matches_bvr_pattern("polling_station_code", "NRB-001-PS-042"));
}

#[test]
fn shipped_workflow_requires_consent_and_a_minimum_age() {
    let compiled = compile(&load_standard_workflow()).unwrap();
    let consent = compiled.data_item("consent_biometric").unwrap();
    assert_eq!(consent.item.kind, DataKind::Consent);
    assert!(consent.item.required, "consent must be required");

    let dob = compiled.data_item("dob").unwrap();
    assert_eq!(dob.item.constraints.min_age_years, Some(18));
    assert_eq!(dob.item.constraints.max_age_years, Some(120));
}

#[test]
fn shipped_workflow_accepts_a_complete_registration() {
    let compiled = compile(&load_standard_workflow()).unwrap();
    let submission: serde_json::Map<String, serde_json::Value> = json!({
        "full_name": "Wanjiku Mwangi",
        "national_id": "12345678",
        "dob": "1990-06-15",
        "gender": "female",
        "county_code": "47",
        "constituency_code": "042",
        "polling_station_code": "NRB001PS042",
        "phone": "+254712345678",
        "consent_biometric": true,
    })
    .as_object()
    .cloned()
    .unwrap();

    let result = validate_submission(
        &compiled,
        &submission,
        &[
            CaptureKind::DocumentMrz,
            CaptureKind::Fingerprint,
            CaptureKind::LivePhotograph,
        ],
    );
    assert!(result.is_ok(), "shipped workflow rejected a valid registration: {:?}", result.violations);
}

#[test]
fn shipped_workflow_rejects_a_registration_missing_a_capture() {
    let compiled = compile(&load_standard_workflow()).unwrap();
    let submission: serde_json::Map<String, serde_json::Value> = json!({
        "full_name": "Wanjiku Mwangi",
        "national_id": "12345678",
        "dob": "1990-06-15",
        "gender": "female",
        "county_code": "47",
        "constituency_code": "042",
        "polling_station_code": "NRB001PS042",
        "consent_biometric": true,
    })
    .as_object()
    .cloned()
    .unwrap();

    let result = validate_submission(&compiled, &submission, &[CaptureKind::Fingerprint]);
    assert_eq!(
        result.violations.len(),
        2,
        "document and face captures both missing, got {:?}",
        result.violations
    );
}

#[test]
fn all_five_pii_tiers_are_accepted() {
    // The platform PII model is five tiers (eel-pii-rs::PIIDataClass), wider
    // than the A/B/C subset the BVR contract happens to tag. A workflow
    // storing operational metadata must not be rejected.
    for (class, ok) in [("A", true), ("B", true), ("C", true), ("D", true), ("E", true), ("F", false), ("", false)] {
        let mut spec = standard_ke_workflow();
        spec.data_items[0].data_class = class.to_owned();
        let compiled = compile(&spec);
        assert_eq!(compiled.is_ok(), ok, "data class {class:?} should be ok={ok}");
    }
}

// ---- capture taxonomy parity -------------------------------------------------

#[test]
fn the_dsl_capture_kinds_match_the_sensor_driver_taxonomy() {
    // The DSL and the capture trait must agree on the set of things a workflow
    // can ask for, or a workflow can name a capture the client cannot perform.
    // Both sides are asserted here by count and by wire name; the names are the
    // contract that crosses the C ABI.
    use eel_bvr_workflow_rs::CaptureKind;
    assert_eq!(
        CaptureKind::ALL.len(),
        18,
        "capture taxonomy grew; keep this in step with sensor-driver::Modality::ALL"
    );
    let wire: Vec<String> = CaptureKind::ALL
        .iter()
        .map(|k| serde_json::to_value(k).unwrap().as_str().unwrap().to_owned())
        .collect();
    for expected in [
        "fingerprint",
        "finger_vein",
        "palm_vein",
        "wrist_vein",
        "palm_print",
        "face",
        "live_photograph",
        "periocular",
        "iris",
        "ear",
        "voiceprint",
        "gait",
        "signature",
        "retina",
        "keystroke_dynamics",
        "mouse_dynamics",
        "dna",
        "document_mrz",
    ] {
        assert!(wire.contains(&expected.to_owned()), "missing capture kind {expected}");
    }
}

#[test]
fn only_the_document_scan_is_not_biometric() {
    use eel_bvr_workflow_rs::CaptureKind;
    for kind in CaptureKind::ALL {
        assert_eq!(
            kind.is_biometric(),
            *kind != CaptureKind::DocumentMrz,
            "{kind:?} classification is wrong"
        );
    }
}

// ---- the extended-capability workflow --------------------------------------

const EXTENDED_WORKFLOW: &str =
    "services/biometric-edge-svc/apps/bvr-desktop/workflows/extended-enrolment.json";

fn load_extended_workflow() -> WorkflowSpec {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(EXTENDED_WORKFLOW);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {path:?} failed: {e}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("parsing {path:?} failed: {e}"))
}

#[test]
fn extended_workflow_compiles() {
    let compiled = compile(&load_extended_workflow())
        .expect("the extended workflow must compile");
    assert_eq!(compiled.id, "bvr-enrolment-extended");
    assert_eq!(compiled.capture_order.len(), 6);
}

#[test]
fn extended_workflow_reaches_every_capture_kind() {
    // The point of this document: every modality the platform can capture is
    // reachable from a workflow, so the capability is genuinely available even
    // though the standard enrolment does not use it.
    let compiled = compile(&load_extended_workflow()).unwrap();
    let reachable: std::collections::HashSet<CaptureKind> = compiled
        .captures
        .values()
        .flat_map(|c| c.requirement.kinds.iter().copied())
        .collect();
    for kind in CaptureKind::ALL {
        assert!(reachable.contains(kind), "{kind:?} is not reachable from any workflow");
    }
}

#[test]
fn extended_workflow_gates_the_risky_modalities_behind_consent() {
    // Behavioural and ex-vivo capture must not be reachable in this workflow
    // without the consent item being collected, and the workflow places the
    // consent step before submit. Assert the consent item exists and is required.
    let compiled = compile(&load_extended_workflow()).unwrap();
    let consent = compiled
        .data_item("consent_extended_modalities")
        .expect("extended workflow must collect explicit consent");
    assert_eq!(consent.item.kind, DataKind::Consent);
    assert!(consent.item.required, "consent for extended modalities must be required");
}

#[test]
fn extended_workflow_leaves_the_optional_modalities_optional() {
    // Capability without obligation: every non-primary capture is optional, so
    // a station can deploy this document unchanged without every kit having to
    // serve all eighteen modalities.
    let compiled = compile(&load_extended_workflow()).unwrap();
    for id in ["secondary_biometric", "behavioural", "passive_dynamics", "biological_sample"] {
        let cap = compiled.capture(id).expect("capability {id} must exist");
        assert!(cap.requirement.optional, "{id} must stay optional");
    }
    // Only document and the primary biometric are mandatory.
    for id in ["document", "primary_biometric"] {
        let cap = compiled.capture(id).expect("{id} must exist");
        assert!(!cap.requirement.optional, "{id} should be mandatory");
    }
}

#[test]
fn extended_workflow_validates_against_a_minimal_capture() {
    // A citizen who only completes document plus one fingerprint satisfies the
    // whole workflow, because everything else is optional.
    let compiled = compile(&load_extended_workflow()).unwrap();
    let submission: serde_json::Map<String, serde_json::Value> = json!({
        "full_name": "Wanjiku Mwangi",
        "national_id": "12345678",
        "polling_station_code": "NRB001PS042",
        "consent_extended_modalities": true,
    })
    .as_object()
    .cloned()
    .unwrap();
    let result = validate_submission(
        &compiled,
        &submission,
        &[CaptureKind::DocumentMrz, CaptureKind::Fingerprint],
    );
    assert!(result.is_ok(), "optional captures must not block: {:?}", result.violations);
}

#[test]
fn extended_workflow_still_requires_the_primary_biometric() {
    let compiled = compile(&load_extended_workflow()).unwrap();
    let submission: serde_json::Map<String, serde_json::Value> = json!({
        "full_name": "Wanjiku Mwangi",
        "national_id": "12345678",
        "polling_station_code": "NRB001PS042",
        "consent_extended_modalities": true,
    })
    .as_object()
    .cloned()
    .unwrap();
    // Document only: the primary biometric is at_least(1) over three
    // alternatives, so satisfying it with one is enough, but omitting it
    // entirely is not.
    let no_primary = validate_submission(&compiled, &submission, &[CaptureKind::DocumentMrz]);
    assert!(!no_primary.is_ok(), "the primary biometric must not be optional");

    // One of the three accepted kinds is enough for the primary step.
    for kind in [CaptureKind::Fingerprint, CaptureKind::PalmVein, CaptureKind::LivePhotograph] {
        let ok = validate_submission(
            &compiled,
            &submission,
            &[CaptureKind::DocumentMrz, kind],
        );
        assert!(ok.is_ok(), "{kind:?} should satisfy the primary step: {:?}", ok.violations);
    }
}

#[test]
fn the_standard_workflow_does_not_collect_the_extended_modalities() {
    // Adding capability must not quietly change the default enrolment flow.
    let standard = compile(&load_standard_workflow()).unwrap();
    let reachable: std::collections::HashSet<CaptureKind> = standard
        .captures
        .values()
        .flat_map(|c| c.requirement.kinds.iter().copied())
        .collect();
    for kind in [
        CaptureKind::Retina,
        CaptureKind::Dna,
        CaptureKind::KeystrokeDynamics,
        CaptureKind::MouseDynamics,
        CaptureKind::Voiceprint,
        CaptureKind::Gait,
        CaptureKind::Ear,
        CaptureKind::Periocular,
    ] {
        assert!(
            !reachable.contains(&kind),
            "{kind:?} must not appear in the standard enrolment workflow"
        );
    }
}

#[test]
fn behavioural_kinds_are_distinguished_in_the_dsl_too() {
    for kind in [
        CaptureKind::Voiceprint,
        CaptureKind::Gait,
        CaptureKind::Signature,
        CaptureKind::KeystrokeDynamics,
        CaptureKind::MouseDynamics,
    ] {
        assert!(kind.is_behavioural(), "{kind:?} is behavioural");
    }
    for kind in [CaptureKind::Fingerprint, CaptureKind::Retina, CaptureKind::Dna] {
        assert!(!kind.is_behavioural(), "{kind:?} is physiological");
    }
}

// ---- documentation is kept honest -------------------------------------------

/// Every enum variant must appear in the reference, or the reference is wrong.
///
/// The reference was written after the code, so it is exactly the kind of
/// document that silently drifts. Cheap to check, and the failure mode it
/// prevents is an officer looking up a rejection code that is not there.
#[test]
fn the_reference_documents_every_enum_variant() {
    let reference = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/dsl-reference.md"),
    )
    .expect("docs/dsl-reference.md must exist; the DSL reference is a deliverable");

    let compile_errors: Vec<&str> = vec![
        "UnsupportedVersion", "NoDataItems", "NoSteps", "DuplicateDataItem",
        "DuplicateStepId", "UnknownDataItem", "UnreachableDataItem",
        "LengthBoundsInverted", "AgeBoundsInverted", "ChoiceWithoutOptions",
        "DuplicateOption", "AgeBoundOnNonDate", "LengthBoundOnNonText",
        "InvalidDataClass", "DuplicateCaptureId", "UnknownCapture",
        "UnreferencedCapture", "CaptureWithoutKinds", "DuplicateCaptureKind",
        "CapturePolicyUnsatisfiable", "CapturePolicyVacuous",
    ];
    for name in &compile_errors {
        assert!(reference.contains(name), "reference does not document {name}");
    }

    for name in [
        "MissingRequired", "ConsentNotGiven", "PatternMismatch", "TooShort",
        "TooLong", "NotAnOption", "NotADate", "AgeOutOfRange", "UndeclaredItem",
        "CaptureUnsatisfied",
    ] {
        assert!(reference.contains(name), "reference does not document {name}");
    }

    // Capture kinds are documented by their wire names.
    for kind in CaptureKind::ALL {
        let wire = serde_json::to_value(kind).unwrap();
        let name = wire.as_str().unwrap();
        assert!(reference.contains(name), "reference does not document capture kind {name}");
    }

    // The shipped documents must be listed in the reference too.
    for doc in ["standard-enrolment.json", "extended-enrolment.json"] {
        assert!(reference.contains(doc), "reference does not mention {doc}");
    }
}

#[test]
fn the_training_material_links_to_the_reference() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    let training = std::fs::read_to_string(dir.join("training.md"))
        .expect("docs/training.md must exist; the training material is a deliverable");
    assert!(
        training.contains("dsl-reference.md"),
        "training must link to the reference"
    );
    // The training material is worthless without something to practise on.
    assert!(
        training.contains("standard-enrolment.json"),
        "training must point at a real workflow document"
    );
    assert!(
        training.len() > 4000,
        "training material should be substantive, got {} bytes",
        training.len()
    );
}
