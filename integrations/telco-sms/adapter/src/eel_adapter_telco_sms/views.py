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

"""SMS adapter models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class SendRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    to: list[str] = Field(..., description="MSISDN list e.g. +254...")
    message: str
    sender_id: str | None = None
    request_id: str = Field(..., description="UUIDv7 idempotency key")
    provider: str = Field(
        "at", description="'at' = Africa's Talking, 'smpp' = Safaricom SMPP"
    )


class SendResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    accepted: int
    rejected: int
    message_ids: list[str]


class ReceiveWebhook(BaseModel):
    model_config = ConfigDict(extra="allow")

    from_: str = Field(..., alias="from")
    to: str
    text: str
    date: str
    id: str
