"""Audit shadow signals for AuditShadowMixin models."""

from __future__ import annotations

import logging
from typing import Any, Type

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
