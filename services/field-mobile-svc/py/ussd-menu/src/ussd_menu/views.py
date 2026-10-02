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

"""USSD request/response models (WP-4.7, M13)."""

from __future__ import annotations

from enum import IntEnum

from pydantic import BaseModel, ConfigDict


class UssdRequest(BaseModel):
    """Africa's Talking USSD webhook payload (form-encoded fields)."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    sessionId: str
    phoneNumber: str
    networkCode: str
    serviceCode: str
    text: str = ""


class UssdResponse(BaseModel):
    """Menu response returned to the gateway."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    text: str
    is_final: bool = False


class MenuStep(IntEnum):
    MAIN = 1
    VERIFY_TOKEN_PART1 = 2
    VERIFY_TOKEN_PART2 = 3
    RESULTS = 4
    GRIEVANCE_TYPE = 5
    GRIEVANCE_CONFIRM = 6


# Maximum characters per USSD page (FR-M13-029)
USSD_MAX_CHARS = 182
