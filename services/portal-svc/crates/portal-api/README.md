# portal-api

REST and gRPC API handlers for public-facing portal endpoints. Routes form publication, Merkle inclusion proofs, gazette queries, §6A inspection endpoints, and referendum explainer delivery.

Sits on top of gazette, inspection, and form aggregates. Fits into portal-svc as the HTTP/gRPC boundary.

## What it does

Exposes REST endpoints (OpenAPI at packages/openapi-contracts/portal/) and gRPC services for results, proofs, gazette, inspection. Handles rate limiting (per-IP, per-session), caching headers (cache-control), and CORS for public CDN. Validates input, serializes responses, manages request tracing.

## Public API

Exposed modules via lib.rs:
- forms: GET /v1/portal/results/*, GET /v1/portal/proof/*
- gazette: GET /v1/portal/gazette/*
- inspection: GET /v1/portal/inspection/*, POST /v1/portal/inspection/objections
- referendum: GET /v1/portal/referendum/*/explainer
- rpc: gRPC service definitions

## Feature flags

None currently.

## Testing

API contract tests:
```bash
cargo test --lib api::
```

Rate limiting:
```bash
cargo test rate_limit::
```

Caching behavior:
```bash
cargo test cache::
```

Integration tests (with openapi-contracts):
```bash
cargo nextest run
```

## Benchmarks

Endpoint throughput (proof serving):
```bash
cargo bench serve_proof_10k_reqs
```

## Dependencies of note

Workspace:
- gazette, inspection: Aggregate queries
- eel-api-server-rs: REST/gRPC scaffolding
- eel-observability-rs: Request tracing

External:
- axum: HTTP router
- tokio: Async runtime
- prost: gRPC serialization

## Related docs

See packages/openapi-contracts/portal/openapi.yaml for API spec.
See docs/runbooks/proof-serving-latency.md for performance tuning.
