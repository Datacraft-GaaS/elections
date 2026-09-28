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
capture_requirements:
  # Named so several steps can share one definition, and so a dangling
  # reference is caught at compile time like a missing data item.
  - id: fingerprints
    label: Left and right fingerprints
    kinds: [fingerprint]
    policy: all_of          # all_of | any_of | { at_least: N }
    optional: false
  # "In any combination, or individually": one step, several acceptable
  # captures, and a policy saying how many are actually needed.
  - id: vascular
    label: Palm or finger vein
    kinds: [palm_vein, finger_vein, wrist_vein]
    policy: { at_least: 1 }
    optional: false
steps:
  - id: identity
    kind: data_entry
    items: [full_name, gender, dob]
  - id: prints
    kind: capture
    capture: fingerprints
  - id: vessel
    kind: capture
    capture: vascular
  - id: review
    kind: review
  - id: submit
    kind: submit
```

### Capture kinds

Capture is modelled separately from biometric *matching* modality, because a
capture need not produce a biometric template and because a matching modality
does not imply a capture path exists.

| Kind | Produces | Standard | Capture path |
|---|---|---|---|
| `fingerprint` | biometric | ISO/IEC 19794-2 | optical contact sensor |
| `finger_vein` | biometric | ISO/IEC 19794-9 | NIR vascular sensor |
| `palm_vein` | biometric | ISO/IEC 19794-9 | NIR vascular sensor |
| `wrist_vein` | biometric | ISO/IEC 19794-9 | NIR vascular sensor |
| `face` | biometric | ISO/IEC 19794-5 | RGB/NIR camera |
| `live_photograph` | biometric | ISO/IEC 30107-3 PAD | camera with liveness challenge |
| `iris` | biometric | ISO/IEC 19794-6 | NIR iris sensor |
| `document_mrz` | **document data** | ICAO 9303 | optical document reader |

`document_mrz` is deliberately not a biometric: its output is MRZ text, so it
must not be routed through biometric template handling.
`CaptureKind::is_biometric` encodes that distinction.

`face` and `live_photograph` are separate kinds so a workflow can demand a
liveness-proven capture, not just an image.

### Capture policies

- `all_of` -- every declared kind must be captured.
- `any_of` -- at least one.
- `{"at_least": N}` -- N or more, where `N` must not exceed the number of
  declared kinds.

`at_least` exists because a registration drive cannot always complete a full
acquisition on one citizen, and the platform already supports degraded capture
through `BiometricSensorDriver::capture_fingerprint_degraded`. A workflow with no
`capture_requirements` at all is valid, for a document-only pre-registration
review.

## Guarantees the compiler enforces

- Duplicate data-item names and duplicate step ids.
- Steps referencing data items that do not exist.
- Data items declared but collected by no step (dead form fields).
- Capture requirements never referenced by a step; steps referencing a
  requirement that does not exist; duplicate requirement ids.
- Capture requirements with no kinds, repeated kinds, `at_least: 0` (vacuous),
  and `at_least` demanding more captures than the requirement lists.
- Inverted length and age bounds; length bounds on non-text kinds; age bounds on
  non-date kinds; `choice` without options; duplicate options.
- Data classes outside `A` / `B` / `C`.

`compile` reports **all** faults in one pass rather than stopping at the first,
so a malformed document is fixed in one edit cycle.

Pattern checks for `national_id`, `phone`, and `polling_station_code` are
delegated to `eel_spec_helpers_rs::matches_bvr_pattern`, keeping the contract
patterns in one place.

## Biometric capture

Capture steps name a requirement; they do not talk to hardware. The client binds
them to the existing `BiometricSensorDriver` trait
(`services/biometric-edge-svc/crates/sensor-driver/src/sensor_driver.rs`) per
ADR-0026, so vendor selection stays a build-time choice and CI runs against
`MockSensorDriver`.

Note the honest limit: that trait currently exposes only `capture_fingerprint`,
`capture_face`, and `capture_iris`. Palm, finger vein, wrist vein, live
photograph, and document MRZ have **no capture method yet**, so a client binding
this model to the trait today would have to stub them. Their *matching* support
exists (`abis-modality-vein` for vascular patterns, `integrations/mrz` for
document parsing); the capture methods do not. Extending the trait is a
prerequisite for those capture kinds, and the DSL deliberately does not pretend
otherwise.

## Scope

This crate is the declarative core only: document model, compiler, and
submission validation. It does not render a UI, speak HTTP to the BVR service, or
drive sensors.

## Naming

All identifiers use the neutral `eel` prefix per ADR-0026. No client name
appears in any identifier.
