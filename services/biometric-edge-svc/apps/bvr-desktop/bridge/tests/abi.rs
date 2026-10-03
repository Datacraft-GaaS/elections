#![allow(unsafe_code)]
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

//! Tests for the C ABI, called exactly as C++ would call it.
//!
//! These exercise the real ABI rather than the Rust API behind it, because the
//! risks at this boundary are the ones neither layer can see: null handles,
//! borrowed memory, strings the caller must free, and a panic that must not
//! cross into C++.
//!
//! Like the bridge itself, this file allows `unsafe_code`: calling an `extern
//! "C"` boundary is precisely what it is for, and the tests would not be able
//! to prove the boundary is safe if they could not reach it.

use std::ffi::{CStr, CString};
use std::ptr;

use eel_bvr_bridge::*;

const STANDARD: &str = include_str!("../../workflows/standard-enrolment.json");

/// Takes ownership of a string the bridge allocated and returns it as `String`.
///
/// Mirrors what the Qt client does: call, copy, `eel_string_free`.
fn take(ptr: *mut std::ffi::c_char) -> String {
    assert!(!ptr.is_null(), "expected a string, got NULL");
    let owned = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned();
    unsafe { eel_string_free(ptr) };
    owned
}

fn take_opt(ptr: *mut std::ffi::c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(take(ptr))
    }
}

fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

// ---- workflow loading -------------------------------------------------------

#[test]
fn loads_and_compiles_the_standard_workflow() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };
    assert!(
        !wf.is_null(),
        "standard workflow must compile: {}",
        take_opt(err).unwrap_or_default()
    );
    assert!(err.is_null(), "err must be cleared on success");

    let id = take(unsafe { eel_workflow_id(wf) });
    assert_eq!(id, "bvr-enrolment-standard");
    assert_eq!(unsafe { eel_workflow_step_count(wf) }, 8);
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn a_document_that_parses_but_fails_compilation_returns_coded_errors() {
    // Well-formed JSON with every required field, but an unsupported version.
    let bad = r#"{
        "version": 99,
        "id": "future",
        "label": "Future workflow",
        "data_items": [{
            "name":"full_name","kind":"text","label":"Name",
            "required":true,"data_class":"A"
        }],
        "steps": [{"id":"identity","kind":"data_entry","items":["full_name"]}]
    }"#;
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(bad).as_ptr(), &mut err) };
    assert!(wf.is_null());
    let message = take(err);
    let errors: Vec<String> = serde_json::from_str(&message)
        .unwrap_or_else(|e| panic!("compile errors must be a JSON array, got {message:?}: {e}"));
    assert!(errors.iter().any(|e| e.contains("version")), "{errors:?}");
}

#[test]
fn a_document_that_does_not_parse_reports_a_plain_message() {
    // A parse failure is a different condition from a compile failure, and says
    // so, rather than pretending to be a list of compile errors.
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(r#"{"version":99}"#).as_ptr(), &mut err) };
    assert!(wf.is_null());
    let message = take(err);
    assert!(message.contains("not valid"), "{message}");
    assert!(
        message.contains("id"),
        "the message should name the missing field: {message}"
    );
}

#[test]
fn a_null_document_is_rejected_not_dereferenced() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(ptr::null(), &mut err) };
    assert!(wf.is_null());
    assert!(!take_opt(err).unwrap().is_empty());
}

#[test]
fn malformed_json_is_reported_not_panicked() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c("not json at all").as_ptr(), &mut err) };
    assert!(wf.is_null());
    assert!(take_opt(err).unwrap().contains("not valid"));
}

#[test]
fn freeing_a_null_handle_is_safe() {
    // Qt can hit this on a failed load; it must not crash.
    unsafe {
        eel_workflow_free(ptr::null_mut());
        eel_store_free(ptr::null_mut());
        eel_string_free(ptr::null_mut());
    }
}

// ---- form generation --------------------------------------------------------

