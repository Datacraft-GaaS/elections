# IFMIS Adapter

Production adapter for National Treasury IFMIS REST API. Handles OAuth2 token management, budget queries, voucher creation, and payment status tracking.

## Purpose

Bridges eel core system (M54, M59 modules) to IFMIS API. Manages OAuth token lifecycle, request formatting, response parsing, error handling from government system, and budget balance validation.

## Layout

- `service.py`: Main adapter service. Handles OAuth token refresh, budget lookups, voucher posting, payment status queries, budget balance validation.
- `views.py`: Pydantic models for IFMIS request/response payloads. Models for BudgetCheck, VoucherCreate, PaymentStatus operations. Error code enums.
- `oauth_client.py`: OAuth2 token manager with automatic refresh.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**IfmisService** (async):
- `check_budget_balance(vote_head, amount)`: Verify budget availability, returns BudgetStatus.
- `create_voucher(vote_head, supplier_id, amount, description)`: Create expenditure voucher, returns VoucherReceipt.
- `get_payment_status(voucher_id)`: Query payment status, returns PaymentStatus.
- `_ensure_valid_token()`: Refresh OAuth token if expired.

**Models** (in views.py):
- `BudgetCheckRequest`: Vote head code, fiscal year, optional amount to check.
- `BudgetCheckResponse`: Total allocation, committed amount, available balance.
- `VoucherCreateRequest`: Vote head, supplier ID, amount, description, invoice reference.
- `VoucherCreateResponse`: Voucher ID, status, created timestamp, reference number.
- `PaymentStatusRequest`: Voucher ID.
- `PaymentStatusResponse`: Status enum (PAID, PENDING, REJECTED), amounts, dates.

## Environment Variables

IFMIS_BASE_URL: IFMIS API base URL
IFMIS_CLIENT_ID: OAuth2 client ID (required)
IFMIS_CLIENT_SECRET: OAuth2 client secret (required)
IFMIS_TOKEN_URL: OAuth2 token endpoint
IFMIS_TIMEOUT_SEC: Request timeout in seconds (default: 30)
IFMIS_CHAOS: "true" enables chaos injection (test only)

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
