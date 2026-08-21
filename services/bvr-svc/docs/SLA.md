# BVR Service SLA

| endpoint | p50 | p95 | p99 |
|---|---|---|---|
| POST /enrolments | 200ms | 500ms | 1000ms |
| GET /voters/{id} | 50ms | 150ms | 300ms |
| PATCH /voters/{id}/status | 100ms | 250ms | 500ms |
| POST /transfers | 100ms | 250ms | 500ms |
| GET /registers/{scope} | streaming | streaming | streaming |
| POST /sar | 100ms | 200ms | 400ms |
| GET /merkle-root/latest | 20ms | 50ms | 100ms |

Batch ingest target: 1 000 rows/sec sustained (ADR-0018).
Register export: 30M voters streamed in under 4 hours, checkpoint-resumable.

Availability: 99.99% (election-critical class).
