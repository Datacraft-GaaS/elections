"""Android shim double: replays legacy payload upgrade responses."""

from __future__ import annotations

from fastapi import FastAPI
from pydantic import BaseModel, ConfigDict

app = FastAPI(title="eel-adapter-legacy-android-shim-double")


class _LegacyIn(BaseModel):
    model_config = ConfigDict(extra="allow")
    kit_id: str = "kit-001"
    op: str = "REG"
    ts: str = "2026-08-21T00:00:00Z"


@app.post("/shim/upgrade")
async def upgrade(body: _LegacyIn) -> dict:
    return {
        "specversion": "1.0",
        "id": "019221e0-0000-7000-8000-000000000001",
        "source": f"urn:eel:kit:{body.kit_id}",
        "type": f"eel.kit.{body.op.lower()}.v1",
        "time": body.ts,
        "datacontenttype": "application/json",
        "data": {"kit_id": body.kit_id, "op": body.op},
        "eel_shim_key_id": "shim-dev-key-01",
        "eel_legacy_sig": None,
        "eel_shim_sig": "double-sig",
    }


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
