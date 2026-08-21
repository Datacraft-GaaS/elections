from django.contrib import admin

from .models import BallotLot


@admin.register(BallotLot)
class BallotLotAdmin(admin.ModelAdmin):
    list_display = (
        "lot_id",
        "print_order_id",
        "printer_facility",
        "quantity_printed",
        "variance_pct",
        "status",
        "received_at",
    )
    list_filter = ("status",)
    search_fields = ("lot_id", "print_order_id")
    readonly_fields = ("lot_id",)
