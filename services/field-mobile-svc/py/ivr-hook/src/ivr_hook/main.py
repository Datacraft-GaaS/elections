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
