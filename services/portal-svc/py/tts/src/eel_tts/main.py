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

"""FastAPI app for the eel TTS worker (M68)."""

from __future__ import annotations

import os
import uuid
from pathlib import Path

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, ConfigDict

from .synth import TTSAdapter, get_adapter

app = FastAPI(title="eel-portal-tts", version="0.1.0")

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

_OUTPUT_DIR = Path(os.environ.get("EEL_TTS_OUTPUT_DIR", "/tmp/eel-tts"))
_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Adapter is resolved once at startup (can be overridden in tests via DI).
_adapter: TTSAdapter | None = None


def _get_adapter() -> TTSAdapter:
    global _adapter
    if _adapter is None:
        _adapter = get_adapter()
    return _adapter


# ---------------------------------------------------------------------------
# Models
# ---------------------------------------------------------------------------


class TTSRenderRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    question_id: str
    lang: str
    text: str


class TTSRenderResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    audio_url: str
    duration_ms: int


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------


@app.post("/tts/render", response_model=TTSRenderResponse)
async def render(req: TTSRenderRequest) -> TTSRenderResponse:
    if not req.text.strip():
        raise HTTPException(status_code=422, detail="text must not be blank")
    if not req.lang.strip():
        raise HTTPException(status_code=422, detail="lang must not be blank")

    adapter = _get_adapter()
    mp3_bytes = await adapter.synthesise(req.text, req.lang)

    file_name = f"{req.question_id}-{req.lang}-{uuid.uuid4().hex[:8]}.mp3"
    out_path = _OUTPUT_DIR / file_name
    out_path.write_bytes(mp3_bytes)

    # Duration stub: 1000 ms (real implementation would decode MP3 header)
    duration_ms = 1000

    audio_url = f"file://{out_path}"
    minio_base = os.environ.get("EEL_TTS_MINIO_BASE")
    if minio_base:
        audio_url = f"{minio_base.rstrip('/')}/{req.question_id}/{req.lang}/{file_name}"

    return TTSRenderResponse(audio_url=audio_url, duration_ms=duration_ms)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    return {"status": "ok"}
