# KNADS Test Double

Mock REST server simulating Kenya National Archives and Documentation Service API for integration testing. Accepts document ingestion requests, simulates archival operations, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without KNADS dependency. Validates adapter API key headers, document metadata formatting, base64 encoding, archival reference tracking, and preservation verification. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_knads
```

Server listens on http://localhost:8008 by default.

To override port:
```bash
PORT=9008 uv run python -m eel_double_knads
```

Environment variable: KNADS_DOUBLE_PORT (default: 8008).

## Chaos Knobs

Control failure modes via environment variables:

KNADS_CHAOS_ENABLED: "true" to activate all chaos (default: false)
KNADS_CHAOS_INGEST_FAILURE_RATE: Fraction [0.0-1.0] of ingestion requests returning 400 (default: 0.0)
KNADS_CHAOS_PRESERVATION_DEGRADED_RATE: Fraction [0.0-1.0] of preservation checks returning DEGRADED (default: 0.0)
KNADS_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
KNADS_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
KNADS_CHAOS_ENABLED=true KNADS_CHAOS_INGEST_FAILURE_RATE=0.1 uv run python -m eel_double_knads
```

## Canned Responses

**Record Ingest** (POST /ingest):
Returns `{"ingest_id": "INGEST-2024-001", "status": "ACCEPTED", "archival_reference": "KNADS-2024-00001", "preservation_url": "https://knads.go.ke/records/KNADS-2024-00001", "ingest_timestamp": "2024-08-21T10:30:00Z"}`.
On chaos: returns 400 `{"error": "Invalid document type"}`.

**Record Retrieve** (GET /records/{archival_reference}):
Returns `{"archival_reference": "KNADS-2024-00001", "document_title": "Polling Results", "document_type": "dataset", "content_type": "application/json", "preservation_format": "PDF/A-3", "retrieval_url": "https://knads.go.ke/download/KNADS-2024-00001", "access_restrictions": "UNRESTRICTED"}`.
On missing reference: returns 404 `{"error": "Record not found"}`.

**Preservation Check** (GET /records/{archival_reference}/preservation-status):
Returns `{"status": "VERIFIED", "last_verified_date": "2024-08-21", "verification_method": "FIXITY_CHECK", "fixity_hash_sha256": "abc123..."}`.
On chaos: returns `{"status": "DEGRADED", "last_verified_date": "2024-08-21"}`.

**Health** (GET /health):
Always returns 200 `{"status": "OK"}`.

## Endpoints Exposed

GET /health - Health check
POST /ingest - Submit record for archival
GET /records/{archival_reference} - Retrieve archived record
GET /records/{archival_reference}/preservation-status - Preservation verification
