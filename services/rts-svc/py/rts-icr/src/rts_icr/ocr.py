import logging

from .models import IcrField

_log_ = logging.getLogger(__name__)


class OcrUnavailableError(RuntimeError):
    """Raised when no ICR handwriting model is available for extraction."""


async def extract_fields(image_path: str, form_type: str = "34a") -> list[IcrField]:
    """Extract form fields from a scanned Form 34A image.

    Extraction requires the PaddleOCR ONNX handwriting-recognition model,
    which is not yet vendored (see `docs/plans/deferral-plan.md` section D).
    Until that model lands, field extraction fails loudly instead of
    fabricating synthetic results.
    """
    _log_.warning(
        "ocr.extract_fields unavailable form_type=%s path=%s (see deferral-plan.md section D)",
        form_type,
        image_path,
    )
    raise OcrUnavailableError(
        "ICR handwriting model not vendored; see docs/plans/deferral-plan.md"
    )
