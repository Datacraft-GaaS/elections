# NHIF-SHIF Health Insurance API

Social Health Authority (formerly NHIF) REST API for employee health insurance contribution submission and beneficiary verification. Supports monthly SHIF (Social Health Insurance Fund) contribution payment and member eligibility checking.

## MoU Status

SHA employer registration with mutual NDA. mTLS client certificate and API key issued by SHA ICT department. Annual compliance audit required.

## Endpoint

Production: https://api.sha.go.ke/v1/
Sandbox: https://api-dev.sha.go.ke/v1/

Note: Formerly nhif.or.ke; migrated to SHA branding 2023.

## Protocol and Authentication

REST over HTTPS with JSON payloads. mTLS client certificate (X.509) plus API-key header authentication.

**mTLS Requirements:**
- Client certificate issued by SHA PKI
- Private key stored securely (env var NHIF_KEY)
- Request includes client certificate in TLS handshake

**API Key Header:**
All requests include: X-API-Key: {api_key}

## Request and Response Shape

**Contribution Submit:**
Request: `POST /contributions {employer_code, month, employees: [{national_id, contribution_amount, insurance_class}...]}`
Response: `{receipt_number, status: "ACCEPTED|REJECTED", accepted_count, errors: [...]}`

**Beneficiary Verify:**
Request: `GET /members/{national_id}`
Response: `{national_id, status: "ACTIVE|INACTIVE", member_number, insurance_class, enrollment_date}`

## Sample Curl

```bash
curl -E /path/to/client.pem --key /path/to/key.pem \
  -H "X-API-Key: $NHIF_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"employer_code":"EMP001","month":"202408","employees":[{"national_id":"12345678","contribution_amount":3000}]}' \
  https://api.sha.go.ke/v1/contributions
```

## Health Probe

GET /health returns 200 OK if service is operational.

## Rate Limits

30 requests/minute per employer code. Bulk contribution accepts up to 3000 employees per request.

## PII Enforcement

National IDs and member numbers are tier-A restricted. Health insurance class is tier-A (privacy sensitive). Contribution amounts are tier-B internal. Logs mask national IDs showing only last 4 digits. Member numbers never logged. All requests encrypted in transit via TLS 1.3+.

## Chaos Toggle

NHIF_CHAOS=true enables chaos injection. Causes random 503 errors and contribution rejections (test only).

## Fallback Behavior

On adapter unavailability, SHIF contribution queued locally. Manual submission available via SHA web portal. Operator reconciles via monthly contribution verification report and manual dashboard.

## Circuit Breaker Policy

Threshold: 2 consecutive failures
Open duration: 60 seconds
Rationale: SHIF remittance is critical for employee coverage; aggressive breaker prevents missed deadlines
Half-open: permits 1 probe request

## Environment Variables

NHIF_BASE_URL: SHA API base URL
NHIF_CERT: Path to mTLS client certificate (PEM format)
NHIF_KEY: Path to mTLS private key (PEM format)
NHIF_EMPLOYER_CODE: Employer code assigned by SHA
NHIF_API_KEY: API key for X-API-Key header
NHIF_TIMEOUT_SEC: Request timeout in seconds (default: 30)
NHIF_CHAOS: "true" enables chaos (test only)

## Related Modules

M55 (hr-payroll SHIF contribution submission)

## Runbook

docs/runbooks/nhif-shif-contribution-reconciliation.md (manual upload, contribution tracking, eligibility)

