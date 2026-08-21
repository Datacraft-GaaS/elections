from datetime import datetime
from typing import Annotated

from pydantic import BaseModel, ConfigDict, field_validator


def _non_empty(v: str) -> str:
    if not v.strip():
        raise ValueError("must not be blank")
    return v


NonEmptyStr = Annotated[str, field_validator("*", mode="before")]


class GlPostingRequest(BaseModel):
    """General ledger posting request sent to the IFMIS adapter."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    voucher_id: str
    budget_line_code: str
    amount_kes: int
    description: str
    posting_date: datetime
    initiated_by: str
    approved_by: str


class GlPostingResponse(BaseModel):
    """Response from the IFMIS adapter after a successful GL posting."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    ifmis_ref: str
    voucher_id: str
    status: str
    posted_at: datetime


class IfmisVoucher(BaseModel):
    """IFMIS voucher representation returned by the adapter."""

    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )

    ifmis_ref: str
    voucher_id: str
    budget_line_code: str
    amount_kes: int
    description: str
    status: str
    posting_date: datetime
    created_at: datetime
