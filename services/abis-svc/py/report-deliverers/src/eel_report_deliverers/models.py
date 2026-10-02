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

"""Pydantic models for delivery requests and receipts."""

from __future__ import annotations

from datetime import datetime
from typing import Any

from pydantic import BaseModel, ConfigDict


class DeliveryRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    artifact_ref: str
    artifact_id: str
    subscription_id: str
    format: str
    sha256: str


class DeliveryReceipt(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    # Gov printer
    submission_id: str | None = None
    submitted_at: datetime | None = None
    # KBC SFTP
    remote_path: str | None = None
    sha256: str | None = None
    uploaded_at: datetime | None = None


class VerifyResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    submission_id: str
    status: str
    verified_at: datetime | None = None
    detail: dict[str, Any] | None = None
