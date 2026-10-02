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
from typing import Annotated, Literal, TypeAlias

from pydantic import AfterValidator, BaseModel, ConfigDict, Field


def _pos(v: float) -> float:
    assert v >= 0, "must be non-negative"
    return v


PositiveFloat: TypeAlias = Annotated[float, AfterValidator(_pos)]


class TelemetryPoint(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    station_code: str
    timestamp: datetime
    votes_cast: int = Field(ge=0)
    ballot_papers_issued: int = Field(ge=0)
    spoilt_ballots: int = Field(ge=0)
    kit_id: str


class AnomalyAlert(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    alert_id: str
    station_code: str
    layer: str
    score: PositiveFloat
    severity: Literal["low", "medium", "high", "critical"]
    details: dict[str, str | float | int]
    detected_at: datetime
    event_type: str = "eel.anomaly.alert.v1"


class AlertBatch(BaseModel):
    model_config = ConfigDict(extra="forbid")

    alerts: list[AnomalyAlert]
    batch_id: str
    generated_at: datetime
