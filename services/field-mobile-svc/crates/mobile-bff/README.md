# mobile-bff

Backend-for-frontend aggregating voter registration, results, and inclusion proof APIs for mobile clients.

Exposes GraphQL and REST endpoints that aggregate bvr-svc (registration), bvi-svc (inclusion proofs), rts-svc (results), and transparency-svc (STH verification). Implements offline sync protocol for caching voter register snapshots and proof chains on device. Handles real-time push notification routing via Firebase Cloud Messaging. Enforces per-device rate limiting and offline-first fallback logic.

Fits into field-mobile-svc as the aggregation layer for all mobile apps: citizen-verify, party-agent, poll-clerk, and observer apps all use mobile-bff to fetch voter data, proofs, and results with seamless offline fallback.

## Public API

Exposed modules via lib.rs:
- graphql: GraphQLServer, schema with query and subscription resolvers
- rest: RestApi, routes for /v1/verify, /v1/results, /v1/proofs
- offline_sync: OfflineSync, sync_protocol_v1(device_id, last_sync_ts) -> SyncBundle
- push: PushNotificationRouter, route_alert(topic, alert) -> FirebaseResult
- rate_limiting: DeviceRateLimiter, check_quota(device_id) -> QuotaDecision

## Feature flags

offline-sync: Enable offline sync protocol (default: true).
graphql: Enable GraphQL endpoint (default: true).
push-notifications: Enable Firebase Cloud Messaging integration (default: true).

## Testing

Unit tests:
```bash
cargo test --lib
```

Offline sync protocol tests:
```bash
cargo test offline_sync::tests
```

GraphQL query tests:
```bash
cargo test graphql::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires upstream mocks):
```bash
cargo test --test '*integration*' -- --ignored
```

Contract tests (GraphQL schema compliance):
```bash
cargo test --test '*contract*'
```

## Benchmarks

Aggregation latency (query multiple upstreams):
```bash
cargo bench --bench aggregation_latency
```

Offline sync throughput (bundle serialization):
```bash
cargo bench --bench sync_throughput
```

## Dependencies of note

Workspace:
- eel-auth-rs: Device authentication and attestation
- eel-observability-rs: Tracing and metrics
- eel-resilience-rs: Circuit breaker for upstream services

External:
- async-graphql: GraphQL framework
- axum: Async HTTP server
- tokio: Async runtime
- reqwest: HTTP client for upstream aggregation
- firebase-cloud-messaging: Push notification delivery

## Related docs

See services/field-mobile-svc/docs/offline-sync.md for sync protocol specification.
See services/field-mobile-svc/docs/bff-aggregation.md for upstream integration.
See docs/runbooks/mobile-bff-degradation.md for operational runbook.
