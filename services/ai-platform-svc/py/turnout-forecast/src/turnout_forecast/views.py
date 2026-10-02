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
