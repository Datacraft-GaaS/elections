"""Django admin registration for M55 HR-Payroll (WP-5.6)."""

from django.contrib import admin

from .models import (
    JobApplication,
    PayrollRun,
    PayslipLine,
    StaffContract,
    StatutoryDeduction,
)

admin.site.register(JobApplication)
admin.site.register(StaffContract)
admin.site.register(PayrollRun)
admin.site.register(PayslipLine)
admin.site.register(StatutoryDeduction)
