# ecourt/double

Canned-response mock for eCourt. Used in CI and dev environments.

## Purpose

Provides predictable responses for eCourt REST API without hitting live upstream. Includes chaos injection for resilience testing and circuit breaker validation.

## Running the mock

```bash
python -m eel_double_ecourt --port 8888
```

Listens on localhost:8888 by default. Set ECOURT_DOUBLE_PORT to override.

```bash
ECOURT_DOUBLE_PORT=9999 python -m eel_double_ecourt
```

## Chaos knobs

Environment variables for injecting faults:

- ECOURT_DOUBLE_CHAOS: none | timeout | error | slow (default: none)
  - timeout: return 504 after 15s (tests timeouts)
  - error: return 503 with retry-after header
  - slow: add 5s latency to response

```bash
ECOURT_DOUBLE_CHAOS=timeout python -m eel_double_ecourt --port 8888
```

## Canned responses

Default response for petition with grounds="irregularity":
- caseNo: "ECR-2025-001"
- filingDate: "2025-08-21T10:30:00Z"
- estimatedHearing: "2025-09-15"
- status: "filed"
- docketURL: "https://ecourt.go.ke/cases/ECR-2025-001"

Request with grounds="fraud" triggers accelerated hearing (5 days). Petition with empty briefStatement returns 400 Bad Request.

## mTLS setup

Double does not validate mTLS client certs (dev only). Pass --cert and --key flags to enable TLS socket listening on 0.0.0.0:8888.
