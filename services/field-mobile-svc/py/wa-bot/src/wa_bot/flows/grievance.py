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

"""M23: Voter grievance intake via WhatsApp."""

from __future__ import annotations

import logging

_log_ = logging.getLogger(__name__)


async def grievance_intake_flow(message: str) -> str:
    """Acknowledge and record a voter grievance.

    Returns a confirmation text for the WhatsApp reply.
    """
    assert message, "message required"
    _log_.info("grievance.flow message_len=%d", len(message))
    # Stub: real impl persists to grievance service
    return (
        "Your grievance has been recorded. Reference: GRV-STUB-001.\n"
        "A returning officer will respond within 48 hours.\n"
        "Dial *123# for urgent issues."
    )
