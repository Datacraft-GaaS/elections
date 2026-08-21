# abis-api

REST and gRPC API handlers for the ABIS. Handles inbound match requests, idempotency via request key hashing, outbox event emission, and adjudication case retrieval.

## What it does

- HTTP API (Axum, port 8080): Accepts POST `/api/v1/abis/match/1n` and `1:1` requests with biometric templates
- gRPC API (port 8443): Match and adjudication services for internal kit-facing calls
- Idempotency: Routes requests by `X-Eel-Idempotency-Key` header; returns cached response if duplicate received
- Outbox relay: Writes events to `eel_outbox` table; separate relay process publishes to Kafka
- Health checks: Liveness (always 200), readiness (503 until attestation quote obtained)
- Request logging: Structured logs with template ID, score, latency, vendor error details

## Public API surface

```rust
pub struct MatchRequest {
    pub probe_template: Vec<u8>,
    pub gallery_shard: Option<u8>,
    pub vendor: String,
    pub metadata: Map<String, Value>,
}

pub struct MatchResponse {
    pub job_id: String,
    pub status: String,
    pub score: Option<f32>,
    pub decision: Option<String>,
}

pub async fn health_check(State(db): State<PgPool>) -> Result<Json<Health>>;
pub async fn submit_match(
    State(db): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<MatchRequest>,
) -> Result<(StatusCode, Json<MatchResponse>)>;
pub async fn get_adjudication_cases(
    State(db): State<PgPool>,
    Query(filter): Query<CaseFilter>,
) -> Result<Json<Vec<AdjudicationCase>>>;
pub async fn decide_case(
    State(db): State<PgPool>,
    Path(case_id): Path<String>,
    Json(decision): Json<CaseDecision>,
) -> Result<StatusCode>;
```

## Feature flags

None. Core API surface always included.

## Test commands

```bash
# Unit tests (request builder, response codec, idempotency key hashing)
cargo test --lib --package eel-abis-api

# Integration tests (HTTP server, PG outbox, gRPC stubs)
cargo test --test '*' --package eel-abis-api -- --test-threads=1

# Contract tests (OpenAPI spec compliance, error codes)
cargo test --test contract --package eel-abis-api -- --nocapture
```

## Bench commands

```bash
# Match request throughput (baseline no-op handler)
cargo bench --package eel-abis-api -- submit_match

# Adjudication case list latency (100 cases, pagination)
cargo bench --package eel-abis-api -- list_cases
```

## Dependencies

- `axum` - HTTP server framework, routing, middleware
- `tower` - Tower middleware stack (logging, timeout, tracing)
- `sqlx` - PostgreSQL outbox write and query execution
- `serde_json` - Request/response JSON codec
- `uuid` - Job ID generation (UUID7)
- `utoipa` - OpenAPI spec generation and validation

## Related docs

- [API-CATALOG.md](../../docs/API-CATALOG.md) - Full OpenAPI spec, error codes, authentication
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Request flow diagram, outbox relay timing
- [EDGE-CASES.md](../../docs/EDGE-CASES.md) - Idempotency key collision handling, duplicate submit detection