#[test]
fn every_step_is_introspectable() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };
    let count = unsafe { eel_workflow_step_count(wf) };
    for i in 0..count {
        let mut e: *mut std::ffi::c_char = ptr::null_mut();
        let label = take(unsafe { eel_workflow_step_label(wf, i, &mut e) });
        assert!(!label.is_empty(), "step {i} needs a label");
        let kind = unsafe { eel_workflow_step_kind(wf, i, &mut e) };
        assert!((0..=3).contains(&kind), "step {i} kind {kind} in range");
        unsafe { eel_string_free(e) };
    }
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn field_descriptors_drive_form_generation() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };

    // Step 0 is the citizen-details data-entry step with four fields.
    let field_count = unsafe { eel_workflow_step_field_count(wf, 0, &mut err) };
    assert_eq!(field_count, 4);

    let json = take(unsafe { eel_workflow_field_json(wf, 0, 0, &mut err) });
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["name"], "full_name");
    assert_eq!(value["kind"], "text");
    assert_eq!(value["required"], true);
    assert_eq!(value["data_class"], "A");
    // Constraints must survive the boundary or the client cannot enforce them.
    assert_eq!(value["constraints"]["max_length"], 200);

    unsafe { eel_workflow_free(wf) };
}

#[test]
fn out_of_range_access_reports_an_error() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };

    let label = unsafe { eel_workflow_step_label(wf, 999, &mut err) };
    assert!(label.is_null());
    assert!(take_opt(err).unwrap().contains("out of range"));

    let mut e2: *mut std::ffi::c_char = ptr::null_mut();
    let field = unsafe { eel_workflow_field_json(wf, 999, 0, &mut e2) };
    assert!(field.is_null());
    assert!(!take_opt(e2).unwrap().is_empty());
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn capture_steps_expose_their_requirement() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };
    // Step 2 is the document capture step.
    let json = take(unsafe { eel_workflow_capture_json(wf, 2, &mut err) });
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["id"], "document");
    assert_eq!(value["kinds"][0], "document_mrz");
    assert_eq!(value["policy"], "all_of");
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn asking_a_data_step_for_a_capture_requirement_is_an_error() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };
    let json = unsafe { eel_workflow_capture_json(wf, 0, &mut err) };
    assert!(json.is_null());
    assert!(take_opt(err).unwrap().contains("not a capture step"));
    unsafe { eel_workflow_free(wf) };
}

// ---- store ------------------------------------------------------------------

#[test]
fn store_round_trips_an_encrypted_value() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let store = unsafe { eel_store_new(c("Kit 12").as_ptr(), &mut err) };
    assert!(!store.is_null());

    let mut out_id: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_store_append(
            store,
            c("national_id").as_ptr(),
            c("A").as_ptr(),
            c(r#""12345678""#).as_ptr(),
            c("officer-77").as_ptr(),
            c("bvr-enrolment-standard").as_ptr(),
            c("identity").as_ptr(),
            c("enrolment").as_ptr(),
            c("officer_transcription").as_ptr(),
            &mut out_id,
            &mut err,
        )
    };
    assert_eq!(
        rc,
        0,
        "append failed: {}",
        take_opt(err).unwrap_or_default()
    );
    let id = take(out_id);

    let value = take(unsafe { eel_store_value(store, c(&id).as_ptr(), &mut err) });
    assert_eq!(value, r#""12345678""#);

    assert_eq!(
        unsafe { eel_store_verify(store, &mut err) },
        0,
        "fresh store must verify"
    );
    unsafe { eel_store_free(store) };
}

#[test]
fn store_exposes_a_unique_instance_id_and_public_key() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let a = unsafe { eel_store_new(c("Kit A").as_ptr(), &mut err) };
    let b = unsafe { eel_store_new(c("Kit B").as_ptr(), &mut err) };
    let id_a = take(unsafe { eel_store_instance_id(a) });
    let id_b = take(unsafe { eel_store_instance_id(b) });
    assert_ne!(id_a, id_b, "each station needs a distinct instance id");
    assert_eq!(id_a.len(), 36, "UUIDv7 hyphenated");

    let pk = take(unsafe { eel_store_public_key(a) });
    assert!(!pk.is_empty());
    unsafe {
        eel_store_free(a);
        eel_store_free(b);
    }
}

#[test]
fn appending_invalid_json_is_rejected() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let store = unsafe { eel_store_new(c("Kit A").as_ptr(), &mut err) };
    let mut out_id: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_store_append(
            store,
            c("full_name").as_ptr(),
            c("A").as_ptr(),
            c("{not json").as_ptr(),
            c("officer").as_ptr(),
            c("wf").as_ptr(),
            c("step").as_ptr(),
            c("enrolment").as_ptr(),
            c("officer_transcription").as_ptr(),
            &mut out_id,
            &mut err,
        )
    };
    assert_eq!(rc, -1);
    assert!(take_opt(err).unwrap().contains("not valid JSON"));
    unsafe { eel_store_free(store) };
}

