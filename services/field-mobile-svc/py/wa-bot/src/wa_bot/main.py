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

"""FastAPI app: WhatsApp Business API webhook (WP-4.7, M13/M23/M35)."""

from __future__ import annotations

from fastapi import FastAPI, HTTPException

from .service import WaBotService
from .views import WhatsAppWebhookPayload

app = FastAPI(title="eel-wa-bot", version="0.1.0")
_svc = WaBotService()


@app.get("/healthz")
async def health() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/webhook/whatsapp")
async def verify_webhook(
    hub_mode: str,
    hub_challenge: str,
    hub_verify_token: str,
) -> int:
    """Meta webhook verification handshake."""
    if hub_verify_token != "eel-verify-token":
        raise HTTPException(status_code=403, detail="invalid token")
    return int(hub_challenge)


@app.post("/webhook/whatsapp")
async def handle_webhook(payload: WhatsAppWebhookPayload) -> dict[str, str]:
    await _svc.handle(payload)
    return {"status": "received"}
