# Government Printer API

Kenya Gazette and official printing services integration. Handles PDF submission for gazette publication, print order creation, and print status tracking.

## MoU Status

Government Printer Service Level Agreement (SLA) signed with NDA. API access and SFTP credentials issued for production use.

## Endpoint

REST API: https://www.kenyagazette.go.ke/api/v1/
SFTP: sftp.kenyagazette.go.ke

## Protocol and Authentication

**REST API:**
HTTPS JSON. Authentication via X-API-Key header.

**SFTP:**
SSH-2 protocol. Authentication via SSH key pair (public key on server, private key managed locally).

## Request and Response Shape

**Gazette PDF Submit (REST):**
Request: `POST /api/v1/gazette/submit {gazette_date, issue_number, pdf_filename, file_size_bytes, content_hash_sha256}`
Response: `{submission_id, status: "RECEIVED|PROCESSING|PUBLISHED", publication_url: "http://...", publication_date: "2024-08-21"}`

**Print Order Create (REST):**
Request: `POST /api/v1/orders {document_id, print_copies, binding_type, delivery_address}`
Response: `{order_id, status: "ACCEPTED|REJECTED", estimated_delivery_date}`

**Print Status Check (REST):**
Request: `GET /api/v1/orders/{order_id}`
Response: `{order_id, status: "ACCEPTED|PRINTED|SHIPPED|DELIVERED", current_stage, estimated_completion_date}`

**SFTP Upload:**
File placement: /incoming/gazette_{yyyymmdd}_{sequence}.pdf

## Sample Curl

```bash
# Gazette submit via REST
curl -H "X-API-Key: $GOV_PRINTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"gazette_date":"2024-08-21","issue_number":"120","pdf_filename":"gazette_2024_08_21.pdf","file_size_bytes":5242880,"content_hash_sha256":"abc123..."}' \
  https://www.kenyagazette.go.ke/api/v1/gazette/submit

# SFTP upload
sftp -i $GOV_PRINTER_SSH_KEY printer@sftp.kenyagazette.go.ke
> put gazette_20240821_001.pdf incoming/
```

## Health Probe

GET /api/v1/health returns 200 if service is operational.

## Rate Limits

REST API: 10 requests/minute. SFTP sessions: one session at a time (sequential uploads enforced by printer workflow).

## PII Enforcement

Documents are official gazette notices with no personal information. Submission metadata (filenames, dates) are internal. No PII-specific masking required, but document access logs restricted to authorized staff.

## Chaos Toggle

GOV_PRINTER_CHAOS=true enables chaos injection. Causes random submission rejections and delivery delays (test only).

## Fallback Behavior

On REST API unavailability, PDF email backup available. Operator sends gazette PDF to printer helpdesk via secure email. Manual publication timeline extends by 2-3 days.

On SFTP unavailability, fallback to FTP backup server, then email to printer operations.

## Circuit Breaker Policy

Threshold: 1 consecutive failure
Open duration: 120 seconds
Rationale: Government printer is slow legacy system with long SLA; even one failure suggests extended maintenance
Half-open: permits 1 probe request

## Environment Variables

GOV_PRINTER_API_KEY: API key for REST authentication
GOV_PRINTER_BASE_URL: Base URL for REST API
GOV_PRINTER_SFTP_HOST: SFTP server hostname
GOV_PRINTER_SFTP_PORT: SFTP port (default: 22)
GOV_PRINTER_SFTP_USER: SFTP username
GOV_PRINTER_SSH_KEY_PATH: Path to SSH private key (PEM format)
GOV_PRINTER_TIMEOUT_SEC: Request timeout in seconds (default: 60)
GOV_PRINTER_CHAOS: "true" enables chaos (test only)

## Related Modules

M52 (election-calendar-gazette), M67 (party-symbol-registry), M39 (ballot-printing-chain)

## Runbook

docs/runbooks/gov-printer-gazette-management.md (gazette publication, print order tracking, fallback procedures)

## Note

Subdirectory gazette-report-poster/ is owned by RPT-C agent (report generation). Do not modify without coordination.

