# M-Pesa Daraja C2B Handler

M-Pesa Daraja C2B (Customer to Business) callback handler service. Receives STK push callbacks from Safaricom, validates payment confirmations, and publishes `eel.payments.mpesa-c2b` Kafka events for downstream payment workflows.

## Overview

Module: M47 (Nomination Fee Payment)

Payment callback processor for nomination fee collection. Receives M-Pesa payment confirmations and triggers candidate registration updates via Kafka events.

## FastAPI Endpoints

- POST `/mpesa/callback/c2b` - Receive C2B payment callback from Safaricom
- POST `/mpesa/callback/stk` - Receive STK push response callback
- GET `/health` - Service health check

## Pydantic Models

- `C2BCallback` - Payment confirmation (TransactionType, TransID, TransAmount, BusinessShortCode, BillRefNumber, MSISDN)
- `STKCallback` - STK push response (MerchantRequestID, CheckoutRequestID, ResultCode, ResultDesc)

## Environment Variables

- `MPESA_CONSUMER_KEY` - Daraja API consumer key
- `MPESA_CONSUMER_SECRET` - Daraja API consumer secret
- `MPESA_SHORTCODE` - Business short code (receiving payments)
- `MPESA_PASSKEY` - Daraja passkey for STK encryption
- `KAFKA_BROKERS` - Kafka broker endpoints
- `DATABASE_URL` - PostgreSQL connection (for idempotency tracking)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn daraja.wsgi:application --workers 4
```

## Idempotency

Duplicate callback guard via TransID uniqueness check. Callbacks with duplicate TransID are silently ignored (idempotent).

## Notes

- STK push timeouts (user doesn't enter PIN) return ResultCode=1031
- Successful payment: ResultCode=0
- Failed payment: ResultCode > 0 (user cancelled, insufficient balance, etc.)
- Callbacks must be configured at Safaricom Daraja portal
- All callbacks are logged for reconciliation