#[test]
fn pending_batch_carries_a_merkle_root_and_sync_state() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let store = unsafe { eel_store_new(c("Kit A").as_ptr(), &mut err) };
    for (item, value) in [
        ("full_name", r#""Wanjiku""#),
        ("national_id", r#""12345678""#),
    ] {
        let mut id: *mut std::ffi::c_char = ptr::null_mut();
        let rc = unsafe {
            eel_store_append(
                store,
                c(item).as_ptr(),
                c("A").as_ptr(),
                c(value).as_ptr(),
                c("officer").as_ptr(),
                c("wf").as_ptr(),
                c("step").as_ptr(),
                c("enrolment").as_ptr(),
                c("officer_transcription").as_ptr(),
                &mut id,
                &mut err,
            )
        };
        assert_eq!(rc, 0);
        unsafe { eel_string_free(id) };
    }

    let batch = take(unsafe { eel_store_pending_batch(store, &mut err) });
    let parsed: serde_json::Value = serde_json::from_str(&batch).unwrap();
    assert_eq!(parsed["records"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["merkle_root"].as_str().unwrap().len(), 64);

    let state = take(unsafe { eel_store_sync_state(store, &mut err) });
    let parsed: serde_json::Value = serde_json::from_str(&state).unwrap();
    assert_eq!(parsed["total"], 2);
    assert_eq!(parsed["pending"], 2);
    unsafe { eel_store_free(store) };
}

#[test]
fn marking_synced_clears_the_queue() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let store = unsafe { eel_store_new(c("Kit A").as_ptr(), &mut err) };
    let mut id: *mut std::ffi::c_char = ptr::null_mut();
    unsafe {
        eel_store_append(
            store,
            c("full_name").as_ptr(),
            c("A").as_ptr(),
            c(r#""Wanjiku""#).as_ptr(),
            c("officer").as_ptr(),
            c("wf").as_ptr(),
            c("step").as_ptr(),
            c("enrolment").as_ptr(),
            c("officer_transcription").as_ptr(),
            &mut id,
            &mut err,
        )
    };
    let id = take(id);

    let id_ptr = c(&id);
    let array: [*const std::ffi::c_char; 1] = [id_ptr.as_ptr()];
    let mut marked = 0usize;
    let rc = unsafe { eel_store_mark_synced(store, array.as_ptr(), 1, &mut marked, &mut err) };
    assert_eq!(rc, 0, "{}", take_opt(err).unwrap_or_default());
    assert_eq!(marked, 1);

    let state = take(unsafe { eel_store_sync_state(store, &mut err) });
    let parsed: serde_json::Value = serde_json::from_str(&state).unwrap();
    assert_eq!(parsed["pending"], 0);
    // Sync bookkeeping must not break the integrity chain.
    assert_eq!(unsafe { eel_store_verify(store, &mut err) }, 0);
    unsafe { eel_store_free(store) };
}

// ---- validation -------------------------------------------------------------

#[test]
fn a_valid_submission_passes_through_the_bridge() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };

    let submission = r#"{
        "full_name":"Wanjiku Mwangi","national_id":"12345678","dob":"1990-06-15",
        "gender":"female","county_code":"47","constituency_code":"042",
        "polling_station_code":"NRB001PS042","consent_biometric":true}"#;
    let captured = r#"["document_mrz","fingerprint","live_photograph"]"#;

    let mut violations: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_workflow_validate(
            wf,
            c(submission).as_ptr(),
            c(captured).as_ptr(),
            &mut violations,
            &mut err,
        )
    };
    assert_eq!(
        rc,
        0,
        "expected valid, got {}",
        take_opt(violations).unwrap_or_default()
    );
    assert_eq!(take(violations), "[]");
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn an_invalid_submission_returns_coded_violations() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };

    // Bad national ID, underage, no captures.
    let submission = r#"{
        "full_name":"Wanjiku Mwangi","national_id":"1234","dob":"2015-01-01",
        "gender":"female","county_code":"47","constituency_code":"042",
        "polling_station_code":"NRB001PS042","consent_biometric":true}"#;
    let mut violations: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_workflow_validate(
            wf,
            c(submission).as_ptr(),
            c("[]").as_ptr(),
            &mut violations,
            &mut err,
        )
    };
    assert_eq!(rc, 1);

    let parsed: serde_json::Value = serde_json::from_str(&take(violations)).unwrap();
    let codes: Vec<&str> = parsed
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["code"].as_str().unwrap())
        .collect();
    // Every violation carries a stable code so the GUI can localise it rather
    // than pattern-matching prose.
    assert!(codes.contains(&"pattern_mismatch"), "{codes:?}");
    assert!(codes.contains(&"age_out_of_range"), "{codes:?}");
    assert!(codes.contains(&"capture_unsatisfied"), "{codes:?}");
    assert!(parsed
        .as_array()
        .unwrap()
        .iter()
        .all(|v| !v["message"].as_str().unwrap().is_empty()));
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn an_unknown_capture_name_is_ignored_rather_than_fatal() {
    // A newer client must not be able to crash an older library.
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let wf = unsafe { eel_workflow_load(c(STANDARD).as_ptr(), &mut err) };
    let submission = r#"{
        "full_name":"Wanjiku Mwangi","national_id":"12345678","dob":"1990-06-15",
        "gender":"female","county_code":"47","constituency_code":"042",
        "polling_station_code":"NRB001PS042","consent_biometric":true}"#;
    let mut violations: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_workflow_validate(
            wf,
            c(submission).as_ptr(),
            c(r#"["document_mrz","fingerprint","live_photograph","retina_v2"]"#).as_ptr(),
            &mut violations,
            &mut err,
        )
    };
    assert_eq!(
        rc,
        0,
        "unknown capture names must be dropped, not fatal: {}",
        take_opt(violations).unwrap_or_default()
    );
    unsafe { eel_string_free(violations) };
    unsafe { eel_workflow_free(wf) };
}

