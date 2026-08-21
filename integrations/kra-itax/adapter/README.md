# KRA iTax Adapter

Production adapter for Kenya Revenue Authority iTax SOAP gateway. Manages mTLS authentication, SOAP request serialization, WS-Security token injection, and SOAP fault parsing.

## Purpose

Bridges eel core system (M55, M12, M22 modules) to KRA iTax SOAP API. Handles TLS client certificate management, WSDL parsing, SOAP envelope construction with security headers, timeout handling, and legacy error code mapping.

## Layout

- `service.py`: Main adapter service. Manages SOAP client (via zeep library), mTLS session setup, WS-Security header construction, SOAP fault exception handling.
- `views.py`: Pydantic models for SOAP request/response envelopes. Models for TaxCompliance, PAYESubmission, P9AGenerate operations. Error code mappings.
- `client.py`: Lazy SOAP client initialization with certificate pinning and timeout configuration.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**KraItaxService** (async):
- `tax_compliance_check(pin)`: Check tax clearance status, returns CertificateStatus.
- `paye_submission(employer_tin, month, deductions)`: Submit monthly PAYE returns, returns SubmissionReceipt.
- `p9a_generate(pin, year)`: Generate P9A annual statement PDF, returns DocumentURL.
- `_inject_ws_security_header(envelope)`: Constructs WS-Security UsernameToken header.
- `_parse_soap_fault(fault_element)`: Extracts KRA error code and message from SOAP fault.

**Security Models** (in views.py):
- `TaxComplianceRequest`: PIN, operation type, optional fiscal year.
- `TaxComplianceResponse`: Status (COMPLIANT, NON_COMPLIANT), certificate URL, expiry date.
- `PAYEDeduction`: Employee PIN, month, amount withheld.
- `PAYESubmissionRequest`: Employer TIN, month, list of deductions.
- `PAYESubmissionResponse`: Submission ID, status, validation errors.

## Environment Variables

KRA_ITAX_WSDL_URL: SOAP WSDL endpoint (required, under NDA)
KRA_ITAX_CERT: Path to mTLS client certificate file (PEM)
KRA_ITAX_KEY: Path to mTLS private key file (PEM)
KRA_ITAX_PIN: KRA agent PIN for WS-Security header
KRA_ITAX_ENV: "sandbox" or "production"
KRA_ITAX_TIMEOUT_SEC: HTTP timeout (default: 30)
KRA_ITAX_CHAOS: "true" enables chaos injection (test only)

## Build and Test

Build package:
```bash
cd adapter
uv build
```

Install in development:
```bash
uv pip install -e .
```

Type check:
```bash
uv run pyright src/
```

Run tests:
```bash
uv run pytest tests/ -vxs
```

Note: Requires KRA WSDL URL in KRA_ITAX_WSDL_URL env var for contract tests.
