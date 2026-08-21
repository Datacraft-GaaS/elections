# ABIS Report Deliverers

Kafka-driven event worker for delivering rendered ABIS reports to downstream channels (email, S3, KNADS, KBC SFTP). Subscribes to eel.abis.report-ready events and coordinates report distribution.

## Overview

Module: M03 (ABIS)

Worker service that consumes already-rendered ABIS reports from Kafka and delivers them to configured channels. This worker does NOT generate reports; it only delivers them. Report generation is owned by RPT-C module.

## Workers Exposed

- `EmailDeliverer` - Send reports via SMTP (internal stakeholders)
- `S3Deliverer` - Upload reports to S3 (long-term storage)
- `KNADSDeliverer` - Submit to KNADS (Kenya National Archive Digital Services)
- `KBCSFTPDeliverer` - Deliver to KBC (Kenya Broadcasting Corporation) via SFTP

## Pydantic Models

- `ReportDeliveryEvent` - Kafka event (report_id: str, channels: list[str], format: str, rendered_url: str)
- `DeliveryResult` - Delivery status (report_id, channel, status, delivered_at, error_msg)

## Environment Variables

- `KAFKA_BROKERS` - Kafka broker endpoints (comma-separated)
- `KAFKA_CONSUMER_GROUP` - Consumer group (default: eel-abis-deliverers)
- `S3_BUCKET` - S3 bucket for report archival
- `S3_PREFIX` - S3 key prefix (default: abis-reports/)
- `KNADS_ADAPTER_URL` - KNADS submission endpoint
- `KBC_SFTP_ADAPTER_URL` - KBC SFTP server details
- `SMTP_URL` - Email server (smtp://user:pass@host:port)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

As Kubernetes Deployment with Kafka consumer persistence:

```bash
python -m report_deliverers.worker
```

## Notes

- RPT-C owns report generation pipeline; this worker only handles delivery
- Failed deliveries are retried 3 times before dead-lettering
- S3 reports use server-side encryption by default
- KNADS and KBC deliveries require valid service credentials in env vars
