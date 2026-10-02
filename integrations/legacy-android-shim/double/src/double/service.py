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
