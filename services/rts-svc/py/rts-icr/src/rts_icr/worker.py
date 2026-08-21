import logging

import httpx

from .models import IcrResult
from .ocr import extract_fields

_log_ = logging.getLogger(__name__)

CONFIDENCE_GATE = 0.90


async def process_job(form_id: str, image_url: str, api_base: str) -> IcrResult:
    """Fetch the image, run OCR, return an IcrResult."""
    fields = await extract_fields(image_url)
    requires_review = any(f.confidence < CONFIDENCE_GATE for f in fields)
    result = IcrResult(
        form_id=form_id,
        fields=fields,
        requires_review=requires_review,
        model_version="stub-v0.1",
    )
    _log_.info(
        "icr.process_job form_id=%s requires_review=%s",
        form_id,
        requires_review,
    )
    return result
