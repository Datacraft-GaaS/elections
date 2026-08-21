# WhatsApp Bot

WhatsApp bot handler for citizen self-service via Meta Cloud API. Handles voter registration verification, polling station lookup, grievance submission, and voter education content delivery via WhatsApp.

## Overview

Module: M13 (Citizen Verify WA), M23 (Grievance WA), M37 (Voter Education WA)

WhatsApp bot providing menu-driven access to election services. Reaches citizens with smartphones and data plans (higher reach than USSD). Integrates with Meta Cloud API for message delivery.

## FastAPI Endpoints

- POST `/wa/webhook` - Receive WhatsApp message events from Meta
- GET `/wa/webhook` - Verification callback (Meta webhook validation)
- GET `/health` - Service health check

## Pydantic Models

- `WAMessage` - Incoming message (from_msisdn: str, message_type: str, text: str | None, interactive: dict | None, image_url: str | None)
- `WABotResponse` - Outgoing response (to: str, template: str | None, text: str | None, interactive_buttons: list | None)

## Environment Variables

- `WA_VERIFY_TOKEN` - Meta webhook verification token
- `WA_ACCESS_TOKEN` - Meta Cloud API access token (bearer token for API calls)
- `WA_PHONE_NUMBER_ID` - Meta phone number ID for sending messages
- `BVR_API_URL` - Voter Registration API
- `GRIEVANCE_API_URL` - Grievance intake API
- `NLLB_ADAPTER_URL` - Translation service (automatic response translation)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
gunicorn wa_bot.wsgi:application --workers 4
```

## Message Types

- **Text Messages**: Voter sends text -> bot replies with info or options
- **Interactive Buttons**: User taps button -> bot processes response
- **Image Upload**: User sends photo (ID card) -> bot processes for verification
- **Template Messages**: Bot sends pre-approved templates (requires Meta approval)

## Features

1. **Voter Verification**: Confirm registration status and polling station
2. **Polling Station Finder**: Search by constituency or ward
3. **Grievance Submission**: Text grievance -> creates case with auto-reply
4. **Voter Education**: Receive fact-check articles and procedural guides

## Supported Languages

Automatic translation to 11 languages via NLLB model (Swahili, English, regional languages).

## Notes

- WhatsApp template approval takes 6-8 weeks with Meta (plan early)
- Message throughput: ~10,000 messages/hour (upgrade available)
- Conversations are rate-limited to 1 message per user per 24 hours (bulk messaging)
- All messages logged for audit; no conversation data retained beyond 48 hours
