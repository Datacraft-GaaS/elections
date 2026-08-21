# gazette

Election calendar and official gazette service for M52 module. Manages CalendarEvent (election dates, deadlines, phases) and GazetteNotice (statutory rules, legal notices) from Government Printer.

Ingests notices via Government Printer API integration, indexes them searchable, publishes to Kafka for downstream notification. Fits into portal-svc as the gazette publication layer.

## What it does

Maintains authoritative election calendar and gazette notices. Handles calendar event lifecycle (announced, published, completed). Indexes notices by date, category, and statutory rule ID. Provides full-text search. Emits gazette_notice.published and calendar_event.updated events.

## Public API

Exposed modules via lib.rs:
- calendar: CalendarEvent, schedule_event(), get_calendar() -> Events
- gazette: GazetteNotice, ingest_notices(batch) -> Notices
- search: search_notices(query) -> Stream(Notice)
- government_printer: fetch_latest_notices() -> Stream

## Feature flags

None currently.

## Testing

Unit tests (calendar, gazette indexing):
```bash
cargo test --lib
```

Integration tests (Government Printer mock):
```bash
cargo nextest run
```

Full-text search correctness:
```bash
cargo test search::
```

## Benchmarks

Gazette indexing latency:
```bash
cargo bench index_1000_notices
```

## Dependencies of note

Workspace:
- eel-events-rs: Notice publication
- eel-observability-rs: Indexing metrics

External:
- reqwest: Government Printer client
- tantivy: Full-text search engine
- tokio-util: Async retry logic

## Related docs

See services/portal-svc/docs/gazette-ingestion.md for Government Printer API format.
See docs/runbooks/gazette-import-failure.md for troubleshooting.
