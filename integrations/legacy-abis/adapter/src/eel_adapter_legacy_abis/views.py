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

"""Legacy ABIS adapter models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class Match1NRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    probe_template: str = Field(
        ..., description="Base64-encoded fingerprint template (PII Restricted)"
    )
    modality: str = Field("fingerprint", description="fingerprint | iris | face")
    threshold: float = Field(0.8, ge=0.0, le=1.0)
    request_id: str


class Match11Request(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    probe_template: str = Field(
        ..., description="Base64-encoded probe template (PII Restricted)"
    )
    reference_id: str = Field(
        ..., description="Registered voter ID for reference lookup"
    )
    modality: str = Field("fingerprint")
    request_id: str


class MatchResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    request_id: str
    matched: bool
    score: float
    candidate_id: str | None = None
    error: str | None = None
