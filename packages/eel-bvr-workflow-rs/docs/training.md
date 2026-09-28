# BVR Enrolment Workflow DSL — Training

A hands-on course for the people who configure voter registration workflows.
Written for two audiences:

- **Registration officers and programme managers** (Part 1) — what the form
  will ask for and why, no programming needed.
- **Client and service developers** (Parts 2–4) — compiling, validating, and
  extending.

Every exercise is runnable. Nothing here is hypothetical.

**Prerequisites:** read
[`docs/dsl-reference.md`](dsl-reference.md) for the field-by-field reference.
This document teaches the reasoning.

---

## Part 1 — Configuring a form (no programming)

### 1.1 What this language is for

A registration form has to change between elections, between counties, and
between voter categories. If the form were built out of hand-written code, every
change would need a developer, a build, and a rollout to every kit in the
country. The DSL makes the form a **document**. Changing what is collected is
editing a JSON file.

The trade is that you can no longer ask for something the language does not
express. Part 1.6 covers where that line sits.

### 1.2 Your first workflow

Open `services/biometric-edge-svc/apps/bvr-desktop/workflows/standard-enrolment.json`.

A workflow has three parts:

| Part | What it is |
|---|---|
| `data_items` | The things to collect and validate |
| `capture_requirements` | The biometrics or documents to acquire, named |
| `steps` | The order: when each item is asked for |

Read it in that order. Then answer these questions:

1. Which items are **required**? Which are optional? How do you tell?
2. `national_id` has no `constraints` block. How is it validated anyway?
3. Why is `polling_station_code` not a `choice`?

**Answers.** (1) `"required": true` / `false`. (2) Its `kind` is `national_id`,
which implies the contract pattern `^[0-9]{8}$`; you do not restate it.
(3) A polling station code is a code, not a fixed list — the set of stations
differs per county, so enumerating options in the document would be wrong.

**Exercise 1.** *Add a `county` text item with exactly two characters, required,
class `B`, asked on the residence step.* Where do you add it — in `data_items`
or in `steps`? **Both**, and that is the thing to internalise: `data_items`
declares it exists, a step's `items` array makes it reachable. Declare it and
forget the step and the compiler rejects the document with
`UnreachableDataItem`.

### 1.3 Choosing a `kind`

`kind` picks the widget and the validator together. There is no separate
"validator" field, deliberately — a field that can be typed into one way and
validated another is a bug waiting to happen.

| You want | Use | Watch out |
|---|---|---|
| A name, a code, free text | `text` | Add `min_length` / `max_length` |
| A date of birth | `date` | Add `min_age_years` |
| The 8-digit national ID | `national_id` | Pattern is implied |
| A Kenyan mobile number | `phone` | Must be `+254…`, not `07…` |
| A polling station code | `station_code` | Uppercase letters and digits only |
| A fixed set of answers | `choice` | **`options` is required** |
| A yes/no the officer must affirm | `consent` | Boolean `true`, not a string |

**Exercise 2.** *A county collects the sub-county as free text, up to 40
characters, and requires it. Write the `data_items` entry.* Then: what is the
`data_class`? Sub-county is operational data, not citizen PII, so `B`.

### 1.4 Choices

A `choice` needs `options`, and the compiler rejects it without:

```json
{
  "name": "id_type",
  "kind": "choice",
  "label": "Identification document type",
  "required": true,
  "data_class": "B",
  "constraints": { "options": ["national_id", "passport", "birth_certificate"] }
}
```

Options may not repeat — a repeated option is a compile error, because it is
almost always a copy-paste slip that makes one option look selectable twice.

### 1.5 Dates and age

`date` is always checked for being a real calendar date. `1990-13-45` is
rejected whether or not you set an age bound.

Age bounds are independent, which matters more than it sounds:

| Document | Effect |
|---|---|
| `min_age_years: 18` only | Under-18s rejected. No upper bound. |
| `max_age_years: 120` only | Over-120s rejected. No minimum. |
| both | Full band. |

