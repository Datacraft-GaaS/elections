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

"""Audit shadow signals for AuditShadowMixin models."""

from __future__ import annotations

import logging
from typing import Any

from django.db.models.signals import post_delete, post_save
from django.dispatch import receiver

_log = logging.getLogger(__name__)


def connect_audit_shadow(model_class: type) -> None:
    """Connect audit shadow signals for a model."""

    @receiver(post_save, sender=model_class)
    def on_save(sender: type, instance: Any, created: bool, **kwargs: Any) -> None:
        op = "INSERT" if created else "UPDATE"
        _log.info("audit_shadow: %s on %s pk=%s", op, sender.__name__, instance.pk)

    @receiver(post_delete, sender=model_class)
    def on_delete(sender: type, instance: Any, **kwargs: Any) -> None:
        _log.info("audit_shadow: DELETE on %s pk=%s", sender.__name__, instance.pk)
