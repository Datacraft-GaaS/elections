# ADR-0027: Desktop/Laptop BVR Registration Client and Declarative Enrolment Workflows

**Date**: 2026-09-28
**Status**: Accepted (core landed; GUI shell outstanding)

## Context

BVR (M01, Biometric Voter Registration) is currently an Android tablet client
(`services/biometric-edge-svc/apps/bvr-kit`) talking to the `bvr` contract at
`packages/openapi-contracts/bvr/openapi.yaml`. Registration also has to run on
ordinary desktop and laptop hardware, not only provisioned tablets.

Two requirements drive this ADR.

1. **Configurable data items.** The set of fields collected and validated must be
   changeable by configuration. A field added or dropped must not require a code
   change and a client release, because registration campaigns are defined
   per-election and vary between counties and voter categories.
2. **A native desktop GUI** on both Linux and Windows, because operators work on
   general-purpose machines that are not MDM-managed and cannot be assumed to run
   a browser.

Design input was requested from **NIST MBARK** (Multimodal Biometric Application
Resource Kit, Information Access Division). The transferable ideas were:

- **Separate the acquisition description from the application.** MBARK's real
  contribution was a middleware boundary with a unified sensor API, not a kiosk
  app. New sensors plug in without restarting or re-platforming.
- **Describe the workflow declaratively.** MBARK adopted BiCCL (NIST IR 7531), a
  small DSL that *formally describes* biometric acquisition — sensors, tasks,
  workflow — and a reference compiler that lowers it to something executable.
  BiCCL is platform-independent and deliberately not MBARK-specific.
- **Ship a virtual sensor.** MBARK's later releases added virtual sensors so
  development and evaluation need no hardware.
- **Design for operator recovery.** MBARK gives operators means to recover from
  both minor mistakes and hardware failure.
- **Usability-tested capture UI.** Derived from NIST's WSABI work
  (NIST IR 8003), not invented ad hoc.

Rejected from MBARK: the .NET implementation, the XML pipeline, and the
sensor plug-in mechanism as such. BiCCL's idea is worth taking; its 2009
realisation is not.

## Decision

Ship a **Qt 6 desktop client** whose declarative core lives in Rust, with the
workflow described by a document compiled at load time.

### 1. Native GUI: Qt 6, via a C ABI bridge from Rust

Qt 6 was chosen over GTK4, WinUI 3, and Tauri.

- **Qt 6** is the only option that is genuinely native on both target platforms
  from one codebase. GTK4 is Linux-only; WinUI 3 is Windows-only; Tauri is a
  webview and is not native, which was the original requirement.
- The Rust core and the Qt UI communicate over a **C ABI bridge** generated with
  `cbindgen`, exposing only `extern "C"` functions over an opaque handle. This
  keeps Rust's ownership and error handling out of the C++ layer.
- Cost accepted: the heaviest build chain of the options. A C++ toolchain
  (CMake plus a Qt 6 development install) becomes a build prerequisite, and Qt
  is LGPL-3.0, which obliges dynamic linking and allows the recipient to
  relink against a modified Qt. That is acceptable for internal distribution but
  **must be reviewed before any public release**.

### 2. The form is generated from a document, not coded

`packages/eel-bvr-workflow-rs` implements a BiCCL-equivalent DSL:

```
WorkflowSpec --compile()--> CompiledWorkflow --validate_submission()--> SubmissionError
```

`compile` resolves step-to-data-item references, rejects uncollected data items,
pre-validates constraints, and reports every fault in one pass. Nothing consumes
an uncompiled spec. `validate_submission` applies the compiled rules to
collected values, independently of any UI, so the desktop client, the Android
`bvr-kit`, and replay of a captured record all validate identically.

This is the piece that makes requirement 1 true rather than aspirational.

### 3. Capture is modelled separately from biometric modality

A workflow's collectable items are not all the same kind of thing. Documents are
scanned, photographs are taken live, and biometrics are captured, and these are
independent choices. The DSL therefore models a **`CaptureKind`** rather than a
biometric modality:

`fingerprint`, `finger_vein`, `palm_vein`, `wrist_vein`, `face`,
`live_photograph`, `iris`, `document_mrz`.

Capture is kept distinct from matching modality for two reasons. A document scan
produces MRZ text, not a biometric template, so it must not be routed through
biometric handling; `CaptureKind::is_biometric` encodes that. And a matching
modality does not imply a capture path exists: palm vein already has a matcher
(`abis-modality-vein`, ISO/IEC 19794-9) but no capture method in the
`sensor-driver` trait, because NIR vascular capture is a different sensor class
from optical contact capture.

**"In any combination or individually"** is expressed with a
`CapturePolicy` over a list of kinds, not with a combinatorial expansion of
steps:

- `all_of` -- every listed kind
- `any_of` -- at least one
- `at_least: N` -- N or more

`at_least` matters operationally: a registration drive cannot always complete a
full acquisition on one citizen, and the platform already supports degraded
capture via `BiometricSensorDriver::capture_fingerprint_degraded`, so a workflow
must be able to accept fewer captures than the ideal.

