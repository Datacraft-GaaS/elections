# USSD Menu

USSD menu handler for *400# citizen self-service channel. Drives session state machine for voter registration verification, polling station lookup, grievance submission, and voter deregistration appeals.

## Overview

Module: M13 (Citizen Verify USSD), M36 (Voter Deregistration Appeal USSD)

USSD (Unstructured Supplementary Service Data) gateway for voters without data access. Provides menu-driven access to core services: verify registration, find polling station, submit grievances, appeal deregistration.

## FastAPI Endpoints

- POST `/ussd/session` - Handle USSD session request
- GET `/health` - Service health check

## Pydantic Models

- `USSDRequest` - Session request (sessionId, serviceCode, phoneNumber, text)
- `USSDResponse` - Session response (message: str, continueSession: bool)

## Environment Variables

- `USSD_GW_SECRET` - USSD gateway authentication secret
- `DATABASE_URL` - PostgreSQL connection string
- `BVR_API_URL` - Voter Registration API (verify registration)
- `GRIEVANCE_API_URL` - Grievance intake API
- `QUEUE_API_URL` - Queue transparency API (polling station info)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn ussd_menu.wsgi:application --workers 4
```

## USSD Menu Flow

1. **Verify Registration**: Enter name/ID -> returns voter status + polling station
2. **Find Polling Station**: Enter constituency -> returns nearby polling station locations
3. **Submit Grievance**: Voice/text grievance -> creates case and returns reference number
4. **Deregistration Appeal**: Review removal reason -> submit appeal -> returns appeal ID

## Session Management

- Sessions are stateless (each request includes full state in `text` field)
- Session timeout: 30 minutes of inactivity
- No session data persisted (USSD gateway manages session)
- Multi-language support (Swahili, English, 5+ regional languages)

## Notes

- USSD is free (funded by government subsidy to telcos)
- Typical response time: 2-3 seconds
- Session text can exceed one screen; pagination provided
- Phone numbers are used only for communication (not linked to voter record)
