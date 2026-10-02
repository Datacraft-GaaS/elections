# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""WhatsApp webhook handler for grievance intake (WP-4.1)."""

from __future__ import annotations

import logging
from typing import Any

from .models import WhatsappWebhook

_log_ = logging.getLogger(__name__)


class WhatsappHandler:
    """Handle incoming WhatsApp Cloud API webhooks for grievance intake."""

    async def handle_webhook(self, payload: WhatsappWebhook) -> dict[str, Any]:
        """Parse webhook payload, extract message, return ack."""
        extracted: list[dict[str, Any]] = []
        for entry in payload.entry:
            for change in entry.get("changes", []):
                value = change.get("value", {})
                for message in value.get("messages", []):
                    msg_id = message.get("id", "")
                    msg_type = message.get("type", "text")
                    from_num = message.get("from", "")
                    body = ""
                    if msg_type == "text":
                        body = message.get("text", {}).get("body", "")
                    _log_.info(
                        "whatsapp.message msg_id=%s from=%s type=%s body_len=%d",
                        msg_id,
                        _mask_phone(from_num),
                        msg_type,
                        len(body),
                    )
                    extracted.append(
                        {
                            "msg_id": msg_id,
                            "from_hash": _hash_phone(from_num),
                            "body": body,
                        }
                    )

        return {"status": "ok", "processed": len(extracted)}

    async def send_reply(
        self, to_number: str, message_text: str, api_token: str
    ) -> dict[str, Any]:
        """Send a WhatsApp reply via Cloud API (stub: returns simulated response)."""
        _log_.info(
            "whatsapp.send_reply to=%s len=%d",
            _mask_phone(to_number),
            len(message_text),
        )
        # Real impl: POST https://graph.facebook.com/v19.0/{phone_number_id}/messages
        return {"messages": [{"id": "stub_msg_id"}]}


def _hash_phone(phone: str) -> str:
    import hashlib

    return hashlib.sha256(phone.encode()).hexdigest()[:16]


def _mask_phone(phone: str) -> str:
    if len(phone) >= 6:
        return phone[:3] + "***" + phone[-3:]
    return "***"
