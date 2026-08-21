# KBC SFTP Test Double

Mock SFTP server simulating Kenya Broadcasting Corporation file drop for integration testing. Accepts file uploads, simulates directory operations, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without KBC SFTP dependency. Validates adapter SSH key authentication, file upload operations, directory listing, and FTP fallback logic. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_kbc_sftp
```

SFTP mock listens on port 2222 by default (not standard 22 to avoid permission issues).

To override port:
```bash
SFTP_PORT=2223 uv run python -m eel_double_kbc_sftp
```

Environment variable: KBC_SFTP_DOUBLE_PORT (default: 2222).

## Chaos Knobs

Control failure modes via environment variables:

KBC_SFTP_CHAOS_ENABLED: "true" to activate all chaos (default: false)
KBC_SFTP_CHAOS_CONNECTION_FAILURE_RATE: Fraction [0.0-1.0] of connection attempts failing (default: 0.0)
KBC_SFTP_CHAOS_UPLOAD_FAILURE_RATE: Fraction [0.0-1.0] of file uploads returning permission denied (default: 0.0)
KBC_SFTP_CHAOS_LATENCY_MS: Milliseconds to delay operations (default: 0)
KBC_SFTP_CHAOS_TIMEOUT_SEC: Force operation timeout after N seconds (default: 0, disabled)

Example:
```bash
KBC_SFTP_CHAOS_ENABLED=true KBC_SFTP_CHAOS_UPLOAD_FAILURE_RATE=0.1 uv run python -m eel_double_kbc_sftp
```

## Canned Responses

**File Upload** (PUT /incoming/eel/results_*.json):
Returns success. File stored in memory with fake stat info: size 1024000 (1MB), mod_date now.
On chaos: returns "Permission denied" or "Disk quota exceeded" error.

**File List** (LS /incoming/eel/):
Returns list of previously uploaded files. Each entry shows filename, size, mod_date.
On missing directory: returns "No such file or directory".

**File Delete** (RM /incoming/eel/results_*.json):
Returns success, removes file from virtual directory.
On missing file: returns "No such file or directory".

**Directory Stats** (STAT /incoming/eel/):
Returns directory info: type=directory, permissions=0755, mod_date=now.

## Authentication

Tests should provide:
- SSH public key matching double's internal test key (or disable key check)
- Username: test_user (hardcoded in double)
- No password authentication

For testing, double accepts any key or can run in no-auth mode (see test fixtures).

## Endpoints Exposed (SFTP Protocol)

SFTP operations:
- PUT - File upload to /incoming/eel/
- GET - File download from /incoming/eel/ (simulated)
- LS - List files in directory
- STAT - Get file/directory statistics
- RM - Remove file
- MKDIR - Create directory (simulated)
