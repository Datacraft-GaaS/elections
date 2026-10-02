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

from pydantic import BaseModel, ConfigDict


class AuditMethod(str):
    BALLOT_POLLING = "ballot_polling"
    COMPARISON = "comparison"
    BATCH_COMPARISON = "batch_comparison"


class AuditRun(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    run_id: str
    election_id: str
    position_code: str
    audit_method: str
    risk_limit: float
    seed: str | None = None
    status: str = "initialised"


class SampledBallot(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    ballot_id: str
    run_id: str
    station_code: str
    batch_id: str
    ballot_position: int
    auditor_interpretation: str | None = None
    discrepancy: bool = False


class TStatResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    t_statistic: float
    risk_achieved: float
    passed: bool
    recount_triggered: bool


class WitnessRecord(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    witness_id: str
    run_id: str
    name: str
    organisation: str
    signature_b64: str | None = None
    witnessed_at: str | None = None
