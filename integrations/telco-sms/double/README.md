# telco-sms/double

Canned-response mock for telco SMS gateways. Simulates Safaricom, Airtel, Telkom APIs.

## Running the mock

```bash
python -m eel_double_telco_sms --port 8082
```

Listens on localhost:8082. Responds to SmsRequest with synthetic sms_id and status.

## Chaos knobs

Environment variables for fault injection:

- TELCO_SMS_DOUBLE_CHAOS: none | timeout | error | duplicate (default: none)
  - timeout: return 504 after 15s
  - error: return 503 with random operator
  - duplicate: return success twice on single request

- SMS_DOUBLE_OPERATOR: safaricom | airtel | telkom (default: all)

- SMS_DOUBLE_DELIVER_RATE: 0.0-1.0 (default: 1.0), fraction of SMS marked delivered

## Canned responses

All requests return: {"sms_id": "uuid-v7", "status": "sent", "timestamp": "2026-08-21T..."}

Phone numbers starting with +255 (Tanzania) return 400 Bad Request (invalid).

## Multi-operator simulation

Each operator endpoint responds independently. Failover logic in main adapter applies.
