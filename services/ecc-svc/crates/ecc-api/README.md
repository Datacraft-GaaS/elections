# ecc-api

REST and gRPC API handlers for Election Command Centre. Routes alerts, grievances, IAB escalations, and dispatch commands.

Sits on top of alert and grievance aggregates. Fits into ecc-svc as the HTTP/gRPC boundary.

## What it does

Exposes REST endpoints (OpenAPI at packages/openapi-contracts/ecc/) and gRPC services for alerts, grievance submission and tracking, IAB escalation workflow, and dispatch commands. Handles alert streaming (server-sent events and gRPC), grievance NLP triage integration, rate limiting (per-channel), and request tracing.

## Public API

Exposed modules via lib.rs:
- alerts: GET /v1/ecc/alerts, StreamAlerts() -> Stream(Alert)
- grievances: POST /v1/ecc/grievances, GET /v1/ecc/grievances/{id}
- iab: GET /v1/ecc/iab/escalations, POST /v1/ecc/iab/escalations/{id}/override
- dispatch: POST /v1/ecc/dispatch/{personnel_id}/command
- rpc: gRPC service definitions

## Feature flags

None currently.

## Testing

API contract tests:
```bash
cargo test --lib api::
```

Alert streaming correctness:
```bash
cargo test alert_stream::
```

Grievance submission and triage:
```bash
cargo test grievance::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Alert streaming throughput (100 concurrent consumers):
```bash
cargo bench stream_alerts_100_consumers
```

## Dependencies of note

Workspace:
- eel-api-server-rs: REST/gRPC scaffolding
- eel-observability-rs: Request tracing

External:
- axum: HTTP router
- tokio: Async runtime
- prost: gRPC serialization
- tokio-tungstenite: WebSocket support (for streaming)

## Related docs

See packages/openapi-contracts/ecc/openapi.yaml for API spec.
See docs/runbooks/alert-delivery-latency.md for performance tuning.
