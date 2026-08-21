# NHIF-SHIF Adapter

Production adapter for Social Health Authority REST API. Handles mTLS authentication, SHIF contribution submission, beneficiary verification, and insurance eligibility checking.

## Purpose

Bridges eel core system (M55 module) to SHA API. Manages TLS client certificate, API key header injection, contribution formatting, member validation, and benefit eligibility verification.

## Layout

- `service.py`: Main adapter service. Handles mTLS session setup, API key header construction, contribution submission, member verification, eligibility checks.
- `views.py`: Pydantic models for SHA request/response payloads. Models for ContributionSubmit, BeneficiaryVerify operations. Insurance class and status enums.
- `client.py`: HTTP client initialization with mTLS and timeout configuration.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**NhifShifService** (async):
- `submit_contributions(employer_code, month, employees)`: Submit bulk SHIF contributions, returns ReceiptNumber.
- `verify_beneficiary(national_id)`: Check member eligibility status, returns MemberStatus.

**Models** (in views.py):
- `EmployeeContribution`: National ID, contribution amount, insurance class.
- `ContributionSubmitRequest`: Employer code, month, list of employees.
- `ContributionSubmitResponse`: Receipt number, status, counts, error details.
- `BeneficiaryStatus`: National ID, member number, status, insurance class, enrollment date.

## Environment Variables

NHIF_BASE_URL: SHA API base URL
NHIF_CERT: Path to mTLS client certificate (PEM format)
NHIF_KEY: Path to mTLS private key (PEM format)
NHIF_EMPLOYER_CODE: Employer code assigned by SHA
NHIF_API_KEY: API key for X-API-Key header
NHIF_TIMEOUT_SEC: Request timeout in seconds (default: 30)
NHIF_CHAOS: "true" enables chaos injection (test only)

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
