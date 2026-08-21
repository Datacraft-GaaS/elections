# M-Pesa B2C Payout

M-Pesa Business-to-Customer (B2C) payout service for bulk salary disbursement. Executes payroll run payouts via M-Pesa transfers to poll workers and election staff. Handles callback confirmation, retries, and reconciliation.

## Overview

Module: M55 (HR Payroll)

Bulk payout processor for election staff salary distribution via M-Pesa. Enables rapid, traceable salary payments without wire transfers or bank checks.

## FastAPI Endpoints

- POST `/mpesa/b2c/payout` - Submit payroll batch for B2C disbursement
- POST `/mpesa/b2c/callback` - Receive M-Pesa callback confirmation
- GET `/mpesa/b2c/payout/{id}/status` - Poll payout status
- GET `/health` - Service health check

## Pydantic Models

- `B2CPayoutRequest` - Batch request (payroll_run_id: str, recipients: list[dict], amount_per: float)
- `B2CCallback` - Callback (ConversationID, TransactionID, ResultCode, ResultDesc, ReceiverPartyPublicName)

## Environment Variables

- `MPESA_CONSUMER_KEY` - Daraja API consumer key
- `MPESA_CONSUMER_SECRET` - Daraja API consumer secret
- `MPESA_SHORTCODE` - Business short code (initiating account)
- `MPESA_INITIATOR_NAME` - Initiator name for M-Pesa logs
- `MPESA_SECURITY_CREDENTIAL` - Encrypted initiator password
- `MPESA_CALLBACK_URL` - Callback endpoint URL for confirmations
- `KAFKA_BROKERS` - Kafka broker endpoints
- `DATABASE_URL` - PostgreSQL connection string

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m mpesa_b2c.worker
```

## Safaricom B2C Daily Limit

Default: 5,000 transactions/day
Upgrade available: contact Safaricom Business Support for limit increase

## Payout Workflow

1. Finance app submits payroll batch
2. B2C initiates bulk transfer (up to 1,000 recipients per batch)
3. M-Pesa processes transfers (typically 5-10 minutes)
4. Callbacks confirm success/failure for each recipient
5. Reconciliation report generated

## Notes

- Bulk limit: 1,000 recipients per request
- Individual transaction limit: 500,000 KES (M-Pesa limit)
- Failed transfers are logged for manual follow-up
- Confirmations sent via SMS to worker phone numbers
- SLA: 250k recipients <= 4 hours (batch processing)
- Retries are automatic (3 attempts before failure)
