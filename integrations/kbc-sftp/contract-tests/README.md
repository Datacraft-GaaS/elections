# KBC SFTP Contract Tests

Integration test suite validating adapter compliance with Kenya Broadcasting Corporation SFTP server contract. Tests SSH key authentication, file upload/download, directory operations, and FTP fallback logic.

## Purpose

Verifies adapter correctly manages SFTP connections, uploads files with proper naming, handles directory operations, and falls back to FTP on SFTP failure. Tests SSH key loading, connection lifecycle, and error recovery.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_sftp_upload.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "fallback" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**SSH Key Authentication:**
- Private key loaded from KBC_SFTP_KEY_PATH
- Key file permissions validated (must be 0600 or 0400)
- Key format validated (PEM format)
- SSH connection established using key pair
- Invalid key file returns error
- Missing key file returns error

**SFTP Connection:**
- Connection to KBC_SFTP_HOST:KBC_SFTP_PORT established
- Connection username set to KBC_SFTP_USER
- Connection timeout honored (KBC_SFTP_TIMEOUT_SEC)
- Connection closed cleanly after operations
- Stale connections detect and refresh

**File Upload:**
- File exists before upload attempt
- File transferred to correct remote path (KBC_UPLOAD_DIR)
- Remote filename formatted correctly
- File size preserved after upload
- File modification date set correctly
- Upload confirmation returned
- Large file (500MB) uploaded successfully
- Duplicate filename handling tested

**Directory Listing:**
- List operation on remote directory succeeds
- Returned file list includes uploaded files
- File stats (size, mod_date) populated correctly
- Directory traversal fails (security check)
- Empty directory handled

**File Deletion:**
- Uploaded file deleted successfully
- Deletion confirmation returned
- Deleted file no longer listed in directory
- Deleting nonexistent file handled gracefully

**FTP Fallback:**
- SFTP failure triggers FTP fallback
- FTP login uses same credentials (username/password or key)
- File uploaded via FTP to same directory (/incoming/eel/)
- FTP fallback logs success for operator visibility
- Both SFTP and FTP uploads deliver same result

**Error Handling:**
- Connection timeout after KBC_SFTP_TIMEOUT_SEC triggers circuit breaker
- Authentication failure (wrong key) returns error
- Permission denied (upload dir not writable) handled
- Disk quota exceeded returned as specific error
- Network connection loss triggers retry

**Circuit Breaker:**
- Aggressive breaker opens after 1 failure (SFTP is stateful)
- Open duration 300 seconds (long recovery for stateful connection)
- Half-open state probes with single request
- Probe failure re-opens circuit

**Chaos Testing:**
- KBC_SFTP_CHAOS=true injects errors at configured rate
- Connection failure rate injected correctly
- Upload failure rate injected correctly
- Latency injection delays operations
- Timeout injection forces adapter retry/fallback
