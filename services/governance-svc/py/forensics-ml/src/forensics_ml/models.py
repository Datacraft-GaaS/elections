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
from enum import Enum
from pydantic import BaseModel, ConfigDict


class AnalysisMethod(str, Enum):
    RD = "RD"
    IV = "IV"
    DID = "DID"


class RdInput(BaseModel):
    model_config = ConfigDict(extra="forbid")
    constituency_id: str
    vote_share: list[float]
    expenditure: list[float]
    threshold: float  # CCT bandwidth threshold
    bandwidth: float | None = None


class IvInput(BaseModel):
    model_config = ConfigDict(extra="forbid")
    constituency_id: str
    outcome: list[float]
    endogenous: list[float]
    instrument: list[float]


class DiDInput(BaseModel):
    model_config = ConfigDict(extra="forbid")
    constituency_id: str
    outcome_pre: list[float]
    outcome_post: list[float]
    treated: list[bool]


class EstimateResult(BaseModel):
    model_config = ConfigDict(extra="forbid")
    constituency_id: str
    method: AnalysisMethod
    estimate: float
    ci_low: float
    ci_high: float
    p_value: float
    model_version: str = "1.0.0"


class ReplicationManifest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    cycle_id: str
    method: AnalysisMethod
    input_hash: str
    output_hash: str
    environment_spec: str
    produced_at: str
