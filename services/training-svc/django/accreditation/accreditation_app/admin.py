from django.contrib import admin
from accreditation_app.models import (
    MediaOrganisation,
    MediaAccreditation,
    AccreditationAuditEntry,
    ObserverOrganisation,
    IndividualObserver,
    CredentialRevocationEntry,
)


@admin.register(MediaOrganisation)
class MediaOrganisationAdmin(admin.ModelAdmin):
    list_display = [
        "name",
        "organisation_type",
        "mck_registered",
        "live_feed_permit",
        "created_at",
    ]
    search_fields = ["name"]
    list_filter = ["organisation_type", "mck_registered"]


@admin.register(MediaAccreditation)
class MediaAccreditationAdmin(admin.ModelAdmin):
    list_display = ["applicant_name", "organisation", "status", "badge_serial"]
    search_fields = ["applicant_name", "id_number"]
    list_filter = ["status"]
    readonly_fields = ["badge_serial", "badge_issued_at", "revoked_at"]


@admin.register(AccreditationAuditEntry)
class AccreditationAuditEntryAdmin(admin.ModelAdmin):
    list_display = [
        "action",
        "actor_id",
        "accreditation_id",
        "timestamp",
        "purpose_code",
    ]
    readonly_fields = ["id", "timestamp"]


@admin.register(ObserverOrganisation)
class ObserverOrganisationAdmin(admin.ModelAdmin):
    list_display = ["name", "org_type", "jurisdiction", "observer_quota", "approved_at"]
    search_fields = ["name"]
    list_filter = ["org_type"]


@admin.register(IndividualObserver)
class IndividualObserverAdmin(admin.ModelAdmin):
    list_display = ["full_name", "organisation", "status", "nationality"]
    search_fields = ["full_name", "id_number"]
    list_filter = ["status"]


@admin.register(CredentialRevocationEntry)
class CredentialRevocationEntryAdmin(admin.ModelAdmin):
    list_display = ["credential_serial", "revoked_at", "officer_id"]
    readonly_fields = ["revoked_at"]
