# NHIF-SHIF Test Double

Mock REST server simulating Social Health Authority API for integration testing. Accepts mTLS connections and API requests, returns canned responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without SHA dependency. Validates adapter mTLS certificate handling, API key headers, contribution formatting, and error responses. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_nhif_shif
```

Server listens on http://localhost:8005 by default.

To override port:
```bash
PORT=9005 uv run python -m eel_double_nhif_shif
```

Environment variable: NHIF_DOUBLE_PORT (default: 8005).

## Chaos Knobs

Control failure modes via environment variables:

NHIF_SHIF_CHAOS_ENABLED: "true" to activate all chaos (default: false)
NHIF_SHIF_CHAOS_REJECTION_RATE: Fraction [0.0-1.0] of contribution rejections (default: 0.0)
NHIF_SHIF_CHAOS_MEMBER_INACTIVE_RATE: Fraction [0.0-1.0] of member lookups returning INACTIVE (default: 0.0)
NHIF_SHIF_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
NHIF_SHIF_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
NHIF_SHIF_CHAOS_ENABLED=true NHIF_SHIF_CHAOS_REJECTION_RATE=0.1 uv run python -m eel_double_nhif_shif
```

## Canned Responses

**Contribution Submit** (POST /contributions):
Returns `{"receipt_number": "SHIF-2024-001", "status": "ACCEPTED", "accepted_count": 100, "rejected_count": 0}`.
On chaos: returns `{"status": "REJECTED", "errors": ["Invalid national IDs", "Duplicate entries"]}` with 400 status.

**Beneficiary Verify** (GET /members/{national_id}):
Returns `{"national_id": "12345678", "status": "ACTIVE", "member_number": "MEM123", "insurance_class": "CIVIL_SERVANT", "enrollment_date": "2024-01-01"}`.
On chaos: returns `{"status": "INACTIVE"}` with 400 status.

**Health** (GET /health):
Always returns 200 `{"status": "OK"}`.

## Endpoints Exposed

GET /health - Health check
POST /contributions - Submit contributions
GET /members/{national_id} - Beneficiary verification
