"""
Django admin registrations for RFID cover tag models.

All admins are read-only audit surfaces except the resolve/retire actions.
Schema is owned by Rust (managed=False); no Django migrations are run.
"""

from __future__ import annotations

from django.contrib import admin, messages
from django.http import HttpRequest
from django.utils import timezone

from .models import CoverStateTransition, CoverTamperAlert, RfidCoverRead, RfidCoverTag


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


# ---------------------------------------------------------------------------
# RfidCoverTag
# ---------------------------------------------------------------------------


@admin.register(RfidCoverTag)
class RfidCoverTagAdmin(admin.ModelAdmin):
    list_display = (
        "epc",
        "manufacturer",
        "paired_with_box",
        "tamper_state",
        "cover_state",
        "chip_family",
        "issued_at",
    )
    list_filter = ("tamper_state", "chip_family", "cover_state")
    search_fields = ("epc", "tag_id", "paired_with_box", "manufacturer")
    readonly_fields = (
        "tag_id",
        "epc",
        "chip_family",
        "paired_with_box",
        "cover_state",
        "manufacturer",
        "issued_at",
        "hsm_key_ref",
        "bluetooth_id",
    )
    actions = ["mark_retired"]
    ordering = ("-issued_at",)

    @admin.action(description="Mark selected tags as Retired")
    def mark_retired(self, request: HttpRequest, queryset) -> None:
        # Direct SQL update is intentional: managed=False means no ORM signals for
        # the Rust-owned table. The Rust service must also be notified via API.
        count = queryset.filter(tamper_state=RfidCoverTag.TamperState.INTACT).update(
            tamper_state=RfidCoverTag.TamperState.RETIRED
        )
        self.message_user(
            request, f"{count} tag(s) marked as Retired.", messages.WARNING
        )


# ---------------------------------------------------------------------------
# RfidCoverRead
# ---------------------------------------------------------------------------


@admin.register(RfidCoverRead)
class RfidCoverReadAdmin(admin.ModelAdmin):
    list_display = (
        "read_id",
        "tag_id",
        "reader_type",
        "reader_serial",
        "challenge_valid",
        "tamper_indicator",
        "read_at",
    )
    list_filter = ("reader_type", "challenge_valid", "tamper_indicator")
    search_fields = ("read_id", "tag__epc", "reader_serial")
    readonly_fields = (
        "read_id",
        "tag",
        "reader_serial",
        "reader_type",
        "challenge_nonce",
        "challenge_response",
        "challenge_valid",
        "tamper_indicator",
        "rssi",
        "latitude",
        "longitude",
        "reader_signature",
        "read_at",
    )
    ordering = ("-read_at",)

    def has_add_permission(self, request: HttpRequest) -> bool:
        return False

    def has_change_permission(self, request: HttpRequest, obj=None) -> bool:
        return False

    def has_delete_permission(self, request: HttpRequest, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# CoverStateTransition
# ---------------------------------------------------------------------------


@admin.register(CoverStateTransition)
class CoverStateTransitionAdmin(admin.ModelAdmin):
    list_display = (
        "transition_id",
        "tag_id",
        "from_state",
        "to_state",
        "officer_id",
        "prev_hash_preview",
        "this_hash_preview",
        "transitioned_at",
    )
    search_fields = ("tag__epc", "officer_id")
    readonly_fields = (
        "transition_id",
        "tag",
        "from_state",
        "to_state",
        "officer_id",
        "reader_read",
        "prev_hash",
        "this_hash",
        "hsm_signature",
        "merkle_anchor_id",
        "transitioned_at",
    )
    ordering = ("transitioned_at",)

    @admin.display(description="prev hash")
    def prev_hash_preview(self, obj: CoverStateTransition) -> str:
        return bytes(obj.prev_hash).hex()[:12] + "..." if obj.prev_hash else ""

    @admin.display(description="this hash")
    def this_hash_preview(self, obj: CoverStateTransition) -> str:
        return bytes(obj.this_hash).hex()[:12] + "..." if obj.this_hash else ""

    def has_add_permission(self, request: HttpRequest) -> bool:
        return False

    def has_change_permission(self, request: HttpRequest, obj=None) -> bool:
        return False

    def has_delete_permission(self, request: HttpRequest, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# CoverTamperAlert
# ---------------------------------------------------------------------------


@admin.register(CoverTamperAlert)
class CoverTamperAlertAdmin(admin.ModelAdmin):
    list_display = (
        "alert_id",
        "tag_id",
        "severity",
        "detected_at",
        "ecc_dispatched",
        "is_resolved",
        "resolved_at",
    )
    list_filter = ("severity", "ecc_dispatched", ResolvedStatusFilter)
    search_fields = ("alert_id", "tag__epc", "dispatch_officer_id")
    readonly_fields = (
        "alert_id",
        "tag",
        "trigger_read",
        "severity",
        "description",
        "geo_lat",
        "geo_lon",
        "expected_state",
        "observed_state",
        "ecc_dispatched",
        "dispatch_officer_id",
        "detected_at",
    )
    actions = ["bulk_resolve_with_note"]

    @admin.display(description="Resolved", boolean=True)
    def is_resolved(self, obj: CoverTamperAlert) -> bool:
        return obj.resolved_at is not None

    @admin.action(description="Bulk resolve selected alerts (mark resolved now)")
    def bulk_resolve_with_note(self, request: HttpRequest, queryset) -> None:
        note = (
            f"Bulk resolved via admin by {request.user} at {timezone.now().isoformat()}"
        )
        count = queryset.filter(resolved_at__isnull=True).update(
            resolved_at=timezone.now(),
            resolution_note=note,
        )
        self.message_user(
            request, f"{count} alert(s) marked resolved.", messages.SUCCESS
        )

    def has_add_permission(self, request: HttpRequest) -> bool:
        return False

    def has_delete_permission(self, request: HttpRequest, obj=None) -> bool:
        return False
