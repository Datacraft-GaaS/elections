# biometric-edge-svc SLA

## Availability

- Target: 99.9% per edge site during election operation window.
- 5 regional edge sites; DR handled by central primary/mirror.

## Latency (per endpoint)

| Endpoint | p50 (ms) | p95 (ms) | p99 (ms) |
|---|---|---|---|
| POST /enrolment/upload | 100 | 300 | 1000 |
| POST /attestation/verify | 50 | 200 | 500 |
| GET /register-extract/{code} | 100 | 400 | 1000 |
| POST /telemetry (heartbeat) | 20 | 100 | 300 |
| POST /distress | 20 | 100 | 300 |
| POST /ota/check | 50 | 200 | 500 |
| POST /mrz/parse | 30 | 150 | 400 |
| POST /sync/pull | 100 | 500 | 1500 |

## Throughput

- Heartbeat: 46,000 kits at 1 heartbeat/minute = 767 req/s peak.
- Enrolment: >=12 enrolments/hr per BVR kit, ~22,000 BVR kits = 73 req/s peak.

## Recovery

- Kit offline queue retains 7 days of data; sync resumes on connectivity restore.
- edge-api is stateless; horizontal scaling is via replicas (3 per edge site).
