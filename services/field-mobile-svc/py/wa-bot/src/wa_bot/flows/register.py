"""M35: Voter register lookup via WhatsApp."""

from __future__ import annotations

import logging

_log_ = logging.getLogger(__name__)


async def register_lookup_flow(national_id_hash: str) -> str:
    """Return register lookup result for the given hashed national ID.

    In production, queries the voter register microservice.
    """
    assert national_id_hash, "national_id_hash required"
    _log_.info("register.flow id_hash=%s", national_id_hash[:8])
    # Stub: real impl calls voter register API
    return (
        f"Register lookup for ID ending ...{national_id_hash[-4:]}:\n"
        "FOUND: YES\n"
        "Polling station: Confirm at eel.example"
    )
