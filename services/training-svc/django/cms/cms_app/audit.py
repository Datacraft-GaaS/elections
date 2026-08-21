import logging
from typing import Any

_log_ = logging.getLogger(__name__)


def log_status_transition(
    content_item_id: str,
    actor_id: str | None,
    from_status: str,
    to_status: str,
    reason: str | None = None,
) -> None:
    _log_.info(
        "content_item status transition: id=%s actor=%s %s -> %s reason=%s",
        content_item_id,
        actor_id,
        from_status,
        to_status,
        reason,
    )


def log_distribution_event(
    content_item_id: str,
    channel: str,
    county_code: str,
    quantity: int,
    extra: dict[str, Any] | None = None,
) -> None:
    _log_.info(
        "distribution event: content_item_id=%s channel=%s county=%s qty=%d extra=%s",
        content_item_id,
        channel,
        county_code,
        quantity,
        extra,
    )
