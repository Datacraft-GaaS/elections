from __future__ import annotations

import asyncio
from datetime import datetime, timedelta, timezone

from httpx import ASGITransport, AsyncClient

from model_registry.service import app, models_store, decision_log_store


BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def _reg_payload(**overrides) -> dict:
    base = {
        "code": "eel-test-model",
        "name": "Test Model",
        "version": "1.0.0",
        "layer": "INTERNAL",
        "architecture": "transformer",
        "training_data_description": "synthetic civic data",
        "eval_accuracy": 0.92,
        "eval_bias_flag": False,
        "known_limitations": "low coverage on dialects",
        "human_in_loop": True,
        "sovereign_hosting": True,
        "rollback_procedure": "revert to previous tag",
    }
    base.update(overrides)
    return base


async def _register(payload: dict | None = None) -> dict:
    async with _client() as c:
        r = await c.post("/v1/ai/registry/models", json=payload or _reg_payload())
    assert r.status_code == 201, r.text
    return r.json()


def setup_function(_):
    models_store.clear()
    decision_log_store.clear()


def test_health_ok():
    async def _run():
        async with _client() as c:
            r = await c.get("/v1/ai/registry/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_register_model():
    async def _run():
        data = await _register()
        assert "id" in data
        assert data["status"] == "PENDING_IAB"
        assert data["is_active"] is False

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_get_model():
    async def _run():
        created = await _register()
        model_id = created["id"]
        async with _client() as c:
            r = await c.get(f"/v1/ai/registry/models/{model_id}")
        assert r.status_code == 200
        assert r.json()["id"] == model_id

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_activate_requires_iab():
    async def _run():
        created = await _register()
        model_id = created["id"]
        async with _client() as c:
            r = await c.patch(
                f"/v1/ai/registry/models/{model_id}/activate",
                json={"ccb_approval_id": "CCB-001", "hsm_signature": "sig"},
            )
        assert r.status_code == 422

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_activate_success():
    async def _run():
        iab_time = (datetime.now(timezone.utc) - timedelta(hours=73)).isoformat()
        payload = _reg_payload(iab_notified_at=iab_time)
        created = await _register(payload)
        model_id = created["id"]
        async with _client() as c:
            r = await c.patch(
                f"/v1/ai/registry/models/{model_id}/activate",
                json={"ccb_approval_id": "CCB-002", "hsm_signature": "sig"},
            )
        assert r.status_code == 200
        body = r.json()
        assert body["is_active"] is True
        assert body["status"] == "ACTIVE"
        assert body["ccb_approval_id"] == "CCB-002"

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_decision_log():
    async def _run():
        raw_input = "voter cast ballot"
        payload = {
            "model_id": "model-abc",
            "model_version": "1.0",
            "interaction_type": "classification",
            "input_hash": raw_input,
            "confidence_score": 0.87,
            "model_output": {"label": "valid"},
        }
        async with _client() as c:
            r = await c.post("/v1/ai/registry/decision-log", json=payload)
        assert r.status_code == 201
        body = r.json()
        assert body["input_hash"] != raw_input
        assert "id" in body

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
