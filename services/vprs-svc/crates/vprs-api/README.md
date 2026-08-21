# vprs-api

REST API for voter registration application submission, status tracking, and IPRS identity verification.

Handles voter application submission with national ID validation via IPRS integration. Implements application state machine (submitted, iprs-verified, approved, rejected, updated). Provides applicant-facing status tracking and admin-facing approval workflow. Integrates with bvr-svc for approved voter enrollment.

Fits into vprs-svc as the public API layer: citizens submit applications, vprs-api validates and routes to domain logic, and approved applications are published to bvr-svc.

## Public API

Exposed modules via lib.rs:
- applications: ApplicationApi, submit_application(applicant_data) -> ApplicationId
- status: StatusApi, get_application_status(application_id) -> ApplicationStatus
- iprs: IPRSIntegration, verify_national_id(national_id, names) -> VerificationResult
- updates: UpdateApi, submit_update(application_id, updates) -> UpdateId

## Feature flags

iprs-integration: Enable IPRS verification (default: true).
cache: Enable verification cache (default: true).

## Testing

Unit tests:
```bash
cargo test --lib
```

Application submission tests:
```bash
cargo test applications::tests
```

IPRS integration tests (with caching):
```bash
cargo test iprs::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires IPRS mock or staging IPRS):
```bash
cargo test --test '*integration*' -- --ignored
```

Contract tests (OpenAPI compliance):
```bash
cargo test --test '*contract*'
```

## Benchmarks

Application submission latency:
```bash
cargo bench --bench submission_latency
```

IPRS verification latency (with and without cache):
```bash
cargo bench --bench iprs_latency
```

## Dependencies of note

Workspace:
- eel-auth-rs: Applicant authentication
- eel-observability-rs: Tracing and metrics
- vprs-domain: Core aggregates

External:
- axum: Async HTTP server
- tokio: Async runtime
- sqlx: PostgreSQL persistence
- reqwest: IPRS gRPC client

## Related docs

See services/vprs-svc/docs/iprs-integration.md for verification flow.
See services/vprs-svc/docs/application-state-machine.md for state transitions.
See docs/runbooks/vprs-api-degradation.md for operational runbook.
