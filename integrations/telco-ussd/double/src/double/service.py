"""USSD double: simulates AT USSD callback for testing.

Knobs:
  USSD_LATENCY_MS - ms delay
"""

from __future__ import annotations

import asyncio
import os

from fastapi import FastAPI, Form, Response

_LATENCY_MS = int(os.environ.get("USSD_LATENCY_MS", "0"))

app = FastAPI(title="eel-adapter-telco-ussd-double")

_sessions: dict[str, list[str]] = {}


@app.post("/ussd/callback")
async def ussd_callback(
    sessionId: str = Form(...),
    serviceCode: str = Form(...),
    phoneNumber: str = Form(...),
    text: str = Form(""),
    networkCode: str = Form(""),
) -> Response:
    if _LATENCY_MS > 0:
        await asyncio.sleep(_LATENCY_MS / 1000)
    if not text:
        return Response(
            content="CON Welcome\n1. Verify\n2. Grievance\n3. Notification",
            media_type="text/plain",
        )
    return Response(content="END OK", media_type="text/plain")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
