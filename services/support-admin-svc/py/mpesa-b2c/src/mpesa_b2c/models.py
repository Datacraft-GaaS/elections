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

"""Pydantic v2 models for M-Pesa B2C disbursement (WP-5.6)."""

from typing import Annotated

from pydantic import AfterValidator, BaseModel, ConfigDict, field_validator


def _positive_amount(v: int) -> int:
    if v <= 0:
        raise ValueError("amount_kes must be positive")
    return v


PositiveKes = Annotated[int, AfterValidator(_positive_amount)]


class DisbursementItem(BaseModel):
    """A single B2C disbursement line."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    staff_id: str
    phone_number: str
    amount_kes: PositiveKes
    remarks: str = "Salary disbursement"


class BatchRequest(BaseModel):
    """A batch of B2C disbursement items submitted together."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    batch_id: str
    items: list[DisbursementItem]


class ItemResult(BaseModel):
    """Per-item outcome from a B2C batch submission."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    staff_id: str
    phone_number: str
    status: str
    conversation_id: str | None = None
    originator_conversation_id: str | None = None
    error: str | None = None


class BatchResult(BaseModel):
    """Aggregate result from a B2C batch submission."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    batch_id: str
    total: int
    succeeded: int
    failed: int
    items: list[ItemResult]


class B2CCallback(BaseModel):
    """M-Pesa callback payload for a B2C result notification."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    Result: dict
