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


class HeadcountRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    station_code: str
    image_source: str
    captured_at: datetime


class HeadcountResult(BaseModel):
    model_config = ConfigDict(extra="forbid")
    result_id: str
    station_code: str
    headcount: int
    confidence: float = Field(ge=0.0, le=1.0)
    source: str
    processed_at: datetime
