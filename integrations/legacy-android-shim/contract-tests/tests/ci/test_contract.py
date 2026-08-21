"""Legacy Android shim contract tests against the double.

Verifies all 20 golden legacy payloads upgrade correctly.
"""

from __future__ import annotations

import json
import os
from pathlib import Path

import httpx
import pytest

DOUBLE_URL = os.environ.get("SHIM_DOUBLE_URL", "http://localhost:8090")
FIXTURES = Path(__file__).parent.parent.parent / "fixtures" / "legacy_payloads.json"


@pytest.fixture
def golden_payloads() -> list[dict]:
    return json.loads(FIXTURES.read_text())


@pytest.mark.asyncio
async def test_golden_corpus(golden_payloads: list[dict]) -> None:
    """All 20 documented legacy payloads must upgrade to valid CloudEvents envelopes."""
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        for payload in golden_payloads:
            resp = await client.post("/shim/upgrade", json=payload)
            assert resp.status_code == 200, f"Failed for kit_id={payload['kit_id']}"
            env = resp.json()
            assert env["specversion"] == "1.0"
            assert "id" in env
            assert "source" in env
            assert "type" in env
            assert env["data"]["kit_id"] == payload["kit_id"]


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
