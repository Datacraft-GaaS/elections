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
