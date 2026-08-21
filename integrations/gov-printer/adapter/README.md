# Government Printer Adapter

Production adapter for Kenya Government Printer REST API and SFTP services. Handles gazette PDF submission, print order management, and status tracking via dual transport channels.

## Purpose

Bridges eel core system (M52, M67, M39 modules) to Government Printer services. Manages API key authentication, SFTP key management, dual transport fallback, SHA-256 file verification, and long-running print job tracking.

## Layout

- `service.py`: Main adapter service. Handles REST API submissions (gazette, print orders), SFTP uploads, status queries, transport fallback logic.
- `views.py`: Pydantic models for REST API payloads and responses. Models for GazetteSubmit, PrintOrderCreate, PrintStatusCheck operations.
- `sftp_client.py`: SSH key loading, SFTP connection management, file upload with integrity verification.
- `api_client.py`: REST API client with API-key header and timeout configuration.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**GovernmentPrinterService** (async):
- `submit_gazette_pdf(date, issue_number, pdf_data)`: Upload gazette PDF via REST, returns SubmissionID.
- `create_print_order(document_id, copies, binding, address)`: Create print job, returns OrderID.
- `check_print_status(order_id)`: Query order status, returns PrintStatus.
- `_upload_via_sftp(filename, pdf_bytes)`: Fallback SFTP upload if REST fails.
- `_verify_file_integrity(pdf_bytes, expected_sha256)`: Validate PDF using content hash.

**Models** (in views.py):
- `GazetteSubmitRequest`: Date, issue number, filename, file size, SHA-256 hash.
- `GazetteSubmitResponse`: Submission ID, status, publication URL, date.
- `PrintOrderRequest`: Document ID, copy count, binding type, delivery address.
- `PrintOrderResponse`: Order ID, status, estimated delivery date.
- `PrintStatusResponse`: Order ID, current status, stage, completion date estimate.

## Environment Variables

GOV_PRINTER_API_KEY: REST API key
GOV_PRINTER_BASE_URL: REST API base URL
GOV_PRINTER_SFTP_HOST: SFTP server hostname
GOV_PRINTER_SFTP_PORT: SFTP port (default: 22)
GOV_PRINTER_SFTP_USER: SFTP username
GOV_PRINTER_SSH_KEY_PATH: Path to SSH private key (PEM)
GOV_PRINTER_TIMEOUT_SEC: Request timeout (default: 60)
GOV_PRINTER_CHAOS: "true" enables chaos injection (test only)

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
