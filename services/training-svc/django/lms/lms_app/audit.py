import logging
from typing import Any

_log_ = logging.getLogger(__name__)


def log_assessment_attempt(
    worker_id: str,
    module_id: str,
    attempt_number: int,
    score_pct: float,
    passed: bool,
) -> None:
    _log_.info(
        "assessment attempt: worker=%s module=%s attempt=%d score=%.2f passed=%s",
        worker_id,
        module_id,
        attempt_number,
        score_pct,
        passed,
    )


def log_certificate_event(
    event: str,
    certificate_id: str,
    worker_id: str,
    extra: dict[str, Any] | None = None,
) -> None:
    _log_.info(
        "certificate event: %s certificate_id=%s worker_id=%s extra=%s",
        event,
        certificate_id,
        worker_id,
        extra,
    )
