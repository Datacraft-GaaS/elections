# crms-api

REST and gRPC API for candidate nominations, party registration, ballot proofs, and party symbol management.

Exposes REST endpoints (OpenAPI) and gRPC services for nomination submission, clearance status queries, ballot PDF generation, party registration, and symbol management. Integrates with crms-domain for state machine validation and event emission. Routes clearance queries to Python aggregator service with caching. Provides candidate and party portal interfaces.

Fits into crms-svc as the public API layer: candidates submit nominations, parties register, clearance status is queried, ballot proofs are generated, and symbols are submitted via crms-api.

## Public API

Exposed modules via lib.rs:
- nominations: NominationApi, submit_nomination(candidate_data) -> NominationId
- clearance: ClearanceApi, get_clearance_status(nomination_id) -> ClearanceStatus
- ballot: BallotApi, generate_proof(constituency_id, candidates) -> ProofPdf
- parties: PartyApi, register_party(party_data) -> PartyId
- symbols: SymbolApi, submit_symbol(party_id, symbol_data) -> SymbolId

## Feature flags

grpc: Enable gRPC endpoints (default: true).
rest: Enable REST endpoints (default: true).
ballot-proof: Enable ballot PDF generation (default: true).

## Testing

Unit tests:
```bash
cargo test --lib
```

Nomination submission tests:
```bash
cargo test nominations::tests
```

Clearance status query tests:
```bash
cargo test clearance::tests
```

Ballot proof generation tests:
```bash
cargo test ballot::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires PostgreSQL, Python PDF service):
```bash
cargo test --test '*integration*' -- --ignored
```

Contract tests (OpenAPI + gRPC compliance):
```bash
cargo test --test '*contract*'
```

## Benchmarks

Nomination submission latency:
```bash
cargo bench --bench nomination_submission
```

Clearance status query latency (with caching):
```bash
cargo bench --bench clearance_query
```

Ballot PDF generation throughput:
```bash
cargo bench --bench ballot_generation
```

## Dependencies of note

Workspace:
- eel-auth-rs: Candidate and party authentication
- eel-observability-rs: Tracing and metrics
- crms-domain: Core aggregates

External:
- axum: Async HTTP server
- tonic: gRPC server framework
- tokio: Async runtime
- sqlx: PostgreSQL persistence
- reqwest: Python clearance service client

## Related docs

See services/crms-svc/docs/clearance-workflow.md for decision flow.
See services/crms-svc/docs/ballot-pdf-generation.md for proof generation.
See docs/runbooks/crms-api-degradation.md for operational runbook.
