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

"""FastAPI application: /health + /deliver/{channel} + /verify/{submission_id}."""

from __future__ import annotations

from datetime import datetime, timezone

from fastapi import FastAPI, HTTPException

from . import gov_printer as gp
from . import kbc_sftp as kbc
from .models import DeliveryReceipt, DeliveryRequest, VerifyResponse

app = FastAPI(title="eel-report-deliverers", version="0.1.0")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "eel-report-deliverers"}


@app.post("/deliver/gov-printer", response_model=DeliveryReceipt)
async def deliver_gov_printer(req: DeliveryRequest) -> DeliveryReceipt:
    """Submit report artefact to the Gov Printer Gazette endpoint."""
    try:
        return await gp.deliver(req)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc


@app.post("/deliver/kbc-sftp", response_model=DeliveryReceipt)
async def deliver_kbc_sftp(req: DeliveryRequest) -> DeliveryReceipt:
    """Bundle and upload report artefact to KBC SFTP."""
    try:
        return await kbc.deliver(req)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc


@app.post("/deliver/{channel}", response_model=DeliveryReceipt)
async def deliver_generic(channel: str, req: DeliveryRequest) -> DeliveryReceipt:
    """Generic dispatch by channel name."""
    if channel == "gov-printer":
        return await deliver_gov_printer(req)
    if channel == "kbc-sftp":
        return await deliver_kbc_sftp(req)
    raise HTTPException(status_code=404, detail=f"unknown channel: {channel}")


@app.get("/verify/{submission_id}", response_model=VerifyResponse)
async def verify(submission_id: str) -> VerifyResponse:
    """Verify a prior submission. Stub: always returns 'accepted'."""
    return VerifyResponse(
        submission_id=submission_id,
        status="accepted",
        verified_at=datetime.now(timezone.utc),
        detail=None,
    )
