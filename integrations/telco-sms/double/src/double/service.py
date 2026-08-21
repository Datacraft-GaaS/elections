"""SMS double: simulates Africa's Talking send endpoint.

Knobs:
  SMS_LATENCY_MS   - ms delay per request
  SMS_ERROR_RATE   - fraction of requests returning 503
"""

from __future__ import annotations

import asyncio
import os
import random
import uuid

from fastapi import FastAPI
from pydantic import BaseModel, ConfigDict

_LATENCY_MS = int(os.environ.get("SMS_LATENCY_MS", "0"))
_ERROR_RATE = float(os.environ.get("SMS_ERROR_RATE", "0"))

app = FastAPI(title="eel-adapter-telco-sms-double")


class _ATSendIn(BaseModel):
    model_config = ConfigDict(extra="allow")
    username: str = "sandbox"
    to: str = ""
    message: str = ""


@app.post("/version1/messaging")
async def messaging(body: _ATSendIn) -> dict:
    if _LATENCY_MS > 0:
        await asyncio.sleep(_LATENCY_MS / 1000)
    if _ERROR_RATE > 0 and random.random() < _ERROR_RATE:
        from fastapi import Response

        return Response(status_code=503)
    recipients = [
        {
            "status": "Success",
            "statusCode": 101,
            "number": num.strip(),
            "cost": "KES 0.8000",
            "messageId": str(uuid.uuid4()),
        }
        for num in body.to.split(",")
        if num.strip()
    ]
    return {
        "SMSMessageData": {
            "Message": "Sent to 1/1 Total Cost: KES 0.8000",
            "Recipients": recipients,
        }
    }


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
