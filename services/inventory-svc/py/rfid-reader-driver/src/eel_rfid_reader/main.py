"""FastAPI application for eel-rfid-reader-driver.

Endpoints:
  POST /reader/register
  POST /reader/{reader_id}/read
  POST /reader/{reader_id}/batch-read
  POST /reader/{reader_id}/challenge
  GET  /reader/{reader_id}/status
  GET  /health
"""

from __future__ import annotations

import datetime
import logging
from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, HTTPException, status
from uuid6 import uuid7

from .adapter import RfidReaderAdapter
from .adapters.synthetic import SyntheticAdapter
from .config import cfg
from .hmac_gen2v2 import record_nonce
from .models import (
    BatchReadRequest,
    ChallengeRequest,
    ChallengeResponse,
    ReadRequest,
    ReadResponse,
    ReaderRegistration,
    ReaderStatus,
    TagState,
    VerifyResult,
)
from .upstream import UpstreamClient

_log_ = logging.getLogger(__name__)


def _uuid7str() -> str:
    return str(uuid7())


def build_adapter() -> RfidReaderAdapter:
    """Instantiate the adapter from EEL_RFID_ADAPTER env."""
    adapter_type = cfg.adapter.lower()
    if adapter_type == "synthetic":
        return SyntheticAdapter(tamper_prob=cfg.synthetic_tamper_prob)
    if adapter_type == "impinj":
        from .adapters.impinj_llrp import ImpinjLlrpAdapter

        host = cfg.upstream_url  # overridden in prod by dedicated env
        import os

        host = os.environ.get("EEL_IMPINJ_HOST", "localhost")
        return ImpinjLlrpAdapter(host=host)
    if adapter_type in ("zebra", "zebra-llrp"):
        from .adapters.zebra_llrp import ZebraLlrpAdapter

        import os

        host = os.environ.get("EEL_ZEBRA_HOST", "localhost")
        return ZebraLlrpAdapter(host=host)
    if adapter_type == "cipherlab":
        from .adapters.cipherlab_rs36 import CipherLabRS36Adapter

        import os

        addr = os.environ.get("EEL_CIPHERLAB_ADDR", "00:00:00:00:00:00")
        return CipherLabRS36Adapter(device_address=addr)
    raise ValueError(f"unknown EEL_RFID_ADAPTER: {cfg.adapter!r}")


# Mutable state stored on the app instance.
_readers: dict[str, dict[str, Any]] = {}  # reader_id -> metadata
_adapter: RfidReaderAdapter | None = None
_upstream: UpstreamClient | None = None


@asynccontextmanager
async def _lifespan(app: FastAPI):  # type: ignore[type-arg]
    global _adapter, _upstream
    _adapter = build_adapter()
    _upstream = UpstreamClient()
    await _upstream.start()
    yield
    await _adapter.close()
    await _upstream.stop()


app = FastAPI(title="eel-rfid-reader-driver", lifespan=_lifespan)


# ---------------------------------------------------------------------------
# POST /reader/register
# ---------------------------------------------------------------------------


@app.post(
    "/reader/register",
    response_model=dict[str, str],
    status_code=status.HTTP_201_CREATED,
)
async def register_reader(reg: ReaderRegistration) -> dict[str, str]:
    """Register a reader (SPIFFE SVID + metadata)."""
    _readers[reg.reader_id] = {
        "svid": reg.svid,
        "location": reg.location,
        "reader_type": reg.reader_type,
        "firmware_version": reg.firmware_version,
        "registered_at": datetime.datetime.utcnow().isoformat(),
        "last_read_ts": None,
        "tag_count": 0,
    }
    _log_.info("reader registered: id=%s type=%s", reg.reader_id, reg.reader_type)
    return {"reader_id": reg.reader_id, "status": "registered"}


# ---------------------------------------------------------------------------
# POST /reader/{reader_id}/read
# ---------------------------------------------------------------------------


