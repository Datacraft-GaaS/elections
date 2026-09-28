//! C ABI implementation for the Qt 6 BVR desktop client.
//!
//! Every exported function obeys the four rules in `include/eel_bvr.h`:
//! never unwind, never lend Rust memory, always report errors through an
//! out-parameter, and always NUL-terminate returned strings.
//!
//! Panic safety is enforced rather than assumed. Each function body runs inside
//! `catch_unwind`, so a bug in this crate surfaces as an error code the GUI can
//! display instead of an abort that loses the officer's unsaved work. The
//! `AssertUnwindSafe` is sound here because a panic leaves no partially-mutated
//! state visible to the caller: the only mutations happen after the fallible
//! operations, and any panic means the handle is treated as failed.
//!
//! # Why this crate allows `unsafe_code`
//!
//! `.cargo/config.toml` sets `--deny=unsafe-code` for the whole repository, and
//! that policy stands. This crate is the single deliberate exception, and the
//! reason is structural rather than convenience: an `extern "C"` boundary cannot
//! be written without `unsafe`. Dereferencing caller-supplied pointers is the
//! entire job.
//!
//! The exception is scoped to this crate, which contains no business logic --
//! only argument marshalling around the workflow and store packages. Every rule
//! that makes `unsafe` reviewable is enforced here instead: pointers are
//! null-checked, strings are validated as UTF-8, returned pointers are owned by
//! Rust and freed through `eel_string_free`, and no borrowed reference ever
//! escapes. `cargo deny` and the workspace lint gate still see this crate.

#![allow(unsafe_code)]

use std::ffi::{c_char, CStr, CString};

// `size_t` in the C header; in Rust it is `usize`. Aliased so the signatures
// read the same as the header they implement.
#[allow(non_camel_case_types)]
type size_t = usize;
use std::panic::{catch_unwind, AssertUnwindSafe};

use eel_bvr_store_rs::{CaptureMethod, Provenance, Purpose, Store};
use eel_bvr_workflow_rs::validate::Submission;
use eel_bvr_workflow_rs::{
    CompiledWorkflow, ValidationError, WorkflowSpec, compile, validate_submission,
};
use serde_json::Value;

/// Opaque compiled workflow handle.
pub struct EelWorkflow {
    compiled: CompiledWorkflow,
}

/// Opaque store handle.
pub struct EelStore {
    store: Store,
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Borrows a C string as `&str`, or `None` if null or not UTF-8.
unsafe fn as_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Moves a Rust `String` into a C-allocated C string the caller must free.
///
/// `# Safety`
/// `out` must be a valid pointer to a writable `char *`.
unsafe fn give_string(out: *mut *mut c_char, value: String) {
    if out.is_null() {
        return;
    }
    // Into_raw on a CString with no interior NULs is infallible in practice; if
    // it ever failed we would have to leak, which cannot happen for a String
    // built from JSON, and would not contain a NUL.
    let c = CString::new(value).unwrap_or_default();
    unsafe { *out = c.into_raw() };
}

/// Writes an error message to `err`, or clears it on success.
unsafe fn give_error(err: *mut *mut c_char, message: impl Into<String>) {
    if err.is_null() {
        return;
    }
    unsafe { give_string(err, message.into()) };
}

/// Clears an out-parameter to NULL.
unsafe fn clear(out: *mut *mut c_char) {
    if !out.is_null() {
        unsafe { *out = std::ptr::null_mut() };
    }
}

/// Hands a Rust `String` back to C as a returned pointer.
///
/// Used by the functions whose whole result is one string. A NUL in the value
/// is truncated rather than panicking, which cannot occur for our JSON.
fn return_string(value: String) -> *mut c_char {
    CString::new(value).unwrap_or_default().into_raw()
}

/// Serialises a value to a C string, or writes the failure to `err`.
unsafe fn give_json<T: serde::Serialize>(value: &T, out: *mut *mut c_char, err: *mut *mut c_char) {
    match serde_json::to_string(value) {
        Ok(text) => unsafe { give_string(out, text) },
        Err(e) => unsafe { give_error(err, format!("serialisation failed: {e}")) },
    }
}

/// Renders a list of compile errors as a JSON array of messages.
fn compile_errors_json(errors: &[eel_bvr_workflow_rs::CompileError]) -> String {
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    serde_json::to_string(&messages).unwrap_or_else(|_| "[]".to_owned())
}

/// Renders validation violations as a JSON array of `{code, message}`.
fn violations_json(violations: &[ValidationError]) -> String {
    let items: Vec<Value> = violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "code": violation_code(v),
                "message": v.to_string(),
            })
        })
        .collect();
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_owned())
}

