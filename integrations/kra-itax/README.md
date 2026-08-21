# KRA iTax SOAP Gateway

Kenya Revenue Authority tax compliance and statutory deduction API. Handles employer PAYE submissions, national ID verification, and tax clearance certificate retrieval for candidate eligibility.

## MoU Status

KRA MoU signed with annual renewal cycle. Endpoint URL under NDA; contact KRA Compliance for production WSDL and certificate credentials.

## Endpoint

Production: https://itax.kra.go.ke/KRA-Portal/wsServices/ (exact URLs under NDA)
Sandbox: Contact KRA for sandbox WSDL and test credentials

## Protocol and Authentication

SOAP 1.1 over HTTPS. Authentication via mTLS client certificate plus WS-Security UsernameToken with KRA PIN embedded in SOAP header.

**mTLS Requirements:**
- Client certificate (X.509) issued by KRA PKI
- Private key stored securely (env var KRA_ITAX_KEY)
- Root CA certificate for server verification

**SOAP Header (WS-Security):**
```xml
<wsse:Security>
  <wsse:UsernameToken>
    <wsse:Username>KRA_PIN</wsse:Username>
    <wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0.xsd#PasswordText">PIN_VALUE</wsse:Password>
  </wsse:UsernameToken>
</wsse:Security>
```

## Request and Response Shape

**TaxCompliance Operation:**
Request: `{pin: "A001234567B", operation: "clearance_check"}`
Response: `{status: "COMPLIANT|NON_COMPLIANT", certificate_url: "http://...", expiry_date: "2027-12-31"}`

**PAYESubmission Operation:**
Request: `{employer_tin: "A001234567B", month: "202408", deductions: [{pin: "...", amount: "..."}...]}`
Response: `{submission_id: "uuid", status: "ACCEPTED|REJECTED", errors: [...]}`

**P9AGenerate Operation:**
Request: `{pin: "A001234567B", year: "2024"}`
Response: `{document_url: "http://...", base64_pdf: "..."}`

## Health Probe

SOAP Ping endpoint verifies gateway availability. Adapter makes test WSDL fetch and validates schema.

## Rate Limits

60 requests/minute per client certificate. Exceeding limit returns 429 Too Many Requests.

## PII Enforcement

KRA PIN and TIN are tier-A restricted. Employer/employee names are tier-A. Salary amounts are tier-B internal. Logs mask PINs, showing only first 3 and last 2 characters. TINs never logged. Names logged as initials only. All SOAP payloads encrypted in transit via TLS 1.3+.

## Chaos Toggle

KRA_ITAX_CHAOS=true enables chaos injection. Causes random SOAP faults and latency spikes (test only).

## Fallback Behavior

On adapter unavailability, clearance checks default to manual review (tax status unknown). HR payroll PAYE submission queued locally and retried hourly. Operator escalates to manual KRA inquiry for resolution.

## Circuit Breaker Policy

Threshold: 2 consecutive failures
Open duration: 60 seconds
Rationale: KRA legacy system is slow; aggressive breaker prevents cascading timeouts
Half-open: permits 1 probe request

## Environment Variables

KRA_ITAX_WSDL_URL: SOAP WSDL URL (required, under NDA)
KRA_ITAX_CERT: Path to mTLS client certificate (PEM format)
KRA_ITAX_KEY: Path to mTLS private key (PEM format)
KRA_ITAX_PIN: KRA agent PIN for WS-Security UsernameToken
KRA_ITAX_ENV: "sandbox" or "production" (default: sandbox)
KRA_ITAX_TIMEOUT_SEC: Request timeout in seconds (default: 30)
KRA_ITAX_CHAOS: "true" enables chaos (test only)

## Related Modules

M55 (hr-payroll statutory deductions), M12 (crms candidate clearance), M22 (campaign finance)

## Runbook

docs/runbooks/kra-itax-compliance-check.md (certificate renewal, PIN rotation, manual override)

