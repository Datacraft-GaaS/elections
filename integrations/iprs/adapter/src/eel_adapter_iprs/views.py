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

"""Request/response models for the IPRS adapter."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class VerifyRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    national_id: str = Field(
        ..., description="14-digit national ID number (PII Restricted)"
    )
    full_name: str | None = Field(
        None, description="Full name for cross-check (PII Restricted)"
    )
    dob: str | None = Field(None, description="Date of birth ISO-8601 (PII Restricted)")
    request_id: str = Field(..., description="UUIDv7 idempotency key")


class VerifyResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    matched: bool
    duplicate: bool = False
    status_code: str
    message: str
