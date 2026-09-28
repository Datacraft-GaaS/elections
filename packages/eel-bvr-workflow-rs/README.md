# eel-bvr-workflow-rs

Declarative enrolment workflows for BVR (M01, Biometric Voter Registration),
modelled on NIST's BiCCL (Biometric Client Configuration Language, NIST IR 7531).

## Why this exists

The desktop/laptop registration client must present a **configurable list of data
items to collect and validate**. Two routes were available:

1. Hand-written widgets per field. Every new field is a code change and a
   release.
2. A declarative document, compiled at load time. Adding a field, reordering
   steps, or dropping an optional field is an edit to a document that a
   registration officer or programme manager can make.

This crate is route 2. The same document drives the desktop client, the Android
`bvr-kit`, and any replay of a captured record, so validation cannot drift
between form factors.

BiCCL's actual contribution was not the concept of describing biometric
acquisition declaratively but making it *formally describable* and compiling it
before use. This crate mirrors that shape: `WorkflowSpec` is the literal
document, `compile` is the reference compiler, and nothing consumes an
uncompiled spec.

## Two-stage design

```
WorkflowSpec  ──compile()──▶  CompiledWorkflow  ──validate_submission()──▶  SubmissionError
 (literal JSON)              (resolved, indexed)          (total)
```

`compile` resolves step-to-data-item references, rejects a data item no step
collects, and pre-validates constraints. `validate_submission` applies the
compiled rules to collected values.

## Document format

```yaml
version: 1
id: bvr-enrolment-standard
label: Standard voter enrolment
data_items:
  - name: full_name
    kind: text              # text | date | national_id | phone | station_code | choice | consent
    label: Full legal name
    required: true
    data_class: A           # mirrors the contract's x-eel-data-class
    constraints:
      min_length: 1
      max_length: 200
  - name: gender
    kind: choice
    label: Gender
    required: true
    data_class: A
    constraints:
      options: [male, female, other, prefer_not_to_say]
  - name: dob
    kind: date
    label: Date of birth
    required: true
    data_class: A
    constraints:
      min_age_years: 18
      max_age_years: 120
  - name: consent
    kind: consent           # must be affirmatively true
    label: Citizen consents to biometric enrolment
    required: true
    data_class: A
steps:
  - id: identity
    kind: data_entry
    items: [full_name, gender, dob]
  - id: left_fingers
    kind: biometric_capture
    modality: fingerprint   # fingerprint | face | iris
    required: true
  - id: review
    kind: review
  - id: submit
    kind: submit
```

## Guarantees the compiler enforces

- Duplicate data-item names and duplicate step ids.
- Steps referencing data items that do not exist.
- Data items declared but collected by no step (dead form fields).
- Inverted length and age bounds; length bounds on non-text kinds; age bounds on
  non-date kinds; `choice` without options; duplicate options.
- Data classes outside `A` / `B` / `C`.

`compile` reports **all** faults in one pass rather than stopping at the first,
so a malformed document is fixed in one edit cycle.

Pattern checks for `national_id`, `phone`, and `polling_station_code` are
delegated to `eel_spec_helpers_rs::matches_bvr_pattern`, keeping the contract
patterns in one place.

## Biometric capture

Capture steps name a modality; they do not talk to hardware. The client binds
them to the existing `BiometricSensorDriver` trait
(`services/biometric-edge-svc/crates/sensor-driver/src/sensor_driver.rs`) per
ADR-0026, so vendor selection stays a build-time choice and CI runs against
`MockSensorDriver`.

## Scope

This crate is the declarative core only: document model, compiler, and
submission validation. It does not render a UI, speak HTTP to the BVR service, or
drive sensors.

## Naming

All identifiers use the neutral `eel` prefix per ADR-0026. No client name
appears in any identifier.
