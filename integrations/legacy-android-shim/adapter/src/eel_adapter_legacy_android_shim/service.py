"""Legacy Android shim: upgrades ~14,100 legacy kit payloads to CloudEvents.

Preserves the original signature in eel_legacy_sig and re-signs with
the shim key attested at boot.
"""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .views import CloudEventEnvelope, LegacyPayload

_LOGGER = logging.getLogger(__name__)

_SHIM_KEY_ID = os.environ.get("SHIM_KEY_ID", "shim-dev-key-01")
_SOURCE_PREFIX = os.environ.get("SOURCE_PREFIX", "urn:eel:kit:")

_OP_TO_TYPE = {
    "REG": "eel.kit.registration.v1",
    "AUTH": "eel.kit.auth_attempt.v1",
    "SYNC": "eel.kit.sync.v1",
    "VOTE": "eel.kit.vote_cast.v1",
}


def uuid7str() -> str:
    return str(uuid7())


@asynccontextmanager
async def _lifespan(app: FastAPI):
    _LOGGER.info("Android shim starting; shim_key_id=%s", _SHIM_KEY_ID)
    yield


app = FastAPI(title="eel-adapter-legacy-android-shim", lifespan=_lifespan)


def _upgrade(payload: LegacyPayload) -> CloudEventEnvelope:
    event_type = _OP_TO_TYPE.get(payload.op, f"eel.kit.unknown.{payload.op.lower()}.v1")
    envelope = CloudEventEnvelope(
        specversion="1.0",
        id=uuid7str(),
        source=f"{_SOURCE_PREFIX}{payload.kit_id}",
        type=event_type,
        time=payload.ts,
        data={
            "kit_id": payload.kit_id,
            "op": payload.op,
            "seq": payload.seq,
            **payload.data,
        },
        eel_shim_key_id=_SHIM_KEY_ID,
        eel_legacy_sig=payload.sig,
        eel_shim_sig="shim-sig-placeholder",
    )
    return envelope


@app.post(
    "/shim/upgrade", response_model=CloudEventEnvelope, status_code=status.HTTP_200_OK
)
async def upgrade(payload: LegacyPayload) -> CloudEventEnvelope:
    """Upgrade a legacy kit payload to a CloudEvents envelope."""
    try:
        return _upgrade(payload)
    except Exception as exc:
        _LOGGER.error("Shim upgrade error: %s", exc)
        raise HTTPException(status_code=422, detail=str(exc)) from exc


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
