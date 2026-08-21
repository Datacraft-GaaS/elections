# integrations/telco-ussd

I08 (USSD leg) Telco USSD Gateway: Safaricom, Airtel Kenya USSD endpoints.

## Purpose

Handles unstructured supplementary service data (USSD) for citizen verification and dereg appeal flows on feature phones. USSD tree navigation: dial *400#, respond to prompts, receive verification status.

## Modules using this adapter

M13 (citizen-verify, *400# flow), M36 (dereg-appeal USSD), M23 (grievance).

## Protocol

TCP/USSD-GW proprietary binary protocol (vendor-specific) plus REST management plane for session setup.

Endpoint: USSD GW host/port under NDA.

## Authentication

Per-USSD-session key (ephemeral) + operator credentials (OAuth2 on management plane).

## PII enforcement

Low tier. MSISDN only (caller phone number). Masked in logs.

## Rate limits

Session concurrency limit: 10,000 concurrent sessions per operator.

## Circuit breaker

Open 10s after 3 failures. Fallback to SMS for timeout sessions.

## Sample flow

1. Mobile user dials *400#
2. USSD GW routes to adapter's USSD handler
3. Adapter returns prompt menu
4. User responds; adapter processes, returns next prompt
5. Final response: "Verification complete, status: APPROVED"

## Fallback

If USSD session times out (>30s), send SMS with verification link instead.

## Chaos toggle

USSD_CHAOS_ENABLED=true (double only). Injects USSD timeout, session drop, protocol error.

## MoU status

Same operator agreements as telco-sms.
