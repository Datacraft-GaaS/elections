"""Integration tests for the FastAPI service.

Uses httpx.AsyncClient against the ASGI app directly.
All tests use the hash-based pseudo-embedding (no ONNX model).
"""

from __future__ import annotations

import base64

import pytest
from httpx import ASGITransport, AsyncClient

from eel_fp_dl_worker.main import app
from eel_fp_dl_worker.model import EMBED_DIM


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _b64_solid(v: int = 128) -> str:
    return base64.b64encode(bytes([v] * (64 * 64))).decode()


def _b64_stripe() -> str:
    pix = []
    for y in range(64):
        row_val = 30 if y % 10 < 5 else 200
        pix.extend([row_val] * 64)
    return base64.b64encode(bytes(pix)).decode()


# ---------------------------------------------------------------------------
# Health
# ---------------------------------------------------------------------------


async def test_health_ok(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert "model_loaded" in data


async def test_health_model_not_loaded(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.get("/health")
    data = resp.json()
    assert data["model_loaded"] is False


# ---------------------------------------------------------------------------
# /embed
# ---------------------------------------------------------------------------


async def test_embed_returns_384_dim(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post("/embed", json={"image_b64": _b64_solid()})
    assert resp.status_code == 200
    data = resp.json()
    assert data["dim"] == EMBED_DIM
    assert len(data["embedding"]) == EMBED_DIM


async def test_embed_invalid_base64(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post("/embed", json={"image_b64": "!!! not base64 !!!"})
    assert resp.status_code == 422


async def test_embed_deterministic(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    img = _b64_stripe()
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        r1 = await client.post("/embed", json={"image_b64": img})
        r2 = await client.post("/embed", json={"image_b64": img})
    assert r1.json()["embedding"] == r2.json()["embedding"]


async def test_embed_values_bounded(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post("/embed", json={"image_b64": _b64_solid()})
    emb = resp.json()["embedding"]
    for v in emb:
        assert -1.1 <= v <= 1.1, f"embedding value {v} out of expected range"


# ---------------------------------------------------------------------------
# /match
# ---------------------------------------------------------------------------


async def test_match_identical_score_near_1(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    img = _b64_stripe()
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post("/match", json={"probe_b64": img, "gallery_b64": img})
    assert resp.status_code == 200
    score = resp.json()["score"]
    assert score > 0.95, f"identical images score={score}"


async def test_match_score_bounded(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/match", json={"probe_b64": _b64_solid(0), "gallery_b64": _b64_solid(255)}
        )
    assert resp.status_code == 200
    score = resp.json()["score"]
    assert 0.0 <= score <= 1.0


async def test_match_different_images_low_score(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/match", json={"probe_b64": _b64_solid(0), "gallery_b64": _b64_solid(255)}
        )
    score = resp.json()["score"]
    assert score < 0.5, f"very different images score={score} unexpectedly high"


async def test_match_invalid_probe(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EEL_FP_DL_ONNX_PATH", raising=False)
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/match", json={"probe_b64": "BAD!", "gallery_b64": _b64_solid()}
        )
    assert resp.status_code == 422
