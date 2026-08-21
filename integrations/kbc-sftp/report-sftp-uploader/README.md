# report-sftp-uploader

Rust adapter that bridges `eel-abis-report-publisher` to the `eel-report-deliverers`
Python service for KBC SFTP bundle uploads.

## Endpoint contract

### POST /deliver/kbc-sftp

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
| `remote_path` | string or null | Full path on the KBC SFTP server |
| `sha256` | string or null | Hex SHA-256 of the uploaded tar.gz bundle |
| `uploaded_at` | ISO-8601 datetime or null | Timestamp of upload |

## Timing SLA

Uploads must land within the weekly upload window: **Friday 18:00 EAT (UTC+3)**.
Late deliveries should be queued and held until the next window by the scheduler.

## File naming convention

```
eel_{subscription_id}_{artifact_id}.tar.gz
```

Each bundle contains:
- `{artifact_id}.{format}` - the artifact bytes
- `manifest.json` - metadata (artifact_id, subscription_id, format, sha256, bundled_at)
- `manifest.sig` - PGP signature (stub if gnupg not installed)

## Retention

20 years. KBC SFTP archives are immutable; never overwrite or delete uploaded files.

## Configuration

| Env var | Default | Purpose |
|---|---|---|
| `EEL_KBC_SFTP_ADAPTER_URL` | `http://localhost:8091` | Base URL of the Python bridge service |
| `EEL_KBC_SFTP_HOST` | `sftp.kbc.example.go.ke` | KBC SFTP host |
| `EEL_KBC_SFTP_PORT` | `22` | KBC SFTP port |
| `EEL_KBC_SFTP_USER` | `eel` | KBC SFTP username |
| `EEL_KBC_SFTP_KEY_PATH` | (required) | Path to private key for SFTP auth |
| `EEL_KBC_SFTP_REMOTE_DIR` | `/uploads/eel` | Remote directory on KBC SFTP |

## Contract tests

`contract-tests/pact.json` contains a Pact v2 contract.
Run against a live Python bridge with `pact-verifier`.
