# intent

Intent preservation crate for M27. Validates pre-marked ballots, records spoilt ballot events, and manages the adjudication workflow for ambiguous marks, enforcing Elections Act §§53-55.

## What it does

On polling day, a voter may mark a ballot in a way that requires clerk judgment (multiple crosses, ambiguous tick, stray mark). M27 provides on-kit (Flutter plugin via FFI) and server-side validation. The kit plugin captures a scan of the ballot, classifies the mark pattern via the on-device ONNX model (from ai-platform-svc), and either accepts or flags for adjudication. The server side records the full event chain, ensures every spoilt ballot is counted (not destroyed), and enforces the dual-control adjudication workflow (two clerks must agree on a rejected ballot).

## Public API surface

- `mod pre_mark` - `PreMarkValidationEvent`, `validate_pre_mark()`, mark pattern enum
- `mod spoilt` - `SpoiltBallotEvent`, `record_spoilt()`, `list_spoilt_by_station()`
- `mod adjudication` - `AdjudicationEvent`, `AdjudicationDecision`, `submit_adjudication()`, `resolve_adjudication()`
- `mod kit_plugin` - FFI interface for Flutter on-kit integration (Dart FFI bindings generated separately)

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `on-kit-ffi` | off | Compile as cdylib for Flutter FFI |
| `onnx-inference` | off | Enable on-device ONNX mark classification |

## Test commands

```bash
cargo nextest run -p intent
cargo nextest run -p intent --test integration -- --include-tag intent
# FFI build
cargo build --lib --features on-kit-ffi
```

## Dependencies of note

- `eel-events-rs` (workspace): event envelope and outbox
- `eel-auth-rs` (workspace): dual-control permission checks
- `ai-platform-svc` crate plugin (runtime): ONNX model inference

## Related docs

- `docs/plans/modules/M27-intent-preservation.md`
- Elections Act §§53-55 spoilt ballot provisions
- `packages/event-schemas/support/intent.*.json`
