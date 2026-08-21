import logging

from .models import IcrField

_log_ = logging.getLogger(__name__)


async def extract_fields(image_path: str, form_type: str = "34a") -> list[IcrField]:
    """Extract form fields from an image using OCR.

    Stub: returns synthetic fields with confidence 0.95.
    TODO: integrate PaddleOCR ONNX model for production.
    """
    _log_.info("ocr.extract_fields form_type=%s path=%s", form_type, image_path)
    return [
        IcrField(
            field_name="valid_votes",
            value="0",
            confidence=0.95,
            bbox=[0.0, 0.0, 1.0, 1.0],
        ),
        IcrField(
            field_name="rejected_votes",
            value="0",
            confidence=0.95,
            bbox=[0.0, 0.1, 1.0, 0.1],
        ),
        IcrField(
            field_name="spoilt_ballots",
            value="0",
            confidence=0.95,
            bbox=[0.0, 0.2, 1.0, 0.1],
        ),
        IcrField(
            field_name="votes_cast",
            value="0",
            confidence=0.95,
            bbox=[0.0, 0.3, 1.0, 0.1],
        ),
    ]
