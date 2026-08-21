# boundary-svc Threat Model

## Trust boundary

- Public HTTP endpoints: no auth. Data class C only. Rate-limited by edge layer.
- Admin HTTP endpoints: JWT (Keycloak) + SpiceDB `boundary.admin` permission.
- Internal gRPC (Geofence.Check): mTLS, SPIFFE identity enforced at transport.

## Key threats

| Threat | Mitigation |
|--------|-----------|
| Forged geofence polygon | `signature` field enforced non-empty at DB level (`CHECK (length(signature) > 0)`) and at domain layer (`validate_signature`). HSM countersign required for EC-OPS-06 flood/curfew relocation. |
| Boundary data tampering | Response headers `X-Eel-Cosign-Sig` + `X-Eel-Merkle-Proof` allow independent verification. Consumers use verifier-rs. |
| Shapefile injection | Import requires admin JWT + SpiceDB. Hash verified before upsert. |
| Privilege escalation | SpiceDB policy enforced per operation. JWT validated against eel.local JWKS. |
| Denial of service | Rate limiting at edge. Geofence.Check is O(n) CPU-only with no DB round-trip. |
| Data exfiltration via bulk endpoint | `/api/v1/boundaries/bulk.geojson.gz` carries only class-C public geometry. No voter records. |

## Data classification

All boundary geometry: class C (public). No PII stored or served by this service.
Submission `payload_ref` points to an object in MinIO; the payload itself is not stored in this service's DB.

## Residual risks

- HSM signer interface (WP-2.2) is stubbed; production deployment must wire the real PKCS#11 call.
- Fairness metrics use proxy VRA-analog; true VRA analysis requires KNBS census microdata integration (deferred).
