"""FastAPI USSD webhook accepting Africa's Talking and Safaricom SMSC payloads.

WP-4.7, M13 (FR-M13-024 to FR-M13-029).
"""

from __future__ import annotations

import logging

from fastapi import FastAPI, Form
from fastapi.responses import PlainTextResponse

from .fsm import parse_steps, route_menu

_log_ = logging.getLogger(__name__)

app = FastAPI(
    title="eel USSD Menu",
    description="USSD menu engine for ballot verification and grievance (WP-4.7)",
    version="0.1.0",
)


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "ussd-menu"}


@app.post("/ussd", response_class=PlainTextResponse)
async def ussd_webhook(
    sessionId: str = Form(...),
    phoneNumber: str = Form(...),
    networkCode: str = Form(...),
    serviceCode: str = Form(...),
    text: str = Form(default=""),
) -> str:
    """Africa's Talking USSD webhook (form-encoded)."""
    _log_.info(
        "ussd.webhook session=%s network=%s text=%r",
        sessionId,
        networkCode,
        text,
    )
    steps = parse_steps(text)
    response_text, is_final = route_menu(steps)
    prefix = "END" if is_final else "CON"
    return f"{prefix} {response_text}"


@app.post("/ussd/safaricom", response_class=PlainTextResponse)
async def ussd_safaricom_webhook(
    sessionId: str = Form(...),
    phoneNumber: str = Form(...),
    networkCode: str = Form(default="Safaricom"),
    serviceCode: str = Form(...),
    text: str = Form(default=""),
) -> str:
    """Safaricom SMSC variant; same FSM, different field names."""
    _log_.info(
        "ussd.safaricom session=%s text=%r",
        sessionId,
        text,
    )
    steps = parse_steps(text)
    response_text, is_final = route_menu(steps)
    prefix = "END" if is_final else "CON"
    return f"{prefix} {response_text}"
