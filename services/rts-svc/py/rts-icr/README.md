# RTS ICR (Intelligent Character Recognition)

Intelligent Character Recognition (ICR) service for Form 34A/B/C handwritten tally extraction. Called by ro-desktop app during form scanning to extract voted tallies from handwritten forms. Returns extracted data with confidence scores and flags ambiguous fields for manual review.

## Overview

Module: M10 (RTS Backend), M09 (RTS Read-Only)

OCR/ICR service specialized in handwritten election form extraction. Supports Forms 34A (presidential/parliamentary tallies), 34B (county tallies), and 34C (supplementary forms).

## FastAPI Endpoints

- POST `/icr/extract` - Extract tallies from form image
- GET `/icr/result/{job_id}` - Poll extraction result status
- GET `/health` - Service health check

## Pydantic Models

- `ICRRequest` - Form image (form_scan_b64: str, form_type: str = '34A')
- `ICRResult` - Extraction (job_id: str, extracted_tallies: dict, confidence: float, flagged_fields: list[str])

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `ICR_MODEL_PATH` - Path to trained ICR model
- `GPU_DEVICE_ID` - CUDA device ID (default: 0)
- `LITELLM_ADAPTER_URL` - LiteLLM API for LLM-assisted disambiguation
- `ICR_CONFIDENCE_THRESHOLD` - Confidence threshold for auto-approval (default: 0.90)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m rts_icr.worker
```

## GPU Requirements

- CUDA 12+ required
- Minimum 4GB VRAM
- Inference time: ~1-2 seconds per form

## Field Flagging

Fields are flagged for manual review when:
- Confidence < threshold
- Handwriting is ambiguous (multiple digits possible)
- Mathematical inconsistencies (e.g., subtotal > grand total)

## Notes

- Forms must be high-quality scans (300+ DPI, good lighting)
- Supports handwritten digits 0-9 and operator symbols (+, -, =)
- Returns extracted tallies in JSON format matching Form 34 structure
- Disputed sections are marked for returning officer manual review
- Model is retrained quarterly with new labeled forms
