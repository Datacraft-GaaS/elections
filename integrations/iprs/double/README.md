# iprs/double

Canned-response mock for IPRS. Used in CI and dev environments.

## Running the mock

```bash
python -m eel_double_iprs --port 8081
```

Listens on localhost:8081 by default. Responds to PersonLookup requests with canned data.

## Chaos knobs

Environment variables for injecting faults:

- IPRS_DOUBLE_CHAOS: none | timeout | error | slow (default: none)
  - timeout: return 504 after 15s (tests timeouts)
  - error: return 503 with retry-after
  - slow: add 5s latency to response

## Canned responses

Default response for ID "12345678":
- full_name: "Jane Doe"
- dob: "1985-03-15"
- gender: "F"
- phone_number: "+254712345678"
- postal_address: "123 Main St, Nairobi"
- registration_status: "active"

Lookup ID "99999999" triggers error response.

## mTLS setup

Double accepts mTLS client certs but does not validate them (dev only). Pass --cert and --key flags to enable TLS.
