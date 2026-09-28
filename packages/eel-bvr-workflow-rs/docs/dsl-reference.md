# BVR Enrolment Workflow DSL — Reference

Complete reference for the declarative language that describes a voter
registration workflow. Implemented by `packages/eel-bvr-workflow-rs`, consumed
by the Qt desktop client through the C ABI in
`services/biometric-edge-svc/apps/bvr-desktop/bridge/include/eel_bvr.h`.

Modelled on NIST BiCCL (NIST IR 7531). The transferable idea is not "describe
biometrics declaratively" but "make acquisition *formally* describable and
compile it before use".

- [Design](#design)
- [Document format](#document-format)
- [Reference](#reference)
  - [WorkflowSpec](#workflowspec)
  - [DataItem](#dataitem)
  - [Constraints](#constraints)
  - [DataKind](#datakind)
  - [CaptureRequirement](#capturerequirement)
  - [CaptureKind](#capturekind)
  - [CapturePolicy](#capturepolicy)
  - [Step and StepKind](#step-and-stepkind)
- [Compilation](#compilation)
- [Validation](#validation)
- [Worked examples](#worked-examples)
- [Limits](#limits)
- [Errors](#errors)

---

## Design

Two stages, and nothing consumes an uncompiled document:

```
WorkflowSpec ──compile()──▶ CompiledWorkflow ──validate_submission()──▶ SubmissionError
  (literal JSON)             (resolved, indexed)         (total)
```

**`WorkflowSpec`** is the literal document. **`CompiledWorkflow`** is the result
of `compile`: step references resolved, data items indexed, every constraint
pre-validated. **`validate_submission`** applies the compiled rules to collected
values.

The split exists so that form generation, validation, and submission are three
independent consumers of one resolved description. A Qt form, an Android
`bvr-kit` form, and a replay of a captured record therefore validate
identically — the rules live in one place, not three.

`compile` is not fail-fast. It collects **every** fault in the document and
returns them as a `Vec`, so a malformed document is fixed in one edit cycle
rather than one error per build.

---

## Document format

A workflow is a single JSON object. Every field below is shown with its type;
optional fields may be omitted.

```json
{
  "version": 1,
  "id": "bvr-enrolment-standard",
  "label": "Standard voter enrolment",

  "data_items": [
    {
      "name": "full_name",
      "kind": "text",
      "label": "Full legal name",
      "required": true,
      "data_class": "A",
      "help": "Optional helper text",
      "constraints": { "min_length": 1, "max_length": 200 }
    }
  ],

  "capture_requirements": [
    {
      "id": "fingerprints",
      "label": "Fingerprints",
      "kinds": ["fingerprint"],
      "policy": "all_of",
      "optional": false
    }
  ],

  "steps": [
    { "id": "identity", "kind": "data_entry", "items": ["full_name"], "label": "Details" },
    { "id": "prints", "kind": "capture", "capture": "fingerprints" },
    { "id": "review", "kind": "review" },
    { "id": "submit", "kind": "submit" }
  ]
}
```

### Value types in a submission

A submission is a JSON object mapping data-item name to value. The types are
deliberately narrow:

| Data kind | JSON type | Example |
|---|---|---|
| `consent` | boolean | `true` |
| everything else | string | `"12345678"`, `"1990-06-15"` |

Values are strings because the server owns typed parsing. A local type on the
client could drift from the contract; a string cannot. Consent is boolean
because "affirmatively answered" and "present but empty" must be
distinguishable, and they are not if both are strings.

---

## Reference

### WorkflowSpec

| Field | Type | Required | Meaning |
|---|---|---|---|
| `version` | integer | yes | Spec schema version. Only `1` is understood. |
| `id` | string | yes | Stable workflow identifier. |
| `label` | string | yes | Human-readable workflow name. |
| `data_items` | array of [DataItem](#dataitem) | yes | Collectable items. May be empty only if the workflow also has no steps. |
| `capture_requirements` | array of [CaptureRequirement](#capturerequirement) | no | Named capture definitions. May be omitted or empty for a workflow that acquires no biometrics. |
| `steps` | array of [Step](#step-and-stepkind) | yes | Ordered steps. |

### DataItem

| Field | Type | Required | Meaning |
|---|---|---|---|
| `name` | string | yes | Stable key, unique in the workflow. Also the submitted field name. |
| `kind` | [DataKind](#datakind) | yes | Widget and validator to use. |
| `label` | string | yes | Text shown to the enrolment officer. |
| `required` | boolean | yes | Whether the item must be satisfied to advance. |
| `data_class` | `"A"`…`"E"` | yes | PII tier. Mirrors `eel-pii-rs::PIIDataClass`. |
| `help` | string | no | Helper text shown with the field. |
| `constraints` | [Constraints](#constraints) | no | Value bounds. Omit when there are none. |

`data_class` accepts all five platform tiers, not just the A/B/C subset the BVR
contract happens to tag on its `Voter` schema. Anything outside A–E is a
compile error.

| Tier | Meaning |
|---|---|
| `A` | Voter PII, maximum restriction |
| `B` | Voter operational data |
| `C` | Publicly available data |
| `D` | Operational metadata |
| `E` | Ephemeral data |

### Constraints

| Field | Type | Applies to | Meaning |
|---|---|---|---|
| `min_length` | integer | `text`, `choice` | Inclusive minimum string length. |
| `max_length` | integer | `text`, `choice` | Inclusive maximum string length. |
| `options` | array of string | `choice` | Permitted values. **Required** for `choice`. |
| `min_age_years` | integer | `date` | Inclusive minimum age in whole years. |
| `max_age_years` | integer | `date` | Inclusive maximum age in whole years. |

Rules:

- `min_length` > `max_length` is a compile error.
- Length bounds on any other kind is a compile error, **even if the bound
  happens to agree with the kind's pattern length**. A pattern-typed field's
  length is the contract's business, not the document's.
- Age bounds on a non-`date` item is a compile error.
- A `choice` with no `options` is a compile error. A repeated option is a
  compile error.
- Each age bound is enforced **independently**. Declaring only `min_age_years`
  is valid and enforces exactly that bound.
- A `date` is always parsed as a real ISO 8601 calendar date, whether or not any
  age bound is declared.

### DataKind

| Value | Widget | Validation |
|---|---|---|
| `text` | Single-line text | `min_length`, `max_length` |
| `date` | Date picker | Must parse as ISO 8601; age bounds if set |
| `national_id` | Text, digits only | Pattern `^[0-9]{8}$` |
| `phone` | Text | Pattern `^\+254[0-9]{9}$` |
| `station_code` | Text | Pattern `^[A-Z0-9]{6,12}$` |
| `choice` | Dropdown | Must be one of `constraints.options` |
| `consent` | Checkbox | Must be `true` when `required` |

Pattern kinds delegate to `eel_spec_helpers_rs::matches_bvr_pattern`, which
mirrors the BVR contract. The client carries no regexes of its own for contract
fields, so a pattern change lands in one place.

> **Known contract discrepancy.** The contract's own example for
> `polling_station_code` is `"NRB-001-PS-042"`, which its own pattern
> `^[A-Z0-9]{6,12}$` does not admit. The **pattern** is enforced. Someone
> should reconcile the example and the pattern in the contract.

### CaptureRequirement

A named, reusable capture definition. Steps reference it by `id`, so one
definition can be shared and a dangling reference is caught at compile time
exactly like a missing data item.

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Stable identifier, unique. Referenced by capture steps. |
| `label` | string | yes | Human-readable name. |
| `kinds` | array of [CaptureKind](#capturekind) | yes | Acceptable acquisitions. May not be empty. |
| `policy` | [CapturePolicy](#capturepolicy) | yes | How many are needed. |
| `optional` | boolean | no | Defaults to `false`. When true the step may be skipped entirely. |

### CaptureKind

Capture is modelled separately from biometric **matching** modality, for two
reasons. A document scan produces MRZ text, not a template, so it must not be
routed into biometric handling. And a matching modality does not imply a capture
path exists: palm vein already has a matcher (`abis-modality-vein`, ISO/IEC
19794-9) but NIR vascular capture is a different sensor class from optical
contact capture.

| Value | Produces | Standard | Notes |
|---|---|---|---|
| `fingerprint` | biometric | ISO/IEC 19794-2 | Contact optical |
| `finger_vein` | biometric | ISO/IEC 19794-9 | NIR vascular |
| `palm_vein` | biometric | ISO/IEC 19794-9 | NIR vascular |
| `wrist_vein` | biometric | ISO/IEC 19794-9 | NIR vascular |
| `palm_print` | biometric | ISO/IEC 19794-2 | Contact |
| `face` | biometric | ISO/IEC 19794-5 | |
| `live_photograph` | biometric | ISO/IEC 19794-5 + 30107-3 PAD | Liveness-challenged |
| `periocular` | biometric | ISO/IEC 19794-5 | Eye region |
| `iris` | biometric | ISO/IEC 19794-6 | |
| `ear` | biometric | none | |
| `retina` | biometric | none | |
| `voiceprint` | biometric | ISO/IEC 19794-13 | Behavioural |
| `gait` | biometric | ISO/IEC 39794-17 | Behavioural; not 19794-6, which is iris |
| `signature` | biometric | ISO/IEC 19794-7 | Behavioural |
| `keystroke_dynamics` | biometric | none | Behavioural, passively collected |
| `mouse_dynamics` | biometric | none | Behavioural, passively collected |
| `dna` | biometric | none | Analysed ex vivo |
| `document_mrz` | **document data** | ICAO 9303 | **Not biometric** |

Eighteen kinds, mirroring `sensor_driver::Modality::ALL`. The DSL and the trait
must agree: a test asserts the wire names match, because a workflow that names a
capture the kit cannot perform is a runtime failure otherwise.

`document_mrz` is the only kind where `is_biometric()` is false.

### CapturePolicy

| Value | Satisfied when |
|---|---|
| `"all_of"` | Every listed kind was captured |
| `"any_of"` | At least one listed kind was captured |
| `{ "at_least": N }` | N or more listed kinds were captured |

`at_least` exists because a registration drive cannot always complete a full
acquisition on one citizen, and the platform already supports degraded capture
through `BiometricSensorDriver::capture_fingerprint_degraded`.

Compile errors: `at_least: 0` is **vacuous** (can never fail), and `at_least`
demanding more than the number of listed kinds is **unsatisfiable**. A repeated
kind is also an error.

### Step and StepKind

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Unique within the workflow. |
| `kind` | one of the below | yes | Discriminator. |
| `label` | string | no | Overrides the generated label. |

`StepKind` is a tagged union; `kind` is the tag.

| `kind` | Extra fields | Meaning |
|---|---|---|
| `data_entry` | `items`: array of data-item names | Collect and validate the listed items |
| `capture` | `capture`: requirement id | Perform the named capture |
| `review` | — | Present the accumulated record |
| `submit` | — | Submit to the BVR enrolment endpoint |

Generated labels when `label` is absent: `Details` for a data-entry step,
`Capture <Kind>` or `<label> (N options)` for a capture step, `Review`, `Submit`.

---

## Compilation

`compile(spec) -> Result<CompiledWorkflow, Vec<CompileError>>`

### Guarantees

On success:

- Every `items` entry in a `data_entry` step names an existing data item.
- Every `capture` entry names an existing capture requirement.
- Every data item is reachable from at least one step.
- Every capture requirement is referenced by at least one step.
- All constraints are internally consistent.
- `data_items` is indexed by name for O(1) lookup; `data_item_order` preserves
  declaration order for stable form layout.

### Rejections

| Condition | Error |
|---|---|
| `version` ≠ 1 | `UnsupportedVersion` |
| no data items | `NoDataItems` |
| no steps | `NoSteps` |
| two data items share a name | `DuplicateDataItem` |
| two steps share an id | `DuplicateStepId` |
| step names an unknown data item | `UnknownDataItem` |
| data item no step collects | `UnreachableDataItem` |
| `min_length` > `max_length` | `LengthBoundsInverted` |
| length bounds on a non-text kind | `LengthBoundOnNonText` |
| `min_age_years` > `max_age_years` | `AgeBoundsInverted` |
| age bounds on a non-`date` | `AgeBoundOnNonDate` |
| `choice` with no `options` | `ChoiceWithoutOptions` |
| repeated option | `DuplicateOption` |
| `data_class` outside A–E | `InvalidDataClass` |
| two capture requirements share an id | `DuplicateCaptureId` |
| step names an unknown capture | `UnknownCapture` |
| capture requirement no step uses | `UnreferencedCapture` |
| capture with no kinds | `CaptureWithoutKinds` |
| repeated kind | `DuplicateCaptureKind` |
| `at_least` exceeds the kind count | `CapturePolicyUnsatisfiable` |
| `at_least: 0` | `CapturePolicyVacuous` |

---

## Validation

`validate_submission(workflow, submission, captured) -> SubmissionError`

`captured` is the list of capture kinds the operator reported acquiring.

- **Total, not fail-fast.** Every violation is returned. An officer sees the
  whole problem at once rather than one error per attempt.
- **Undeclared fields are rejected.** A submission carrying a field the
  workflow does not declare is a `UndeclaredItem` violation, so a stale or
  tampered payload cannot smuggle extra fields into the submit call.
- **Absent is uniform.** A missing key, a `null`, an empty string, and
  whitespace-only all count as absent, so `required` applies the same way
  whatever the widget produced.
- **Consent is separate.** An unticked `consent` item is
  `ConsentNotGiven` when required, not `MissingRequired`, so the UI can say
  something meaningful.
- **Age is truncated, not rounded.** Someone is admitted on the day before
  their birthday and excluded the day after — the conservative reading for a
  minimum-age rule.

| Violation | Raised when |
|---|---|
| `MissingRequired` | Required item absent or blank |
| `ConsentNotGiven` | Required consent not `true` |
| `PatternMismatch` | Value fails its kind's contract pattern |
| `TooShort` / `TooLong` | Value outside length bounds |
| `NotAnOption` | Value not in `options` |
| `NotADate` | Value is not a real ISO 8601 date |
| `AgeOutOfRange` | Implied age outside bounds |
| `UndeclaredItem` | Submission carries an unknown field |
| `CaptureUnsatisfied` | A required capture step's policy is unmet |

---

## Shipped documents

Two workflows ship with the client, both under
`services/biometric-edge-svc/apps/bvr-desktop/workflows/`, and both pinned by
tests so they cannot drift from the contract or from this reference.

| Document | Purpose |
|---|---|
| `standard-enrolment.json` | The default enrolment. Citizen details, residence, document MRZ, fingerprints, liveness-checked photograph, consent, review, submit. |
| `extended-enrolment.json` | Optional and alternative modalities. Reaches all eighteen capture kinds; only the document and one primary biometric are mandatory, so a kit can deploy it unchanged without serving everything. |

`extended-enrolment.json` is the reference example for optional captures and for
the `at_least` policies below.

## Worked examples

### Individually or in any combination

One step, several acceptable acquisitions, a policy saying how many are needed.
This is the point of `CapturePolicy` — without it, "any of these three" would
need three separate steps and the officer could not skip one.

```json
{
  "capture_requirements": [
    {
      "id": "any_vascular",
      "label": "Palm or finger or wrist vein",
      "kinds": ["palm_vein", "finger_vein", "wrist_vein"],
      "policy": { "at_least": 1 }
    }
  ],
  "steps": [{ "id": "vessel", "kind": "capture", "capture": "any_vascular" }]
}
```

### Degraded acquisition

Accept fewer than the ideal, because a drive cannot always complete a full
acquisition on one citizen.

```json
{
  "id": "combo",
  "label": "Any acceptable capture",
  "kinds": ["fingerprint", "palm_vein", "finger_vein"],
  "policy": { "at_least": 2 }
}
```

Two of three satisfies this; one does not.

### Document plus biometrics

```json
{
  "capture_requirements": [
    { "id": "doc",  "label": "Identity document", "kinds": ["document_mrz"], "policy": "all_of" },
    { "id": "bio",  "label": "Primary biometric",  "kinds": ["fingerprint", "palm_vein", "live_photograph"], "policy": { "at_least": 1 } }
  ]
}
```

Note `bio` is `at_least: 1`, not `all_of`. `all_of` over three *alternative*
modalities demands all three — a mistake this DSL's compiler cannot detect,
because the document is internally consistent, but which the tests caught while
this feature was being built.

### No biometrics at all

A document-only pre-registration review. Omit `capture_requirements` entirely
and have no `capture` steps.

---

## Limits

Deliberate, and enforced rather than half-expressed:

- **No arbitrary branching or loops.** A workflow that would need them is
  rejected by omission, not approximated. Sequencing and capture policy cover
  the real requirements.
- **No expressions.** Constraints are literal bounds, not expressions. A
  document that needs one is a document that needs a new `DataKind`.
- **No data transformation.** The DSL describes what is collected and validated.
  Deriving one item from another is out of scope; `CaptureMethod::Derived` in
  the store records that it happened, but the derivation itself lives in code.
- **Capture does not bind hardware.** A `capture` step names a requirement. The
  client binds it to `BiometricSensorDriver` per ADR-0026. A modality the kit
  cannot serve is reported as unsupported, not faked.

---

## Errors

Both error enums implement `Display` via `thiserror`, so a message is
human-readable without a formatter.

**Rust.** `CompileError` and `ValidationError` are plain enums; match on
variants rather than parsing messages.

**C ABI.** The bridge converts them to JSON:

- `eel_workflow_load` — on failure returns `NULL` and writes a **JSON array of
  compile-error message strings** to `*err`. On success `*err` is `NULL`.
- `eel_workflow_validate` — returns `0` valid, `1` invalid, `-1` error. On `1`,
  `*out_violations` holds a JSON array of `{ "code", "message" }`. `code` is
  stable and localisable; `message` is for logs. **Match on `code`**, not on
  the message text.

A document that fails to *parse* is a different condition from one that fails to
*compile*: it returns a plain message naming the parse failure, not a list of
compile errors.

---

## Related

- `packages/eel-bvr-workflow-rs/README.md` — overview and guarantees
- `packages/eel-bvr-workflow-rs/TRAINING.md` — tutorial and exercises
- `docs/architecture/decisions/0027-desktop-bvr-client.md` — ADR
- ADR-0026 — hardware latitude and the driver abstraction
- `services/biometric-edge-svc/apps/bvr-desktop/workflows/` — shipped documents
