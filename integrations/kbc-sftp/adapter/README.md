# KBC SFTP Adapter

Production adapter for Kenya Broadcasting Corporation SFTP file drop. Handles SSH key authentication, file upload/download, directory listing, and file deletion operations.

## Purpose

Bridges eel core system (M14, M52 modules) to KBC SFTP server. Manages SSH key loading, SFTP connection lifecycle, file upload with integrity verification, fallback to FTP, and connection error recovery.

## Layout

- `service.py`: Main adapter service. Handles SFTP connection management, file upload, directory listing, file deletion, FTP fallback logic.
- `views.py`: Pydantic models for file operations (upload request/response, list request/response).
- `sftp_client.py`: SSH key loading, SFTP session initialization, connection pooling, timeout handling.
- `ftp_fallback.py`: FTP client for fallback when SFTP unavailable.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**KbcSftpService** (async):
- `upload_file(local_path, remote_filename)`: Upload file to SFTP, returns FileStats.
- `list_files(remote_dir)`: List files in remote directory, returns list of FileInfo.
- `delete_file(remote_filename)`: Remove file from remote directory, returns success status.
- `_fallback_ftp_upload(local_path, remote_filename)`: Fallback FTP upload if SFTP fails.

**Models** (in views.py):
- `FileUploadRequest`: Local path, remote filename, optional metadata.
- `FileUploadResponse`: Remote filename, size, modification date, upload timestamp.
- `FileListRequest`: Remote directory path.
- `FileListResponse`: List of files with size, mod_date, permissions.
- `FileInfo`: Filename, size bytes, modification date, is_directory flag.

## Environment Variables

KBC_SFTP_HOST: SFTP server hostname
KBC_SFTP_PORT: SFTP port (default: 22)
KBC_SFTP_USER: SFTP username
KBC_SFTP_KEY_PATH: Path to SSH private key (PEM format)
KBC_UPLOAD_DIR: Remote upload directory (default: /incoming/eel/)
KBC_SFTP_TIMEOUT_SEC: Connection timeout (default: 30)
KBC_SFTP_CHAOS: "true" enables chaos injection (test only)

## Build and Test

Build package:
```bash
cd adapter
uv build
```

Install in development:
```bash
uv pip install -e .
```

Type check:
```bash
uv run pyright src/
```

Run tests:
```bash
uv run pytest tests/ -vxs
```
