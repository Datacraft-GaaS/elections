# telco-sms/adapter

Python package wrapping Safaricom, Airtel, Telkom SMS gateway APIs.

## Layout

- `src/eel_adapter_telco_sms/__init__.py` - public exports
- `src/eel_adapter_telco_sms/service.py` - SmsAdapter, multi-operator failover logic
- `src/eel_adapter_telco_sms/views.py` - Pydantic models (SmsRequest, SmsResponse)
- `src/eel_adapter_telco_sms/oauth.py` - per-operator OAuth2 token management

## Key classes

SmsAdapter: wraps three operators, handles OAuth2, failover logic, rate limiting.

SmsRequest: phone_number (str), message (str), operator_hint (optional).

SmsResponse: sms_id, status (sent | queued | failed), timestamp.

## Environment variables

- SMS_SAFARICOM_CLIENT_ID, SMS_SAFARICOM_CLIENT_SECRET
- SMS_AIRTEL_CLIENT_ID, SMS_AIRTEL_CLIENT_SECRET
- SMS_TELKOM_CLIENT_ID, SMS_TELKOM_CLIENT_SECRET
- SMS_RATE_LIMIT_PER_SEC: default 100
- SMS_FAILOVER_ENABLED: default true

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Multi-operator failover

Tries operators in order: Safaricom -> Airtel -> Telkom. Respects per-operator circuit breaker state.
