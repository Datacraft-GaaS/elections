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

_logger = logging.getLogger(__name__)


class AuditLogger:
    """Structured audit logger for governance admin actions.

    Emits a single structured record per action. Deployments can attach a
    handler to the ``governance_app.audit`` logger to persist records to the
    audit shadow store.
    """

    def log_action(
        self, actor: str, action: str, target_id: str, detail: str = ""
    ) -> None:
        _logger.info(
            "audit actor=%s action=%s target_id=%s detail=%s",
            actor,
            action,
            target_id,
            detail,
            extra={
                "audit_event": {
                    "actor": actor,
                    "action": action,
                    "target_id": target_id,
                    "detail": detail,
                }
            },
        )