Requirements are **named and referenced by id**, so one definition can be shared
across steps and a dangling reference is caught at compile time exactly like a
missing data item.

`face` and `live_photograph` are separate kinds so a workflow can demand a
liveness-proven capture (ISO/IEC 30107-3 PAD), not merely an image.

### 4. Capture reuses the existing driver abstraction; no vendor SDKs

Per ADR-0026, fingerprint capture goes through **libfprint via fprintd** and face
capture through **OpenCV**. Capture steps in a workflow name a modality only; the
client binds them to the existing `BiometricSensorDriver` trait
(`services/biometric-edge-svc/crates/sensor-driver/src/sensor_driver.rs`).

Consequences:

- The desktop client adds **no new vendor integration**. It reuses the trait,
  its `MockSensorDriver` double, and the existing OSS stacks.
- Vendor selection remains a build-time choice; CI needs no hardware.
- MBARK's virtual-sensor idea is already satisfied by `MockSensorDriver`, so no
  new simulator is required.

**Stated limit.** `BiometricSensorDriver` exposes only `capture_fingerprint`,
`capture_face`, and `capture_iris`. Palm, finger vein, wrist vein, live
photograph, and document MRZ have **no capture method**. Their matching and
parsing support exists (`abis-modality-vein`, `integrations/mrz`), but extending
the trait is a prerequisite before a client can serve those capture kinds. The
DSL models them so workflows can be authored and reviewed now; it does not imply
the hardware path is ready.

### 5. Validation stays in the contract, not in the client

Field patterns (`national_id`, `phone`, `polling_station_code`) are enforced by
`eel_spec_helpers_rs::matches_bvr_pattern`, mirroring the contract. The client
does not carry its own regexes for contract fields.

## Consequences

**Positive**

- Adding, removing, or reordering data items is a document edit.
- One declarative core serves desktop, Android, and replay, so validation cannot
  drift between form factors.
- No new vendor hardware integration and no new native biometrics dependency.
- The compiler rejects malformed operator documents before the client runs,
  rather than half-enrolling a citizen.

**Negative**

- Qt 6 adds a C++ toolchain and an LGPL-3.0 obligation, and LGPL terms need
  review before public distribution.
- A declarative DSL is a new surface to specify and version; documents must be
  treated as configuration and change-controlled.
- The C ABI bridge is a real seam: it can be miscalled, and it must be kept
  narrow and panic-free across the boundary.
- The DSL deliberately does not express arbitrary branching or loops. A workflow
  that would need them is rejected rather than half-expressed.
- Several modelled capture kinds have no capture method behind them yet (see
  above), so the model is ahead of the hardware integration.

**Neutral**

- The contract's own example for `polling_station_code`
  (`"NRB-001-PS-042"`) does not satisfy its own pattern `^[A-Z0-9]{6,12}$`. The
  pattern is enforced and the test asserts the disagreement explicitly. Someone
  should reconcile the example and the pattern in the contract.

## Alternatives Considered

1. **Hand-written widgets per data item.** Rejected: contradicts requirement 1;
   every field becomes a code change and a release.
2. **GTK4 / libadwaita.** Genuinely native and lighter, but Linux-only, so
   Windows needs a second codebase.
3. **WinUI 3 / windows-rs.** Native on Windows and better suited to MDM-managed
   estates, but Linux needs a second codebase.
4. **Tauri.** One codebase across both platforms and no compile-time C++
   toolchain, but it is a webview, not a native GUI, which was the requirement.
5. **Vendor fingerprint SDKs on desktop.** Rejected: contradicts ADR-0026 and
   reintroduces the lock-in that ADR exists to prevent.

## Status

Landed: the declarative core (`packages/eel-bvr-workflow-rs`, 53 tests) and the
contract pattern validator (`packages/eel-spec-helpers-rs`).

Outstanding: the Qt 6 shell, the C ABI bridge, binding capture steps to
`BiometricSensorDriver` in the desktop process, extending
`BiometricSensorDriver` with palm, finger vein, wrist vein, live photograph, and
document MRZ capture, packaging for Linux and Windows, and operator usability
testing of the capture UI per NIST IR 8003.

## References

- ADR-0026 (`docs/architecture/decisions/0026-hardware-latitude.md`)
- ADR-0011 (spec-first), ADR-0025 (OSS RFID stack)
- BVR contract: `packages/openapi-contracts/bvr/openapi.yaml`
- `packages/eel-bvr-workflow-rs/README.md`
- NIST MBARK: https://www.nist.gov/services-resources/software/multimodal-biometric-application-resource-kit-mbark
- BiCCL, NIST IR 7531
- NIST IR 8003, mobile touchscreen interface for multimodal biometric capture
- libfprint/fprintd: https://fprint.freedesktop.org/ (LGPL-2.1)
- Qt 6: https://www.qt.io/product/qt6 (LGPL-3.0)
