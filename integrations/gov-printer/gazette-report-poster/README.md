# gazette-report-poster

Rust adapter that bridges `eel-abis-report-publisher` (the delivery pipeline)
to the `eel-report-deliverers` Python service for Gov Printer submissions.

## Endpoint contract

### POST /deliver/gov-printer

**Request body** (JSON):

| Field | Type | Description |
|---|---|---|
| `artifact_ref` | string | MinIO presigned URL or internal ref to the artifact bytes |
| `artifact_id` | UUID | Artifact UUID from `eel_report_artifacts` |
| `subscription_id` | UUID | Subscription UUID from `eel_report_subscriptions` |
| `format` | string | Artifact format (`pdf`, `docx`, etc.) |
| `sha256` | string | Hex-encoded SHA-256 of the artifact bytes |

**Response body** (JSON, HTTP 200):

| Field | Type | Description |
|---|---|---|
| `submission_id` | string or null | Gazette submission identifier returned by Gov Printer |
| `submitted_at` | ISO-8601 datetime or null | Timestamp of submission |

**Error responses**: 4xx for permanent failures, 5xx for transient (will be retried by the pipeline).

## Retention

Gazette entries are permanent. No deletion or TTL is applied.

## Configuration

| Env var | Default | Purpose |
|---|---|---|
| `EEL_GOV_PRINTER_ADAPTER_URL` | `http://localhost:8091` | Base URL of the Python bridge service |

## Contract tests

`contract-tests/pact.json` contains a Pact v2 contract.
Run against a live Python bridge with `pact-verifier`.