An earlier version of the DSL required **both** to be set before checking
either, so `min_age_years: 18` alone compiled cleanly and enforced nothing. That
is silent under-enforcement in a registration flow, which is the worst kind of
bug, and it is fixed. Do not assume the same trap elsewhere: check the
behaviour, do not assume it.

Age is **truncated, not rounded**. Someone is admitted on the day before their
birthday and excluded the day after.

### 1.6 What the language will not do

The DSL deliberately has no branching, no loops, and no expressions. A workflow
that would need them is refused by omission rather than half-expressed.

Practical consequence: if you find yourself wanting a conditional — "only ask
for X if the citizen is a refugee" — you cannot express it here, and you should
raise it rather than working around it. A half-expressed conditional reads as a
rule and is not one.

---

## Part 2 — Captures

### 2.1 Naming, then referencing

Captures are declared once in `capture_requirements` with an `id`, and steps
reference that id:

```json
"capture_requirements": [
  { "id": "fingerprints", "label": "Fingerprints", "kinds": ["fingerprint"], "policy": "all_of" }
],
"steps": [
  { "id": "prints", "kind": "capture", "capture": "fingerprints" }
]
```

Why not inline the requirement into the step? Because the same definition is
often used twice, and because a dangling reference is now caught at compile time
exactly like a missing data item. Two failure modes, one mechanism.

**Exercise 3.** *Write a workflow that captures a document and any one
biometric.* You need two requirements and two steps. The biometric requirement
lists `fingerprint`, `palm_vein`, `live_photograph`.

**The trap.** You will want `"policy": "all_of"` for the biometric step. Do not.
`all_of` over three *alternative* modalities demands **all three**. For
"any one of these", use `{"at_least": 1}`. This exact mistake was made while
building the extended workflow, and the document compiled cleanly — it is
semantically wrong, not syntactically wrong, so no compiler can catch it. The
tests did.

### 2.2 The three policies

| Policy | Means | Use when |
|---|---|---|
| `all_of` | Every listed kind | The list is a checklist, not alternatives |
| `any_of` | At least one | Same as `at_least: 1`; prefer `at_least` for consistency |
| `at_least: N` | N or more | Alternatives, and degraded acquisition |

`at_least` is the interesting one. A registration drive cannot always complete
a full acquisition on one citizen — a finger is injured, a palm is callused, the
citizen cannot stand for gait. The platform supports degraded capture
(`capture_fingerprint_degraded`), and `at_least` is how a workflow says "two of
these three is a successful enrolment".

**Exercise 4.** *A county enrols with fingerprint, palm vein, or finger vein,
and will accept two of the three. Write the requirement.*

**Exercise 5.** *A kit might have only a fingerprint reader. What should a
workflow that requires `at_least: 2` of three vascular-capable kinds do?*
Answer: the driver reports the modality unsupported and the client greys out the
step. The workflow is not wrong; the kit is under-specified. Check
`BiometricSensorDriver::supported_modalities` before deployment.

### 2.3 Optional captures

`"optional": true` means the step can be skipped with nothing captured. Use it
for anything beyond the mandatory minimum, so a station can deploy a
capability-rich document unchanged on a kit that only serves part of it.

The shipped `extended-enrolment.json` does exactly this: only the document and
one primary biometric are mandatory; every other modality is optional, and a
citizen who completes only those two still satisfies the whole workflow.

### 2.4 The eighteen kinds

