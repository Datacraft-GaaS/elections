# bvr-batch

Batch ingestion pipeline for CRS/NRB/KRA delta updates (1k records/sec throughput).

Fetches voter data deltas from CRS (Central Registration System), NRB (National Registration Bureau), and KRA (Kenya Revenue Authority) via scheduled jobs. Parses MRZ (Machine Readable Zone) from ID images. Deduplicates voters against existing register via biometric ABIS federation. Emits bvr.voter.enrolled and bvr.voter.updated events. Handles replay-prevention (batch_id + timestamp dedup) and backfill (gap recovery from CRS via offset queries).

Fits into bvr-svc as the data ingestion layer: fetches upstream data, transforms and deduplicates, emits events for domain to process.

## Public API

Exposed modules via lib.rs:
- ingestion_pipeline: IngestionPipeline, run_crs_delta_ingest(offset) -> IngestionResult
- mrz_parser: MrzParser, parse_id_image(image_bytes) -> MrzData
- dedup: DeduplicateVoter, find_duplicate(voter) -> DeduplicationResult
- batch_tracking: BatchManifest, track_batch(batch_id, record_count, hash)

## Feature flags

None currently.

## Testing

Unit tests (MRZ parsing, dedup logic):
```bash
cargo test --lib
```

MRZ parsing with fixtures:
```bash
cargo test mrz_parser:: --features test-fixtures
```

Dedup consistency tests:
```bash
cargo test dedup::
```

Integration tests (with mock CRS):
```bash
cargo nextest run
```

## Benchmarks

Ingest throughput (1k records/sec):
```bash
cargo bench ingest_1k_per_sec
```

MRZ parsing latency (if benches/ exists):
```bash
cargo bench parse_mrz_image
```

## Dependencies of note

Workspace:
- bvr-domain: Voter events
- eel-events-rs: Event emission

External:
- reqwest: CRS/NRB/KRA HTTP client
- image, tesseract: MRZ extraction
- tokio: Async batching

## Related docs

See services/bvr-svc/docs/crs-api-spec.md for CRS data format.
See services/bvr-svc/docs/mrz-parsing.md for OCR/MRZ strategy.
See services/bvr-svc/docs/batch-ingest-protocol.md for replay prevention.
See docs/runbooks/bvr-ingest-lag.md for troubleshooting.
