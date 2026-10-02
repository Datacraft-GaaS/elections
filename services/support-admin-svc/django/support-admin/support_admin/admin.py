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

"""Django admin registration for support-admin (WP-5.5, M25, M27)."""

from django.contrib import admin

from .models import (
    AdjudicationCatalogueEntry,
    AdjudicationEvent,
    IDPReassignment,
    RejectionRateAlert,
    S44AOverrideRequest,
    SpecialVoterFacility,
)


# ---- M25: Special Voter Facilities ------------------------------------------


@admin.register(SpecialVoterFacility)
class SpecialVoterFacilityAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "category",
        "county_code",
        "voting_date",
        "session_state",
        "published_voter_count",
        "violation_pending_ack",
    )
    list_filter = ("category", "session_state", "voting_date", "violation_pending_ack")
    search_fields = ("county_code", "presiding_officer_id")
    readonly_fields = ("id", "created_at", "updated_at")
    ordering = ("voting_date", "county_code")


@admin.register(S44AOverrideRequest)
class S44AOverrideAdmin(admin.ModelAdmin):
    """S.44A dual-officer override queue (IAB review panel).

    Requires IAB_AUDITOR or COMMISSION_OPS role. All rows are read-only
    after creation; audit trail must be preserved (no delete).
    """

    list_display = (
        "id",
        "facility",
        "reason_code",
        "requesting_officer_id",
        "countersigning_officer_id",
        "approved",
        "created_at",
    )
    list_filter = ("approved", "reason_code")
    search_fields = ("requesting_officer_id", "countersigning_officer_id")
    readonly_fields = (
        "id",
        "facility",
        "reason_code",
        "requesting_officer_id",
        "countersigning_officer_id",
        "approved",
        "created_at",
    )

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False

    def has_change_permission(self, request, obj=None):
        return False


@admin.register(IDPReassignment)
class IDPReassignmentAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "original_station_code",
        "idp_facility",
        "displacement_source",
        "effective_date",
        "reassigned_at",
    )
    list_filter = ("displacement_source", "effective_date")
    search_fields = ("original_station_code",)
    readonly_fields = ("id", "created_at")

    def has_delete_permission(self, request, obj=None):
        return False


# ---- M27: Intent Preservation -----------------------------------------------


@admin.register(AdjudicationCatalogueEntry)
class AdjudicationCatalogueAdmin(admin.ModelAdmin):
    """Adjudication reason-code catalogue browser.

    Standard codes (A1-A8, V1-V3) are read-only. the Commission legal may add
    supplementary S-codes via this panel.
    """

    list_display = ("code", "decision", "description_en", "is_standard", "created_at")
    list_filter = ("decision", "is_standard")
    search_fields = ("code", "description_en", "description_sw")
    readonly_fields = ("created_at",)

    def has_change_permission(self, request, obj=None):
        # Standard codes locked; supplementary codes may be edited.
        if obj is not None and obj.is_standard:
            return False
        return super().has_change_permission(request, obj)

    def has_delete_permission(self, request, obj=None):
        if obj is not None and obj.is_standard:
            return False
        return super().has_delete_permission(request, obj)


@admin.register(AdjudicationEvent)
class AdjudicationEventAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "ballot_id",
        "station_code",
        "contest_id",
        "reason_code",
        "decision",
        "adjudicator_id",
        "adjudicated_at",
    )
    list_filter = ("decision", "reason_code", "contest_id")
    search_fields = ("ballot_id", "station_code", "adjudicator_id")
    readonly_fields = ("id", "created_at")

    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(RejectionRateAlert)
class RejectionRateAlertAdmin(admin.ModelAdmin):
    """Rejection-rate outlier dashboard surfaced to the Commission ECC (M27)."""

    list_display = (
        "id",
        "station_code",
        "contest_id",
        "rejection_rate_pct",
        "z_score",
        "acknowledged",
        "acknowledged_by",
        "alerted_at",
    )
    list_filter = ("acknowledged", "contest_id")
    search_fields = ("station_code",)
    readonly_fields = (
        "id",
        "station_code",
        "contest_id",
        "rejection_rate",
        "national_mean",
        "z_score",
        "alerted_at",
        "created_at",
    )

    @admin.display(description="Rate (%)")
    def rejection_rate_pct(self, obj) -> str:
        return f"{obj.rejection_rate * 100:.2f}%"

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False
