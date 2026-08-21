# abis-report-publisher

Report publication, delivery scheduling, and channel adapters. Manages lifecycle of generated adjudication decision reports: uploads to MinIO, schedules delivery (email, SMS, push notification), and tracks delivery status.

## What it does

- MinIO uploads: Stores rendered DOCX and Markdown reports in MinIO buckets organized by date and case ID
- Delivery scheduling: Routes reports to delivery channels (email, SMS OTP, push notification) based on recipient preferences
- Channel adapters: Abstracts email (Stalwart), SMS (Safaricom), push (Firebase Cloud Messaging) behind common interface
- Delivery tracking: Records delivery status (queued, sent, failed, bounced) in `eel_report_deliveries` table
- Retry logic: Exponential backoff for transient failures; gives up after max retries with alert
- Event emission: Publishes `abis.report.delivered.v1` and `abis.report.delivery.failed.v1` to Kafka

## Public API surface

```rust
pub struct ReportContent {
    pub case_id: String,
    pub format: ReportFormat,
    pub docx: Option<Vec<u8>>,
    pub markdown: Option<String>,
}

pub enum ReportFormat { DOCX, Markdown, HTML }

pub struct DeliveryRequest {
    pub report: ReportContent,
    pub recipient: String,
    pub channels: Vec<DeliveryChannel>,
}

pub enum DeliveryChannel { Email, SMS, Push }

pub async fn publish_report(req: DeliveryRequest) -> Result<()>;
pub async fn upload_to_minio(report: &ReportContent) -> Result<String>;
pub async fn schedule_delivery(recipient: &str, channels: &[DeliveryChannel]) -> Result<()>;
pub async fn process_delivery_queue();
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (channel routing, retry logic, delivery state machine)
cargo test --lib --package eel-abis-report-publisher

# Integration tests (MinIO mock, email/SMS adapter stubs, PG delivery tracking)
cargo test --test '*' --package eel-abis-report-publisher -- --test-threads=1
```

## Bench commands

```bash
# MinIO upload time (10MB DOCX)
cargo bench --package eel-abis-report-publisher -- minio_upload

# Delivery channel routing (1000 recipients across channels)
cargo bench --package eel-abis-report-publisher -- route_deliveries
```

## Dependencies

- `s3` (or `minio`) - MinIO client for bucket operations
- `tokio` - Async delivery queue worker
- `sqlx` - Delivery tracking table writes
- `reqwest` - HTTP adapters for email and SMS APIs
- `rdkafka` - Event publication to Kafka

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Report flow diagram, delivery channel architecture
- [OPERATIONS.md](../../docs/OPERATIONS.md) - MinIO bucket setup, delivery failure troubleshooting, retry tuning
