# abis-reports-adhoc

Ad-hoc reports authoring, preview, and publish engine for ABIS. Allows operators to generate custom reports on demand with filters (date range, decision outcome, venue) and preview results before publication.

## What it does

- Report authoring: Accepts date range, filters (adjudicator, venue, score range), and format preference (DOCX/Markdown)
- Preview rendering: Generates report sample (first 10 cases) and returns markdown preview without HSM signing or tlog anchoring
- Full generation: On approval, renders complete report, calls HSM signer, anchors to tlog, uploads to MinIO
- Filter validation: Enforces date range constraints; rejects requests older than 90 days (retention policy)
- Operator UI: Stores request metadata (author, timestamp, filter params) for audit trail
- Event emission: Publishes `abis.report.adhoc.generated.v1` to Kafka on successful publication

## Public API surface

```rust
pub struct AdhocReportRequest {
    pub start_date: Date,
    pub end_date: Date,
    pub filters: ReportFilters,
    pub format: ReportFormat,
}

pub struct ReportFilters {
    pub adjudicator_id: Option<String>,
    pub venue_id: Option<String>,
    pub score_range: Option<(f32, f32)>,
    pub decision: Option<CaseDecision>,
}

pub struct PreviewResponse {
    pub case_count: usize,
    pub sample_markdown: String,
}

pub struct PublishResponse {
    pub report_id: String,
    pub minio_url: String,
}

pub async fn preview_report(req: AdhocReportRequest) -> Result<PreviewResponse>;
pub async fn publish_report(req: AdhocReportRequest) -> Result<PublishResponse>;
pub async fn list_adhoc_reports(limit: usize) -> Result<Vec<GeneratedReport>>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (filter validation, date range checking, retention policy)
cargo test --lib --package eel-abis-reports-adhoc

# Integration tests (preview rendering, report publication, PG audit trail)
cargo test --test '*' --package eel-abis-reports-adhoc -- --test-threads=1
```

## Bench commands

```bash
# Preview render (first 10 cases from 1000 match set)
cargo bench --package eel-abis-reports-adhoc -- preview_render

# Full report generation (500 cases with HSM sign + tlog anchor)
cargo bench --package eel-abis-reports-adhoc -- publish_report
```

## Dependencies

- `sqlx` - Query matching cases by filter, audit trail writes
- `eel-abis-render-docx`, `eel-abis-render-markdown` - Renderer crates
- `eel-abis-reports` - Shared HSM signer and tlog anchor functions
- `tokio` - Async report generation and MinIO upload
- `serde` - Filter serialization for audit trail storage

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - On-demand report workflow diagram
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Adhoc report retention policy, filter performance tuning