Full table in the [reference](dsl-reference.md#capturekind). Three things to
internalise:

**`document_mrz` is not a biometric.** Its output is MRZ text, not a template.
It must not be routed into biometric template storage or retention. The DSL and
the capture trait both mark it, and tests assert the classification.

**Collection mode is not one thing.** `keystroke_dynamics`, `mouse_dynamics`,
and `dna` do not fit a timed capture call — the first two accumulate during a
session, the third is analysed away from the subject. The client uses
`collection_mode` to pick the right UI. If you add a modality, you are
committing to how it is acquired.

**Some kinds have no binding standard.** `ear`, `retina`, `keystroke_dynamics`,
`mouse_dynamics`, and `dna` cite none. That is deliberate: naming a standard
that does not govern the data would be worse than naming none in a tender
deliverable. Do not add a standard number to make a table look complete without
checking it.

---

## Part 3 — For developers

### 3.1 Compile before you trust

```rust
use eel_bvr_workflow_rs::{compile, WorkflowSpec};

let spec: WorkflowSpec = serde_json::from_str(&document)?;
let workflow = compile(&spec)?;   // every fault, not just the first
```

Two rules:

1. **Nothing consumes an uncompiled spec.** If you find yourself needing to
   resolve a step reference by hand, you are bypassing the compiler.
2. **`compile` returns a `Vec<CompileError>`, not the first error.** Show the
   officer all of them. One error per build means a three-round-trip fix for a
   document with three mistakes.

### 3.2 Generate forms from `CompiledWorkflow`

`CompiledWorkflow` exists to make form generation mechanical:

```rust
for step in &workflow.steps {
    match &step.step.kind {
        StepKind::DataEntry { .. } => {
            for name in &step.items {
                let field = workflow.data_item(name).expect("compiled");
                // field.item.kind selects the widget;
                // field.item.constraints carries the bounds to enforce.
            }
        }
        StepKind::Capture { capture } => {
            let requirement = workflow.capture(capture).expect("compiled");
            // requirement.requirement.kinds lists the acceptable acquisitions.
        }
        _ => {}
    }
}
```

Use `data_item_order` for layout, not the `HashMap` iteration order — form
order must be stable, or an officer's muscle memory breaks between kits.

### 3.3 Validate on the client *and* on the server

`validate_submission` is not a substitute for server-side validation. It exists
so the officer gets told at the point of entry rather than after a failed sync
on a kit with no connectivity. Both sides must run it; they are the same code,
which is the point.

```rust
let result = validate_submission(&workflow, &submission, &captured_kinds);
if !result.is_ok() {
    for violation in &result.violations {
        // violation is a ValidationError; Display gives a message.
    }
}
```

`captured` is what the operator reported acquiring. It is not inferred from
hardware presence — a sensor being connected is not a capture having happened.

### 3.4 Persist with provenance

Every value is sealed through `eel_bvr_store_rs`, which records the platform's
five-axis audit reference per record: **where** (station and instance), **when**,
**who** (officer), **why** (purpose), **how** (workflow, step, method). Plus an
`audit_id` UUIDv7, a `proofHash`, and a signature.

If you add a code path that writes a datum, it goes through the store. A value
written any other way has no provenance and cannot be traced, which makes it
useless in an election audit no matter how well the rest of the system works.

### 3.5 Test your workflows

Tests in `packages/eel-bvr-workflow-rs/tests/workflow.rs` are the pattern. A
workflow test should:

- compile the document, asserting it is accepted;
- assert the compiler **rejects** a specific malformation and for the right
  reason;
- validate a complete submission;
- validate a submission missing one required element, and one missing a capture.

Prefer asserting on a specific `ValidationError` variant over counting
violations. "There is one violation" passes for the wrong reason just as easily
as for the right one.

Pin shipped documents with tests, as `standard-enrolment.json` and
`extended-enrolment.json` are. A shipped document is configuration, and
configuration drifts silently.

### 3.6 Run the checks

```bash
# Compile and test the DSL.
cargo test -p eel-bvr-workflow-rs

# Check a document without launching a UI. Headless; use in CI.
cargo build -p eel-bvr-bridge --release --locked
./bvr-desktop/eel-bvr-desktop --check --workflow path/to/workflow.json

# Integrity of a new document against the contract, across the workspace.
cargo check --workspace --locked --all-targets
```

Exit codes, so a deployment script can branch on them:

| Code | Meaning |
|---|---|
| `0` | Workflow compiled and the store verified |
| `2` | Workflow rejected (see the compile errors on stderr) |
| `3` | Store could not be created |
| `4` | **Store failed its integrity check** |
| `5` | No officer identity supplied |
| `6` | `--store-key` was not 32 bytes |

`4` is the one that matters. A tampered store must not be used for
registration, and `--check` is how a deployment pipeline finds that out before
a kit reaches a county.

`--check` also validates the store key, because a smoke test that skips the
misconfiguration it exists to catch is worse than no smoke test. The key seals
the data key at rest and is never written to disk; in a deployed station it
comes from a TPM or HSM and never leaves the hardware.

---

## Part 4 — Worked example: a refugee-specific workflow

A realistic end-to-end exercise. Say a county runs a supplementary registration
drive for refugees, who present a UNHCR card rather than a national ID.

**Step 1 — decide what changes.** A refugee has no `national_id`. The
`national_id` item cannot simply be dropped from the standard document, because
the `Voter` contract marks it required. This is a genuine constraint, not a DSL
limitation, and it is the right place to stop and ask rather than work around.

**Step 2 — what the DSL can do.** If the backend accepts an alternative
identifier, the shape is an additional required item plus a `choice` recording
which document type was seen:

```json
{
  "name": "document_type",
  "kind": "choice",
  "label": "Document presented",
  "required": true,
  "data_class": "B",
  "constraints": { "options": ["national_id", "unhcr_card"] }
},
{
  "name": "unhcr_number",
  "kind": "text",
  "label": "UNHCR reference number",
  "required": false,
  "data_class": "A",
  "constraints": { "min_length": 6, "max_length": 20 }
}
```

**Step 3 — what it cannot do.** "Only ask for `unhcr_number` when
`document_type` is `unhcr_card`" is a conditional, and the DSL has none. Options
are: make it always visible with a clear label, or put the conditional in client
code and lose the property that the document alone describes the form.

Choose the first. A form that always offers the field is understandable; a form
whose shape depends on invisible client logic is not, and a reviewer reading only
the document cannot tell what will be asked.

**Exercise 6.** *Write the full workflow for this drive: a `document_type`
choice, an optional `unhcr_number`, a required date of birth and full name, an
optional phone, mandatory document MRZ capture, and one biometric from
fingerprint, palm vein, or live photograph.* Then compile it:

```bash
cargo build -p eel-bvr-bridge --release --locked
./bvr-desktop/eel-bvr-desktop --check --workflow my-workflow.json
```

**Exercise 7.** *Deliberately break it four ways and read the errors.* Add a
`data_items` entry that no step references. Set `min_length: 100` with
`max_length: 10`. Add a `choice` with no `options`. Reference a capture id that
does not exist. Note that all four are reported together, not one per run.

**Exercise 8.** *Write the Rust test for your workflow.* Assert it compiles,
assert a submission with every required item and one capture passes, and assert
a submission with no capture fails with `CaptureUnsatisfied`. Follow
`the_shipped_standard_workflow_compiles` in the existing test file.

---

## Part 5 — What to escalate

Raise these rather than working around them:

| Situation | Why it is not yours to solve |
|---|---|
| The form needs a conditional | No branching in the DSL; a workaround hides the rule |
| A required field is not in the contract | Contract change, not a workflow change |
| A new capture kind is needed | Requires the capture trait, standards position, and hardware |
| A pattern looks wrong | It comes from the BVR contract; fix it there |
| Consent wording is unclear | Legal, not engineering |
| An optional modality is needed in a county | Deployment decision plus the consent and retention review |

The last one deserves emphasis. `Modality::is_behavioural()` enumerates
`voiceprint`, `gait`, `signature`, `keystroke_dynamics`, and `mouse_dynamics`
precisely so that a consent and retention review can list the behavioural set
from one place. That review is a separate piece of work from making the capture
capability exist, and both are needed before any of those modalities runs in a
county.
