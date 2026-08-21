# IVR Hook

IVR webhook handler for field support line. Receives IVR call events from the telco IVR platform, creates field support tickets, and routes to on-call engineers.

## Overview

Module: M32 (Mobile Field Support - IVR support line)

Call event processor for election-day field support line. Poll workers, supervisors, and observers call a toll-free number to report incidents. IVR collects information and creates tickets.

## FastAPI Endpoints

- POST `/ivr/callback` - Receive IVR call event from telco platform
- GET `/ivr/menu/{session_id}` - Render IVR menu for session
- GET `/health` - Service health check

## Pydantic Models

- `IVRCallEvent` - Call event (session_id, caller_msisdn: str, dtmf_input: str, station_id: str | None, call_duration_secs: int)
- `FieldSupportTicket` - Support ticket (id, ivr_session_id, priority, description, assigned_to, created_at)

## Environment Variables

- `IVR_GW_SECRET` - IVR gateway authentication secret
- `KAFKA_BROKERS` - Kafka broker endpoints
- `DATABASE_URL` - PostgreSQL connection string
- `FIELD_SUPPORT_API_URL` - Field support ticketing system API

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn ivr_hook.wsgi:application --workers 2
```

## IVR Flow

1. Caller dials toll-free number
2. IVR asks for station code (DTMF input)
3. IVR records incident description (audio or DTMF)
4. Ticket created and assigned to on-call engineer
5. SMS confirmation sent to caller with ticket number

## Notes

- MSISDN (phone number) is logged for follow-up communication
- Audio recordings are encrypted and retained for 30 days
- Priority is auto-assigned based on incident keywords in transcription
- Tickets integrate with field support app for engineer dispatch