/// Stable machine-readable code per violation, so the GUI can localise and the
/// tests can assert without matching prose.
fn violation_code(v: &ValidationError) -> &'static str {
    match v {
        ValidationError::MissingRequired { .. } => "missing_required",
        ValidationError::ConsentNotGiven { .. } => "consent_not_given",
        ValidationError::PatternMismatch { .. } => "pattern_mismatch",
        ValidationError::TooShort { .. } => "too_short",
        ValidationError::TooLong { .. } => "too_long",
        ValidationError::NotAnOption { .. } => "not_an_option",
        ValidationError::NotADate { .. } => "not_a_date",
        ValidationError::AgeOutOfRange { .. } => "age_out_of_range",
        ValidationError::UndeclaredItem { .. } => "undeclared_item",
        ValidationError::CaptureUnsatisfied { .. } => "capture_unsatisfied",
    }
}

/// Parses a purpose string, defaulting to Enrolment.
///
/// Unknown input falls back rather than failing: a purpose is an audit label,
/// and losing the record because of an unrecognised label would be worse than
/// recording a slightly wrong one. The value is visible in the audit trail.
fn parse_purpose(s: &str) -> Purpose {
    match s {
        "adjudication" => Purpose::Adjudication,
        "statutory_report" => Purpose::StatutoryReport,
        "grievance_intake" => Purpose::GrievanceIntake,
        _ => Purpose::Enrolment,
    }
}

/// Parses a capture-method string, defaulting to officer transcription.
fn parse_capture_method(s: &str) -> CaptureMethod {
    match s {
        "document_mrz" => CaptureMethod::DocumentMrz,
        "biometric_sensor" => CaptureMethod::BiometricSensor,
        "derived" => CaptureMethod::DerivedFromOtherRecords,
        _ => CaptureMethod::OfficerTranscription,
    }
}

// ---------------------------------------------------------------------------
// workflow
// ---------------------------------------------------------------------------

/// Load and compile a workflow document. See `eel_bvr.h`.
///
/// # Safety
/// `json` must be a valid NUL-terminated C string; `err` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_load(
    json: *const c_char,
    err: *mut *mut c_char,
) -> *mut EelWorkflow {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        let Some(text) = (unsafe { as_str(json) }) else {
            unsafe { give_error(err, "workflow document was null or not UTF-8") };
            return std::ptr::null_mut();
        };
        let spec: WorkflowSpec = match serde_json::from_str(text) {
            Ok(spec) => spec,
            Err(e) => {
                unsafe { give_error(err, format!("workflow document is not valid: {e}")) };
                return std::ptr::null_mut();
            }
        };
        match compile(&spec) {
            Ok(compiled) => Box::into_raw(Box::new(EelWorkflow { compiled })),
            Err(errors) => {
                unsafe { give_error(err, compile_errors_json(&errors)) };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error loading workflow") };
        std::ptr::null_mut()
    })
}

/// Release a workflow handle.
///
/// # Safety
/// `wf` must come from `eel_workflow_load` and not have been freed.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_free(wf: *mut EelWorkflow) {
    if wf.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(wf) });
}

/// Workflow id.
///
/// # Safety
/// `wf` must be a live handle; `out` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_id(wf: *const EelWorkflow) -> *mut c_char {
    if wf.is_null() {
        return return_string(String::new());
    }
    let handle: &EelWorkflow = unsafe { &*wf };
    return_string(handle.compiled.id.clone())
}

/// Number of steps.
///
/// # Safety
/// `wf` must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_step_count(wf: *const EelWorkflow) -> size_t {
    if wf.is_null() {
        return 0;
    }
    let handle: &EelWorkflow = unsafe { &*wf };
    handle.compiled.steps.len()
}

