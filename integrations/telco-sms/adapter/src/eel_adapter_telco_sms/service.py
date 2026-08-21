"""SMS gateway adapter: Africa's Talking REST + Safaricom SMPP."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import httpx
from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import ReceiveWebhook, SendRequest, SendResult

_LOGGER = logging.getLogger(__name__)

_AT_API_KEY = os.environ.get("AT_API_KEY", "")
_AT_USERNAME = os.environ.get("AT_USERNAME", "sandbox")
_AT_BASE = os.environ.get("AT_BASE_URL", "https://api.africastalking.com/version1")

_http: httpx.AsyncClient | None = None


def uuid7str() -> str:
    return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
    global _http
    _http = httpx.AsyncClient(timeout=15.0)
    yield
    await _http.aclose()


app = FastAPI(title="eel-adapter-telco-sms", lifespan=_lifespan)


async def _send_at(req: SendRequest) -> SendResult:
    assert _http is not None
    resp = await _http.post(
        f"{_AT_BASE}/messaging",
        data={
            "username": _AT_USERNAME,
            "to": ",".join(req.to),
            "message": req.message,
            **({"from": req.sender_id} if req.sender_id else {}),
        },
        headers={"apiKey": _AT_API_KEY, "Accept": "application/json"},
    )
    resp.raise_for_status()
    data = resp.json()
    recipients = data.get("SMSMessageData", {}).get("Recipients", [])
    accepted = [r for r in recipients if r.get("status") == "Success"]
    rejected = [r for r in recipients if r.get("status") != "Success"]
    return SendResult(
        request_id=req.request_id,
        accepted=len(accepted),
        rejected=len(rejected),
        message_ids=[r.get("messageId", uuid7str()) for r in accepted],
    )


@app.post("/sms/send", response_model=SendResult, status_code=status.HTTP_200_OK)
async def send(req: SendRequest) -> SendResult:
    """Send an SMS via Africa's Talking or Safaricom SMPP."""
    if req.provider == "at":
        try:
            return await _send_at(req)
        except httpx.HTTPStatusError as exc:
            _LOGGER.error("AT upstream error: %s", exc)
            raise HTTPException(status_code=502, detail="AT upstream error") from exc
    raise HTTPException(status_code=400, detail=f"Unknown provider: {req.provider!r}")


@app.post("/sms/receive", status_code=status.HTTP_204_NO_CONTENT)
async def receive(webhook: ReceiveWebhook) -> None:
    """Inbound SMS webhook from AT/Safaricom."""
    _LOGGER.info("Inbound SMS from %s: %s", webhook.from_, webhook.text)


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
