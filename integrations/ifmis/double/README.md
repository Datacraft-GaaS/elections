# IFMIS Test Double

Mock REST server simulating National Treasury IFMIS API for integration testing. Accepts API requests, returns canned responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without IFMIS dependency. Validates adapter OAuth token handling, budget query formatting, voucher creation, and error responses. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_ifmis
```

Server listens on http://localhost:8003 by default.

To override port:
```bash
PORT=9003 uv run python -m eel_double_ifmis
```

Environment variable: IFMIS_DOUBLE_PORT (default: 8003).

## Chaos Knobs

Control failure modes via environment variables:

IFMIS_CHAOS_ENABLED: "true" to activate all chaos (default: false)
IFMIS_CHAOS_BUDGET_ERROR_RATE: Fraction [0.0-1.0] of budget check requests returning 404 (default: 0.0)
IFMIS_CHAOS_VOUCHER_REJECTION_RATE: Fraction [0.0-1.0] of voucher creation returning 422 UNPROCESSABLE_ENTITY (default: 0.0)
IFMIS_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
IFMIS_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
IFMIS_CHAOS_ENABLED=true IFMIS_CHAOS_BUDGET_ERROR_RATE=0.1 uv run python -m eel_double_ifmis
```

## Canned Responses

**OAuth Token Endpoint** (/oauth/token):
Returns `{"access_token": "test-token-xxx", "token_type": "Bearer", "expires_in": 1800}`.
On error: returns `{"error": "invalid_client"}` with 401 status.

**Budget Check** (GET /api/v1/budget/check?code=E123):
Returns `{"budget_code": "E123", "fiscal_year": "2024", "total_allocation": 5000000, "total_committed": 3200000, "available_balance": 1800000}`.
On missing code: returns 404 `{"error": "Budget code not found"}`.

**Voucher Creation** (POST /api/v1/vouchers):
Returns `{"voucher_id": "V-2024-001", "status": "CREATED", "created_timestamp": "2024-08-21T10:30:00Z", "reference_number": "REF-2024-001"}`.
On insufficient budget: returns 422 `{"error": "Insufficient budget balance"}`.
On invalid supplier: returns 400 `{"error": "Supplier ID not recognized"}`.

**Payment Status** (GET /api/v1/payments/{voucher_id}/status):
Returns `{"voucher_id": "V-2024-001", "status": "PENDING", "paid_amount": 0, "paid_date": null}`.
On nonexistent voucher: returns 404 `{"error": "Voucher not found"}`.

## Endpoints Exposed

GET /oauth/token - OAuth2 token endpoint
GET /api/v1/health - Health check (always returns 200)
GET /api/v1/budget/check - Budget allocation query
POST /api/v1/vouchers - Create voucher
GET /api/v1/payments/{voucher_id}/status - Payment status query

All endpoints return JSON and support standard HTTP status codes (200, 400, 401, 404, 422, 500).
