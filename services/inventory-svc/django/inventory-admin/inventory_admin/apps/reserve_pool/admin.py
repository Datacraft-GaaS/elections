from django.contrib import admin

from .models import ReservePool


@admin.register(ReservePool)
class ReservePoolAdmin(admin.ModelAdmin):
    list_display = (
        "pool_id",
        "scope",
        "asset_type",
        "count",
        "alert_threshold",
        "is_critical",
        "updated_at",
    )
    list_filter = ("asset_type", "scope")
    search_fields = ("pool_id", "scope", "asset_type")
    readonly_fields = ("updated_at",)

    @admin.display(boolean=True, description="Critical")
    def is_critical(self, obj: ReservePool) -> bool:
        return obj.count < obj.alert_threshold
