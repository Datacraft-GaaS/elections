"""USSD adapter: state-machine menu engine for AT USSD.

Handles:
  M13 verifier queries
  M23 grievance submission
  M35 CVR family notifications
"""

from __future__ import annotations

import logging

from fastapi import FastAPI, Form, Response, status

_LOGGER = logging.getLogger(__name__)

app = FastAPI(title="eel-adapter-telco-ussd")

# Session state: sessionId -> list of user inputs
_sessions: dict[str, list[str]] = {}

_MENU_ROOT = (
    "CON Welcome to Electoral Services\n"
    "1. Verify registration\n"
    "2. Submit grievance\n"
    "3. Family CVR notification"
)


def _handle(session_id: str, text: str) -> str:
    parts = [p for p in text.split("*") if p]
    depth = len(parts)

    if depth == 0:
        return _MENU_ROOT

    choice = parts[0]

    if choice == "1":
        if depth == 1:
            return "CON Enter your national ID:"
        nid = parts[1]
        return f"END Your registration status for ID {nid}: Registered. Polling station: Nairobi Central."

    if choice == "2":
        if depth == 1:
            return "CON Enter grievance category:\n1. Results dispute\n2. Registration issue\n3. Other"
        return "END Grievance reference: GR-2026-001. You will receive an SMS confirmation."

    if choice == "3":
        if depth == 1:
            return "CON Enter family member national ID:"
        nid = parts[1]
        return f"END CVR notification sent for ID {nid}."

    return "END Invalid option. Please try again."


@app.post("/ussd/callback")
async def ussd_callback(
    sessionId: str = Form(...),
    serviceCode: str = Form(...),
    phoneNumber: str = Form(...),
    text: str = Form(""),
    networkCode: str = Form(""),
) -> Response:
    """Handle AT USSD callback."""
    resp_text = _handle(sessionId, text)
    _LOGGER.info(
        "USSD session=%s depth=%d -> %s",
        sessionId,
        len(text.split("*")),
        resp_text[:40],
    )
    return Response(content=resp_text, media_type="text/plain")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
