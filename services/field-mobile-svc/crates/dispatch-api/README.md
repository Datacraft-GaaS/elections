# dispatch-api

Personnel dispatch and attendance API supporting 250k concurrent check-ins in 90-min election window.

Handles massive concurrent check-in load via partitioned event log, GPS-based tracking, and real-time assignment updates. Tracks personnel location, check-in/check-out events, and assignment state. Integrates with MDM enrollment for role-based access control and geo-fencing. Provides high-throughput ingestion and low-latency query for field operations dashboard.

Fits into field-mobile-svc as the critical path for M33 personnel dispatch: field staff check in via mobile app, dispatch API logs events, and ops dashboard monitors station coverage and personnel availability in real-time.

## Public API

Exposed modules via lib.rs:
- dispatch: DispatchService trait, check_in(personnel_id, gps_coords, timestamp) -> AttendanceId
- assignments: AssignmentStore, list_assignments(personnel_id) -> AssignmentList
- tracking: LocationTracker, update_location(personnel_id, lat, lon, accuracy)
- query: DispatchQuery, get_station_coverage(station_id) -> CoverageReport

## Feature flags

None currently defined. Full concurrent dispatch always enabled.

## Testing

Unit tests:
```bash
cargo test --lib
```

Concurrent check-in tests:
```bash
cargo test dispatch::concurrent_tests
```

Event log partitioning tests:
```bash
cargo test event_log::tests
```

Run all with:
```bash
cargo nextest run
```

Integration tests (requires PostgreSQL with partitioned table setup):
```bash
cargo test --test '*integration*' -- --ignored
```

Load tests (250k check-ins in 90 min window):
```bash
cargo test --test load_check_in -- --ignored --test-threads=1
```

## Benchmarks

Check-in throughput and latency:
```bash
cargo bench --bench dispatch_throughput
```

Tracks p50/p95/p99 check-in latency under load.

## Dependencies of note

Workspace:
- eel-auth-rs: MDM enrollment verification
- eel-observability-rs: Tracing and metrics
- eel-resilience-rs: Bulkhead isolation per station

External:
- axum: Async HTTP server
- tokio: Async runtime
- sqlx: PostgreSQL partitioned table access
- dashmap: Concurrent location cache

## Related docs

See services/field-mobile-svc/docs/dispatch-slo.md for throughput SLOs.
See services/field-mobile-svc/docs/partition-strategy.md for event log partitioning.
See docs/runbooks/dispatch-degradation.md for operational runbook.
