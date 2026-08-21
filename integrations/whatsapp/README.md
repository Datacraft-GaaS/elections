# WhatsApp Business Cloud API

Meta WhatsApp Business Cloud integration for citizen engagement via messaging. Handles message sending (text, templates, interactive), webhook reception for incoming messages and status callbacks, and message delivery verification.

## MoU Status

Meta Business Service Provider (BSP) agreement with NDA. Phone number and WABA (WhatsApp Business Account) registered and verified. Monthly billing through Meta account. Template approval cycle 6-8 weeks.

## Endpoint

Production: https://graph.facebook.com/v20.0/{phone-number-id}/messages
Webhook: Your public HTTPS URL (configured in Meta dashboard)

## Protocol and Authentication

REST over HTTPS with JSON payloads. Bearer token authentication via Meta System User token issued to WABA account.

**Token Acquisition:**
Meta Business Account dashboard -> Settings -> System Users -> create system user with all_messaging_app permissions -> generate access token

Token TTL: typically 60+ days; automatic refresh via refresh token or re-issue via Meta dashboard.

## Request and Response Shape

**Send Message (Text):**
Request: `POST {messaging_product: "whatsapp", to: "254700000000", type: "text", text: {body: "Hello"}}`
Response: `{messages: [{id: "wamid.xxx"}]}`

**Send Message (Template):**
Request: `POST {messaging_product: "whatsapp", to: "254700000000", type: "template", template: {name: "election_notice", language: {code: "en"}, components: [{type: "body", parameters: [{type: "text", text: "2027"}]}]}}`
Response: `{messages: [{id: "wamid.xxx"}]}`

**Send Message (Interactive - List/Button):**
Request: `POST {messaging_product: "whatsapp", to: "254700000000", type: "interactive", interactive: {type: "list", header: {...}, body: {...}, action: {sections: [...]}}}`
Response: `{messages: [{id: "wamid.xxx"}]}`

**Webhook Callback (Message Received):**
Incoming: `{entry: [{changes: [{value: {messages: [{id: "wamid.xxx", type: "text", text: {body: "Received text"}}]}}]}]}`

**Webhook Callback (Delivery Status):**
Incoming: `{entry: [{changes: [{value: {statuses: [{id: "wamid.xxx", status: "delivered|read|failed", timestamp: "...", recipient_id: "254700000000"}]}}]}]}`

## Sample Curl

```bash
# Text message
curl -H "Authorization: Bearer $WA_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"messaging_product":"whatsapp","to":"254700000000","type":"text","text":{"body":"Hello from eel"}}' \
  https://graph.facebook.com/v20.0/$PHONE_ID/messages

# Template message (pre-approved)
curl -H "Authorization: Bearer $WA_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"messaging_product":"whatsapp","to":"254700000000","type":"template","template":{"name":"election_notice","language":{"code":"en"}}}' \
  https://graph.facebook.com/v20.0/$PHONE_ID/messages
```

## Health Probe

GET https://graph.facebook.com/v20.0/me?access_token=... returns 200 if token is valid and account is active.

## Rate Limits

Unverified WABA: 250 messages in 24-hour rolling window per unique phone number (conversation model).
Verified WABA: higher tier with negotiated limits. Check Meta dashboard for your account limits.

## PII Enforcement

MSISDN (phone number) is tier-A restricted. Message body and template parameters are tier-B internal. Logs mask phone numbers showing only last 4 digits. Message bodies logged only as metadata (sent/delivered status), not full text. No caching of message history on-disk except transient state (in-flight message IDs for delivery tracking).

## Chaos Toggle

WA_CHAOS=true enables chaos injection. Causes random message delivery failures and rate limit responses (test only).

## Fallback Behavior

On adapter unavailability or rate limit hit, messages queued locally. Fallback channel is telco-sms (SMS via Safaricom). Operator dashboard shows queued messages and can retry or switch channels.

## Circuit Breaker Policy

Threshold: 5 consecutive failures
Open duration: 30 seconds
Rationale: WhatsApp outages are often temporary; aggressive breaker allows rapid recovery
Half-open: permits 1 probe request

## Template Approval

New message templates submitted to Meta for approval (6-8 week review cycle). Once approved, templates are cached in adapter and can be reused without re-approval. Template rejection reasons logged for operator investigation.

## Environment Variables

WA_PHONE_NUMBER_ID: Phone number ID (from Meta dashboard)
WA_ACCESS_TOKEN: System User access token (Bearer token)
WA_VERIFY_TOKEN: Token for webhook signature verification
WA_WEBHOOK_SECRET: Secret key for webhook HMAC validation
WA_BSP_URL: Base URL for Meta Graph API (default: https://graph.facebook.com/v20.0/)
WA_TIMEOUT_SEC: Request timeout in seconds (default: 30)
WA_CHAOS: "true" enables chaos (test only)

## Related Modules

M13 (citizen-verify WA channel), M23 (grievance WA channel), M37 (voter education), M68 (referendum explainer)

## Runbook

docs/runbooks/whatsapp-template-management.md (template approval workflow, fallback SMS, message queue management)

