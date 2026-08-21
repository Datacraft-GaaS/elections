"""M23: Voter grievance intake via WhatsApp."""

from __future__ import annotations

import logging

_log_ = logging.getLogger(__name__)


async def grievance_intake_flow(message: str) -> str:
    """Acknowledge and record a voter grievance.

    Returns a confirmation text for the WhatsApp reply.
    """
    assert message, "message required"
    _log_.info("grievance.flow message_len=%d", len(message))
    # Stub: real impl persists to grievance service
    return (
        "Your grievance has been recorded. Reference: GRV-STUB-001.\n"
        "A returning officer will respond within 48 hours.\n"
        "Dial *123# for urgent issues."
    )
