# WhatsApp Adapter

Production adapter for Meta WhatsApp Business Cloud API. Handles message sending (text, templates, interactive), webhook reception for incoming messages and delivery status, and webhook signature verification.

## Purpose

Bridges eel core system (M13, M23, M37, M68 modules) to Meta WhatsApp Business API. Manages Bearer token, message formatting, template caching, webhook HMAC validation, and fallback channel coordination.

## Layout

- `service.py`: Main adapter service. Handles message sending (text, template, interactive), webhook reception, delivery status tracking, template cache refresh.
- `views.py`: Pydantic models for WhatsApp API payloads. Models for SendMessage, WebhookCallback operations. Message type and status enums.
- `webhook.py`: Webhook signature verification (HMAC-SHA256), request parsing, idempotency handling.
- `templates.py`: Template registry and cache. Handles template params validation and substitution.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**WhatsAppService** (async):
- `send_text_message(to, body)`: Send text message, returns MessageID.
- `send_template_message(to, template_name, params)`: Send pre-approved template, returns MessageID.
- `send_interactive_message(to, interactive_payload)`: Send list/button interactive, returns MessageID.
- `verify_webhook_signature(request_body, signature)`: Validate incoming webhook HMAC.
- `process_webhook_callback(webhook_data)`: Parse delivery status or incoming message.

**Models** (in views.py):
- `SendMessageRequest`: Phone number, message type, body/template/interactive payload.
- `SendMessageResponse`: Message ID list, status indicators.
- `WebhookMessage`: Incoming message data (phone, body, type).
- `WebhookStatus`: Delivery status (delivered, read, failed), timestamp, phone.
- `TemplateComponent`: Template header, body, footer, buttons.

## Environment Variables

WA_PHONE_NUMBER_ID: Phone number ID from Meta dashboard
WA_ACCESS_TOKEN: System User Bearer token
WA_VERIFY_TOKEN: Webhook verify token (for initial Meta verification)
WA_WEBHOOK_SECRET: Secret for HMAC-SHA256 signature validation
WA_BSP_URL: Meta Graph API base URL
WA_TIMEOUT_SEC: Request timeout in seconds (default: 30)
WA_CHAOS: "true" enables chaos injection (test only)

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
