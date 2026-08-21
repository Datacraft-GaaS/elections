# KNADS Adapter

Production adapter for Kenya National Archives and Documentation Service REST API. Handles document ingestion, retrieval, and preservation verification for long-term archival.

## Purpose

Bridges eel core system (M69, M14 modules) to KNADS API. Manages API key authentication, document metadata formatting, base64 encoding/decoding, archival reference tracking, and preservation status verification.

## Layout

- `service.py`: Main adapter service. Handles record ingestion, retrieval, preservation checking, metadata enrichment, and archival status tracking.
- `views.py`: Pydantic models for KNADS API payloads. Models for RecordIngest, RecordRetrieve, PreservationCheck operations. Document type and status enums.
- `client.py`: HTTP client initialization with API-key header and timeout configuration.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**KnadsService** (async):
- `ingest_record(collection_id, title, doc_type, content)`: Submit document for archival, returns IngestReceipt.
- `retrieve_record(archival_reference)`: Fetch archived document, returns RecordData.
- `check_preservation_status(archival_reference)`: Verify archival preservation status, returns PreservationStatus.

**Models** (in views.py):
- `RecordIngestRequest`: Collection ID, title, type, content type, metadata dict, payload.
- `RecordIngestResponse`: Ingest ID, status, archival reference, preservation URL, timestamp.
- `RecordData`: Title, type, content type, metadata, preservation format, retrieval URL, access level.
- `PreservationStatus`: Status enum (VERIFIED, DEGRADED, LOST), last verified date, verification method, SHA-256 hash.

## Environment Variables

KNADS_BASE_URL: KNADS API base URL
KNADS_API_KEY: API key for X-API-Key header
KNADS_COLLECTION_ID: Default collection ID for ingestions
KNADS_TIMEOUT_SEC: Request timeout in seconds (default: 60)
KNADS_CHAOS: "true" enables chaos injection (test only)

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
