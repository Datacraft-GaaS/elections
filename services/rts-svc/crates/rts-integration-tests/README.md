# rts-integration-tests

Integration test harness for RTS with testcontainers (PostgreSQL, Kafka, Redis).

Provides fixtures: RtsTestEnv (stands up DB, Kafka, service), realistic form election dataset (46k stations, Nairobi + coast + highlands distribution), chaos injection helpers. Enables integration test suite (tests/) to run against real services without mocks.

Fits into rts-svc as the test infrastructure: enables high-confidence integration coverage.

## Public API

Exposed modules via lib.rs:
- test_env: RtsTestEnv, create() -> TestEnv, with_database(), with_kafka()
- fixtures: ElectionDataset, generate_46k_stations(), sample_form34(station_id)
- chaos: ChaosInject, kill_primary(), partition_kafka(), delay_hsm(duration)
- assertions: FormAssert, verify_form_store(form_id, expected_counts)

## Feature flags

test-fixtures: Additional test data generators (large datasets).

## Testing

Run integration tests:
```bash
cargo test --test '*' --all-features
```

Specific integration suite:
```bash
cargo test --test integration_rts_domain
```

## Benchmarks

Fixture generation time (if benches/ exists):
```bash
cargo bench fixture_generation
```

## Dependencies of note

Workspace:
- rts-domain: Domain aggregates
- eel-eventstore-rs: Event store fixtures

External:
- testcontainers: PostgreSQL, Kafka containers
- tokio: Async test runtime
- sqlx: Database access
- rdkafka: Kafka client

## Related docs

See services/rts-svc/tests/ for test suite organization.
See docs/testing/integration-test-patterns.md for patterns.
