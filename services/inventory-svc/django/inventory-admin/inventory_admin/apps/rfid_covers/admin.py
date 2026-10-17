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

"""Django admin registrations for the RFID cover custody tables.

Schema owned by V051 (`managed=False`); Django runs no migrations for it.

Everything here is read-only, and that is a property of the schema rather than a
policy choice. V051 installs a BEFORE UPDATE/DELETE trigger that raises
unconditionally on both `eel_cover_state_transitions` and
`eel_cover_tamper_alerts`, so any admin write to either table fails at the
database. The registrations previously offered two actions that did exactly
that -- a tag "mark retired" UPDATE and a bulk alert resolve -- and neither had
ever run against Postgres.
"""

from __future__ import annotations

from django.contrib import admin
from django.http import HttpRequest

from .models import (
    CoverStateTransition,
    CoverTamperAlert,
    RfidCoverRead,
    RfidCoverTag,
    as_bytes,
)


class ResolvedStatusFilter(admin.SimpleListFilter):
    """Filter tamper alerts by resolved/pending status."""

    title = "resolved status"
    parameter_name = "resolved"

    def lookups(self, request, model_admin):
        return [
            ("yes", "Resolved"),
            ("no", "Pending"),
        ]

    def queryset(self, request, queryset):
        if self.value() == "yes":
            return queryset.filter(resolved_at__isnull=False)
        if self.value() == "no":
            return queryset.filter(resolved_at__isnull=True)
        return queryset


class ReadOnlyAdmin(admin.ModelAdmin):
    """No add, no change, no delete.

    `has_change_permission` returning False keeps the object page viewable
    without a Save button, which is what an audit surface wants: the row is the
    evidence and the trigger would refuse the write anyway.
    """

    def has_add_permission(self, request: HttpRequest) -> bool:
        return False

    def has_change_permission(self, request: HttpRequest, obj=None) -> bool:
        return False

    def has_delete_permission(self, request: HttpRequest, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# RfidCoverTag
# ---------------------------------------------------------------------------


@admin.register(RfidCoverTag)
class RfidCoverTagAdmin(ReadOnlyAdmin):
    list_display = (
        "epc_hex",
        "manufacturer",
        "paired_with_box",
        "tamper_state",
        "cover_state",
        "chip_family",
        "issued_at",
        "retired_at",
    )
    list_filter = ("tamper_state", "chip_family")
    search_fields = ("tag_id", "paired_with_box", "manufacturer", "lot_id")
    readonly_fields = (
        "tag_id",
        "epc_hex",
        "chip_family",
        "lot_id",
        "paired_with_box",
        "paired_at",
        "manufacturer",
        "issued_at",
        "tag_secret_hsm_key_id",
        "tamper_state",
        "tamper_detected_at",
        "tamper_detected_by",
        "retired_at",
        "cover_state",
    )
    ordering = ("-issued_at",)

    def get_queryset(self, request: HttpRequest):
        # `cover_state` is the chain's latest `to_state`, not a column, so the
        # list needs the annotation to sort and display it. Without this the
        # field resolves to nothing and the column renders empty.
        return super().get_queryset(request).with_current_state()

    @admin.display(description="EPC", ordering="epc")
    def epc_hex(self, obj: RfidCoverTag) -> str:
        return obj.epc_hex

    @admin.display(description="Cover state")
    def cover_state(self, obj: RfidCoverTag) -> str:
        return getattr(obj, "cover_state", None) or "—"


# ---------------------------------------------------------------------------
# RfidCoverRead
# ---------------------------------------------------------------------------


@admin.register(RfidCoverRead)
class RfidCoverReadAdmin(ReadOnlyAdmin):
    list_display = (
        "read_id",
        "tag",
        "reader_type",
        "reader_id",
        "challenge_valid",
        "tamper_indicator",
        "read_at",
    )
    list_filter = ("reader_type", "challenge_valid", "tamper_indicator")
    search_fields = ("read_id", "tag__tag_id", "reader_id")
    readonly_fields = (
        "read_id",
        "tag",
        "reader_id",
        "reader_type",
        "read_at",
        "geo_lat",
        "geo_lon",
        "geo_accuracy_m",
        "tag_challenge",
        "tag_response",
        "challenge_valid",
        "tamper_indicator",
        "reader_signature",
        "custody_event_id",
        "correlation_id",
    )
    ordering = ("-read_at",)
    date_hierarchy = "read_at"


# ---------------------------------------------------------------------------
# CoverStateTransition
# ---------------------------------------------------------------------------


@admin.register(CoverStateTransition)
class CoverStateTransitionAdmin(ReadOnlyAdmin):
    list_display = (
        "transition_id",
        "tag",
        "from_state",
        "to_state",
        "officer_id",
        "prev_hash_preview",
        "this_hash_preview",
        "transitioned_at",
    )
    list_filter = ("to_state",)
    search_fields = ("tag__tag_id", "officer_id", "officer_svid")
    readonly_fields = (
        "transition_id",
        "tag",
        "from_state",
        "to_state",
        "officer_id",
        "officer_svid",
        "triggered_by_read",
        "prev_hash",
        "this_hash",
        "officer_signature",
        "transitioned_at",
    )
    ordering = ("transitioned_at",)
    date_hierarchy = "transitioned_at"

    @admin.display(description="prev hash")
    def prev_hash_preview(self, obj: CoverStateTransition) -> str:
        return as_bytes(obj.prev_hash).hex()[:12] + "..." if obj.prev_hash else ""

    @admin.display(description="this hash")
    def this_hash_preview(self, obj: CoverStateTransition) -> str:
        return as_bytes(obj.this_hash).hex()[:12] + "..." if obj.this_hash else ""


# ---------------------------------------------------------------------------
# CoverTamperAlert
# ---------------------------------------------------------------------------


@admin.register(CoverTamperAlert)
class CoverTamperAlertAdmin(ReadOnlyAdmin):
    list_display = (
        "alert_id",
        "tag",
        "severity",
        "detected_at",
        "ecc_dispatched",
        "is_resolved",
        "resolved_at",
    )
    list_filter = ("severity", "ecc_dispatched", ResolvedStatusFilter)
    search_fields = ("alert_id", "tag__tag_id", "resolution_note")
    readonly_fields = (
        "alert_id",
        "tag",
        "triggering_read",
        "severity",
        "expected_state",
        "observed_state",
        "detected_at",
        "ecc_dispatched",
        "ecc_dispatched_at",
        "resolved_at",
        "resolution_note",
    )
    ordering = ("-detected_at",)
    date_hierarchy = "detected_at"

    @admin.display(description="Resolved", boolean=True)
    def is_resolved(self, obj: CoverTamperAlert) -> bool:
        return obj.resolved_at is not None
