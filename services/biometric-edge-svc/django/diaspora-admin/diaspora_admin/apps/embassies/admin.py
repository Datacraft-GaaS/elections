"""Django admin registration for embassy station models (WP-5.8, M24)."""

from __future__ import annotations

from django.contrib import admin, messages
from django.http import HttpRequest

from .models import (
    CustodyEvent,
    DiasporaRoll,
    EmbassyStation,
    Form34ADiaspora,
    PouchCustody,
)


class CustodyEventInline(admin.TabularInline):
    model = CustodyEvent
    extra = 0
    readonly_fields = (
        "transferred_at",
        "from_person",
        "to_person",
        "location",
        "gps_lat",
        "gps_lon",
        "signature",
        "seq",
    )
    can_delete = False


@admin.register(EmbassyStation)
class EmbassyStationAdmin(admin.ModelAdmin):
    list_display = (
        "station_code",
        "city",
        "country_iso",
        "timezone",
        "kits_deployed",
        "is_open_display",
        "opened_at",
        "closed_at",
    )
    search_fields = ("station_code", "city", "country_iso")
    list_filter = ("country_iso",)
    readonly_fields = ("id", "created_at")

    @admin.display(boolean=True, description="Open")
    def is_open_display(self, obj: EmbassyStation) -> bool:
        return obj.is_open


@admin.register(DiasporaRoll)
class DiasporaRollAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "embassy_station",
        "voter_count",
        "frozen",
        "snapshot_taken_at",
    )
    search_fields = ("embassy_station__station_code", "hash")
    list_filter = ("frozen", "embassy_station__country_iso")
    readonly_fields = ("id", "hash", "created_at")


def approve_forms(
    modeladmin: Form34ADiasporaAdmin, request: HttpRequest, queryset
) -> None:
    updated = queryset.update(status=Form34ADiaspora.STATUS_APPROVED)
    modeladmin.message_user(request, f"{updated} form(s) approved.", messages.SUCCESS)


approve_forms.short_description = "Approve selected Form 34A entries"  # type: ignore[attr-defined]


def dispute_forms(
    modeladmin: Form34ADiasporaAdmin, request: HttpRequest, queryset
) -> None:
    updated = queryset.update(status=Form34ADiaspora.STATUS_DISPUTED)
    modeladmin.message_user(
        request, f"{updated} form(s) marked as disputed.", messages.WARNING
    )


dispute_forms.short_description = "Mark selected Form 34A entries as disputed"  # type: ignore[attr-defined]


@admin.register(Form34ADiaspora)
class Form34ADiasporaAdmin(admin.ModelAdmin):
    list_display = (
        "form_id",
        "embassy_station",
        "status",
        "rejected",
        "disputed",
        "transmitted_at",
        "created_at",
    )
    search_fields = ("form_id", "embassy_station__station_code")
    list_filter = ("status", "embassy_station__country_iso", "transmitted_at")
    readonly_fields = ("id", "hsm_signature", "transmission_batch_id", "created_at")
    actions = [approve_forms, dispute_forms]


@admin.register(PouchCustody)
class PouchCustodyAdmin(admin.ModelAdmin):
    list_display = ("id", "seal_id", "sealed_at", "received_at_nairobi", "created_at")
    search_fields = ("seal_id",)
    list_filter = ("received_at_nairobi",)
    readonly_fields = ("id", "created_at")
    inlines = [CustodyEventInline]


@admin.register(CustodyEvent)
class CustodyEventAdmin(admin.ModelAdmin):
    list_display = (
        "pouch",
        "seq",
        "from_person",
        "to_person",
        "location",
        "transferred_at",
    )
    search_fields = ("from_person", "to_person", "location", "pouch__seal_id")
    list_filter = ("transferred_at",)
    readonly_fields = (
        "pouch",
        "seq",
        "transferred_at",
        "from_person",
        "to_person",
        "location",
        "gps_lat",
        "gps_lon",
        "signature",
    )
