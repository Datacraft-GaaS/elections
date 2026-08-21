# KBC SFTP File Drop

Kenya Broadcasting Corporation SFTP file drop for election results and media distribution. Handles file upload for broadcast publication, file listing, and deletion management.

## MoU Status

KBC broadcast agreement with NDA. SFTP upload account credentials and key pair issued for production use. Annual review and credential rotation required.

## Endpoint

SFTP Server: sftp.kbc.co.ke (exact hostname under NDA)
SFTP Port: 22
Upload Directory: /incoming/eel/ (or variant under NDA)

## Protocol and Authentication

SFTP (SSH-2). Authentication via SSH key pair:
- Private key managed locally (env var KBC_SFTP_KEY_PATH, PEM format)
- Public key registered on KBC SFTP server
- Username: KBC_SFTP_USER

## Request and Response Shape

**File Upload:**
Operation: Upload file via SFTP to /incoming/eel/
Input: Local file path, remote filename
Output: Upload success confirmation, file stat (size, permissions)

**File List:**
Operation: List files in /incoming/eel/ or /archive/
Input: Remote directory path
Output: List of files with size, modification date, permissions

**File Delete:**
Operation: Remove file from /incoming/eel/
Input: Remote filename
Output: Deletion confirmation

## Sample Usage

```bash
# Upload file via SFTP
sftp -i $KBC_SFTP_KEY_PATH $KBC_SFTP_USER@sftp.kbc.co.ke
> put results_2024_08_21.json incoming/eel/
> quit

# List uploaded files
sftp -i $KBC_SFTP_KEY_PATH $KBC_SFTP_USER@sftp.kbc.co.ke
> ls incoming/eel/
> quit
```

## Health Probe

SFTP connection test: attempt directory listing. If connection succeeds and lists directory, KBC SFTP is reachable and authenticated.

## Rate Limits

One SFTP session at a time (SFTP is stateful). Upload max 500MB/file. No rate limit on number of operations/minute, but serialized connection pattern enforced.

## PII Enforcement

Files are official election results with no personal information. Upload metadata (filenames, timestamps) are internal. No PII-specific masking required. File access logs restricted to authorized KBC staff.

## Chaos Toggle

KBC_SFTP_CHAOS=true enables chaos injection. Causes random connection failures and upload rejections (test only).

## Fallback Behavior

On SFTP unavailability, fallback to FTP backup server (ftp.kbc.co.ke). If FTP also unavailable, email results PDF to KBC operations team. Manual upload timeline extends by 4-6 hours.

## Circuit Breaker Policy

Threshold: 1 consecutive failure
Open duration: 300 seconds
Rationale: SFTP is stateful connection; even one failure suggests server maintenance or network issue; aggressive timeout prevents cascading timeouts
Half-open: permits 1 probe request

## Environment Variables

KBC_SFTP_HOST: SFTP server hostname
KBC_SFTP_PORT: SFTP port (default: 22)
KBC_SFTP_USER: SFTP username for authentication
KBC_SFTP_KEY_PATH: Path to SSH private key (PEM format)
KBC_UPLOAD_DIR: Remote directory for uploads (default: /incoming/eel/)
KBC_SFTP_TIMEOUT_SEC: Connection timeout in seconds (default: 30)
KBC_SFTP_CHAOS: "true" enables chaos (test only)

## Related Modules

M14 (reproducibility-bundle KBC publication), M52 (gazette)

## Runbook

docs/runbooks/kbc-sftp-file-management.md (file upload, archive management, FTP fallback)

## Note

Subdirectory report-sftp-uploader/ is owned by RPT-C agent (report publishing). Do not modify without coordination.

