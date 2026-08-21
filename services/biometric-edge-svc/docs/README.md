# biometric-edge-svc

Edge proxy and kit management service for biometric voter registration (M01) and identification (M05) kits (WP-3.8).

## Components

| Component | Language | Port | Purpose |
|---|---|---|---|
| crates/edge-api | Rust / axum | 8090 | HTTP gateway; binds all OpenAPI paths; proxies to bvr-svc and bvi-svc |
| crates/kit-attest | Rust | library | In-process attestation verification (Play Integrity, App Attest, TPM, legacy) |
| apps/bvr-kit | Flutter | — | Enrolment kit app (>=12 enrolments/hr) |
| apps/bvi-kit | Flutter | — | Identification kit app (offline 7-day queue, boot <=90 s) |
| apps/legacy-android | Flutter | — | Android 8+ parity shim; see PARITY.md for feature matrix |

## API paths

All paths require SPIFFE/SPIRE SVID mTLS. See `packages/openapi-contracts/biometric-edge/openapi.yaml`.

| Path | Method | Handler |
|---|---|---|
| /api/v1/biometric-edge/enrolments | POST | enrolment upload |
| /api/v1/biometric-edge/identifications | POST | BVI identification |
| /api/v1/biometric-edge/config | GET | config bundle download |
| /api/v1/biometric-edge/attestation | POST | kit attestation verify |
| /api/v1/biometric-edge/heartbeat | POST | kit heartbeat |
| /api/v1/edge/enrolment/upload | POST | alias for enrolments |
| /api/v1/edge/mrz/parse | POST | in-process MRZ parse |
| /api/v1/edge/attestation/verify | POST | alias for attestation |
| /api/v1/edge/register-extract/{code} | GET | proxied bvi-svc config |
| /api/v1/edge/sync/pull | POST | delta-sync pull |
| /api/v1/edge/sync/ack | POST | sync batch ack |
| /api/v1/edge/ota/check | POST | OTA manifest check |
| /api/v1/edge/telemetry | POST | alias for heartbeat |
| /api/v1/edge/distress | POST | silent duress alert (EC-IDN-08) |

## Build

```
cd services/biometric-edge-svc
cargo build --release         # builds edge-api + kit-attest
cargo test                    # runs all unit tests
```

Flutter apps require Flutter >= 3.19 on the PATH:

```
cd apps/bvr-kit && flutter test
cd apps/bvi-kit && flutter test
cd apps/legacy-android && flutter test
```

## Load test

```
k6 run --vus 460 --duration 60s loadtest/edge_k6.js
```
