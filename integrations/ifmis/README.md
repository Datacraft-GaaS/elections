# IFMIS Budget and Expenditure API

Integrated Financial Management Information System API for budget queries, voucher creation, and payment status checks. Supports budget allocation lookups, expenditure posting, and payment reconciliation for election operations.

## MoU Status

National Treasury MoU signed. API access via OAuth2 credentials issued to election commission agency account. Annual renewal required.

## Endpoint

Production: https://ifmis.go.ke/api/v1/
Sandbox: https://ifmis-dev.go.ke/api/v1/

## Protocol and Authentication

REST over HTTPS with JSON payloads. OAuth2 client-credentials flow for token generation via National Treasury IdP.

**Token Endpoint:**
https://auth.treasury.go.ke/oauth/token
Parameters: grant_type=client_credentials, client_id, client_secret
Token TTL: 1800 seconds (30 minutes)

## Request and Response Shape

**Budget Check Query:**
Request: `GET /api/v1/budget/check?code=E123&fy=2024`
Response: `{budget_code: "E123", fiscal_year: "2024", total_allocation: 5000000, total_committed: 3200000, available_balance: 1800000}`

**Voucher Creation:**
Request: `POST /api/v1/vouchers {vote_head: "E123", supplier_id: "S001", amount: 100000, description: "Election materials", invoice_ref: "INV-2024-001"}`
Response: `{voucher_id: "V-2024-001", status: "CREATED", created_timestamp: "2024-08-21T10:30:00Z", reference_number: "REF-2024-001"}`

**Payment Status Query:**
Request: `GET /api/v1/payments/V-2024-001/status`
Response: `{voucher_id: "V-2024-001", status: "PAID|PENDING|REJECTED", paid_amount: 100000, paid_date: "2024-08-21"}`

## Sample Curl

```bash
# Budget check
curl -H "Authorization: Bearer $TOKEN" \
  https://ifmis.go.ke/api/v1/budget/check?code=E123

# Voucher creation
curl -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"vote_head":"E123","supplier_id":"S001","amount":100000,"description":"Election materials"}' \
  https://ifmis.go.ke/api/v1/vouchers

# Payment status
curl -H "Authorization: Bearer $TOKEN" \
  https://ifmis.go.ke/api/v1/payments/V-2024-001/status
```

## Health Probe

GET /api/v1/health returns 200 OK if service is operational.

## Rate Limits

100 requests/minute per OAuth client. Exceeding limit returns 429 with Retry-After header.

## PII Enforcement

Supplier and payee data (names, account numbers) classified as medium sensitivity. Voucher amounts are internal. Logs redact supplier names and mask account numbers. No personally identifiable information stored in request logs.

## Chaos Toggle

IFMIS_CHAOS=true enables chaos injection. Causes random 503 errors and latency spikes (test only).

## Fallback Behavior

On adapter unavailability, voucher requests queued locally. Manual offline voucher workflow available via admin interface. Operator can manually post to IFMIS when system recovers and reconcile via batch upload.

## Circuit Breaker Policy

Threshold: 3 consecutive failures
Open duration: 30 seconds
Rationale: IFMIS is government system with occasional maintenance windows
Half-open: permits 1 probe request

## Environment Variables

IFMIS_BASE_URL: Base URL for IFMIS API
IFMIS_CLIENT_ID: OAuth2 client ID from Treasury
IFMIS_CLIENT_SECRET: OAuth2 client secret from Treasury
IFMIS_TOKEN_URL: OAuth2 token endpoint
IFMIS_TIMEOUT_SEC: Request timeout in seconds (default: 30)
IFMIS_CHAOS: "true" enables chaos (test only)

## Related Modules

M54 (budget-expenditure), M59 (oag-interface)

## Runbook

docs/runbooks/ifmis-voucher-reconciliation.md (manual voucher posting, payment tracking)

