# telco-ussd/double

Canned-response mock for telco USSD gateways. Simulates Safaricom and Airtel USSD flows.

## Running the mock

```bash
python -m eel_double_telco_ussd --port 8083
```

Listens on localhost:8083. Responds to UssdRequest with canned menu prompts.

## Chaos knobs

Environment variables for fault injection:

- TELCO_USSD_DOUBLE_CHAOS: none | timeout | error | drop (default: none)
  - timeout: return 504 after 30s
  - error: return 503 Service Unavailable
  - drop: simulate USSD session drop (mid-menu)

- USSD_DOUBLE_OPERATOR: safaricom | airtel (default: all)

- USSD_DOUBLE_SESSION_TIMEOUT_SECS: default 300

## Canned menu

```
Welcome to eel voter verification
1. Enter via MSISDN
2. Enter via National ID
3. Exit
```

Selecting 1: prompts for phone number, validates, returns "Verification complete".

Selecting 3: returns "END" to terminate session.

## Multi-operator simulation

Each operator endpoint responds independently. Session state tracked per msisdn + session_id.
