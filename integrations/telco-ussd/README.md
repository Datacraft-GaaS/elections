# integrations/telco-ussd

USSD adapter for Africa's Talking with a menu-engine state machine.

Handles: M13 verifier, M23 grievance, M35 CVR family notifications.

## Wire protocol

AT USSD callback: `POST /ussd/callback` (form-encoded: sessionId, serviceCode,
phoneNumber, text, networkCode). Response: plain text starting with `CON`
(continues) or `END` (terminates).

## Circuit breaker

Stateless - no upstream calls in base flow. Back-end service calls use
eel-resilience-py circuit breaker with 5-failure threshold, 30 s open.

## Double knobs

| Env var | Default |
|---|---|
| USSD_LATENCY_MS | 0 |
