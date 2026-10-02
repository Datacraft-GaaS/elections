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

import logging
from typing import Any

_log_ = logging.getLogger(__name__)


def log_status_transition(
    content_item_id: str,
    actor_id: str | None,
    from_status: str,
    to_status: str,
    reason: str | None = None,
) -> None:
    _log_.info(
        "content_item status transition: id=%s actor=%s %s -> %s reason=%s",
        content_item_id,
        actor_id,
        from_status,
        to_status,
        reason,
    )


def log_distribution_event(
    content_item_id: str,
    channel: str,
    county_code: str,
    quantity: int,
    extra: dict[str, Any] | None = None,
) -> None:
    _log_.info(
        "distribution event: content_item_id=%s channel=%s county=%s qty=%d extra=%s",
        content_item_id,
        channel,
        county_code,
        quantity,
        extra,
    )
