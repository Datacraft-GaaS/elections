from django.contrib import admin

from .models import Budget, BudgetLine, CoBReport, PaymentVoucher, ProcurementRecord


@admin.register(Budget)
class BudgetAdmin(admin.ModelAdmin):
    list_display = ["id", "financial_year", "status", "total_kes", "created_at"]
    list_filter = ["status", "financial_year"]
    search_fields = ["id", "election_id", "financial_year"]


@admin.register(BudgetLine)
class BudgetLineAdmin(admin.ModelAdmin):
    list_display = ["id", "budget", "programme_code", "allocated_kes", "spent_kes"]
    list_filter = ["budget"]
    search_fields = ["programme_code", "description"]


@admin.register(ProcurementRecord)
class ProcurementRecordAdmin(admin.ModelAdmin):
    list_display = ["id", "ppra_ref", "vendor_id", "amount_kes", "status"]
    list_filter = ["status", "procurement_method"]
    search_fields = ["ppra_ref", "vendor_id"]


@admin.register(PaymentVoucher)
class PaymentVoucherAdmin(admin.ModelAdmin):
    list_display = ["id", "procurement", "amount_kes", "status", "initiated_by"]
    list_filter = ["status"]
    search_fields = ["id", "initiated_by", "ifmis_ref"]


@admin.register(CoBReport)
class CoBReportAdmin(admin.ModelAdmin):
    list_display = ["id", "budget", "reporting_period", "total_spent", "variance"]
    list_filter = ["reporting_period"]
    search_fields = ["budget__id", "reporting_period"]
