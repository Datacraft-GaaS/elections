# ai-gateway

Rust API gateway for ONNX inference routing, model attestation validation, and per-consumer rate limiting.

Routes inference requests to Python ONNX backends with automatic failover and circuit breaking. Enforces model signature validation (all production models must be registered and Ed25519-signed). Applies per-consumer rate limits via token bucket algorithm. Audit-logs every inference decision for compliance and forensics.

Fits into ai-platform-svc as the critical path: requests land on the gateway, model attestation is checked, load is distributed across Python workers, and telemetry is captured for observability.

## Public API

Exposed modules via lib.rs:
- gateway: Server trait, start(config) -> Service
- routing: InferenceRouter, route_request(consumer_id, model_id) -> WorkerPool
- attestation: ModelValidator, validate_signature(model_id, signature) -> Result
- rate_limiting: RateLimiter, check_quota(consumer_id) -> QuotaDecision
- audit: AuditLogger, log_inference_decision(consumer, model, latency, status)

## Feature flags

None currently defined. Inference routing and validation always active.

## Testing

Unit tests:
```bash
cargo test --lib
```

Model validation tests (signature verification):
```bash
cargo test attestation::tests
```

Rate limiting tests (token bucket correctness):
```bash
cargo test rate_limiting::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires Python ONNX server running):
```bash
cargo test --test '*integration*' -- --ignored
```

## Benchmarks

Inference routing latency:
```bash
cargo bench --bench gateway_latency
```

Tracks request routing time, model validation time, and end-to-end inference latency.

## Dependencies of note

Workspace:
- eel-auth-rs: Consumer authentication and authorization
- eel-observability-rs: Tracing and metrics
- eel-resilience-rs: Circuit breaker and retry logic

External:
- axum: Async HTTP server framework
- tokio: Async runtime
- sqlx: Audit log persistence to PostgreSQL
- ed25519-dalek: Signature validation

## Related docs

See services/ai-platform-svc/docs/gateway-slo.md for latency SLOs.
See services/ai-platform-svc/docs/attestation.md for model signature validation procedures.
See docs/runbooks/ai-gateway-degradation.md for operational runbook.
