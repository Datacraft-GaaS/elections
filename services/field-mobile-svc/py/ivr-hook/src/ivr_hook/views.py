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

"""IVR Pydantic models."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class AtVoicePayload(BaseModel):
    """Africa's Talking voice webhook payload (form-encoded)."""

    model_config = ConfigDict(extra="ignore", validate_by_name=True)

    sessionId: str
    phoneNumber: str
    dtmfDigits: str | None = None
    isActive: str = "1"


class IvrSession(BaseModel):
    """In-memory IVR session state."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    session_id: str
    phone: str
    state: str = "main_menu"
    lang: str = "en"


class SsmlResponse(BaseModel):
    """SSML/text response to return to Africa's Talking."""

    model_config = ConfigDict(extra="forbid", validate_by_name=True)

    content: str
    is_terminal: bool = False
