# integrations/legacy-abis

I05 Legacy ABIS (incumbent AFIS vendor SOAP API) biometric adapter.

## Purpose

Migration bridge only: wraps legacy ABIS vendor SOAP/gRPC endpoints for 1:1 and 1:N biometric matching. Used during Wave 2-3 transition; sun-setting post-Wave 3. All new biometric processing uses eel-biometric-svc (Rust).

## Modules using this adapter

M03 (abis-svc migration bridge only).

## Protocol

SOAP 1.1 over HTTPS (legacy) or gRPC (optional).

## Authentication

WS-Security UsernameToken + mTLS client certificate.

## PII enforcement

Maximum tier. Fingerprint templates, biographic data, match confidence scores. Field-level encryption for storage. No raw payload logging.

## Rate limits

10 req/min (legacy system limit). Conservative to avoid blocking vendor AFIS.

## Circuit breaker

Open 60s after 1 failure (fragile legacy system). Very conservative: single error triggers open.

## Fallback

Queue adjudication case for manual review. Route to eel-biometric-svc for new-format templates if available.

## Endpoints

- POST /abis/match/1n - 1:N biometric search
- POST /abis/match/11 - 1:1 biometric verification

## Sunset

Planned removal end of Wave 3 (Q3 2027). See ADR-0022 (legacy system migration).

## Related

See adapter/, double/, contract-tests/ subdirectories.
