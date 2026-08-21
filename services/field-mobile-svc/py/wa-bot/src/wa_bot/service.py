"""WaBotService: routes inbound WhatsApp messages to voter-service flows."""

from __future__ import annotations

import hashlib
import logging

from .flows.grievance import grievance_intake_flow
from .flows.register import register_lookup_flow
from .flows.verifier import voter_verify_flow
from .views import WhatsAppWebhookPayload

_log_ = logging.getLogger(__name__)

_VERIFY_KEYWORDS = {"verify", "confirm", "check"}
_GRIEVANCE_KEYWORDS = {"grievance", "complaint", "problem", "issue"}
_REGISTER_KEYWORDS = {"register", "listed", "roll"}


class WaBotService:
    """Routes inbound WA messages to the appropriate voter-service flow."""

    async def handle(self, payload: WhatsAppWebhookPayload) -> None:
        """Process all messages in the webhook payload."""
        for entry in payload.entry:
            for change in entry.changes:
                messages = change.value.get("messages", [])
                for msg in messages:
                    await self._dispatch(msg)

    async def _dispatch(self, msg: dict) -> str | None:
        from_number: str = msg.get("from", "")
        body: str = msg.get("text", {}).get("body", "").strip().lower()
        _log_.info("wa.dispatch from=%s body=%r", from_number, body[:40])

        tokens = set(body.split())
        if tokens & _VERIFY_KEYWORDS:
            id_hash = hashlib.sha256(from_number.encode()).hexdigest()
            return await voter_verify_flow(id_hash)
        if tokens & _GRIEVANCE_KEYWORDS:
            return await grievance_intake_flow(body)
        if tokens & _REGISTER_KEYWORDS:
            id_hash = hashlib.sha256(from_number.encode()).hexdigest()
            return await register_lookup_flow(id_hash)

        _log_.info("wa.dispatch no matching flow body=%r", body[:40])
        return None
