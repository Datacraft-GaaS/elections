from django.contrib import admin
from .models import Voter, AdjudicationCase, SarRequestModel, AuditEvent


@admin.register(Voter)
class VoterAdmin(admin.ModelAdmin):
    """Voter search with PII masking. Purpose of processing recorded on every reveal."""

    list_display = (
        "voter_id",
        "county_code",
        "status",
        "election_scope",
        "is_canary",
        "created_at",
    )
    list_filter = ("status", "county_code", "is_canary")
    search_fields = ("voter_id",)
    readonly_fields = (
        "voter_id",
        "county_code",
        "status",
        "election_scope",
        "is_canary",
        "created_at",
        "updated_at",
    )

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False

    def has_change_permission(self, request, obj=None):
        return False


@admin.register(AdjudicationCase)
class AdjudicationCaseAdmin(admin.ModelAdmin):
    """Adjudication queue for dedup conflicts."""

    list_display = ("case_id", "modality", "status", "investigator_id", "created_at")
    list_filter = ("status", "modality")
    readonly_fields = (
        "case_id",
        "voter_ids",
        "modality",
        "status",
        "investigator_id",
        "created_at",
    )

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(SarRequestModel)
class SarRequestAdmin(admin.ModelAdmin):
    """SAR/DSAR request handling."""

    list_display = ("sar_id", "voter_id", "status", "created_at", "resolved_at")
    list_filter = ("status",)
    readonly_fields = ("sar_id", "voter_id", "status", "created_at", "resolved_at")

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(AuditEvent)
class AuditEventAdmin(admin.ModelAdmin):
    """Audit trail view."""

    list_display = ("event_id", "actor", "action", "resource", "ts")
    list_filter = ("action",)
    readonly_fields = ("event_id", "actor", "action", "resource", "ts", "signed_by")

    def has_add_permission(self, request):
        return False

    def has_delete_permission(self, request, obj=None):
        return False

    def has_change_permission(self, request, obj=None):
        return False