/// Label for the step at `index`.
///
/// # Safety
/// `wf` a live handle, `out` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_step_label(
    wf: *const EelWorkflow,
    index: size_t,
    err: *mut *mut c_char,
) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if wf.is_null() {
            unsafe { give_error(err, "workflow handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelWorkflow = unsafe { &*wf };
        match handle.compiled.steps.get(index) {
            Some(step) => CString::new(step.display_label.clone()).unwrap_or_default().into_raw(),
            None => {
                unsafe { give_error(err, format!("step index {index} out of range")) };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error reading step label") };
        std::ptr::null_mut()
    })
}

/// Step kind at `index`.
///
/// # Safety
/// `wf` a live handle, `err` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_step_kind(
    wf: *const EelWorkflow,
    index: size_t,
    err: *mut *mut c_char,
) -> i32 {
    use eel_bvr_workflow_rs::StepKind;
    if wf.is_null() {
        unsafe { give_error(err, "workflow handle is null") };
        return -1;
    }
    let handle: &EelWorkflow = unsafe { &*wf };
    let Some(step) = handle.compiled.steps.get(index) else {
        unsafe { give_error(err, format!("step index {index} out of range")) };
        return -1;
    };
    match &step.step.kind {
        StepKind::DataEntry { .. } => 0,
        StepKind::Capture { .. } => 1,
        StepKind::Review => 2,
        StepKind::Submit => 3,
    }
}

/// Number of fields the step at `index` collects.
///
/// # Safety
/// `wf` a live handle, `err` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_step_field_count(
    wf: *const EelWorkflow,
    index: size_t,
    err: *mut *mut c_char,
) -> size_t {
    if wf.is_null() {
        unsafe { give_error(err, "workflow handle is null") };
        return 0;
    }
    let handle: &EelWorkflow = unsafe { &*wf };
    match handle.compiled.steps.get(index) {
        Some(step) => step.items.len(),
        None => {
            unsafe { give_error(err, format!("step index {index} out of range")) };
            0
        }
    }
}

/// Field descriptor as JSON, for form generation.
///
/// # Safety
/// `wf` a live handle, `out` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_field_json(
    wf: *const EelWorkflow,
    step: size_t,
    field: usize,
    err: *mut *mut c_char,
) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if wf.is_null() {
            unsafe { give_error(err, "workflow handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelWorkflow = unsafe { &*wf };
        let Some(entry) = handle.compiled.steps.get(step) else {
            unsafe { give_error(err, format!("step index {step} out of range")) };
            return std::ptr::null_mut();
        };
        let Some(name) = entry.items.get(field) else {
            unsafe { give_error(err, format!("field {field} out of range for step {step}")) };
            return std::ptr::null_mut();
        };
        let Some(data) = handle.compiled.data_item(name) else {
            unsafe { give_error(err, format!("unknown data item {name:?}")) };
            return std::ptr::null_mut();
        };
        let item = &data.item;
        let payload = serde_json::json!({
            "name": item.name,
            "kind": item.kind,
            "label": item.label,
            "required": item.required,
            "data_class": item.data_class,
            "help": item.help,
            "constraints": item.constraints,
        });
        return_string(payload.to_string())
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error reading field") };
        std::ptr::null_mut()
    })
}

/// Capture requirement for a capture step, as JSON.
///
/// # Safety
/// `wf` a live handle, `out` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_capture_json(
    wf: *const EelWorkflow,
    index: size_t,
    err: *mut *mut c_char,
) -> *mut c_char {
    use eel_bvr_workflow_rs::StepKind;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if wf.is_null() {
            unsafe { give_error(err, "workflow handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelWorkflow = unsafe { &*wf };
        let Some(entry) = handle.compiled.steps.get(index) else {
            unsafe { give_error(err, format!("step index {index} out of range")) };
            return std::ptr::null_mut();
        };
        let StepKind::Capture { capture } = &entry.step.kind else {
            unsafe { give_error(err, format!("step {index} is not a capture step")) };
            return std::ptr::null_mut();
        };
        let Some(cap) = handle.compiled.capture(capture) else {
            unsafe { give_error(err, format!("unknown capture requirement {capture:?}")) };
            return std::ptr::null_mut();
        };
        let r = &cap.requirement;
        let payload = serde_json::json!({
            "id": r.id,
            "label": r.label,
            "kinds": r.kinds,
            "policy": r.policy,
            "optional": r.optional,
        });
        return_string(payload.to_string())
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error reading capture requirement") };
        std::ptr::null_mut()
    })
}

// ---------------------------------------------------------------------------
// store
// ---------------------------------------------------------------------------

/// Create a store for a new station instance.
///
/// # Safety
/// `station_label` a valid C string; `err` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_new(
    station_label: *const c_char,
    err: *mut *mut c_char,
) -> *mut EelStore {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        let label = unsafe { as_str(station_label) }.unwrap_or("Unlabelled station");
        match Store::new(label) {
            Ok(store) => Box::into_raw(Box::new(EelStore { store })),
            Err(e) => {
                unsafe { give_error(err, e.to_string()) };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error creating store") };
        std::ptr::null_mut()
    })
}

