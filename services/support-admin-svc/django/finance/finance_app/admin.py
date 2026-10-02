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
