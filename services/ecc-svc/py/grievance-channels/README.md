# Grievance Channels

Grievance intake worker for processing incoming grievances from multiple channels (WhatsApp, USSD, SMS, kiosk). Deduplicates, validates, and publishes grievance events to Kafka. Supports whistleblower mode with encrypted identity protection.

## Overview

Module: M23 (Grievance Channels)

Multi-channel intake system for election-related grievances. Routes complaints to ECC (Election Command Centre) for resolution. Supports confidential reporting with ephemeral encrypted profiles for whistleblowers.

## FastAPI Endpoints

- POST `/grievance/wa-webhook` - WhatsApp webhook callback
- POST `/grievance/ussd-callback` - USSD session callback
- POST `/grievance/sms` - SMS submission endpoint
- POST `/grievance/kiosk` - Kiosk POST endpoint (at polling stations)
- GET `/health` - Service health check

## Pydantic Models

- `GrievanceCase` - Grievance record (id, channel: str, description, evidence: list[str], status, created_at)
- `WhistleblowerProfile` - Encrypted identity (encrypted_identity: bytes, session_token: str, expiry_ts)

## Environment Variables

- `KAFKA_BROKERS` - Kafka broker endpoints
- `DATABASE_URL` - PostgreSQL connection string
- `WA_VERIFY_TOKEN` - WhatsApp webhook verification token
- `USSD_GW_SECRET` - USSD gateway authentication secret
- `ECC_API_URL` - Election Command Centre API endpoint
- `WHISTLEBLOWER_ENCRYPTION_KEY` - AES-256 key for identity encryption

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m grievance_channels.worker
```

## Whistleblower Mode

- Identity encrypted and stored separately from grievance text
- Session tokens expire after 7 days
- No IP/phone number stored with grievance content
- Requires security review before identity disclosure

## Notes

- Duplicates detected via content hash (same channel within 10 min window)
- Evidence attachments (photos/video) stored in secure S3 bucket
- Grievances published to Kafka for ECC ingestion and case management
- All interactions are logged for transparency (with identity protection)
