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

"""Django models for support-admin (WP-5.5, M25, M27).

Read-projection copies of the Rust event-sourced aggregates for the Django
admin UI. Authoritative store is the event log in support-admin-svc. No
business logic lives here.
"""

from __future__ import annotations

import uuid

from django.db import models


# ---- M25: Special Voter Facilities ------------------------------------------


class SpecialVoterFacility(models.Model):
    """Projection of SpecialVoterFacility aggregate (M25, data class B)."""

    CATEGORY_CHOICES = [
        ("REMAND_PRISON", "Remand Prison"),
        ("HOSPITAL", "Hospital"),
        ("BARRACKS", "KDF / GSU / Prison Service"),
        ("REFUGEE_CAMP", "Refugee Camp"),
        ("IDP_CAMP", "IDP Camp"),
    ]
    SESSION_STATE_CHOICES = [
        ("SEALED", "Sealed"),
        ("DUAL_BIOMETRIC_PENDING", "Dual Biometric Pending"),
        ("OPEN", "Open"),
        ("SUSPENDED", "Suspended"),
        ("CLOSED", "Closed"),
    ]

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    category = models.CharField(max_length=32, choices=CATEGORY_CHOICES)
    county_code = models.CharField(max_length=8, db_index=True)
    presiding_officer_id = models.CharField(max_length=128)
    voting_date = models.DateField()
    published_voter_count = models.PositiveIntegerField(default=0)
    session_state = models.CharField(
        max_length=32, choices=SESSION_STATE_CHOICES, default="SEALED"
    )
    violation_pending_ack = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = "support_admin_facilities"
        ordering = ["voting_date", "county_code"]
        verbose_name = "Special Voter Facility"
        verbose_name_plural = "Special Voter Facilities"

    def __str__(self) -> str:
        return f"{self.get_category_display()} ({self.county_code}, {self.voting_date})"


class S44AOverrideRequest(models.Model):
    """Projection of s.44A dual-officer override log (M25 guardrail)."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    facility = models.ForeignKey(
        SpecialVoterFacility, on_delete=models.PROTECT, related_name="s44a_overrides"
    )
    reason_code = models.CharField(max_length=64)
    requesting_officer_id = models.CharField(max_length=128)
    countersigning_officer_id = models.CharField(max_length=128)
    approved = models.BooleanField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "support_admin_s44a_overrides"
        ordering = ["-created_at"]
        verbose_name = "S.44A Override"
        verbose_name_plural = "S.44A Override Queue"

    def __str__(self) -> str:
        status = "approved" if self.approved else "pending"
        return f"S44A({self.facility_id}, {self.reason_code}, {status})"


class IDPReassignment(models.Model):
    """Projection of IDPReassignment aggregate (M25, data class B)."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    idp_facility = models.ForeignKey(
        SpecialVoterFacility, on_delete=models.PROTECT, related_name="idp_reassignments"
    )
    original_station_code = models.CharField(max_length=32, db_index=True)
    displacement_source = models.CharField(max_length=64)
    effective_date = models.DateField()
    reassigned_at = models.DateTimeField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "support_admin_idp_reassignments"
        ordering = ["-reassigned_at"]
        verbose_name = "IDP Reassignment"

    def __str__(self) -> str:
        return f"IDP({self.original_station_code} -> {self.idp_facility_id})"


# ---- M27: Intent Preservation -----------------------------------------------


class AdjudicationCatalogueEntry(models.Model):
    """Projection of the adjudication reason-code catalogue (M27, A1-A8 / V1-V3).

    Rows are seeded from the Rust catalogue; the Django UI is read-only for
    standard codes but allows the Commission legal team to add jurisdiction-specific
    supplementary codes (prefix S1-S9) via admin.
    """

    DECISION_CHOICES = [
        ("REJECT", "Reject"),
        ("ACCEPT", "Accept"),
    ]

    code = models.CharField(max_length=8, primary_key=True)
    decision = models.CharField(max_length=8, choices=DECISION_CHOICES)
    description_en = models.CharField(max_length=256)
    description_sw = models.CharField(max_length=256)
    is_standard = models.BooleanField(default=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "support_admin_adjudication_catalogue"
        ordering = ["code"]
        verbose_name = "Adjudication Reason Code"

    def __str__(self) -> str:
        return f"{self.code}: {self.description_en[:60]}"


class AdjudicationEvent(models.Model):
    """Projection of AdjudicationEvent aggregate (M27)."""

    DECISION_CHOICES = [("REJECT", "Reject"), ("ACCEPT", "Accept")]

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    ballot_id = models.CharField(max_length=128, db_index=True)
    station_code = models.CharField(max_length=32, db_index=True)
    contest_id = models.CharField(max_length=128)
    adjudicator_id = models.CharField(max_length=128)
    reason_code = models.CharField(max_length=8)
    decision = models.CharField(max_length=8, choices=DECISION_CHOICES)
    # MinIO reference; null for acceptance codes with no photo required.
    photo_evidence_ref = models.CharField(max_length=512, blank=True, null=True)
    adjudicated_at = models.DateTimeField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "support_admin_adjudication_events"
        ordering = ["-adjudicated_at"]
        verbose_name = "Adjudication Event"

    def __str__(self) -> str:
        return f"Adjudication({self.ballot_id}, {self.reason_code}, {self.decision})"


class RejectionRateAlert(models.Model):
    """Projection of RejectionRateAlert aggregate (M27); surfaced to ECC."""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4)
    station_code = models.CharField(max_length=32, db_index=True)
    contest_id = models.CharField(max_length=128)
    rejection_rate = models.FloatField()
    national_mean = models.FloatField()
    z_score = models.FloatField()
    acknowledged = models.BooleanField(default=False)
    acknowledged_by = models.CharField(max_length=128, blank=True, null=True)
    alerted_at = models.DateTimeField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "support_admin_rejection_rate_alerts"
        ordering = ["-z_score"]
        verbose_name = "Rejection Rate Alert"

    def __str__(self) -> str:
        return (
            f"Alert({self.station_code}, z={self.z_score:.2f}, ack={self.acknowledged})"
        )