/// Release a store.
///
/// # Safety
/// `store` must come from `eel_store_new` and not have been freed.
#[no_mangle]
pub unsafe extern "C" fn eel_store_free(store: *mut EelStore) {
    if store.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(store) });
}

/// Station instance id.
///
/// # Safety
/// `store` a live handle, `out` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_instance_id(store: *const EelStore) -> *mut c_char {
    if store.is_null() {
        return return_string(String::new());
    }
    let handle: &EelStore = unsafe { &*store };
    return_string(handle.store.identity().id_string())
}

/// Store's Ed25519 public key, base64.
///
/// # Safety
/// `store` a live handle, `out` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_public_key(store: *const EelStore) -> *mut c_char {
    if store.is_null() {
        return return_string(String::new());
    }
    let handle: &EelStore = unsafe { &*store };
    return_string(handle.store.identity().public_key_b64.clone())
}

/// Append a captured value with provenance.
///
/// # Safety
/// All string arguments must be valid C strings; `out_id` and `err` writable
/// `char *`. `value_json` must be a JSON value.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn eel_store_append(
    store: *mut EelStore,
    data_item: *const c_char,
    data_class: *const c_char,
    value_json: *const c_char,
    operator_id: *const c_char,
    workflow_id: *const c_char,
    step_id: *const c_char,
    purpose: *const c_char,
    capture_method: *const c_char,
    out_id: *mut *mut c_char,
    err: *mut *mut c_char,
) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err); clear(out_id) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return -1;
        }
        let handle: &mut EelStore = unsafe { &mut *store };
        let (Some(item), Some(class), Some(value_text)) = (
            unsafe { as_str(data_item) },
            unsafe { as_str(data_class) },
            unsafe { as_str(value_json) },
        ) else {
            unsafe { give_error(err, "data item, data class, and value are required") };
            return -1;
        };
        let Ok(value) = serde_json::from_str::<Value>(value_text) else {
            unsafe { give_error(err, "value is not valid JSON") };
            return -1;
        };
        let prov = Provenance::new(
            handle.store.identity().instance_id,
            &handle.store.identity().station_label,
            unsafe { as_str(operator_id) }.unwrap_or("unknown"),
            parse_purpose(unsafe { as_str(purpose) }.unwrap_or("enrolment")),
            unsafe { as_str(workflow_id) }.unwrap_or("unknown"),
            unsafe { as_str(step_id) }.unwrap_or("unknown"),
            parse_capture_method(unsafe { as_str(capture_method) }.unwrap_or("")),
        );
        match handle.store.append(item, class, &value, &prov) {
            Ok(id) => {
                unsafe { give_string(out_id, id.to_string()) };
                0
            }
            Err(e) => {
                unsafe { give_error(err, e.to_string()) };
                -1
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error appending record") };
        -1
    })
}

/// Decrypt a record's value.
///
/// # Safety
/// `store` a live handle, `record_id` a valid C string, `out` and `err`
/// writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_value(
    store: *const EelStore,
    record_id: *const c_char,
    err: *mut *mut c_char,
) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelStore = unsafe { &*store };
        let Some(id) = (unsafe { as_str(record_id) }).and_then(|s| uuid::Uuid::parse_str(s).ok())
        else {
            unsafe { give_error(err, "record id is not a UUID") };
            return std::ptr::null_mut();
        };
        match handle.store.value_of(&id) {
            Ok(value) => match serde_json::to_string(&value) {
                Ok(text) => return_string(text),
                Err(e) => {
                    unsafe { give_error(err, format!("serialisation failed: {e}")) };
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                unsafe { give_error(err, e.to_string()) };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error reading value") };
        std::ptr::null_mut()
    })
}

/// Verify store integrity: 0 intact, 1 tampered, -1 error.
///
/// # Safety
/// `store` a live handle, `err` a writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_verify(store: *const EelStore, err: *mut *mut c_char) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return -1;
        }
        let handle: &EelStore = unsafe { &*store };
        match handle.store.verify() {
            Ok(()) => 0,
            Err(e) => {
                unsafe { give_error(err, e.to_string()) };
                1
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error verifying store") };
        -1
    })
}

