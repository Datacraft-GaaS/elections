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

"""FastAPI app: Africa's Talking IVR webhook (WP-4.7)."""

from __future__ import annotations

from fastapi import FastAPI, Form
from fastapi.responses import PlainTextResponse

from .service import IvrService

app = FastAPI(title="eel-ivr-hook", version="0.1.0")
_svc = IvrService()


@app.get("/healthz")
async def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post("/ivr/voice", response_class=PlainTextResponse)
async def voice_webhook(
    sessionId: str = Form(...),
    phoneNumber: str = Form(...),
    dtmfDigits: str | None = Form(None),
) -> str:
    """Africa's Talking IVR callback - returns SSML/text."""
    return await _svc.handle_call(sessionId, phoneNumber, dtmfDigits)