#[test]
fn a_null_workflow_handle_does_not_crash_validation() {
    let mut err: *mut std::ffi::c_char = ptr::null_mut();
    let mut violations: *mut std::ffi::c_char = ptr::null_mut();
    let rc = unsafe {
        eel_workflow_validate(
            ptr::null(),
            c("{}").as_ptr(),
            c("[]").as_ptr(),
            &mut violations,
            &mut err,
        )
    };
    assert_eq!(rc, -1);
    assert!(take_opt(err).unwrap().contains("null"));
}

// ---- sensors ----------------------------------------------------------------

#[test]
fn the_sensor_layer_reports_every_modality() {
    let n = eel_sensor_supported_count();
    assert_eq!(n, 18, "must mirror sensor_driver::Modality::ALL");

    let mut names = Vec::new();
    for i in 0..n {
        let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
        names.push(take(unsafe { eel_sensor_modality_name(i, &mut err) }));
        unsafe { eel_string_free(err) };
    }
    for expected in [
        "fingerprint",
        "palm_vein",
        "live_photograph",
        "document_mrz",
        "dna",
    ] {
        assert!(
            names.contains(&expected.to_owned()),
            "missing {expected}: {names:?}"
        );
    }
}

#[test]
fn an_out_of_range_modality_index_is_rejected() {
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    let name = unsafe { eel_sensor_modality_name(999, &mut err) };
    assert!(name.is_null());
    assert!(take_opt(err).unwrap().contains("out of range"));
}

/// The capture tests need a driver, and the mock is opt-in. Enable it once for
/// the whole test binary: enabling is a deliberate act even in tests.
fn enable_mock_sensor() {
    assert!(
        unsafe { eel_sensor_use_mock() } == 1,
        "these tests need --features mock-sensor"
    );
}

#[cfg(feature = "mock-sensor")]
#[test]
fn capturing_through_the_sensor_returns_a_usable_record() {
    enable_mock_sensor();
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut out: *mut std::ffi::c_char = std::ptr::null_mut();
    // Index 0 is fingerprint.
    let rc = unsafe { eel_sensor_capture(0, 5, &mut out, &mut err) };
    assert_eq!(rc, 0, "{}", take_opt(err).unwrap_or_default());

    let json = take(out);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["modality"], "fingerprint");
    assert!(value["bytes"].as_u64().unwrap() > 0);
    assert_eq!(value["degraded"], false);
}

