# Government Printer Test Double

Mock REST and SFTP server simulating Kenya Government Printer services for integration testing. Accepts gazette submissions and print orders, returns canned responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without Government Printer dependency. Validates adapter API key headers, SHA-256 file verification, print order formatting, and dual transport fallback. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_gov_printer
```

REST server listens on http://localhost:8007 by default. SFTP emulation (no real SFTP; files stored in memory).

To override port:
```bash
PORT=9007 uv run python -m eel_double_gov_printer
```

Environment variable: GOV_PRINTER_DOUBLE_PORT (default: 8007).

## Chaos Knobs

Control failure modes via environment variables:

GOV_PRINTER_CHAOS_ENABLED: "true" to activate all chaos (default: false)
GOV_PRINTER_CHAOS_SUBMISSION_REJECTION_RATE: Fraction [0.0-1.0] of gazette submissions returning 400 (default: 0.0)
GOV_PRINTER_CHAOS_ORDER_DELAY: Minutes to add to estimated delivery date (default: 0)
GOV_PRINTER_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
GOV_PRINTER_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
GOV_PRINTER_CHAOS_ENABLED=true GOV_PRINTER_CHAOS_SUBMISSION_REJECTION_RATE=0.1 uv run python -m eel_double_gov_printer
```

## Canned Responses

**Gazette Submit** (POST /api/v1/gazette/submit):
Returns `{"submission_id": "GAZ-2024-001", "status": "RECEIVED", "publication_url": "http://gazette.go.ke/2024/008/GAZ-2024-001.pdf", "publication_date": "2024-08-22"}`.
On invalid hash: returns 400 `{"error": "File integrity check failed"}`.
On chaos: returns 400 `{"error": "Processing error"}`.

**Print Order Create** (POST /api/v1/orders):
Returns `{"order_id": "ORD-2024-001", "status": "ACCEPTED", "estimated_delivery_date": "2024-09-04"}`.
On invalid document_id: returns 400 `{"error": "Document not found"}`.

**Print Status** (GET /api/v1/orders/{order_id}):
Returns `{"order_id": "ORD-2024-001", "status": "ACCEPTED", "current_stage": "queued", "estimated_completion_date": "2024-09-04"}`.
On missing order: returns 404 `{"error": "Order not found"}`.

**Health** (GET /api/v1/health):
Always returns 200 `{"status": "OK"}`.

## Endpoints Exposed

GET /api/v1/health - Health check
POST /api/v1/gazette/submit - Gazette submission
POST /api/v1/orders - Create print order
GET /api/v1/orders/{order_id} - Print status query
