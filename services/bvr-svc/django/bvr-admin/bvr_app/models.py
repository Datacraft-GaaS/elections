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

from django.db import models


class Voter(models.Model):
    """Read-only view of the voter register. Canonical data in Rust service."""

    voter_id = models.UUIDField(primary_key=True)
    county_code = models.CharField(max_length=3)
    status = models.CharField(max_length=32)
    election_scope = models.CharField(max_length=64)
    is_canary = models.BooleanField(default=False)
    created_at = models.DateTimeField()
    updated_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "voters"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"Voter({self.voter_id})"


class AdjudicationCase(models.Model):
    case_id = models.UUIDField(primary_key=True)
    voter_ids = models.JSONField()
    modality = models.CharField(max_length=32)
    status = models.CharField(max_length=32)
    investigator_id = models.UUIDField(null=True, blank=True)
    created_at = models.DateTimeField()

    class Meta:
        managed = False
        db_table = "dedup_orders"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"AdjudicationCase({self.case_id})"


class SarRequestModel(models.Model):
    sar_id = models.UUIDField(primary_key=True)
    voter_id = models.UUIDField()
    status = models.CharField(max_length=32)
    created_at = models.DateTimeField()
    resolved_at = models.DateTimeField(null=True, blank=True)

    class Meta:
        managed = False
        db_table = "sar_requests"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return f"SAR({self.sar_id})"


class AuditEvent(models.Model):
    event_id = models.UUIDField(primary_key=True)
    actor = models.CharField(max_length=255)
    action = models.CharField(max_length=128)
    resource = models.CharField(max_length=255)
    ts = models.DateTimeField()
    signed_by = models.CharField(max_length=128, blank=True)

    class Meta:
        managed = False
        db_table = "audit_events"
        ordering = ["-ts"]

    def __str__(self) -> str:
        return f"AuditEvent({self.action} by {self.actor})"
