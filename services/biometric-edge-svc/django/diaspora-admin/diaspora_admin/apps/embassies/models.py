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

"""Django ORM models mirroring the Rust diaspora domain (WP-5.8, M24)."""

from __future__ import annotations

import uuid

from django.db import models


class EmbassyStation(models.Model):
    """An embassy or consulate polling station abroad."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    country_iso = models.CharField(max_length=2, help_text="ISO 3166-1 alpha-2")
    city = models.CharField(max_length=128)
    timezone = models.CharField(max_length=64, help_text="IANA timezone string")
    address = models.TextField()
    station_code = models.CharField(max_length=32, unique=True)
    kits_deployed = models.PositiveIntegerField(default=0)
    opened_at = models.DateTimeField(null=True, blank=True)
    closed_at = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ["country_iso", "city"]
        verbose_name = "Embassy Station"
        verbose_name_plural = "Embassy Stations"

    def __str__(self) -> str:
        return f"{self.station_code} - {self.city} ({self.country_iso})"

    @property
    def is_open(self) -> bool:
        return self.opened_at is not None and self.closed_at is None


class DiasporaRoll(models.Model):
    """A frozen snapshot of eligible diaspora voters at a station."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    embassy_station = models.ForeignKey(
        EmbassyStation,
        on_delete=models.PROTECT,
        related_name="rolls",
    )
    snapshot_taken_at = models.DateTimeField()
    voter_count = models.BigIntegerField(default=0)
    hash = models.CharField(max_length=64, help_text="BLAKE3 hex digest")
    frozen = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ["-snapshot_taken_at"]
        verbose_name = "Diaspora Roll"
        verbose_name_plural = "Diaspora Rolls"

    def __str__(self) -> str:
        return f"Roll {self.id} [{self.embassy_station.station_code}]"


class Form34ADiaspora(models.Model):
    """Official Form 34A results for a diaspora polling station."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    embassy_station = models.ForeignKey(
        EmbassyStation,
        on_delete=models.PROTECT,
        related_name="forms_34a",
    )
    form_id = models.CharField(max_length=64, unique=True)
    results = models.JSONField(default=dict, help_text="candidate_id -> vote count")
    rejected = models.BigIntegerField(default=0)
    disputed = models.BigIntegerField(default=0)
    transmitted_at = models.DateTimeField(null=True, blank=True)
    hsm_signature = models.TextField(null=True, blank=True)
    transmission_batch_id = models.UUIDField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    STATUS_PENDING = "pending"
    STATUS_APPROVED = "approved"
    STATUS_DISPUTED = "disputed"
    STATUS_CHOICES = [
        (STATUS_PENDING, "Pending"),
        (STATUS_APPROVED, "Approved"),
        (STATUS_DISPUTED, "Disputed"),
    ]
    status = models.CharField(
        max_length=16, choices=STATUS_CHOICES, default=STATUS_PENDING
    )

    class Meta:
        ordering = ["-created_at"]
        verbose_name = "Form 34A (Diaspora)"
        verbose_name_plural = "Forms 34A (Diaspora)"

    def __str__(self) -> str:
        return f"Form34A {self.form_id}"


class PouchCustody(models.Model):
    """Physical ballot pouch for air transport to Nairobi."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    # form_ids stored as JSON array of UUID strings
    form_ids = models.JSONField(default=list)
    sealed_at = models.DateTimeField(null=True, blank=True)
    seal_id = models.CharField(max_length=128, null=True, blank=True)
    received_at_nairobi = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ["-created_at"]
        verbose_name = "Pouch Custody"
        verbose_name_plural = "Pouch Custodies"

    def __str__(self) -> str:
        seal = self.seal_id or "unsealed"
        return f"Pouch {str(self.id)[:8]} [{seal}]"


class CustodyEvent(models.Model):
    """A single custody handover event in the pouch chain."""

    pouch = models.ForeignKey(
        PouchCustody,
        on_delete=models.CASCADE,
        related_name="custody_events",
    )
    transferred_at = models.DateTimeField()
    from_person = models.CharField(max_length=256)
    to_person = models.CharField(max_length=256)
    location = models.CharField(max_length=256)
    gps_lat = models.FloatField(null=True, blank=True)
    gps_lon = models.FloatField(null=True, blank=True)
    signature = models.TextField()
    seq = models.PositiveIntegerField(default=0, help_text="Transfer sequence number")

    class Meta:
        ordering = ["pouch", "seq"]
        unique_together = [("pouch", "seq")]
        verbose_name = "Custody Event"
        verbose_name_plural = "Custody Events"

    def __str__(self) -> str:
        return f"Transfer #{self.seq}: {self.from_person} -> {self.to_person}"
