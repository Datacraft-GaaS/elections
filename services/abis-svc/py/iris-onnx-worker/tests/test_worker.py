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

import base64

import numpy as np
import pytest
from httpx import ASGITransport, AsyncClient

from eel_iris_worker.main import app


def make_request(width: int = 64, height: int = 64, fill: int = 128) -> dict:
    pixels = np.full((height, width), fill, dtype=np.uint8)
    image_b64 = base64.b64encode(pixels.tobytes()).decode()
    return {"image_b64": image_b64, "width": width, "height": height}


@pytest.fixture
async def client():
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as c:
        yield c


async def test_health(client: AsyncClient):
    r = await client.get("/health")
    assert r.status_code == 200
    data = r.json()
    assert data["status"] == "ok"
    assert "models_loaded" in data


async def test_segment_returns_circles(client: AsyncClient):
    r = await client.post("/segment", json=make_request())
    assert r.status_code == 200
    data = r.json()
    assert "pupil" in data
    assert "limbus" in data
    assert data["pupil"]["radius"] > 0
    assert data["limbus"]["radius"] > data["pupil"]["radius"]


async def test_segment_mask_b64_correct_length(client: AsyncClient):
    req = make_request(width=32, height=32)
    r = await client.post("/segment", json=req)
    assert r.status_code == 200
    mask = base64.b64decode(r.json()["mask_b64"])
    assert len(mask) == 32 * 32


async def test_encode_returns_correct_sizes(client: AsyncClient):
    r = await client.post("/encode", json=make_request())
    assert r.status_code == 200
    data = r.json()
    code = base64.b64decode(data["iris_code_b64"])
    mask = base64.b64decode(data["mask_b64"])
    assert len(code) == 256
    assert len(mask) == 256


async def test_encode_quality_in_range(client: AsyncClient):
    r = await client.post("/encode", json=make_request())
    assert r.status_code == 200
    q = r.json()["quality"]
    assert 0.0 <= q <= 1.0


async def test_quality_metrics_in_range(client: AsyncClient):
    r = await client.post("/quality", json=make_request())
    assert r.status_code == 200
    data = r.json()
    for key in ("sharpness", "contrast", "dilation", "occlusion_fraction", "overall"):
        assert 0.0 <= data[key] <= 1.0, f"{key} out of range: {data[key]}"


async def test_quality_gaze_angle_nonnegative(client: AsyncClient):
    r = await client.post("/quality", json=make_request())
    assert r.json()["gaze_angle_deg"] >= 0.0


async def test_lens_detect_returns_kind(client: AsyncClient):
    r = await client.post("/lens-detect", json=make_request())
    assert r.status_code == 200
    data = r.json()
    assert data["kind"] in ("none", "transparent", "coloured", "patterned")
    assert 0.0 <= data["confidence"] <= 1.0


async def test_lens_detect_plain_image_no_lens(client: AsyncClient):
    r = await client.post("/lens-detect", json=make_request(fill=128))
    assert r.json()["has_lens"] is False


async def test_embed_returns_256d(client: AsyncClient):
    r = await client.post("/embed", json=make_request())
    assert r.status_code == 200
    emb = r.json()["embedding"]
    assert len(emb) == 256


async def test_embed_is_unit_norm(client: AsyncClient):
    r = await client.post("/embed", json=make_request())
    emb = r.json()["embedding"]
    norm = float(sum(x**2 for x in emb) ** 0.5)
    assert abs(norm - 1.0) < 0.01, f"norm {norm} not close to 1.0"


async def test_invalid_base64_returns_400(client: AsyncClient):
    req = {"image_b64": "!!!notbase64!!!", "width": 8, "height": 8}
    r = await client.post("/segment", json=req)
    assert r.status_code == 400


async def test_identical_images_identical_encode(client: AsyncClient):
    req = make_request(fill=100)
    r1 = await client.post("/encode", json=req)
    r2 = await client.post("/encode", json=req)
    assert r1.json()["iris_code_b64"] == r2.json()["iris_code_b64"]
