# KRA iTax Test Double

Mock SOAP server simulating Kenya Revenue Authority iTax gateway for integration testing. Accepts SOAP envelopes with WS-Security headers, returns canned SOAP responses, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without KRA dependency. Validates adapter WSDL parsing, SOAP envelope construction, WS-Security header injection, and SOAP fault handling. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_kra_itax
```

Server listens on http://localhost:8002 by default.

To override port:
```bash
PORT=9002 uv run python -m eel_double_kra_itax
```

Environment variable: KRA_ITAX_DOUBLE_PORT (default: 8002).

## Chaos Knobs

Control failure modes via environment variables:

KRA_ITAX_CHAOS_ENABLED: "true" to activate all chaos (default: false)
KRA_ITAX_CHAOS_FAULT_RATE: Fraction [0.0-1.0] of requests returning SOAP fault (default: 0.0)
KRA_ITAX_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
KRA_ITAX_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)
KRA_ITAX_CHAOS_SECURITY_FAULT: "true" returns security fault on missing WS-Security header (default: false)

Example:
```bash
KRA_ITAX_CHAOS_ENABLED=true KRA_ITAX_CHAOS_FAULT_RATE=0.2 uv run python -m eel_double_kra_itax
```

## Canned Responses

**WSDL Endpoint** (GET /soap/wsdl):
Returns complete WSDL schema matching KRA iTax operations. Cached in memory.

**TaxCompliance Operation:**
SOAP Response:
```xml
<soap:Body>
  <TaxComplianceCheckResponse>
    <status>COMPLIANT</status>
    <certificate_url>http://itax.kra.go.ke/cert/A001234567B_2024.pdf</certificate_url>
    <expiry_date>2027-12-31</expiry_date>
  </TaxComplianceCheckResponse>
</soap:Body>
```

On chaos: returns SOAP fault with `<faultstring>Service temporarily unavailable</faultstring>`.

**PAYESubmission Operation:**
SOAP Response:
```xml
<soap:Body>
  <PAYESubmissionResponse>
    <submission_id>sub-12345</submission_id>
    <status>ACCEPTED</status>
    <accepted_count>25</accepted_count>
    <rejected_count>0</rejected_count>
  </PAYESubmissionResponse>
</soap:Body>
```

On invalid TIN: returns SOAP fault with `<faultstring>Invalid Employer TIN</faultstring>`.

**P9AGenerate Operation:**
SOAP Response:
```xml
<soap:Body>
  <P9AGenerateResponse>
    <document_url>http://itax.kra.go.ke/p9a/A001234567B_2024.pdf</document_url>
    <generated_date>2024-08-21</generated_date>
  </P9AGenerateResponse>
</soap:Body>
```

All responses include WS-Addressing headers matching request MessageID.

## Endpoints Exposed

POST /soap - SOAP endpoint (accepts SOAP 1.1 envelopes)
GET /soap/wsdl - WSDL schema fetch

WSDL operations simulated: TaxComplianceCheck, PAYESubmission, P9AGenerate
