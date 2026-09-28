//! Compile and validate behaviour for declarative BVR enrolment workflows.
//!
//! The fixtures here double as the reference example for the desktop client:
//! `standard_ke_workflow()` is the shape an IEBC registration officer would
//! actually be handed.

use eel_bvr_workflow_rs::{
    BiometricModality, Constraints, DataItem, DataKind, Step, StepKind, ValidationError,
    WorkflowSpec, compile, validate_submission,
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
                kind: StepKind::BiometricCapture {
                    modality: BiometricModality::Fingerprint,
                    required: true,
                },
                label: None,
            },
            Step {
                id: "face".to_owned(),
                kind: StepKind::BiometricCapture {
                    modality: BiometricModality::Face,
                    required: true,
                },
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

fn both_modalities() -> Vec<BiometricModality> {
    vec![BiometricModality::Fingerprint, BiometricModality::Face]
}

// ---- compilation ------------------------------------------------------------

#[test]
fn standard_workflow_compiles() {
    let compiled = compile(&standard_ke_workflow()).expect("standard workflow must compile");
    assert_eq!(compiled.id, "bvr-enrolment-standard");
    assert!(compiled.has_biometric_capture);
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
        "kind": "biometric_capture",
        "modality": "fingerprint",
        "required": true
    });
    let step: Step = serde_json::from_value(json).unwrap();
    assert_eq!(
        step.kind,
        StepKind::BiometricCapture {
            modality: BiometricModality::Fingerprint,
            required: true
        }
    );
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
    let mut spec = standard_ke_workflow();
    spec.data_items[0].data_class = "D".to_owned();
    let errors = compile(&spec).unwrap_err();
    assert!(errors.contains(&eel_bvr_workflow_rs::CompileError::InvalidDataClass {
        name: "full_name".to_owned(),
        class: "D".to_owned()
    }));
}

#[test]
fn compilation_reports_every_fault_not_just_the_first() {
    let mut spec = standard_ke_workflow();
    spec.data_items[0].data_class = "Z".to_owned();
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
    let result = validate_submission(
        &compiled,
        &valid_submission(),
        &[BiometricModality::Fingerprint],
    );
    assert!(result.violations.contains(&ValidationError::CaptureMissing { modality: "face" }));
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
    spec.steps[0].kind = StepKind::DataEntry {
        items: vec!["full_name".to_owned(), "national_id".to_owned(), "dob".to_owned()],
    };

    let compiled = compile(&spec).expect("reduced workflow must still compile");
    assert!(compiled.has_biometric_capture, "fingerprint capture still present");
    assert_eq!(compiled.steps.len(), 5);

    let mut submission = valid_submission();
    submission.remove("phone");
    submission.remove("gender");
    submission.remove("polling_station_code");
    let result = validate_submission(&compiled, &submission, &[BiometricModality::Fingerprint]);
    assert!(result.is_ok(), "got {:?}", result.violations);
}
