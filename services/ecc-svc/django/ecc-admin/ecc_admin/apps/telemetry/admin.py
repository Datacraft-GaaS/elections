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

"""Django admin registrations for mobile telemetry models.

All admins are read-only audit surfaces. Schema owned by V052 (managed=False).
"""

from __future__ import annotations

from django.contrib import admin

from .models import (
    TelemetryActivityFrame,
    TelemetryAlert,
    TelemetryDevice,
    TelemetryGeoDwell,
    TelemetryQueryAudit,
    TelemetrySession,
    TelemetrySnapshot,
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


class ReadOnlyAdmin(admin.ModelAdmin):
    """Base for every telemetry surface: no add, no change, no delete.

    The tables are append-only in Postgres (V052 installs a guard trigger on
    signals, alerts and snapshots), so an admin that offered a save button would
    offer one that always raises.
    """

    def has_add_permission(self, request) -> bool:
        return False

    def has_change_permission(self, request, obj=None) -> bool:
        return False

    def has_delete_permission(self, request, obj=None) -> bool:
        return False


# ---------------------------------------------------------------------------
# TelemetryDevice
# ---------------------------------------------------------------------------


@admin.register(TelemetryDevice)
class TelemetryDeviceAdmin(ReadOnlyAdmin):
    list_display = (
        "device_id",
        "kit_id",
        "app_id",
        "persona_role",
        "first_seen_at",
        "last_seen_at",
        "revoked_at",
    )
    list_filter = ("persona_role",)
    search_fields = ("device_id", "kit_id", "app_id", "spiffe_id")
    readonly_fields = (
        "device_id",
        "kit_id",
        "app_id",
        "persona_role",
        "spiffe_id",
        "hmac_key_hsm_ref",
        "first_seen_at",
        "last_seen_at",
        "revoked_at",
        "revocation_reason",
    )
    date_hierarchy = "last_seen_at"


# ---------------------------------------------------------------------------
# TelemetrySession
# ---------------------------------------------------------------------------


@admin.register(TelemetrySession)
class TelemetrySessionAdmin(ReadOnlyAdmin):
    list_display = (
        "session_id",
        "device_id",
        "actor_svid",
        "entered_at",
        "exited_at",
        "exit_reason",
        "signals_count",
    )
    list_filter = ("exit_reason",)
    search_fields = ("session_id", "device_id", "actor_svid")
    readonly_fields = (
        "session_id",
        "device_id",
        "actor_svid",
        "entered_at",
        "exited_at",
        "exit_reason",
        "signals_count",
    )
    date_hierarchy = "entered_at"


# ---------------------------------------------------------------------------
# TelemetrySignal -- deliberately not registered
# ---------------------------------------------------------------------------

# `eel_telemetry_signals` is partitioned by `occurred_at`, so its primary key is
# the composite (signal_id, occurred_at); a single-column key cannot be unique
# on a partitioned table. Django's admin refuses composite-PK models outright
# (`ImproperlyConfigured: ... has a composite primary key, so it cannot be
# registered with admin`), so this table has no admin surface. Modelling it with
# a single-column key to make it registrable would assert a uniqueness the table
# does not have -- the wrong fix. Signals are read through the telemetry-query
# API; the evidence-export view below is the console's own window onto them.


# ---------------------------------------------------------------------------
# TelemetryActivityFrame
# ---------------------------------------------------------------------------


@admin.register(TelemetryActivityFrame)
class TelemetryActivityFrameAdmin(ReadOnlyAdmin):
    list_display = (
        "frame_id",
        "device_id",
        "second_bucket",
        "motion_class",
        "accel_max_g",
        "battery_pct",
        "network_class",
    )
    list_filter = ("motion_class", "network_class")
    search_fields = ("frame_id", "device_id")
    readonly_fields = (
        "frame_id",
        "device_id",
        "second_bucket",
        "accel_x_avg",
        "accel_y_avg",
        "accel_z_avg",
        "accel_max_g",
        "gyro_x_avg",
        "gyro_y_avg",
        "gyro_z_avg",
        "motion_class",
        "battery_pct",
        "battery_temp_c",
        "network_class",
        "lat_coarse",
        "lon_coarse",
    )
    date_hierarchy = "second_bucket"


# ---------------------------------------------------------------------------
# TelemetryGeoDwell
# ---------------------------------------------------------------------------


@admin.register(TelemetryGeoDwell)
class TelemetryGeoDwellAdmin(ReadOnlyAdmin):
    list_display = (
        "dwell_id",
        "device_id",
        "polygon_ref",
        "entered_at",
        "exited_at",
        "dwell_seconds",
    )
    list_filter = ("polygon_ref",)
    search_fields = ("dwell_id", "device_id", "polygon_ref")
    readonly_fields = (
        "dwell_id",
        "device_id",
        "polygon_ref",
        "entered_at",
        "exited_at",
        "dwell_seconds",
    )
    date_hierarchy = "entered_at"


# ---------------------------------------------------------------------------
# TelemetryAlert
# ---------------------------------------------------------------------------


@admin.register(TelemetryAlert)
class TelemetryAlertAdmin(ReadOnlyAdmin):
    list_display = (
        "alert_id",
        "device_id",
        "alert_class",
        "severity",
        "detected_at",
        "ecc_dispatched",
        "resolved_at",
    )
    list_filter = ("severity", "alert_class", "ecc_dispatched", PendingAlertFilter)
    search_fields = ("alert_id", "device_id", "resolution_note")
    readonly_fields = (
        "alert_id",
        "device_id",
        "alert_class",
        "severity",
        "detected_at",
        "triggering_signal",
        "payload",
        "ecc_dispatched",
        "ecc_dispatched_at",
        "resolved_at",
        "resolution_note",
    )
    date_hierarchy = "detected_at"


# ---------------------------------------------------------------------------
# TelemetryQueryAudit
# ---------------------------------------------------------------------------


@admin.register(TelemetryQueryAudit)
class TelemetryQueryAuditAdmin(ReadOnlyAdmin):
    list_display = (
        "query_id",
        "actor_svid",
        "actor_role",
        "executed_at",
        "elapsed_ms",
        "row_count",
        "evidence_bundle_ref",
    )
    list_filter = ("actor_role",)
    search_fields = ("query_id", "actor_svid", "query_text", "evidence_bundle_ref")
    readonly_fields = (
        "query_id",
        "actor_svid",
        "actor_role",
        "query_dsl",
        "query_text",
        "executed_at",
        "elapsed_ms",
        "row_count",
        "evidence_bundle_hash",
        "evidence_bundle_ref",
        "correlation_id",
    )
    date_hierarchy = "executed_at"


# ---------------------------------------------------------------------------
# TelemetrySnapshot
# ---------------------------------------------------------------------------


@admin.register(TelemetrySnapshot)
class TelemetrySnapshotAdmin(ReadOnlyAdmin):
    list_display = (
        "snapshot_id",
        "snapshot_date",
        "device_id",
        "snapshot_taken_at",
        "tlog_leaf_index",
    )
    search_fields = ("snapshot_id", "device_id")
    readonly_fields = (
        "snapshot_id",
        "snapshot_date",
        "device_id",
        "device_merkle_root",
        "snapshot_taken_at",
        "tlog_leaf_index",
        "tlog_root_hash",
        "inclusion_proof",
    )
    date_hierarchy = "snapshot_date"
