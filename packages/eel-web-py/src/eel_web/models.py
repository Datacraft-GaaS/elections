"""eel_web models: outbox, idempotency, dedup, audit shadow."""

from __future__ import annotations

from django.db import models


class OutboxRecord(models.Model):
    """Transactional outbox table."""

    id = models.UUIDField(primary_key=True)
    event_type = models.CharField(max_length=255)
    stream_id = models.UUIDField()
    payload = models.TextField(null=True, blank=True)
    blob_ref = models.TextField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    published_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        db_table = "outbox"


class IdempotencyKey(models.Model):
    key = models.CharField(max_length=255, unique=True)
    status_code = models.IntegerField()
    response_body = models.BinaryField()
    request_body_hash = models.BinaryField()
    created_at = models.DateTimeField(auto_now_add=True)
    expires_at = models.DateTimeField()

    class Meta:
        db_table = "idempotency_keys"


class DeduplicatedEvent(models.Model):
    event_id = models.CharField(max_length=255, unique=True)
    processed_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "dedup_events"


class AuditShadowBase(models.Model):
    """Abstract base for audit shadow history tables."""

    operation = models.CharField(max_length=10)  # INSERT/UPDATE/DELETE
    old_data = models.JSONField(null=True, blank=True)
    new_data = models.JSONField(null=True, blank=True)
    changed_at = models.DateTimeField(auto_now_add=True)
    changed_by = models.CharField(max_length=255, null=True, blank=True)

    class Meta:
        abstract = True


class AuditShadowMixin:
    """Mixin for Django models. Hooks into post_save/post_delete signals
    to write audit history via eel_web.signals.

    Usage:
            class MyModel(AuditShadowMixin, models.Model):
                    ...
    """
