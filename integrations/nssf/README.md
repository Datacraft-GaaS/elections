# NSSF Contribution API

National Social Security Fund REST API for employer contribution submission and member verification. Supports payroll contribution remittance, member account balance queries, and contribution statement retrieval.

## MoU Status

NSSF employer registration and NDA signed. mTLS client certificate and API key issued by NSSF ICT department. Annual compliance audit required.

## Endpoint

Production: https://api.nssf.or.ke/v1/
Sandbox: https://api-staging.nssf.or.ke/v1/

## Protocol and Authentication

REST over HTTPS with JSON payloads. mTLS client certificate (X.509) plus API-key header authentication.

**mTLS Requirements:**
- Client certificate issued by NSSF PKI
- Private key stored securely (env var NSSF_KEY)
- Request includes client certificate in TLS handshake

**API Key Header:**
All requests include: X-API-Key: {api_key}

## Request and Response Shape

**Contribution Submit:**
Request: `POST /contributions {employer_code, period_month, contributions: [{member_id, amount, deduction_date}...]}`
Response: `{submission_id, status: "ACCEPTED|REJECTED", accepted_count, rejected_count, total_amount, errors: [...]}`

**Member Verify:**
Request: `GET /members/{member_id}`
Response: `{member_id, name, status: "ACTIVE|DORMANT|SUSPENDED", account_balance, last_contribution_date}`

**Statement Fetch:**
Request: `GET /statements/{member_id}?year=2024`
Response: `{statement_url, document_base64, generated_date}`

## Sample Curl

```bash
curl -E /path/to/client.pem --key /path/to/key.pem \
  -H "X-API-Key: $NSSF_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"employer_code":"EMP001","period_month":"202408","contributions":[{"member_id":"MEM123","amount":5000}]}' \
  https://api.nssf.or.ke/v1/contributions
```

## Health Probe

GET /health returns 200 OK if service is operational.

## Rate Limits

30 requests/minute per employer code. Bulk contribution submission accepts up to 5000 members per request.

## PII Enforcement

National ID numbers and member IDs are tier-A restricted. Payroll amounts are tier-B internal. Logs mask national IDs showing only first 3 and last 2 characters. Member IDs never logged in plaintext. All requests encrypted in transit via TLS 1.3+.

## Chaos Toggle

NSSF_CHAOS=true enables chaos injection. Causes random 503 errors and submission rejections (test only).

## Fallback Behavior

On adapter unavailability, contribution remittance queued locally. Manual bulk upload available via NSSF web portal. Operator reconciles via monthly NSSF contribution report and manual verification dashboard.

## Circuit Breaker Policy

Threshold: 2 consecutive failures
Open duration: 60 seconds
Rationale: NSSF processing is time-critical for employer remittance deadlines
Half-open: permits 1 probe request

## Environment Variables

NSSF_BASE_URL: NSSF API base URL
NSSF_CERT: Path to mTLS client certificate (PEM format)
NSSF_KEY: Path to mTLS private key (PEM format)
NSSF_EMPLOYER_CODE: Employer code assigned by NSSF
NSSF_API_KEY: API key for X-API-Key header
NSSF_TIMEOUT_SEC: Request timeout in seconds (default: 30)
NSSF_CHAOS: "true" enables chaos (test only)

## Related Modules

M55 (hr-payroll NSSF contribution submission)

## Runbook

docs/runbooks/nssf-contribution-reconciliation.md (manual upload, contribution tracking)

