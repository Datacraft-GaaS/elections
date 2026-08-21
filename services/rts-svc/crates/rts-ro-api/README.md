# rts-ro-api

Module M09 RO (Returning Officer) workstation API for ICR (Intelligent Character Recognition) and dispute handling.

Provides API for RO desktop app (Tauri, apps/rts-ro/). Handles ICR result extraction (Python ML service calls for Form 34 handwriting recognition). Records disputes (voter rejected result, forms unclear, suspected fraud). Tracks dispute lifecycle (opened, adjudicated, closed). Provides dispute case search and status queries for RO review.

Fits into rts-svc as the RO workstation gateway: RO desktop calls ro-api, ro-api coordinates with ML and domain, updates form status and dispute records.

## Public API

Exposed modules via lib.rs:
- icr_handler: ICRHandler, extract_form_icr(form34_image) -> ICRResult
- disputes: DisputeCase, raise_dispute(form_id, reason, evidence), list_open_cases()
- adjudication: Adjudication, resolve_dispute(case_id, resolution) -> DisputeResolution
- workstation: WorkstationContext, set_ro_credentials(ro_id), get_ro_profile() -> RoProfile

## Feature flags

None currently.

## Testing

Unit tests (dispute case logic):
```bash
cargo test --lib
```

ICR result mapping tests:
```bash
cargo test icr_handler::
```

Dispute case state machine tests:
```bash
cargo test disputes::lifecycle
```

Integration tests (with Python ML service mock):
```bash
cargo nextest run --features test-ml-mock
```

## Benchmarks

ICR extraction latency (if benches/ exists):
```bash
cargo bench icr_extract_form34
```

## Dependencies of note

Workspace:
- rts-domain: Form updates
- eel-observability-rs: Dispute case tracing

External:
- reqwest: Python ML service client
- tokio: Async runtime
- serde_json: ICR result parsing

## Related docs

See services/rts-svc/docs/m09-ro-protocol.md for workstation API spec.
See services/rts-svc/py/icr/ for ML extraction service.
See services/rts-svc/docs/dispute-lifecycle.md for case workflows.
See apps/rts-ro/README.md for desktop app details.
