"""
Django admin registrations for mobile telemetry models.

All admins are read-only audit surfaces. Schema owned by Rust (managed=False).
"""

from __future__ import annotations

from django.contrib import admin

from .models import (
    TelemetryAlert,
    TelemetryDeviceSnapshot,
    TelemetryMerkleSnapshot,
    TelemetryOfficerSession,
    TelemetryQueryAudit,
    TelemetrySignal,
)


class PendingAlertFilter(admin.SimpleListFilter):
    """Filter alerts by resolved/pending status."""

    title = "resolved status"
    parameter_name = "resolved"

    def lookups(self, request, model_admin):
        return [("yes", "Resolved"), ("no", "Pending")]

    def queryset(self, request, queryset):
        if self.value() == "yes":
            return queryset.filter(resolved_at__isnull=False)
        if self.value() == "no":
            return queryset.filter(resolved_at__isnull=True)
        return queryset


# ---------------------------------------------------------------------------
# TelemetrySignal
# ---------------------------------------------------------------------------


@admin.register(TelemetrySignal)
class TelemetrySignalAdmin(admin.ModelAdmin):
    list_display = (
        "signal_id",
        "device_id",
        "signal_class",
        "occurred_at",
        "severity",
        "county",
        "station_id",
        "officer_id",
    )
    list_filter = ("signal_class", "severity", "county")
    search_fields = ("signal_id", "device_id", "officer_id", "station_id", "county")
    readonly_fields = (
        "signal_id",
        "device_id",
        "signal_class",
        "occurred_at",
        "payload",
        "geo_lat",
        "geo_lon",
        "severity",
        "station_id",
        "county",
        "officer_id",
        "motion_class",
        "network_class",
        "battery_pct",
        "attestation_status",
        "prev_hash",
        "chain_seq",
    )
    date_hierarchy = "occurred_at"

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryAlert
# ---------------------------------------------------------------------------


@admin.register(TelemetryAlert)
class TelemetryAlertAdmin(admin.ModelAdmin):
    list_display = (
        "alert_id",
        "device_id",
        "signal_class",
        "severity",
        "detected_at",
        "resolved_at",
        "county",
        "station_id",
    )
    list_filter = ("severity", "signal_class", "county", PendingAlertFilter)
    search_fields = ("alert_id", "device_id", "officer_id", "station_id", "description")
    readonly_fields = (
        "alert_id",
        "device_id",
        "signal_class",
        "severity",
        "description",
        "detected_at",
        "resolved_at",
        "resolver_svid",
        "station_id",
        "county",
        "officer_id",
    )
    date_hierarchy = "detected_at"

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryDeviceSnapshot
# ---------------------------------------------------------------------------


@admin.register(TelemetryDeviceSnapshot)
class TelemetryDeviceSnapshotAdmin(admin.ModelAdmin):
    list_display = (
        "device_id",
        "snapshot_date",
        "last_battery_pct",
        "last_attestation_status",
        "signal_count",
        "drop_count",
        "session_count",
    )
    list_filter = ("last_attestation_status",)
    search_fields = ("device_id", "snapshot_id")
    readonly_fields = (
        "snapshot_id",
        "device_id",
        "snapshot_date",
        "last_battery_pct",
        "last_geo_lat",
        "last_geo_lon",
        "last_attestation_status",
        "signal_count",
        "drop_count",
        "session_count",
        "merkle_root",
        "tlog_anchor",
    )
    date_hierarchy = "snapshot_date"

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryOfficerSession
# ---------------------------------------------------------------------------


@admin.register(TelemetryOfficerSession)
class TelemetryOfficerSessionAdmin(admin.ModelAdmin):
    list_display = (
        "officer_id",
        "device_id",
        "login_at",
        "logout_at",
        "station_id",
        "county",
        "biometric_verified",
        "anomaly_flag",
    )
    list_filter = ("biometric_verified", "anomaly_flag", "county")
    search_fields = ("officer_id", "device_id", "station_id")
    readonly_fields = (
        "session_id",
        "officer_id",
        "device_id",
        "login_at",
        "logout_at",
        "station_id",
        "county",
        "geo_lat",
        "geo_lon",
        "biometric_verified",
        "anomaly_flag",
        "anomaly_description",
    )
    date_hierarchy = "login_at"

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryMerkleSnapshot
# ---------------------------------------------------------------------------


@admin.register(TelemetryMerkleSnapshot)
class TelemetryMerkleSnapshotAdmin(admin.ModelAdmin):
    list_display = (
        "device_id",
        "period_start",
        "period_end",
        "leaf_count",
        "merkle_root",
        "anchored_at",
    )
    search_fields = ("device_id", "merkle_root", "tlog_entry_id")
    readonly_fields = (
        "snapshot_id",
        "device_id",
        "period_start",
        "period_end",
        "leaf_count",
        "merkle_root",
        "tlog_entry_id",
        "anchored_at",
    )

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryQueryAudit
# ---------------------------------------------------------------------------


@admin.register(TelemetryQueryAudit)
class TelemetryQueryAuditAdmin(admin.ModelAdmin):
    list_display = (
        "query_id",
        "actor_svid",
        "executed_at",
        "row_count",
        "evidence_exported",
        "preset_name",
    )
    list_filter = ("evidence_exported",)
    search_fields = ("query_id", "actor_svid", "preset_name", "nl_text")
    readonly_fields = (
        "query_id",
        "actor_svid",
        "query_dsl",
        "executed_at",
        "row_count",
        "result_sha256",
        "evidence_exported",
        "preset_name",
        "nl_text",
    )
    date_hierarchy = "executed_at"

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False
