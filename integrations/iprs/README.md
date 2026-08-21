# integrations/iprs

I01 IPRS (Integrated Population Registration System) national-ID validation adapter.

## Purpose

Validates citizen identity against IPRS: fetches name, DOB, gender, phone, postal address for voter registration and BVI (biometric verification integration).

## Endpoint

REST primary, SOAP legacy fallback. URL template: `https://api.iprs.go.ke/v2/{resource}` (exact URL under MoU NDA).

## Authentication

SAML 2.0 bearer assertion + mTLS client certificate. See `adapter/saml.py` for token refresh flow.

## Modules using this adapter

M01 (bvr-device), M02 (bvr-backend), M04 (vprs), M35 (cvr), M36 (dereg-appeal), M53 (voter-id-card).

## Request/response

PersonLookupRequest: id_number (str), lookup_type (national_id | passport).
PersonLookupResponse: full_name, dob (YYYY-MM-DD), gender (M | F | X), phone_number, postal_address, registration_status.

## Sample curl

```bash
curl -E client.pem --key client.key \
  -H "Authorization: Bearer $SAML_TOKEN" \
  https://api.iprs.go.ke/v2/persons/12345678
```

## PII enforcement

Maximum tier. All responses encrypted at rest. No logging of raw payloads. Field-level encryption via eel-pii-rs.

## Rate limits

60 req/min per client cert; burst 10.

## Circuit breaker

Open after 5 consecutive failures; half-open after 5s; probe 2 req per half-open phase.

## Health probe

GET /health -> 200 {"status":"ok"}.

## Chaos toggle

IPRS_CHAOS_ENABLED=true (double only). Injects timeout, schema error, duplicate responses.

## Fallback

Degrade to cached last-known IPRS data (TTL 4h). Surface M02 degraded mode to UI.

## MoU status

Signed Phase 1 (voter reg). Phase 2 (real-time BVI) pending.

## Related

See adapter/, double/, contract-tests/ subdirectories.
