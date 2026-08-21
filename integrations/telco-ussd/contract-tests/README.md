# telco-ussd/contract-tests

Consumer/provider Pact contracts for telco USSD adapter.

## Purpose

Verifies USSD gateway schema, session lifecycle, error codes, and menu state machine behavior.

## Running tests

```bash
uv run pytest -vxs tests/ci
```

## What is verified

- UssdRequest schema (msisdn, session_id, user_input, service_code)
- UssdResponse schema (message, continue_session)
- Session state transitions (initial prompt -> user input -> next prompt -> END)
- HTTP 400 (invalid MSISDN), 504 (timeout), 503 (operator down)
- Session timeout: if idle > SESSION_TIMEOUT_SECS, return END
- Menu state machine: all paths terminate within 5 prompts
- Response latency (p99 < 10s per prompt)

## Provider verification

To verify against real USSD gateways (QA only):

1. Set USSD_SAFARICOM_API_KEY, USSD_AIRTEL_API_KEY.
2. Run pytest with --pact-verify flag.
3. Check pact.json for gateway response mismatches.

## PII in contract tests

Do not commit real MSISDNs. Use synthetic test numbers (+254-test-series).
