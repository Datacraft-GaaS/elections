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

"""TTS worker tests. Target: 4+ passing."""

from __future__ import annotations

import asyncio
import os

import pytest

os.environ.setdefault("EEL_TTS_ADAPTER", "stub")


# ---------------------------------------------------------------------------
# Stub adapter tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_stub_adapter_produces_valid_mp3_header():
    from eel_tts.synth import StubAdapter

    adapter = StubAdapter()
    data = await adapter.synthesise("Hello world", "en")
    # ID3v2 magic bytes
    assert data[:3] == b"ID3", "should start with ID3 header"
    # MPEG sync word somewhere after ID3 header
    assert b"\xff\xfb" in data, "should contain MPEG sync bytes"
    assert len(data) > 50, "should be non-trivially sized"


@pytest.mark.asyncio
async def test_stub_adapter_returns_bytes_for_any_lang():
    from eel_tts.synth import StubAdapter

    adapter = StubAdapter()
    for lang in ["en", "sw", "ki", "luo"]:
        data = await adapter.synthesise("Test", lang)
        assert isinstance(data, bytes)
        assert len(data) > 0


# ---------------------------------------------------------------------------
# Render endpoint tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_render_endpoint_returns_200():
    from httpx import ASGITransport, AsyncClient

    from eel_tts.main import app

    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/tts/render",
            json={
                "question_id": "q-001",
                "lang": "en",
                "text": "Should the constitution be amended?",
            },
        )
    assert resp.status_code == 200
    body = resp.json()
    assert "audio_url" in body
    assert body["duration_ms"] == 1000


@pytest.mark.asyncio
async def test_render_endpoint_blank_text_returns_422():
    from httpx import ASGITransport, AsyncClient

    from eel_tts.main import app

    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        resp = await client.post(
            "/tts/render",
            json={"question_id": "q-001", "lang": "en", "text": "   "},
        )
    assert resp.status_code == 422


# ---------------------------------------------------------------------------
# Batch worker test
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_batch_worker_handles_missing_aiokafka_gracefully():
    """Worker should log and return cleanly when aiokafka is absent."""
    import sys

    # Temporarily hide aiokafka if present
    real_aiokafka = sys.modules.pop("aiokafka", None)
    try:
        # Re-import to ensure the import path is re-evaluated
        import importlib

        import eel_tts.queue as q_mod

        importlib.reload(q_mod)
        # Should complete without raising
        loop = asyncio.get_event_loop()
        await loop.create_task(q_mod.run_worker())
    finally:
        if real_aiokafka is not None:
            sys.modules["aiokafka"] = real_aiokafka
