from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field


class TurnoutIngest(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    station_code: str
    hour: int = Field(ge=0, le=23)
    registered_voters: int = Field(gt=0)
    votes_cast_so_far: int = Field(ge=0)
    recorded_at: datetime


class TurnoutForecast(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    forecast_id: str
    station_code: str
    forecasted_turnout_pct: float = Field(ge=0.0, le=100.0)
    lower_bound_pct: float
    upper_bound_pct: float
    variance_alert: bool
    generated_at: datetime


class VarianceAlert(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    alert_id: str
    station_code: str
    forecasted_pct: float
    actual_pct: float
    deviation_pct: float
    alerted_at: datetime
