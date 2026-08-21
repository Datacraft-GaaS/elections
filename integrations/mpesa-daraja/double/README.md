# M-Pesa Daraja Test Double

Mock server simulating Safaricom Daraja API for integration testing. Accepts all M-Pesa API payloads, returns canned responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without real Daraja dependency. Validates adapter request formatting, callback parsing, and error handling. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_mpesa_daraja
```

Server listens on http://localhost:8001 by default.

To override port:
```bash
PORT=9001 uv run python -m eel_double_mpesa_daraja
```

Environment variable: MPESA_DOUBLE_PORT (default: 8001).

## Chaos Knobs

Control failure modes via environment variables:

MPESA_CHAOS_ENABLED: "true" to activate all chaos (default: false)
MPESA_CHAOS_TOKEN_ERROR_RATE: Fraction [0.0-1.0] of OAuth token requests returning 403 (default: 0.0)
MPESA_CHAOS_CALLBACK_LATENCY_MS: Milliseconds to delay STK/B2C responses (default: 0)
MPESA_CHAOS_CALLBACK_ERROR_RATE: Fraction [0.0-1.0] of callback responses returning 500 (default: 0.0)
MPESA_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
MPESA_CHAOS_ENABLED=true MPESA_CHAOS_CALLBACK_ERROR_RATE=0.1 uv run python -m eel_double_mpesa_daraja
```

## Canned Responses

**OAuth Token Endpoint** (/oauth/v1/generate):
Returns {"access_token": "test-token-xxx", "token_type": "Bearer", "expires_in": 3600}.
On error: returns {"error": "invalid_client"} with 403 status.

**C2B Callback** (via http://localhost:8001/mpesa/c2b/validate):
Returns {"ResultCode": "0", "ResultDesc": "Accepted"}.

**STK Push** (/mpesa/stkpush/v1/processrequest):
Returns {"MerchantRequestID": "uuid-xxx", "CheckoutRequestID": "ws-xxx", "ResponseCode": "0", "ResponseDescription": "Success", "CustomerMessage": "Please enter PIN"}.
On chaos: returns {"ResponseCode": "500", "ResponseDescription": "Internal Server Error"}.

**B2C Payment** (/mpesa/b2c/v3/paymentrequest):
Returns {"ConversationID": "conversation-xxx", "OriginatorConversationID": "originator-xxx", "ResponseCode": "0", "ResponseDescription": "Accepted"}.
Async callback (to configured ResultURL after 2 second delay) delivers {"Result": {"ResultType": 0, "ResultCode": 0, "ResultDesc": "success", "TransactionID": "LHG..."}}.

## Endpoints Exposed

POST /oauth/v1/generate - OAuth token endpoint
POST /mpesa/c2b/register - Register C2B URLs
POST /mpesa/c2b/validate - Validation callback endpoint
POST /mpesa/c2b/confirm - Confirmation callback endpoint
POST /mpesa/stkpush/v1/processrequest - STK push initiation
POST /mpesa/b2c/v3/paymentrequest - B2C payment request

All endpoints accept standard Daraja request bodies and return schemas matching Daraja production API.
