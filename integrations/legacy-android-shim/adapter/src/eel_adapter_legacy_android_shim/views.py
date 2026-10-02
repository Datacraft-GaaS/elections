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

"""Legacy Android shim models."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class LegacyPayload(BaseModel):
    """Documented legacy pre-CloudEvents kit payload format."""

    model_config = ConfigDict(extra="allow")

    kit_id: str = Field(..., description="Legacy kit device ID")
    op: str = Field(..., description="Operation code e.g. 'REG', 'AUTH', 'SYNC'")
    ts: str = Field(..., description="Timestamp ISO-8601")
    data: dict[str, Any] = Field(default_factory=dict)
    sig: str | None = Field(None, description="Legacy ECDSA signature hex")
    seq: int | None = None


class CloudEventEnvelope(BaseModel):
    """eel CloudEvents 1.0 envelope."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    specversion: str = "1.0"
    id: str
    source: str
    type: str
    time: str
    datacontenttype: str = "application/json"
    data: dict[str, Any]
    # eel platform extensions
    eel_shim_key_id: str | None = None
    eel_legacy_sig: str | None = None
    eel_shim_sig: str | None = None
