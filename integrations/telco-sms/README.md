# integrations/telco-sms

I08 (SMS leg) Telco SMS Gateway: Safaricom, Airtel Kenya, Telkom Kenya REST APIs.

## Purpose

Sends SMS notifications for voter registration verification, grievance follow-up, and dereg appeal status. Multi-operator failover (Safaricom primary, Airtel secondary, Telkom tertiary).

## Modules using this adapter

M08 (rts-device), M13 (citizen-verify), M23 (grievance), M36 (dereg-appeal).

## Endpoints

- Safaricom: `https://api.safaricom.co.ke/sms/v1/...`
- Airtel: `https://api.airtel.co.ke/sms/v1/...`
- Telkom: `https://api.telkom.co.ke/sms/v1/...`

Exact URLs under operator NDAs.

## Authentication

OAuth2 client-credentials per operator. Shortcode provisioned by each operator (shared credential set).

## PII enforcement

Low tier. Phone number only, masked in logs (e.g., +2547****678).

## Rate limits

100 SMS/sec per shortcode; burst 200. Operator-enforced; adapter respects via token bucket.

## Circuit breaker

Open 30s after 5 failures per operator. Multi-operator failover: if primary exhausted, try secondary, then tertiary.

## Sample curl

```bash
curl -X POST https://api.safaricom.co.ke/sms/v1/send \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{"phone":"+254712345678","message":"Your OTP: 123456"}'
```

## Health probe

GET /sms/v1/status -> {"status":"ok"}.

## Fallback

Queue SMS for retry. Multi-operator failover. SMS remains queued if all operators exhausted; retry on circuit recovery.

## Chaos toggle

TELCO_SMS_CHAOS=true (double only). Injects timeout, delivery failure, duplicate frames.

## MoU status

Operator agreements in place (NDA). Shortcodes provisioned per phase.
