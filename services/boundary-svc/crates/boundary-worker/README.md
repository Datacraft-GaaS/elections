# boundary-worker

Background worker for NLC/KNBS shapefile import, boundary sync, and transparency log publishing.

Runs scheduled shapefile import from NLC (National Land Commission) and KNBS data sources. Parses GeoJSON/Shapefile, detects boundaries changed, emits BoundaryChange events. Publishes boundary snapshots to transparency-svc transparency log. Handles retry and backoff on transient failures.

Fits into boundary-svc as the data pipeline: fetches upstream data, transforms it, emits events, coordinates with transparency log.

## Public API

Exposed modules via lib.rs:
- import_worker: ImportWorker, run_scheduled_import() -> Result
- shapefile_parser: parse_shapefile(bytes) -> Boundaries
- nlc_client: NLCClient, fetch_boundaries(layer) -> Stream
- tlog_publisher: publish_to_transparency_log(snapshot) -> ProofHash

## Feature flags

None currently.

## Testing

Unit tests (parsing, scheduling):
```bash
cargo test --lib
```

Shapefile parsing tests (with fixtures):
```bash
cargo test shapefile_parser:: --features test-fixtures
```

Integration tests with mocked NLC:
```bash
cargo nextest run
```

## Benchmarks

Shapefile parsing latency (if benches/ exists):
```bash
cargo bench shapefile_parse_10k_features
```

## Dependencies of note

Workspace:
- boundary-domain: BoundaryChange emission
- eel-events-rs: Event publication
- eel-observability-rs: Import progress tracing

External:
- geozero: Shapefile/GeoJSON parsing
- reqwest: NLC HTTP API client
- tokio-util: Retry/backoff handling

## Related docs

See services/boundary-svc/docs/nlc-import.md for NLC data format spec.
See services/boundary-svc/docs/shapefile-parser.md for parsing strategy.
See docs/runbooks/boundary-import-failure.md for troubleshooting.