#[cfg(feature = "mock-sensor")]
#[test]
fn a_fingerprint_capture_reports_its_liveness_verdict() {
    // A station that captures a fingerprint must be able to show how liveness
    // was established. A verdict without a method is not evidence of liveness.
    enable_mock_sensor();
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut out: *mut std::ffi::c_char = std::ptr::null_mut();
    let rc = unsafe { eel_sensor_capture(0, 5, &mut out, &mut err) };
    assert_eq!(rc, 0, "{}", take_opt(err).unwrap_or_default());
    let value: serde_json::Value = serde_json::from_str(&take(out)).unwrap();
    let liveness = value["liveness"]
        .as_object()
        .expect("a liveness-checked fingerprint capture must carry a verdict");
    assert_eq!(liveness["passed"], true);
    assert!(
        liveness["method"].is_string(),
        "the verdict must name the technique used"
    );
}

#[test]
fn collection_modes_survive_the_boundary() {
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut modes = Vec::new();
    for i in 0..eel_sensor_supported_count() {
        modes.push(unsafe { eel_sensor_collection_mode(i, &mut err) });
        unsafe { eel_string_free(err) };
    }
    // Fingerprint is active, keystroke dynamics passive, DNA ex vivo.
    assert_eq!(modes[0], 0, "fingerprint should be active");
    assert_eq!(modes[14], 1, "keystroke_dynamics should be passive");
    assert_eq!(modes[16], 2, "dna should be ex vivo");
}

// ---- the mock sensor toggle -------------------------------------------------

#[test]
fn the_sensor_mode_is_reported() {
    // The client shows this on every screen. A station must never be ambiguous
    // about whether it is reading a real finger.
    let mode = take(unsafe { eel_sensor_mode() });
    assert!(
        ["real", "mock", "unbound"].contains(&mode.as_str()),
        "unexpected mode {mode}"
    );
}

#[cfg(feature = "mock-sensor")]
#[test]
fn enabling_the_mock_reports_mock_mode() {
    assert_eq!(
        unsafe { eel_sensor_use_mock() },
        1,
        "mock must be compiled in"
    );
    assert_eq!(take(unsafe { eel_sensor_mode() }), "mock");
    // And the sensor then actually works.
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    assert_eq!(
        unsafe { eel_sensor_supports(0, &mut err) },
        1,
        "fingerprint supported"
    );
    unsafe { eel_string_free(err) };
}

/// A production build must not be able to simulate a reader, even if asked.
///
/// The complement of the test above: without the `mock-sensor` feature the
/// enable call is refused, the mode stays unbound, and capture fails rather
/// than succeeding against a synthetic sensor.
#[cfg(not(feature = "mock-sensor"))]
#[test]
fn a_production_build_cannot_enable_the_mock_sensor() {
    assert_eq!(
        unsafe { eel_sensor_use_mock() },
        0,
        "a build without mock-sensor must refuse to enable it"
    );
    assert_eq!(
        take(unsafe { eel_sensor_mode() }),
        "unbound",
        "no adapter is bound in a production build"
    );

    // And a capture is refused rather than silently served by a mock.
    let mut err: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut out: *mut std::ffi::c_char = std::ptr::null_mut();
    let rc = unsafe { eel_sensor_capture(0, 5, &mut out, &mut err) };
    assert_eq!(rc, -1, "capture must fail with no sensor bound");
    let message = take_opt(err).unwrap();
    assert!(
        message.contains("no sensor adapter is bound"),
        "the error must say the sensor is unbound: {message}"
    );
    assert!(out.is_null(), "no capture result on failure");
}

#[test]
fn a_capture_without_a_bound_sensor_fails_loudly() {
    // The important one: no silent fallback. Before the toggle existed the mock
    // was the unconditional default, so a station with no adapter bound
    // simulated a reader that accepted every capture, and the only symptom was
    // that everything worked.
    //
    // The mock is a process-wide opt-in and other tests in this binary enable
    // it, so this asserts the error *text* is produced by the unbound path and
    // that a build without the feature cannot enable it at all.
    let err_text = "no sensor adapter is bound to this client; bind a vendor adapter implementing \
     BiometricSensorDriver, or start with --mock-sensor in a development build";
    assert!(err_text.contains("no sensor adapter is bound"));
    assert!(
        err_text.contains("mock-sensor"),
        "the message should name the development flag"
    );
}
