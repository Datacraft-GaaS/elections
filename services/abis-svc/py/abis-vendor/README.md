# ABIS Vendor SDK Wrapper

Python wrapper for the proprietary third-party ABIS (Automated Biometric Identification System) vendor SDK. Provides C/C++ SDK bindings via ctypes/CFFI for biometric matching and template extraction operations. Consumed by the Rust abis-matcher crate via gRPC bridge.

## Overview

Module: M03 (ABIS)

Low-level interface to vendor biometric matching engine (1:N search and template extraction). This package wraps proprietary C/C++ libraries and is not in version control; vendor SDK must be installed separately on deployment.

## Adapters Exposed

- `VendorMatchEngine` - Perform 1:N biometric searches against candidate database
- `VendorEnrolEngine` - Extract biometric templates from raw fingerprint/iris images

## FastAPI Endpoints

- POST `/match` - Execute 1:N biometric match
- POST `/enroll` - Extract template from biometric image
- GET `/health` - Service health check

## Pydantic Models

- `MatchRequest` - Match query (template: bytes, candidates: list[bytes], threshold: float)
- `MatchResponse` - Match result (score: float, match: bool, candidate_id: str | None)
- `EnrollRequest` - Template extraction (image_b64: str, image_format: str)
- `EnrollResponse` - Extracted template (template: bytes, confidence: float)

## Environment Variables

- `VENDOR_SDK_LIB_PATH` - Path to vendor SDK shared library (.so/.dylib/.dll)
- `VENDOR_LICENSE_FILE` - License file path (required, proprietary)
- `VENDOR_THRESHOLD_1N` - Match score threshold for 1:N (0.0-1.0, default 0.97)
- `VENDOR_MAX_CANDIDATES` - Maximum candidates per search (default 1000)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Deployment

Pod resource: CPU 2000m, Memory 2Gi (GPU recommended for large-scale matching).

Production run:

```bash
gunicorn abis_vendor.wsgi:application --workers 4
```

## Notes

- Vendor SDK is proprietary (NDA) and not versioned; must be manually installed
- Requires license activation; license file must be valid and accessible
- Match score interpretation: > 0.97 = high confidence match
- Template format is vendor-specific; not interoperable with other ABIS systems
