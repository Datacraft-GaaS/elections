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

"""Pydantic v2 request/response models for eel-cvr-ocr."""

from __future__ import annotations

from enum import Enum
from typing import Annotated

from pydantic import BaseModel, ConfigDict, Field, AfterValidator


def _nonempty(v: str) -> str:
    assert len(v) > 0, "must not be empty"
    return v


NonEmpty = Annotated[str, AfterValidator(_nonempty)]


class OcrModality(str, Enum):
    death_certificate = "death_certificate"
    id_correction = "id_correction"


class OcrRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    # Raw image bytes are received as multipart form upload, not JSON.
    # This model describes the JSON metadata part of the request.
    modality: OcrModality
    image_filename: NonEmpty = Field(description="Original filename for audit trail")
    hint_national_id: str | None = Field(
        default=None,
        description="Optional pre-filled national_id to anchor OCR extraction",
    )


class DeathCertificateFields(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    deceased_name: str | None = None
    national_id: str | None = None
    date_of_death: str | None = None
    place_of_death: str | None = None
    cert_serial: str | None = None


class IdCorrectionFields(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    full_name: str | None = None
    national_id: str | None = None
    date_of_birth: str | None = None
    gender: str | None = None
    county: str | None = None


class OcrResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    modality: OcrModality
    confidence: float = Field(ge=0.0, le=1.0)
    extracted: DeathCertificateFields | IdCorrectionFields
    adapter: str = Field(description="'tesseract' or 'stub'")
    warnings: list[str] = Field(default_factory=list)


class HealthResponse(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    status: str
    tesseract_available: bool
    version: str