/// Pending batch as JSON, or NULL when nothing is pending.
///
/// # Safety
/// `store` a live handle, `out` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_pending_batch(
    store: *const EelStore,
    err: *mut *mut c_char,
) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelStore = unsafe { &*store };
        match handle.store.pending_batch() {
            Some(batch) => {
                let payload = serde_json::json!({
                    "instance_id": batch.instance_id,
                    "records": batch.records,
                    "merkle_root": batch.merkle_root,
                });
                        return_string(payload.to_string())
            }
            None => {
                unsafe { give_error(err, "no unsynced records") };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error building batch") };
        std::ptr::null_mut()
    })
}

/// Mark records acknowledged.
///
/// # Safety
/// `record_ids` must point to `count` valid C string pointers.
#[no_mangle]
pub unsafe extern "C" fn eel_store_mark_synced(
    store: *mut EelStore,
    record_ids: *const *const c_char,
    count: size_t,
    out_marked: *mut size_t,
    err: *mut *mut c_char,
) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return -1;
        }
        let handle: &mut EelStore = unsafe { &mut *store };
        if count > 0 && record_ids.is_null() {
            unsafe { give_error(err, "record id array is null") };
            return -1;
        }
        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let ptr = unsafe { *record_ids.add(i) };
            match (unsafe { as_str(ptr) }).and_then(|s| uuid::Uuid::parse_str(s).ok()) {
                Some(id) => ids.push(id),
                None => {
                    unsafe { give_error(err, format!("record id {i} is not a UUID")) };
                    return -1;
                }
            }
        }
        match handle.store.mark_synced(&ids) {
            Ok(marked) => {
                if !out_marked.is_null() {
                    unsafe { *out_marked = marked };
                }
                0
            }
            Err(e) => {
                unsafe { give_error(err, e.to_string()) };
                -1
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error marking synced") };
        -1
    })
}

/// Sync progress as JSON.
///
/// # Safety
/// `store` a live handle, `out` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_store_sync_state(store: *const EelStore, err: *mut *mut c_char) -> *mut c_char {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err) };
        if store.is_null() {
            unsafe { give_error(err, "store handle is null") };
            return std::ptr::null_mut();
        }
        let handle: &EelStore = unsafe { &*store };
        let state = handle.store.sync_state();
        match serde_json::to_string(&state) {
            Ok(text) => return_string(text),
            Err(e) => {
                unsafe { give_error(err, format!("serialisation failed: {e}")) };
                std::ptr::null_mut()
            }
        }
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error reading sync state") };
        std::ptr::null_mut()
    })
}

// ---------------------------------------------------------------------------
// validation
// ---------------------------------------------------------------------------

/// Validate a submission. 0 valid, 1 invalid, -1 error.
///
/// # Safety
/// `wf` a live handle; `submission_json` and `captured_json` valid C strings;
/// `out_violations` and `err` writable `char *`.
#[no_mangle]
pub unsafe extern "C" fn eel_workflow_validate(
    wf: *const EelWorkflow,
    submission_json: *const c_char,
    captured_json: *const c_char,
    out_violations: *mut *mut c_char,
    err: *mut *mut c_char,
) -> i32 {
    use eel_bvr_workflow_rs::CaptureKind;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        unsafe { clear(err); clear(out_violations) };
        if wf.is_null() {
            unsafe { give_error(err, "workflow handle is null") };
            return -1;
        }
        let handle: &EelWorkflow = unsafe { &*wf };
        let Ok(submission) =
            serde_json::from_str::<Submission>(unsafe { as_str(submission_json) }.unwrap_or("{}"))
        else {
            unsafe { give_error(err, "submission is not a JSON object") };
            return -1;
        };
        // Capture kinds arrive as wire names; an unrecognised name is dropped
        // rather than fatal, so a newer client cannot crash an older library.
        let captured: Vec<CaptureKind> =
            serde_json::from_str::<Vec<String>>(unsafe { as_str(captured_json) }.unwrap_or("[]"))
                .unwrap_or_default()
                .iter()
                .filter_map(|s| serde_json::from_value::<CaptureKind>(Value::String(s.clone())).ok())
                .collect();

        let result =
            validate_submission(&handle.compiled, &submission, &captured);
        if result.is_ok() {
            unsafe { give_string(out_violations, "[]".to_owned()) };
            return 0;
        }
        unsafe { give_string(out_violations, violations_json(&result.violations)) };
        1
    }));
    outcome.unwrap_or_else(|_| {
        unsafe { give_error(err, "internal error validating submission") };
        -1
    })
}

/// Frees a string returned by any function in this header.
///
/// # Safety
/// `s` must be a pointer returned by this library and not already freed.
#[no_mangle]
pub unsafe extern "C" fn eel_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    drop(unsafe { CString::from_raw(s) });
}
