# rts-api

Public REST + gRPC API handlers for result submission, tally queries, and declaration retrieval.

Exposes form submission endpoints (from kits and RO workstations), tally summaries (running national/constituency counts), declaration and certificate queries. Implements OpenAPI schema at packages/openapi-contracts/rts/. Delegates to rts-domain for validation and rts-tally for computation. Handles load-shedding (graceful rejection at 60k concurrent requests) and rate-limiting per station.

Fits into rts-svc as the public HTTP/gRPC transport layer: receives forms, queries results, returns formatted responses.

## Public API

Exposed modules via lib.rs:
- handlers: FormHandler, submit_form34(form_bytes, sig), get_tally(constituency_id)
- errors: FormError, TallyError for HTTP marshaling
- openapi: OpenAPI spec generation
- loadshedding: LoadShedder, check_capacity() with graceful rejection

## Feature flags

None currently.

## Testing

Unit tests (handler logic):
```bash
cargo test --lib
```

OpenAPI contract tests:
```bash
cargo test contract
```

Load shedding tests (simulated surge):
```bash
cargo test loadshedding::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Form submit handler latency (if benches/ exists):
```bash
cargo bench form_submit_handler
```

## Dependencies of note

Workspace:
- rts-domain: Form validation
- rts-tally: Tally computation
- eel-api-server-rs: HTTP/gRPC framework
- eel-loadshedding-rs: Capacity management
- eel-observability-rs: Request tracing

External:
- axum: HTTP framework
- tonic: gRPC framework

## Related docs

See services/rts-svc/docs/api-spec.md for OpenAPI + gRPC spec.
See services/rts-svc/docs/form34-validation.md for submission requirements.
See services/rts-svc/docs/load-shedding-strategy.md for capacity management.
