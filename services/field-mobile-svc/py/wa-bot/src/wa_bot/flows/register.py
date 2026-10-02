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

"""M35: Voter register lookup via WhatsApp."""

from __future__ import annotations

import logging

_log_ = logging.getLogger(__name__)


async def register_lookup_flow(national_id_hash: str) -> str:
    """Return register lookup result for the given hashed national ID.

    In production, queries the voter register microservice.
    """
    assert national_id_hash, "national_id_hash required"
    _log_.info("register.flow id_hash=%s", national_id_hash[:8])
    # Stub: real impl calls voter register API
    return (
        f"Register lookup for ID ending ...{national_id_hash[-4:]}:\n"
        "FOUND: YES\n"
        "Polling station: Confirm at eel.example"
    )
