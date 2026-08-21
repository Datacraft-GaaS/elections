# NSSF Adapter

Production adapter for National Social Security Fund REST API. Handles mTLS authentication, contribution submission, member verification, and statement retrieval.

## Purpose

Bridges eel core system (M55 module) to NSSF API. Manages TLS client certificate, API key header injection, bulk contribution formatting, member validation, and contribution reconciliation.

## Layout

- `service.py`: Main adapter service. Handles mTLS session setup, API key header construction, contribution submission, member lookups, statement fetching.
- `views.py`: Pydantic models for NSSF request/response payloads. Models for ContributionSubmit, MemberVerify, StatementFetch operations. Error code enums.
- `client.py`: HTTP client initialization with mTLS and timeout configuration.
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**NssfService** (async):
- `submit_contributions(employer_code, period, contributions)`: Submit bulk payroll contributions, returns SubmissionReceipt.
- `verify_member(member_id)`: Check member account status, returns MemberStatus.
- `fetch_statement(member_id, year)`: Retrieve annual statement PDF, returns StatementURL.

**Models** (in views.py):
- `ContributionRecord`: Member ID, contribution amount, deduction date.
- `ContributionSubmitRequest`: Employer code, period month, list of contributions.
- `ContributionSubmitResponse`: Submission ID, status, counts, error details.
- `MemberStatus`: Member ID, name, account status, balance, last contribution date.
- `StatementRequest`: Member ID, fiscal year.
- `StatementResponse`: URL or base64-encoded PDF, generation date.

## Environment Variables

NSSF_BASE_URL: NSSF API base URL
NSSF_CERT: Path to mTLS client certificate (PEM format)
NSSF_KEY: Path to mTLS private key (PEM format)
NSSF_EMPLOYER_CODE: Employer code for submissions
NSSF_API_KEY: API key for X-API-Key header
NSSF_TIMEOUT_SEC: Request timeout in seconds (default: 30)
NSSF_CHAOS: "true" enables chaos injection (test only)

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
