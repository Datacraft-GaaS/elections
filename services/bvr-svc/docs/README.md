# bvr-svc

Biometric Voter Registration service. Authoritative store for the Kenya voter
register.

## Crates

| crate | role |
|---|---|
| bvr-domain | Voter aggregate, domain events, canary, snapshot types |
| bvr-batch  | Batch ingest (1k rows/sec), streamed register export, SAR workflow |
| bvr-api    | axum REST API and gRPC server; all OpenAPI paths bound |

## Django admin

`django/bvr-admin` - back-office for Returning Officers.

Features: voter search (PII masked, purpose logged), register-freeze workflow,
adjudication queue, bulk-export dual approval, audit trail, SAR handling.

## Running locally

```
cargo build --release -p bvr-api -p bvr-batch -p bvr-domain
cargo test -p bvr-domain -p bvr-batch -p bvr-api
cd services/bvr-svc/django/bvr-admin && python manage.py check
```

## Load test

```
cd services/bvr-svc/loadtest
k6 run enrolment_loadtest.js
# Full target: TARGET_RPS=1000 k6 run enrolment_loadtest.js
```
