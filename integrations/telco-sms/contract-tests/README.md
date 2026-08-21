# telco-sms/contract-tests

Consumer/provider Pact contracts for telco SMS adapter.

## Purpose

Verifies SMS gateway schema, OAuth2 authentication, error codes, rate-limit behavior, and multi-operator failover logic.

## Running tests

```bash
uv run pytest -vxs tests/ci
```

## What is verified

- SmsRequest schema (phone_number, message, operator_hint)
- SmsResponse schema (sms_id, status, timestamp)
- OAuth2 client-credentials flow per operator
- HTTP 400 (invalid phone), 429 (rate limit), 503 (operator down)
- Rate-limit headers (X-RateLimit-Limit, X-RateLimit-Remaining, Retry-After)
- Failover: if Safaricom returns 503, try Airtel
- Response latency (p99 < 3s per operator)

## Provider verification

To verify against real operator endpoints (QA only):

1. Set SMS_SAFARICOM_CLIENT_ID, SMS_SAFARICOM_CLIENT_SECRET, etc.
2. Run pytest with --pact-verify flag.
3. Check pact.json for operator response mismatches.

## PII in contract tests

Do not commit real phone numbers. Use synthetic test numbers (+254-test-series).