@app.post(
    "/reader/{reader_id}/read",
    response_model=ReadResponse,
)
async def submit_read(reader_id: str, req: ReadRequest) -> ReadResponse:
    """Accept a single-tag read from a registered reader."""
    _require_reader(reader_id)
    assert _adapter is not None
    request_id = _uuid7str()
    ts = datetime.datetime.utcnow().isoformat()

    _readers[reader_id]["last_read_ts"] = ts
    _readers[reader_id]["tag_count"] = _readers[reader_id].get("tag_count", 0) + 1

    tag_state = TagState.tampered if req.tamper_indicator else TagState.ready

    # Forward to upstream; non-fatal if upstream is down.
    if _upstream is not None:
        try:
            svid = _readers[reader_id].get("svid", "")
            await _upstream.forward_read(reader_id, req, svid=svid)
        except Exception as exc:  # noqa: BLE001
            _log_.warning("upstream forward failed for reader=%s: %s", reader_id, exc)

    return ReadResponse(
        request_id=request_id,
        tag_epc=req.tag_epc,
        accepted=True,
        tag_state=tag_state,
    )


# ---------------------------------------------------------------------------
# POST /reader/{reader_id}/batch-read
# ---------------------------------------------------------------------------


@app.post(
    "/reader/{reader_id}/batch-read",
    response_model=list[ReadResponse],
)
async def batch_read(reader_id: str, req: BatchReadRequest) -> list[ReadResponse]:
    """Accept a burst of up to 1000 tag reads."""
    _require_reader(reader_id)
    assert _adapter is not None and _upstream is not None
    ts = datetime.datetime.utcnow().isoformat()
    _readers[reader_id]["last_read_ts"] = ts
    _readers[reader_id]["tag_count"] = _readers[reader_id].get("tag_count", 0) + len(
        req.reads
    )

    results: list[ReadResponse] = []
    for read in req.reads:
        tag_state = TagState.tampered if read.tamper_indicator else TagState.ready
        results.append(
            ReadResponse(
                request_id=_uuid7str(),
                tag_epc=read.tag_epc,
                accepted=True,
                tag_state=tag_state,
            )
        )

    # Best-effort upstream forwarding in background.
    try:
        svid = _readers[reader_id].get("svid", "")
        await _upstream.forward_batch(reader_id, req.reads, svid=svid)
    except Exception as exc:  # noqa: BLE001
        _log_.warning("batch upstream forward error: %s", exc)

    return results


# ---------------------------------------------------------------------------
# POST /reader/{reader_id}/challenge
# ---------------------------------------------------------------------------


@app.post(
    "/reader/{reader_id}/challenge",
    response_model=ChallengeResponse,
)
async def issue_challenge(reader_id: str, req: ChallengeRequest) -> ChallengeResponse:
    """Issue a Gen2v2 HMAC challenge for a tag EPC."""
    _require_reader(reader_id)
    assert _adapter is not None
    nonce_hex, challenge_hex = await _adapter.issue_challenge(req.tag_epc)
    return ChallengeResponse(
        tag_epc=req.tag_epc,
        nonce_hex=nonce_hex,
        challenge_hex=challenge_hex,
    )


# ---------------------------------------------------------------------------
# GET /reader/{reader_id}/status
# ---------------------------------------------------------------------------


@app.get(
    "/reader/{reader_id}/status",
    response_model=ReaderStatus,
)
async def reader_status(reader_id: str) -> ReaderStatus:
    """Return health and last-read info for a reader."""
    _require_reader(reader_id)
    meta = _readers[reader_id]
    return ReaderStatus(
        reader_id=reader_id,
        healthy=True,
        last_read_ts=meta.get("last_read_ts"),
        tag_count=meta.get("tag_count", 0),
        adapter_type=cfg.adapter,
    )


# ---------------------------------------------------------------------------
# GET /health
# ---------------------------------------------------------------------------


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "adapter": cfg.adapter}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _require_reader(reader_id: str) -> None:
    if reader_id not in _readers:
        raise HTTPException(
            status_code=404,
            detail=f"reader {reader_id!r} not registered",
        )
