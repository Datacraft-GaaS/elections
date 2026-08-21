# telco-ussd/adapter

Python package wrapping Safaricom and Airtel USSD gateways.

## Layout

- `src/eel_adapter_telco_ussd/__init__.py` - public exports
- `src/eel_adapter_telco_ussd/service.py` - UssdAdapter, session management
- `src/eel_adapter_telco_ussd/views.py` - Pydantic models (UssdRequest, UssdResponse)
- `src/eel_adapter_telco_ussd/menu.py` - USSD menu state machine

## Key classes

UssdAdapter: manages USSD session lifecycle, routes to menu handlers.

UssdRequest: msisdn (str), session_id (str), user_input (str), service_code (str).

UssdResponse: message (str), continue_session (bool).

## Environment variables

- USSD_SAFARICOM_API_KEY
- USSD_AIRTEL_API_KEY
- USSD_SESSION_TIMEOUT_SECS: default 300 (5 minutes)
- USSD_MAX_CONCURRENT_SESSIONS: default 10000

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Menu state machine

USSD dialogs modeled as state machine. Each state returns prompt text + next state. Timeout or user abort triggers fallback to SMS.
