# IFMIS Integration

IFMIS (Integrated Financial Management Information System) integration worker. Submits payment vouchers to Kenya's national IFMIS system, polls payment status, reconciles with internal budget records, and publishes `eel.finance.payment-status` Kafka events for downstream workflows.

## Overview

Module: M54 (Budget Expenditure)

Finance integration service that connects election budget/spending to Kenya's government accounting system (IFMIS). Enables end-to-end financial tracking from election budget to government settlement.

## FastAPI Endpoints

- POST `/ifmis/voucher` - Submit payment voucher to IFMIS
- GET `/ifmis/voucher/{id}/status` - Poll payment status
- GET `/health` - Service health check

## Pydantic Models

- `VoucherSubmission` - Voucher details (voucher_id: str, amount: float, payee_code: str, budget_line: str)
- `PaymentStatus` - Payment status (voucher_id, ifmis_ref, status: str, settled_at: str | None)

## Environment Variables

- `IFMIS_ADAPTER_URL` - IFMIS API gateway endpoint
- `KAFKA_BROKERS` - Kafka broker endpoints
- `DATABASE_URL` - PostgreSQL connection string
- `IFMIS_CLIENT_ID` - IFMIS OAuth client ID
- `IFMIS_CLIENT_SECRET` - IFMIS OAuth client secret

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m ifmis.worker
```

## Payment Workflow

1. Finance app submits voucher -> calls `/ifmis/voucher`
2. IFMIS validation and approval process (typically 5-7 business days)
3. Payment settlement and confirmation
4. Status published to Kafka `eel.finance.payment-status` event
5. Finance portal updates with payment confirmation

## Notes

- Voucher submission is async (returns immediately, processes in background)
- Status polling recommended every 24 hours (high volume reduces latency)
- IFMIS maintains master copy of record (authoritative source)
- Failed vouchers are logged and require resubmission
- Integration requires valid GOK (Government of Kenya) credentials
