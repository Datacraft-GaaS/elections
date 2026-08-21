# M-Pesa Daraja Adapter

Production adapter for Safaricom Daraja merchant payment API integration. Handles OAuth2 token management, C2B callback parsing, B2C payout requests, and STK push for interactive PIN entry flows.

## Purpose

Bridges the eel core system (M47, M55, M13 modules) to Safaricom Daraja API. Manages authentication, request/response transformation, retry logic, and callback validation. Implements circuit breaker pattern for resilience.

## Layout

- `service.py`: Main adapter service. Handles token lifecycle, C2B registration, B2C payment initiation, STK push, and callback validation.
- `views.py`: Pydantic models for all API payloads. Request shapes for STK push, B2C payment, C2B callback validation. Response models with enums for transaction types and status codes.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**MpesaDarajaService** (async):
- `generate_token()`: Obtains OAuth2 access token, caches with TTL.
- `register_c2b_urls()`: Registers validation and confirmation URLs with Daraja.
- `initiate_b2c_payment()`: Initiates B2C bulk payout, returns ConversationID.
- `stk_push()`: Initiates STK (SIM Toolkit) prompt on customer phone.
- `validate_c2b_callback()`: Verifies C2B callback authenticity and parses payload.

**Request/Response Models** (in views.py):
- `C2BCallbackRequest`: Parsed C2B callback with validated MSISDN, amount, transaction ID.
- `STKPushRequest`: STK initiation parameters (amount, phone, account reference).
- `STKPushResponse`: Response with MerchantRequestID, CheckoutRequestID, response code.
- `B2CPaymentRequest`: B2C payout parameters (amount, recipient phone, command ID).
- `B2CResultCallback`: Async result callback from Daraja for B2C transaction.

## Environment Variables

MPESA_ENV: "sandbox" or "production" (default: sandbox)
MPESA_CONSUMER_KEY: OAuth consumer key (required)
MPESA_CONSUMER_SECRET: OAuth consumer secret (required)
MPESA_SHORTCODE: Business short code (required)
MPESA_PASSKEY: Online passkey for STK encoding (required)
MPESA_CALLBACK_URL: Public webhook URL for C2B callbacks (required)
MPESA_TIMEOUT_SEC: HTTP timeout in seconds (default: 30)
MPESA_CHAOS: "true" enables chaos injection (test only)

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

Tests are in ../contract-tests/; run from repo root.
