# NSSF Test Double

Mock REST server simulating National Social Security Fund API for integration testing. Accepts mTLS connections and API requests, returns canned responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without NSSF dependency. Validates adapter mTLS certificate handling, API key headers, contribution formatting, and error responses. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_nssf
```

Server listens on http://localhost:8004 by default (mTLS disabled for testing; use self-signed cert in CI).

To override port:
```bash
PORT=9004 uv run python -m eel_double_nssf
```

Environment variable: NSSF_DOUBLE_PORT (default: 8004).

## Chaos Knobs

Control failure modes via environment variables:

NSSF_CHAOS_ENABLED: "true" to activate all chaos (default: false)
NSSF_CHAOS_REJECTION_RATE: Fraction [0.0-1.0] of contribution submissions rejected (default: 0.0)
NSSF_CHAOS_MEMBER_NOT_FOUND_RATE: Fraction [0.0-1.0] of member lookups returning 404 (default: 0.0)
NSSF_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
NSSF_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
NSSF_CHAOS_ENABLED=true NSSF_CHAOS_REJECTION_RATE=0.1 uv run python -m eel_double_nssf
```

## Canned Responses

**Contribution Submit** (POST /contributions):
Returns `{"submission_id": "SUB-2024-001", "status": "ACCEPTED", "accepted_count": 50, "rejected_count": 0, "total_amount": 250000}`.
On chaos: returns `{"status": "REJECTED", "errors": ["Invalid member IDs", "Duplicate entries"]}` with 400 status.

**Member Verify** (GET /members/{member_id}):
Returns `{"member_id": "MEM123", "name": "John Doe", "status": "ACTIVE", "account_balance": 125000, "last_contribution_date": "2024-08-01"}`.
On not found: returns 404 `{"error": "Member not found"}`.

**Statement Fetch** (GET /statements/{member_id}?year=2024):
Returns `{"statement_url": "https://nssf.or.ke/stmt/MEM123_2024.pdf", "generated_date": "2024-08-21"}`.
On invalid year: returns 400 `{"error": "Statement not available for year"}`.

**Health** (GET /health):
Always returns 200 `{"status": "OK"}`.

## Endpoints Exposed

GET /health - Health check
POST /contributions - Submit contributions
GET /members/{member_id} - Member verification
GET /statements/{member_id} - Statement fetch
