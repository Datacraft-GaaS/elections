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
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict


class DetectionReportStatus(str, Enum):
    PENDING = "PENDING"
    PUBLISHED = "PUBLISHED"
    ANCHORED = "ANCHORED"


class WeeklyDetectionReport(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    id: str
    report_week: str
    layer_summaries: dict[str, Any]
    pii_strip_stats: dict[str, Any]
    published_at: datetime | None
    tlog_entry: str | None
    status: DetectionReportStatus


class ModelDecisionSummary(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    model_id: str
    model_version: str
    decision_count: int
    human_override_count: int
    avg_confidence: float
    period_start: datetime
    period_end: datetime


class PiiStripStats(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_by_name=True, validate_by_alias=True)

    total_calls: int
    total_tokens_replaced: int
    bypass_count: int
    period_start: datetime
    period_end: datetime
