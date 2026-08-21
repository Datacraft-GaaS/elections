# WhatsApp Test Double

Mock REST server simulating Meta WhatsApp Business Cloud API for integration testing. Accepts WhatsApp API payloads, returns canned responses, supports webhook callback simulation, and enables chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without Meta dependency. Validates adapter message formatting (text, template, interactive), webhook signature verification, delivery status callbacks, and error handling. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_whatsapp
```

Server listens on http://localhost:8006 by default.

To override port:
```bash
PORT=9006 uv run python -m eel_double_whatsapp
```

Environment variable: WA_DOUBLE_PORT (default: 8006).

## Chaos Knobs

Control failure modes via environment variables:

WA_CHAOS_ENABLED: "true" to activate all chaos (default: false)
WA_CHAOS_MESSAGE_FAILURE_RATE: Fraction [0.0-1.0] of message sends returning 400 error (default: 0.0)
WA_CHAOS_RATE_LIMIT_RATE: Fraction [0.0-1.0] of requests returning 429 rate limit (default: 0.0)
WA_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
WA_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
WA_CHAOS_ENABLED=true WA_CHAOS_MESSAGE_FAILURE_RATE=0.1 uv run python -m eel_double_whatsapp
```

## Canned Responses

**Send Message** (POST /{phone_id}/messages):
Returns `{"messages": [{"id": "wamid.123456"}]}`.
On chaos: returns `{"error": {"message": "Invalid parameter", "code": 400}}` with 400 status or `{"error": {"message": "Rate limit exceeded", "code": 429}}` with 429 status.

**Get Me (Token Verification)** (GET /me?access_token=...):
Returns `{"object": "user", "name": "WhatsApp System User", "id": "123456"}` if token is "test-token-xxx".
On invalid token: returns 403 `{"error": "Invalid access token"}`.

**Webhook Initial Verification** (GET /webhook):
Validates hub.challenge parameter and returns it if hub.verify_token matches WA_VERIFY_TOKEN.

## Webhook Simulation

Double does not auto-send webhooks; tests must explicitly call to receive simulated delivery status:

```bash
curl -X POST http://localhost:8006/webhook-simulate \
  -H "Content-Type: application/json" \
  -d '{"event_type":"message_status","message_id":"wamid.123","status":"delivered","recipient":"254700000000"}'
```

This allows tests to control timing of delivery status callbacks.

## Endpoints Exposed

POST /{phone_id}/messages - Send message
GET /me - Token verification
GET /webhook - Webhook URL registration (Meta verification)
POST /webhook-simulate - Simulate webhook callback (test only, not in production)
