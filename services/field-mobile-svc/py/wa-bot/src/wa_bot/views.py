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

"""WhatsApp webhook Pydantic models."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict


class WaTextBody(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    body: str


class WaMessage(BaseModel):
    id: str
    from_: str
    to: str
    type: str
    text: WaTextBody | None = None

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, populate_by_name=True
    )


class WaChange(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    value: dict[str, Any]
    field: str


class WaEntry(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    id: str
    changes: list[WaChange]


class WhatsAppWebhookPayload(BaseModel):
    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    object: str
    entry: list[WaEntry]


class FlowResponse(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    text: str
    to: str
