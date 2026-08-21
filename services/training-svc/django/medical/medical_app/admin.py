from django.contrib import admin
from medical_app.models import (
    MedicalIncident,
    DispatchRequest,
    ClusterAlert,
    MedicalUnit,
)


@admin.register(MedicalIncident)
class MedicalIncidentAdmin(admin.ModelAdmin):
    list_display = [
        "id",
        "station_id",
        "incident_type",
        "severity",
        "status",
        "reported_at",
    ]
    list_filter = ["incident_type", "severity", "status"]
    readonly_fields = ["reported_at"]


@admin.register(DispatchRequest)
class DispatchRequestAdmin(admin.ModelAdmin):
    list_display = ["id", "incident", "dispatched_to", "dispatched_at", "eta_minutes"]
    list_filter = ["dispatched_to"]
    readonly_fields = ["dispatched_at"]


@admin.register(ClusterAlert)
class ClusterAlertAdmin(admin.ModelAdmin):
    list_display = [
        "id",
        "county_code",
        "incident_count",
        "alert_threshold",
        "raised_at",
        "resolved_at",
    ]
    readonly_fields = ["raised_at"]


@admin.register(MedicalUnit)
class MedicalUnitAdmin(admin.ModelAdmin):
    list_display = [
        "unit_id",
        "unit_type",
        "county",
        "sub_county",
        "capacity",
        "locked",
    ]
    list_filter = ["unit_type", "county", "locked"]
    search_fields = ["unit_id"]
