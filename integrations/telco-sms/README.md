# integrations/telco-sms

I08 SMS gateway adapter: Africa's Talking REST + Safaricom SMPP.

## Wire protocol

- Africa's Talking: `POST https://api.africastalking.com/version1/messaging`
- Safaricom SMPP: SMPP 3.4 over TCP (smpplib)

Internal endpoints:
- `POST /sms/send`
- `POST /sms/receive` (inbound webhook)

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 |
| open duration | 30 s |
| retry budget | 3, jitter 1-5 s |

## Double knobs

| Env var | Default |
|---|---|
| SMS_LATENCY_MS | 0 |
| SMS_ERROR_RATE | 0 |

## SLA

AT p99 < 3 s. Adapter hard timeout 15 s.
