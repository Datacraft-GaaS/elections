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

"""M13: Voter registration status check via WhatsApp."""

from __future__ import annotations

import logging

_log_ = logging.getLogger(__name__)


async def voter_verify_flow(voter_id_hash: str) -> str:
    """Return a text response for voter registration status.

    In production, queries the voter register API with the hashed ID.
    Returns a plain-text response suitable for a WhatsApp message.
    """
    assert voter_id_hash, "voter_id_hash required"
    _log_.info("verifier.flow voter_id_hash=%s", voter_id_hash[:8])
    # Stub: real impl calls the voter register API
    return (
        f"Registration status for ID ending ...{voter_id_hash[-4:]}:\n"
        "STATUS: REGISTERED\n"
        "Station: TBD - check closer to election day."
    )
