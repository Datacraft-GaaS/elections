# rts-publisher

Transparency log publisher and Kafka outbox relay for RTS results.

Consumes rts.form34.published and rts.tally.updated events. Publishes form snapshots and tally to transparency-svc via gRPC (append to log, retrieve inclusion proof). Relays events to Kafka for downstream consumers (audit systems, observer apps). Handles transient failures (retry, exponential backoff). Coordinates proof publication with rts-projections.

Fits into rts-svc as the outbound event pipeline: takes RTS events, publishes to transparency log and Kafka, coordinates with downstream consumers.

## Public API

Exposed modules via lib.rs:
- publisher: Publisher, publish_to_tlog(snapshot) -> ProofHash
- kafka_relay: KafkaRelay, emit_event(rts_event) -> PublishResult
- retry: RetryPolicy, with_exponential_backoff(max_retries, initial_delay)
- proof_handler: ProofHandler, handle_tlog_proof(proof) -> ProofAck

## Feature flags

None currently.

## Testing

Unit tests (retry logic, event mapping):
```bash
cargo test --lib
```

Kafka relay tests (with testcontainers):
```bash
cargo test kafka_relay::
```

Transparency log publisher mocking:
```bash
cargo test publisher:: --features test-mocks
```

Integration tests (real Kafka + transparency-svc mock):
```bash
cargo nextest run
```

## Benchmarks

Event publishing latency (if benches/ exists):
```bash
cargo bench publish_1k_events
```

## Dependencies of note

Workspace:
- rts-domain: Domain events
- eel-eventstore-rs: Event subscription
- eel-events-rs: Event schema
- eel-observability-rs: Publishing tracing

External:
- rdkafka: Kafka producer
- tonic: gRPC client for transparency-svc
- tokio: Async runtime
- backoff: Retry strategy

## Related docs

See services/rts-svc/docs/kafka-topics.md for event topics and schemas.
See services/transparency-svc/docs/tlog-api.md for transparency log gRPC.
See docs/runbooks/rts-publisher-lag.md for troubleshooting.
