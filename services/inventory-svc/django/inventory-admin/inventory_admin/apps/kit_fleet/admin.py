from django.contrib import admin

from .models import Kit


@admin.register(Kit)
class KitAdmin(admin.ModelAdmin):
    list_display = (
        "kit_id",
        "serial",
        "kit_type",
        "status",
        "station_code",
        "last_seen_at",
    )
    list_filter = ("status", "kit_type")
    search_fields = ("kit_id", "serial", "station_code")
    readonly_fields = ("kit_id", "serial", "last_seen_at", "created_at")
    ordering = ("-last_seen_at",)
