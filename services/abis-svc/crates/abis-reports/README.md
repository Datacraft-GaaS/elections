# abis-reports

Reports engine, renderers, HSM signer, and tlog anchoring for the ABIS service. Orchestrates generation of scheduled (daily/weekly) and adhoc adjudication decision reports with full audit trail and cryptographic proof.

## What it does

- Scheduled reports: Runs daily and weekly report jobs via Tokio scheduler; batches all adjudicated cases since last report
- Renderer orchestration: Calls `abis-render-docx` and `abis-render-markdown` in parallel for each case
- HSM signing: Calls HSM signer service (port 8444) to sign report hash; embeds signature in document footer
- Tlog anchoring: Submits report to transparency log (Trillian STH entry); stores anchor proof in `eel_reports` table
- Audit trail: Records report generation timestamp, signer identity, content hash, tlog anchor, and delivery status
- Report retrieval: Exposes GET `/api/v1/abis/reports/scheduled` endpoint for operator UI to fetch past reports

## Public API surface

```rust
pub struct ReportSpec {
    pub start_date: Date,
    pub end_date: Date,
    pub format: ReportFormat,
}

pub struct GeneratedReport {
    pub id: String,
    pub generated_at: DateTime,
    pub case_count: usize,
    pub docx_path: String,
    pub markdown_path: String,
    pub hsm_signature: String,
    pub tlog_anchor: TlogAnchor,
}

pub async fn generate_scheduled_report(spec: ReportSpec) -> Result<GeneratedReport>;
pub async fn generate_adhoc_report(spec: ReportSpec) -> Result<GeneratedReport>;
pub async fn sign_report_hash(content_hash: &[u8]) -> Result<HsmSignature>;
pub async fn anchor_tlog(report_hash: &[u8]) -> Result<TlogAnchor>;
pub async fn list_reports(start: Date, end: Date) -> Result<Vec<GeneratedReport>>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (report spec validation, date range logic, hash calculation)
cargo test --lib --package eel-abis-reports

# Integration tests (scheduler startup, HSM signer mock, tlog anchor simulation)
cargo test --test '*' --package eel-abis-reports -- --test-threads=1
```

## Bench commands

```bash
# Scheduled report generation (100 cases, DOCX + Markdown render + HSM sign)
cargo bench --package eel-abis-reports -- generate_report

# Tlog anchor submission (measure round-trip time)
cargo bench --package eel-abis-reports -- tlog_anchor

# Report list query (1000 reports, pagination)
cargo bench --package eel-abis-reports -- list_reports
```

## Dependencies

- `tokio-schedule` - Cron-based scheduler for daily/weekly report jobs
- `eel-abis-render-docx`, `eel-abis-render-markdown` - Renderer crates
- `sqlx` - Report metadata writes, list queries
- `sha2` - Report content hash for signing
- `tonic` - gRPC client for HSM signer and tlog services
- `eel-crypto-rs` - HSM integration utilities

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Report generation pipeline, HSM signing flow, tlog anchoring
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Manual report generation, HSM certificate rotation, tlog anchor verification
